use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::NSError;

// Load the public framework. Look up its class dynamically so display sleep
// remains available on macOS 11–12, before SMAppService was introduced.
#[link(name = "ServiceManagement", kind = "framework")]
unsafe extern "C" {}

pub enum Status {
    NotRegistered,
    Enabled,
    RequiresApproval,
    NotFound,
}

pub struct LoginItem(Retained<AnyObject>);

impl LoginItem {
    pub fn main_app() -> Option<Self> {
        let class = AnyClass::get(c"SMAppService")?;
        // SAFETY: SMAppService.mainAppService returns a non-null SMAppService.
        Some(Self(unsafe { msg_send![class, mainAppService] }))
    }

    pub fn status(&self) -> Status {
        // SAFETY: The receiver is an SMAppService; status returns NSInteger.
        let status: isize = unsafe { msg_send![&*self.0, status] };
        match status {
            0 => Status::NotRegistered,
            1 => Status::Enabled,
            2 => Status::RequiresApproval,
            _ => Status::NotFound,
        }
    }

    pub fn register(&self) -> Result<(), Retained<NSError>> {
        // SAFETY: This is the public BOOL/NSError** registration API. objc2
        // supplies the error pointer and retains any returned NSError.
        unsafe { msg_send![&*self.0, registerAndReturnError: _] }
    }

    pub fn unregister(&self) -> Result<(), Retained<NSError>> {
        // SAFETY: This is the corresponding public BOOL/NSError** API.
        unsafe { msg_send![&*self.0, unregisterAndReturnError: _] }
    }

    pub fn open_settings(&self) {
        // SAFETY: The receiver's class is SMAppService, available since macOS
        // 13, which implements this class method with no arguments or result.
        let _: () = unsafe { msg_send![self.0.class(), openSystemSettingsLoginItems] };
    }
}
