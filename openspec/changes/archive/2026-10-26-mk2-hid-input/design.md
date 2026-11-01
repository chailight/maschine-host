## Context

See proposal.md for why this change exists. The core already accepts normalized control events from `HardwareAdapter` and routes them through groups, pads, encoder pages, MIDI, and the sampler. The shipped adapters are a mock and an unavailable adapter. The 2026-10-26 spike in `docs/hardware-spike/EVIDENCE.md` identified the full MK2 HID interface and did not decode its report descriptor.

## Goals / Non-Goals

**Goals:**

- Read the full MK2 HID interface on macOS and emit the existing control events for controls a labeled capture has confirmed.
- Keep automated tests on the mock adapter, including on a machine where the MK2 is plugged in.
- Leave the controller, MIDI, and sampler contracts unchanged, and exercise them with events from the new reader.

**Non-Goals:**

- HID output or feature reports, LCD pixels, and LED levels on the device.
- Linux hidraw, a Raspberry Pi session, DFU, and the class-compliant MIDI port as pad input.
- Assigning the remaining silk-screen controls to page, panic, octave, or any new control ID before their bytes are confirmed.
- Copying an upstream report map or linking a GPL HID library.

## Decisions

1. **Read through the platform HID API, and only on macOS.** The interface is already claimed by Apple's HID driver. A user-space HID client matches vendor `0x17cc`, product `0x1140`, and usage page `0xFF01`. A direct USB claim would fight that driver and the device-level clients recorded in the spike. Encdr remains unsuitable because it is GPL-3.0-or-later and requires `wgpu`. Linux hidraw waits for a later change.
2. **A checked-in capture, not a guessed descriptor, is the control map.** The implementation records raw input reports while the operator names each pad, group, and screen encoder. The parser emits an event only where that capture assigns a field to an existing control ID. Bytes without an assignment produce no event. Page, panic, and octave stay silent until a capture names that role.
3. **Press and release ship before an ADC curve.** Where the capture identifies a pressure field, the adapter scales it into the existing 1–127 value and still ignores a zero-valued press. Where it identifies only press and release, the press value is 100 and the documentation says that number is not a measured velocity.
4. **The binary opens HID; tests do not.** `cargo test` and `--self-check` keep constructing the mock adapter, so a connected MK2 cannot change CI. The normal macOS process opens the HID interface when it is present. If the open fails, it logs a recoverable error, names the failure, and does not mark the controller connected.
5. **Feedback submit stays a no-op for this adapter.** Semantic display state still updates inside the process. The adapter does not write those buffers to the device.
6. **The remaining buttons use the same timed capture as the pads, groups, and screen knobs.** `--capture-hid` opens the same interface and logs a report only when its bytes change. The operator performs the named controls in order, with a pause between them. The names are applied afterwards from that order. Volume, Swing, Tempo, and the data encoder are each one detent clockwise, then one detent counter-clockwise. Every other named control is a single press and release. The capture sends no output or feature reports and does not add assignment fields by itself.

## Risks / Trade-offs

- [NIHardwareAgent or Logitech Options+ may refuse a second reader] → The process reports the failure and stays disconnected. It does not quit those processes.
- [A labeled capture can miss a silk-screen button] → That control produces no event until a later capture adds it. Group, pad, and encoder behavior still runs for the controls that were named.
- [Fixed press value 100 is not the pad's pressure] → Acceptable until a capture names the pressure field. The value is documented as a stand-in.
- [Opening HID on the development Mac makes manual runs depend on who else holds the interface] → Mock tests stay independent of that contention.

## Migration Plan

No configuration schema change. Existing YAML keeps working. Removing the HID adapter returns the process to the unavailable adapter. Rollback does not affect saved projects or MIDI routes.

## Open Questions

- Which of the surveyed silk-screen controls, if any, should become page, panic, or octave. They stay unemitted until a later assignment says so.
