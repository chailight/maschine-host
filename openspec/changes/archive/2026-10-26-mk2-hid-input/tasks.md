## 1. Labeled capture

- [x] 1.1 Add a macOS capture mode that opens the full MK2 HID interface, logs raw input reports, and emits no control events.
- [x] 1.2 Record a labeled session for all 16 pads, groups A–H, and the eight screen encoders. Store that assignment in this repository, separate from any upstream descriptor.
- [x] 1.3 If the open is refused, record the error. Do not quit `NIHardwareAgent` or Logitech Options+.

## 2. Input parser

- [x] 2.1 Parse confirmed capture fields into the existing control events, including press, release, and encoder delta.
- [x] 2.2 Drop unconfirmed fields. Do not read the class-compliant MIDI, audio-control, or DFU interfaces, and do not send HID output or feature reports.
- [x] 2.3 Test the parser with captured fixtures: a confirmed press and release, a zero-valued press ignored, and an unconfirmed byte producing no event.

## 3. Runtime wiring

- [x] 3.1 Make the macOS process read the full MK2 HID interface when it is present.
- [x] 3.2 On a refused read, report a recoverable error and leave the controller disconnected.
- [x] 3.3 Keep `cargo test` and `--self-check` on the mock adapter while the MK2 is plugged in.

## 4. Existing capabilities

- [x] 4.1 With live HID events, check group selection, pad routing, release after a group change, encoder pages, external MIDI, and local WAV playback against their current requirements.
- [x] 4.2 Replace each placeholder Purpose in `openspec/specs/` with one or two sentences taken from that spec's requirements. State that controller input comes from the full MK2 HID interface when it is present, and that panel and LED writes remain outside these specs' fulfilled behavior.

## 5. Operator notes

- [x] 5.1 Document the capture, the stand-in press value when no pressure field was confirmed, and that this change writes neither panels nor LEDs.

## 6. Remaining silk-screen survey

- [x] 6.1 Record the remaining silk-screen controls with the same timed `--capture-hid` listen used for the pads, groups, and screen knobs. Do not emit control events. Do not send HID output or feature reports.
- [x] 6.2 After that labeled session, store only the gestures whose bytes are distinct. Do not assign a control to page, panic, or octave unless that role was named.
