# Sleep Monitors

A tiny Rust macOS menu bar app. Click the display icon to put all monitors to
sleep. Press a key or move the mouse to wake them. Right-click (or Control-click)
the icon for **Start at Login** and **Quit Sleep Monitors**.

The app has no window or Dock icon. It requests display sleep only; it does not
request system sleep or change macOS power settings. The Mac can still sleep
later according to its existing settings.

## Download

Download the app ZIP from [GitHub Releases](https://github.com/clammet/macos-sleep-monitors/releases/latest),
extract it, and open **Sleep Monitors.app**. The initial prebuilt release is for
Apple silicon Macs (arm64).

## Build and run

Requires macOS 11 or newer, Rust, and the Xcode Command Line Tools.

```sh
make
open "dist/Sleep Monitors.app"
```

Or use `make run`. Keep the app in a stable location, such as Applications, then
enable **Start at Login** in its right-click menu. Applications is recommended,
but is not required. The option uses macOS's public
`SMAppService.mainApp` API and requires macOS 13 or newer; display sleep still
works on macOS 11 and 12.

A checkmark means login startup is enabled. A dash means macOS requires approval
in System Settings → General → Login Items. Clicking the option again disables
login startup, including a pending registration. The menu refreshes its state
each time it opens, including changes made in System Settings.

The build creates an app for the current architecture and signs it locally
with an ad-hoc signature. It is not notarized for distribution to other Macs.

To create a release ZIP and SHA-256 checksum in `dist`, run:

```sh
make release
```

## Implementation

The UI uses AppKit through the `objc2` Rust bindings. A click fires on mouse-up,
then waits 250 milliseconds before requesting sleep, allowing the initiating
click to finish first.

The native backend dynamically resolves `SLSDisplayManagerRequestDisplaysIdle`,
the display-sleep function used by current builds of Apple's
[`pmset`](https://github.com/apple-oss-distributions/PowerManagement/blob/main/pmset/pmset.m).
This is a private macOS API. If it is missing or returns an error, the app runs
`pmset displaysleepnow` instead. Neither backend requires administrator access.
No shell is involved in the fallback.

To check whether the native backend is available without sleeping the displays:

```sh
cargo run -- --check
```

Run `make check` for formatting, Clippy, and bundle metadata checks.

## Icons

Both icons use Lucide's [monitor-off](https://lucide.dev/icons/monitor-off).
The menu bar uses an 18-point vector template that adapts to macOS appearance.
The app icon uses the same symbol in white on a slate rounded-square background,
packaged as an ICNS with standard and Retina sizes.

The generated assets are checked in, so building the app needs no icon tools.
To regenerate them, install `librsvg` (`brew install librsvg`) and run `make icons`.
See [the icon source and license](assets/lucide/README.md).

## HDMI recovery research

See [display recovery notes](docs/display-recovery.md) for the PG279Q
investigation, suggested HDMI experiments, and custom timing options.
