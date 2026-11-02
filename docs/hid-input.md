# MK2 HID input

On macOS the process opens the full MK2 HID interface, vendor `17cc`, product `1140`, usage page `FF01`. It does not read the class-compliant MIDI interface, the audio-control interface, or the DFU interface. `--capture-hid` and `--listen` do not send HID output or feature reports. Group and pad lamps are a separate check, described in `docs/led-lamps.md`.

## Capture

This logs raw input reports and emits no control events:

```sh
cargo run -- --capture-hid 5
```

The number is how many seconds to listen. A refused open prints `HID read refused:` and leaves the controller disconnected. The process does not quit `NIHardwareAgent` or Logitech Options+.

## Listen

This opens the same HID interface and prints each confirmed control. It does not send MIDI or write the panels. Stop it with Control-C.

```sh
cargo run -- --listen
```

A pad line includes velocity, for example `pad:0 press 31`. Buttons print `volume press` and `volume release`. Screen knobs print `encoder:0 +1`. The shared encoder prints `encoder +1` or `encoder -1`.

## Remaining controls

Use the same listen as the pad, group, and screen-knob session. It logs every changed input report for the given number of seconds and does not name controls while it runs. Leave a short pause between gestures. Nothing is written to the assignment, and no output or feature report is sent.

```sh
cargo run -- --capture-hid 240 > docs/hardware-spike/survey-session.log 2> docs/hardware-spike/survey-session.err
```

Leave the pads, groups, and screen knobs alone. For Volume, Swing, Tempo, and the data encoder, turn one detent clockwise, pause, then one detent counter-clockwise. Every other control is a single press and release. Shift is pressed by itself. The eight buttons above the displays go from left to right. Left and Right are named separately for the display column, the master section, and the transport section. Let the timer finish so the log is flushed.

The timed listen on 2026-10-26 is stored in `docs/hid-buttons.yaml`. The running host does not load that file. `--listen` does. Volume, Swing, and Tempo are button bits. Encoder motion is report `0x01` byte 8: one clockwise detent steps `0x0f` to `0x00` and then upward, and the step back is `0x00` to `0x0f`. Restart is byte 5 bit 0. The master encoder click is byte 3 bit 7.

## Assignment

`docs/hid-assignment.yaml` is the labeled session from 2026-10-26. It maps 16 pads, groups A–H, and the eight screen knobs. Page, panic, and octave were not pressed in that session, so those controls stay silent. The map is not taken from another project's descriptor.

Report `0x20` carries pad pressure. Each pad is a 12-bit value: the low four bits of the marker byte sit above the value byte. A value above the pressure floor scales to velocity with `value * 127 / 4095`, and any such value is at least 1. Bytes 33–64 repeat the same 16 pads. A press is emitted only when both copies are above the floor, so a one-count flicker in one copy does not become a pad event. The floor is 123, from `docs/hardware-spike/pad-floor.log`: a hard hit on the top-right pad reached 2625 and 2364, and the pads it also moved reached at most 122, 105, 102, and 90. A lighter hit on that same pad reached 1312 and 1024 and moved no neighbor. The live host keeps the note held until pressure has stayed at or below the floor for 36 reports, so a dip as the finger lifts does not start a second note.

Report `0x01` byte 4 is the group bitfield, A in bit 0 through H in bit 7. Those buttons have no pressure byte, so a press uses `stand_in_press` 100.

The same report holds the eight knobs as little-endian 10-bit counters, starting at byte 9. Clockwise increases the counter. The smallest step in the session was 4 counts, and one clockwise twist ran for several hundred counts. The parser emits `wrapped_delta / 4`, so a 4-count step is +1 and a long twist still walks the 0–127 parameter until it clamps.

A bit field with no `pressure_byte` uses `stand_in_press`. A confirmed pressure of zero produces no press.

`cargo test` and `--self-check` keep using the mock adapter. They do not open the HID interface, including when the MK2 is plugged in.
