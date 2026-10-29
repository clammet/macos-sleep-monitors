# ASUS PG279Q display recovery

Research notes, 29 October 2026.

The reported fault is an HDMI-connected PG279Q that sometimes shows a mostly
black picture with intermittent pink or corrupted blocks after sleep or an
extended input switch. Disabling the monitor's Deep Sleep options did not help.
Repeated display sleep and wake eventually restores the picture.

Those symptoms suggest a signal or HDMI receiver/link initialization problem,
but do not establish whether the cause is macOS, the cable, the monitor, or a
particular output format. I did not find a verified custom timing that fixes
this exact combination.

## Most useful next experiments

Change one thing at a time and repeat the sleep/input-switch scenario.

1. **Try a fresh connection initialization.** If BetterDisplay is installed,
   use the ASUS display's **Manage Display → Reinitialize Display** action when
   the fault occurs. This refreshes a supported Apple-silicon display through
   its EDID connection path and is a stronger recovery operation than merely
   redetecting displays. It does not itself require Pro. If it reliably helps
   after sleep, enable **Reinitialize this display on wake** in the display's
   Additional Settings. This wake trigger does not cover every input switch
   while the Mac remains awake.

2. **Test a conservative HDMI format.** In BetterDisplay's **Color Mode** menu,
   inspect the active format and, if offered, try **8-bit RGB, SDR**, at a fixed
   **60 Hz**. This is a diagnostic experiment, not a confirmed fix for pink
   blocks. The PG279Q manual lists HDMI support for both 2560 × 1440 at 60 Hz
   and 1920 × 1080 at 60 Hz; temporarily trying 1080p tests a lower-bandwidth
   signal. Change the encoding and resolution separately so the result is
   interpretable. A macOS color profile changes rendered colors, not this
   HDMI transport format.

3. **Compare the physical signal path.** Test another HDMI cable, and connect
   directly to the Mac if a dock, adapter, or switch is involved. If available,
   compare a direct USB-C-to-DisplayPort connection to the monitor. These tests
   help isolate the HDMI path; DisplayPort is not a guaranteed fix. The ASUS
   supports DisplayPort 1.2 and HDMI 1.4.

BetterDisplay's CLI equivalent of the first experiment is:

```sh
betterdisplaycli perform -productName="ROG PG279Q" -reinitialize
```

That requires BetterDisplay to be installed and running, with its CLI helper
available. The Sleep Monitors app has no BetterDisplay dependency.

## Can macOS use custom signal timings?

Yes, on supported Apple-silicon connections, a prepared custom EDID can change
the timings advertised to macOS. BetterDisplay Pro can apply that as a software
override. Its importer does not edit individual timing fields; the binary must
first be prepared in an EDID editor using timings the monitor supports. The
Mac's display driver can still reject a timing.

This is different from adding a scaled or HiDPI desktop resolution. Scaling
alone does not necessarily change the physical signal timing.

There is no established porch, sync-width, or pixel-clock adjustment to
recommend for this fault. First compare the existing supported modes and
connection formats above. If investigating an EDID override later, export the
original EDID and use a reversible software override. Writing EDID into the
monitor's hardware is unnecessary for that experiment.

The historical ASUS PG279Q firmware service program is not evidence of a fix
for this problem: ASUS described an NVIDIA-specific vertical-line issue, with
updates requiring service-center equipment.

## Sources

- [ASUS PG279 series manual](https://dlcdnets.asus.com/pub/ASUS/LCD%20Monitors/PG279/PG279_English.pdf), pages 3-6 and 3-8: ports, panel capabilities, and supported HDMI modes.
- [BetterDisplay: Management and Quick Toggles](https://betterdisplay.pro/guide/interface-reference/app-menu/device-menu/management-and-toggles/): reinitialization and licensing.
- [BetterDisplay: Reinitialize automatically after sleep](https://betterdisplay.pro/guide/faq/connections/reinitialize-on-wake/): wake recovery and its limits.
- [BetterDisplay: Color Mode](https://betterdisplay.pro/guide/interface-reference/app-menu/device-menu/color-mode/): encoding, bit depth, range, SDR/HDR, and available formats.
- [BetterDisplay: CLI reference](https://betterdisplay.pro/guide/integration/cli-reference/): reinitialization and device selection.
- [BetterDisplay: Custom display timings through EDID](https://betterdisplay.pro/guide/faq/scaling/import-custom-edid/): supported software overrides and timing-editor requirements.
- [Apple: External display dark or low resolution](https://support.apple.com/en-us/102501): cables, adapters, direct connections, and display detection.
- [ASUS: PG279Q firmware update program](https://rog-forum.asus.com/t5/rog-series-monitors-pg-xg/firmware-update-for-rog-swift-pg279q/td-p/682048): the distinct NVIDIA vertical-line issue.
