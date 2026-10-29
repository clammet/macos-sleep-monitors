#![cfg(target_os = "macos")]
#![deny(unsafe_op_in_unsafe_fn)]

mod login_item;

use std::cell::OnceCell;
use std::ffi::{c_char, c_void};
use std::process::Command;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSApplication, NSApplicationActivationPolicy,
    NSApplicationDelegate, NSControlStateValueMixed, NSControlStateValueOff, NSControlStateValueOn,
    NSEventMask, NSEventModifierFlags, NSEventType, NSImage, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem,
};
use objc2_foundation::{
    MainThreadMarker, NSData, NSNotification, NSObject, NSObjectProtocol, NSSize, NSString,
    NSTimer, ns_string,
};

type SleepDisplays = unsafe extern "C" fn() -> i32;

unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

fn native_sleep_function() -> Option<SleepDisplays> {
    // macOS RTLD_DEFAULT searches the libraries already loaded by AppKit.
    // Resolve this private API at runtime so its removal cannot prevent launch.
    let address = unsafe {
        dlsym(
            (-2_isize) as *mut c_void,
            c"SLSDisplayManagerRequestDisplaysIdle".as_ptr(),
        )
    };
    if address.is_null() {
        None
    } else {
        // SAFETY: Apple's pmset uses this symbol as CGError function(void).
        Some(unsafe { std::mem::transmute::<*mut c_void, SleepDisplays>(address) })
    }
}

fn sleep_displays() -> Result<(), String> {
    if let Some(sleep) = native_sleep_function() {
        // SAFETY: The resolved function takes no arguments and is called on the
        // main thread. It requests display idle, never system sleep.
        let error = unsafe { sleep() };
        if error == 0 {
            return Ok(());
        }
        eprintln!("Native display sleep failed ({error}); trying pmset.");
    }

    let output = Command::new("pmset")
        .arg("displaysleepnow")
        .output()
        .map_err(|error| format!("Could not run pmset: {error}"))?;
    // Some pmset versions print an error but still return exit status zero.
    if output.status.success() && output.stderr.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "pmset failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[derive(Default)]
struct AppIvars {
    status_item: OnceCell<Retained<NSStatusItem>>,
    menu: OnceCell<Retained<NSMenu>>,
    login_item: OnceCell<Retained<NSMenuItem>>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements. AppKit and the ivars
    // are only accessed on the main thread, and App does not implement Drop.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppIvars]
    struct App;

    // SAFETY: NSObjectProtocol adds no requirements.
    unsafe impl NSObjectProtocol for App {}

    // SAFETY: All delegate callbacks have the signatures declared by AppKit.
    unsafe impl NSApplicationDelegate for App {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _: &NSNotification) {
            self.install_status_item();
        }
    }

    impl App {
        #[unsafe(method(clicked:))]
        fn clicked(&self, _: &NSObject) {
            let app = NSApplication::sharedApplication(self.mtm());
            if let Some(event) = app.currentEvent() {
                if event.r#type() == NSEventType::RightMouseUp
                    || event.modifierFlags().contains(NSEventModifierFlags::Control)
                {
                    let item = self.ivars().status_item.get().unwrap();
                    let button = item.button(self.mtm()).unwrap();
                    self.refresh_login_item();
                    NSMenu::popUpContextMenu_withEvent_forView(
                        self.ivars().menu.get().unwrap(), &event, &button,
                    );
                    return;
                }
            }

            // Wait until the click has finished, so it does not immediately
            // wake the displays again. A one-shot timer retains its target.
            // SAFETY: sleepDisplays: has NSTimer's expected callback signature.
            unsafe {
                NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                    0.25, self, sel!(sleepDisplays:), None, false,
                );
            }
        }

        #[unsafe(method(sleepDisplays:))]
        fn sleep(&self, _: &NSTimer) {
            if let Err(error) = sleep_displays() {
                eprintln!("{error}");
                let alert = NSAlert::new(self.mtm());
                alert.setMessageText(ns_string!("Could not sleep the displays"));
                alert.setInformativeText(&NSString::from_str(&error));
                alert.runModal();
            }
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _: &NSObject) {
            NSApplication::sharedApplication(self.mtm()).terminate(None);
        }

        #[unsafe(method(toggleStartAtLogin:))]
        fn toggle_start_at_login(&self, _: &NSMenuItem) {
            let Some(service) = login_item::LoginItem::main_app() else {
                return;
            };
            let result = match service.status() {
                login_item::Status::Enabled | login_item::Status::RequiresApproval => {
                    service.unregister()
                }
                _ => service.register(),
            };
            self.refresh_login_item();
            if let Err(error) = result {
                let alert = NSAlert::new(self.mtm());
                alert.setMessageText(ns_string!("Could not update Start at Login"));
                alert.setInformativeText(&error.localizedDescription());
                alert.runModal();
            } else if matches!(service.status(), login_item::Status::RequiresApproval) {
                let alert = NSAlert::new(self.mtm());
                alert.setMessageText(ns_string!("Allow Sleep Monitors at login"));
                alert.setInformativeText(ns_string!(
                    "Enable Sleep Monitors in System Settings → General → Login Items."
                ));
                alert.addButtonWithTitle(ns_string!("Open Login Items"));
                alert.addButtonWithTitle(ns_string!("Later"));
                if alert.runModal() == NSAlertFirstButtonReturn {
                    service.open_settings();
                }
            }
        }
    }
);

