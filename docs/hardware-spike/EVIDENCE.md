# Hardware spike evidence

The 2026-10-25 notes below were taken with no MK2 attached. On 2026-10-26 the same Mac enumerated a live full-size MK2. That recheck identified the device and its USB interfaces. It did not read a pad, move an encoder, or write an LED or LCD report.

## Recheck 2026-10-26

`system_profiler SPUSBDataType` and `ioreg` showed:

| Field | Value |
| --- | --- |
| Name | Maschine Controller MK2 |
| Manufacturer | Native Instruments |
| Vendor / product | `0x17cc` / `0x1140` |
| Serial | `4AF076D5` |
| Device release | `bcdDevice` `0x001a` (system profiler version `0.1a`) |
| USB version | `bcdUSB` `0x0200` |
| Link | High speed, up to 480 Mb/s. Device speed property `2`. Current required 480 mA, available 500 mA. |
| Location | `0x01122120`, USB address 11 |
| Device class | `0xEF` / `0x02` / `0x01` (composite device with an interface association) |

The path under the AppleT8132 USB 3.1 bus is four USB 2.0 hubs deep:

1. SATECHI USB 2.0 companion hub, `310e:2924`, location `0x01100000`
2. Terminus USB 2.0 hub, `1a40:0801`, location `0x01120000`
3. VIA Labs USB 2.0 companion hub, `2109:2813`, location `0x01122000`
4. MOAI USB 2.0 hub, `14cd:8601`, location `0x01122100`
5. Maschine Controller MK2, location `0x01122120`

The Mac mini has no USB-A port, so the controller enters through an external hub. It links at high speed, and a USB 3 hub publishes its USB 2.0 companion as a separate hub node. System Information therefore shows a deeper tree than the number of physical boxes.

Active configuration 1 has four interfaces. Endpoint addresses were not printed by the registry, so only the interface class and endpoint count are recorded.

| Interface | Class | Endpoints | Exclusive owner |
| --- | --- | --- | --- |
| 0, "Maschine Controller MK2 Audio Ctrl" | audio control `0x01` / `0x01` / `0x20`, alternate 0 | 0 | none recorded |
| 1, "Maschine Controller MK2 MIDI" | MIDI streaming `0x01` / `0x03` / `0x00` | 2 | `MIDIServer` pid 960 |
| 2, "Maschine Controller MK2 HID" | HID `0x03` / `0x00` / `0x00` | 2 | `AppleUserUSBHostHIDDevice` |
| 3, "Maschine Controller MK2 DFU" | application-specific `0xFE` / `0x01` / `0x01` | 0 | none recorded |

CoreAudio does not list a Maschine device. The default system output was Mac mini Speakers, and the default output device was Fireface 800 (B60). This configuration's audio-control interface has no endpoints, and no other alternate setting was listed.

The HID device reports vendor usage page `0xFF01`, usage `0`, max input report 65 bytes, max output report 265 bytes, and max feature report 33 bytes. The report descriptor was left in the registry and was not copied or decoded. No report was sent. Those sizes do not identify an LCD or LED layout.

Two `IOHIDLibUserClient` objects are open on that HID device: `NIHardwareAgent` pid 2932, and `logioptionsplus_agent` pid 6134. `NIHostIntegrationAgent` and `NTKDaemon` were running and were not interface clients. Device-level USB user clients, separate from the HID and MIDI interfaces, belonged to UA Connect, Auracle X, MIDIServer, and Google Chrome.

CoreMIDI, through `com.apple.AppleMIDIUSBDriver`, has one online device named "Maschine Controller MK2" (unique id `2013998621`) with an online source and destination of that same name. A second device of the same name (unique id `690836201`) is offline. Online virtual ports named "Maschine MK2 Virtual Input" and "Maschine MK2 Virtual Output" have no driver owner. Maschine.app was not running. Nothing was sent to or read from any of these ports. The class-compliant MIDI interface is not pad input.

No Raspberry Pi was present. Pads, encoders, LEDs, and both panels remain untested. The host still has no USB writer.

## This machine

- Darwin 24.6.0, macOS 15.7.7, arm64.
- `system_profiler SPUSBDataType` listed hubs, storage, a webcam, and an iConnectivity iConnectMIDI4+. Vendor `0x17cc` did not appear.
- These Native Instruments processes were running: `NIHardwareAgent`, `NIHostIntegrationAgent`, `NTKDaemon`.
- Output devices reported by `system_profiler SPAudioDataType`: Mac mini Speakers (default system output, 48000 Hz, built-in), VG2719-2K over HDMI (48000 Hz), iConnectMIDI4+ (44100 Hz, 4 in / 2 out), Binaural Renderer Link (virtual, 48000 Hz). No Pisound.
- A cpal/CoreMIDI probe inside the agent sandbox returned a CoreAudio error and "MIDI support could not be initialized". That is an environment failure, not a measurement of those devices. `system_profiler` remains the device list.

