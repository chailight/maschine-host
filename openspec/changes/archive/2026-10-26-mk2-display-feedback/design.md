## Context

See proposal.md for why this change exists. The live HID client already opens vendor `17cc`, product `1140`, usage page `FF01` and reads input reports. `LiveInput::submit` accepts the semantic frame and sends nothing. `feedback::render` already paints held pads in one RGB color and idle pads in another, and it paints the selected group bright and the others dim. Group lamps have no color yet. Feedback is submitted only when `feedback_due` says so, and note routing runs before that submit.

Encdr 0.7.8 names a 16-pad RGB output and eight RGB group lamps (`group_a` through `group_h`) on this product id. Those names are a hypothesis. The group lamps sit in a buffer that also names transport and edit buttons. This repository is GPL-3.0-or-later. Encdr is GPL-3.0-or-later and is not a dependency.

## Goals / Non-Goals

**Goals:**

- Prove one pad lamp and one group lamp on the attached MK2 before any map is stored.
- After that match, drive every group and pad lamp from the existing semantic frame, using one active color and one resting color.
- Keep LCD bytes and feature reports out of every report the host sends. Silk-screen lamps follow the button gesture once the stored map is matched.

**Non-Goals:**

- Linking Encdr or pulling `wgpu`.
- Copying Encdr's screen buffers.
- Quitting `NIHardwareAgent` as part of the implementation.

## Decisions

1. **The check is a separate command from the running host.** The normal submit path sends group and pad LED reports only when a stored map says the check matched. The check command lights one pad, then one group, and the operator says whether that lamp and no other lamp changed. A mismatch stores nothing. Alternative: light all 16 pads and all 8 groups in the first run. Rejected because a wrong order would light the panel before we know the map.

2. **Encdr's pad order and group names are the hypothesis, not the stored map.** Our pad 0 is the physical bottom-left pad. The check does not assume Encdr's `pad_1` or `group_a` is that control. Only the positions the operator matches are written down, with Encdr's copyright notice beside the adapted map. Alternative: vendor `ni_maschine_mk2.json`. Rejected because the file also describes panels and button lamps this change does not send.

3. **A report that would change a button is not a match.** Encdr's group buffer also names button lamps. The group check has to light the group and leave every other lamp in that report unchanged. If it cannot, the check does not match, nothing is stored, and the host sends no group or pad LED report. The pad buffer is treated as pads only. Alternative: zero the button bytes in the shared report. Rejected because turning a button off is a change.

4. **Each group has its own color, from the host config.** The selected group and a held pad use that color at full brightness. Other groups use a dimmer shade of their own color. Idle pads use a dimmer shade of the selected group, at one quarter of the full channel values. Off is (`0, 0, 0`). Bright and dim remain the semantic levels. A missing color uses the built-in palette for that group.

5. **Writes stay on the open HID input device and use the existing feedback throttle.** Output reports only. No feature report. On macOS the report id remains byte 0 of the output report, and the color offsets are wire indexes. The report is one byte longer than the payload. Putting the colors one byte later lit the bottom-left pad blue-white and the pad to its right red, and split Group A white and red. With the colors moved back, the bottom-left pad is solid white and Group A is white on only part of the button. Filling the three bytes that follow Group A's color made the whole button white and left every other lamp unchanged. Each group button is that same six-byte width, packed through byte 48. Bytes 49–56 are the transport lamps (Restart, Left, Right, Grid, Play, Rec, Erase, Shift) and stay 0 unless that button is on. The other silk-screen lamps are a separate one-byte report. `submit` returns a recoverable error on a failed write, the error is logged, and feedback is marked sent so note routing is not blocked or retried. On disconnect, while the device is still open, submit the off colors. If the device is already gone, log the error and continue shutdown. The capture and listen commands do not submit. Alternative: a second USB claim for output. Rejected because this Mac already opens the HID interface for input.

## Risks / Trade-offs

- [NIHardwareAgent repaints the panel during the check] → Stop and store nothing. Do not quit the agent in this change.
- [One matching pad does not prove the other fifteen] → The operator can reject the map. The host sends nothing until they accept it.
- [The group report cannot spare the button lamps] → The check does not match. No map is stored, and neither group nor pad LED reports are sent.
- [Adapted Encdr offsets are a derivative of GPL-3.0-or-later work] → Keep Encdr's copyright notice with the map. Do not add the crate.

## Migration Plan

No configuration migration. Stopping the process leaves the lamps as they were, aside from whatever the check itself lit. Removing the stored map returns submit to sending nothing.

## Open Questions

- None. The visible pair of colors can be swapped later if either color is hard to see; that does not change the requirement of one active color and one resting color.