impl App {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppIvars::default());
        // SAFETY: NSObject's init method accepts no arguments.
        unsafe { msg_send![super(this), init] }
    }

    fn install_status_item(&self) {
        let mtm = self.mtm();
        let item = NSStatusBar::systemStatusBar().statusItemWithLength(28.0);
        let button = item.button(mtm).expect("status item should have a button");
        let icon = NSData::with_bytes(include_bytes!("../assets/MonitorOffTemplate.pdf"));
        let image = NSImage::initWithData(NSImage::alloc(), &icon).or_else(|| {
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                ns_string!("display"),
                Some(ns_string!("Sleep Monitors")),
            )
        });
        if let Some(image) = image {
            image.setSize(NSSize::new(18.0, 18.0));
            image.setTemplate(true);
            button.setImage(Some(&image));
        } else {
            button.setTitle(ns_string!("Sleep"));
        }
        button.setToolTip(Some(ns_string!(
            "Sleep Monitors — click to sleep displays; right-click for options"
        )));
        button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
        // SAFETY: self remains alive throughout app.run(), and implements clicked:.
        unsafe {
            button.setTarget(Some(self));
            button.setAction(Some(sel!(clicked:)));
        }

        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);
        // SAFETY: self implements toggleStartAtLogin: with the menu action signature.
        let login = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                ns_string!("Start at Login"),
                Some(sel!(toggleStartAtLogin:)),
                ns_string!(""),
            )
        };
        // SAFETY: self remains alive throughout app.run() and implements this action.
        unsafe { login.setTarget(Some(self)) };
        menu.addItem(&login);
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        self.ivars().login_item.set(login).unwrap();
        self.refresh_login_item();
        // SAFETY: quit: is implemented above with the menu action signature.
        let quit = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                ns_string!("Quit Sleep Monitors"),
                Some(sel!(quit:)),
                ns_string!("q"),
            )
        };
        // SAFETY: self remains alive throughout app.run() and implements quit:.
        unsafe { quit.setTarget(Some(self)) };
        menu.addItem(&quit);
        self.ivars().menu.set(menu).unwrap();
        self.ivars().status_item.set(item).unwrap();
    }

    fn refresh_login_item(&self) {
        let item = self.ivars().login_item.get().unwrap();
        let Some(service) = login_item::LoginItem::main_app() else {
            item.setEnabled(false);
            item.setToolTip(Some(ns_string!(
                "Start at Login requires macOS 13 or newer."
            )));
            return;
        };
        let state = match service.status() {
            login_item::Status::Enabled => NSControlStateValueOn,
            login_item::Status::RequiresApproval => NSControlStateValueMixed,
            _ => NSControlStateValueOff,
        };
        item.setState(state);
        item.setToolTip(Some(ns_string!(
            "Launch Sleep Monitors automatically when you log in. A dash means approval is pending in System Settings."
        )));
    }
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    if std::env::args().nth(1).as_deref() == Some("--check") {
        println!(
            "Native display sleep available: {}",
            native_sleep_function().is_some()
        );
        return;
    }

    let delegate = App::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    app.run();
}