## Upstream inspection

Sources were read from GitHub on 2026-10-25. No source was copied into this repository.

| Project | License observed | Last activity observed | What it actually contains |
| --- | --- | --- | --- |
| [wrl/maschine.rs](https://github.com/wrl/maschine.rs) | LGPL-3.0 | Commit 2020-06-20 | `src/devices/mk2/mikro.rs` only. README: Maschine Mikro MK2, Linux hidraw, not a macOS target. |
| [SnovaxZ/MaschineMK2_linux](https://github.com/SnovaxZ/MaschineMK2_linux) | GPL-3.0 | Push 2026-04-15 | Fork of maschine.rs. README: pads and lights work, encoders are "not quite 100% functional", and screens are both a feature and a future todo. |
| [Witzman/MaschineMK2_linux](https://github.com/Witzman/MaschineMK2_linux) | GPL-3.0 | Push 2026-08-11 | Fork of SnovaxZ. Device implementation is still `src/devices/mk2/mikro.rs`. |
| [Witzman/Generative-Techno-ZynthianMaschine-MKII](https://github.com/Witzman/Generative-Techno-ZynthianMaschine-MKII) | GPL-3.0 | Push 2026-10-10 | Zynthian instrument. Daemon credits claim MK2 report maps, a 255x64 display protocol, and an OSC drawing API. Its vendored `daemon/README.upstream.md` still says 128x64, udev product `1200`, and LGPL, while the GitHub license API and the LICENSE file are GPL-3.0. The project README says the unauthenticated port-9001 editor was removed in September 2026 and that OSC for lights and screens is loopback-only. The drawing paths are not specified in that upstream README. |
| [RufusIbiza/Encdr](https://github.com/RufusIbiza/Encdr) 0.7.8 | GPL-3.0-or-later | crates.io 2026-10-09 | Descriptor `encdr/descriptors/ni_maschine_mk2.json` names "NI Maschine Mk2", vendor `0x17cc`, product `0x1140`. `ni_maschine_mikro_mk2.json` uses product `0x1200`. `wgpu` is a required dependency, not a feature. |

### Full MK2 descriptor facts

From `ni_maschine_mk2.json`, not from a device:

- Two screens, `left` and `right`, each `width` 256, `height` 64, `pixel_format` `mono`.
- Three LED groups: 16 pads; 31 single buttons; 16 further LEDs that include `group_a` through `group_h`.
- A 24-byte input named `buttons` with 56 named items, including eight `screen_encoder_*` entries.
- A 33-byte input named `pads`. Its comment says pad data is a 16-channel 12-bit stream. This host does not convert those counts to velocity.
- `quirks.detach_kernel_driver` is true.
- The descriptor lists two interfaces. Encdr's hardware page overview says one combined interface. That disagreement was not resolved on a device.
- The crates.io summary says 47 buttons and 11 encoders. The descriptor's button packet has eight screen encoders, and `volume`, `tempo`, and `swing` appear as button names. This host accepts eight normalized encoder events only.

An Encdr changelog line calls product `0x1200` a Maschine Mk2. The descriptor files inspected the same day give `0x1140` to the full Mk2 and `0x1200` to the Mikro Mk2. The descriptor files are the identity used here.

Encdr's own docs say macOS and Windows may need Native Instruments background services stopped before a library can claim the USB interfaces, and that Linux hidraw needs a udev rule. Their published udev example uses mode `0666`. This repository does not.

## What was not done

- No pad, button, or encoder was read on Raspberry Pi ARM64.
- No RGB pad, Group LED, or LCD was updated on Raspberry Pi ARM64 or on this Mac.
- No USB report was captured.
- No round-trip audio latency was measured.
- Bitbox CC numbers were not taken from firmware documentation. The sample profile leaves them null.

## OpenSpec check

`openspec validate maschine2-host-mvp --type change --strict` passed. The installed spec-driven template also asks for a proposal Capabilities section, a spec Purpose, and design Decisions. The existing artifacts do not follow that outline, and the scenarios repeat the WHEN text inside THEN. The CLI accepted them, so the artifacts were left in place. The code follows the behavior in those scenarios and in `design.md`.
