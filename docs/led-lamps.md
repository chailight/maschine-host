# Group and pad lamps

The running host sends group, pad, and button LED reports because `docs/led-map.yaml` records a matched check. LCD panels are a separate check, described in `docs/lcd-panels.md`. No HID feature report is sent. Each group button uses two color triples so the whole button lights.

Each group has its own color in the host config. The selected group and each held pad use that group's full color. The other groups use a dimmer shade of their own color. Idle pads use a dimmer shade of the selected group. Off is black. Disconnect sends off while the HID device is still open.

Silk-screen buttons use one brightness, on or off. Scene, Pattern, Navigate, Duplicate, Select, Solo, and Mute light one at a time. Pad Mode lights while its setup screen is showing. Volume, Swing, Tempo, and Note Repeat light one at a time. The eight buttons above the displays toggle. Master Left, Master Right, and Enter light only while held, as do Control, Step, Browse, Sampling, the display Left and Right buttons, All, Auto Write, and the transport row (Restart, Left, Right, Grid, Play, Rec, Erase, Shift). The data-encoder click has no lamp byte.

## Check

This lights the bottom-left pad, waits, turns it off, then lights Group A. It does not rewrite the stored map. `NIHardwareAgent` is left running.

```sh
cargo run -- --light-check
```

Press Enter after looking at the pads, then again after looking at the groups and the buttons. The stored match is the bottom-left pad, then Group A alone, with no button lamp changing.

`--capture-hid` and `--listen` do not send output or feature reports. `--self-check` uses the mock adapter and does not open the HID interface.
