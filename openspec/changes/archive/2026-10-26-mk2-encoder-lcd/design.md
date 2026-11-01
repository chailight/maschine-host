## Context

See proposal.md for why this change exists. The controller already stores one value per encoder per page and wraps `page_left` / `page_right`. The semantic frame already has two 256×64 buffers, and the live HID writer already sends matched lamp reports on the display interval. The Left and Right buttons beside the displays are momentary lamps and do not select a page. No LCD report has been confirmed on this MK2. Encdr 0.7.8 names two 256×64 mono panels for product `1140`. That naming is a hypothesis. This repository is GPL-3.0-or-later. Encdr is GPL-3.0-or-later and is not a dependency.

## Goals / Non-Goals

**Goals:**

- Prove one panel report on the attached MK2 before any panel map is stored.
- After that match, send both panels from the active encoder page without putting panel bytes in a lamp report.
- Bind the existing display Left and Right presses to the existing page shift, and keep their momentary lamps.

**Non-Goals:**

- Linking Encdr or pulling `wgpu`.
- Quitting `NIHardwareAgent`.
- Assigning real encoder actions.
- Replacing the Bitbox delay, reverb, and filter pages with generic pages.

## Decisions

1. **The panel check is a separate command, and its map is separate from the lamp map.** The running host sends panel reports only when the stored panel map says the check matched. An unmatched panel map skips those reports and still sends the matched lamp reports. The check paints the left panel only. The operator accepts it only if that panel shows the commanded picture and the right panel and the lamps stay unchanged. A mismatch stores nothing. Alternative: paint both panels on the first run. Rejected because a swapped left/right report would write the wrong panel before the map is known.

2. **The device report descriptor chooses the panel reports.** Encdr's one-shot `0xE0` blit was written and the left panel did not change, so that shape is not stored. The attached MK2 describes output report `0xE0` as four 16-bit fields and 256 bytes, and `0xE1` the same way for the right panel. A captured product `1140` frame sets those fields to x, y, a row width of 32 bytes, and a row count of 8. Sending a width of 256 reached the left panel and cut lines through it, so the check uses the 32-byte width. Only a shape the operator accepts is stored. Feature reports stay unused. Alternative: vendor `ni_maschine_mk2.json`. Rejected because the file is not limited to the two panels.

3. **The pixels sent to the device are the encoder page, not the current group summary.** The host model keeps the existing group and page text. Each 256×64 buffer is four columns of 64 pixels, one above each knob. The page name is one line across the top. In each column the configured label sits above the value stored for `(page, encoder)`, and the value is on the bottom line. Disconnect submits blank panels while the device is still open. Alternative: stack every encoder as a left-aligned row. Rejected because the text then sits over the first knob only.

4. **Page buttons reuse the current momentary controls.** A press of display Left calls the existing previous-page shift. A press of display Right calls the existing next-page shift. Release does not change the page. The lamp stays on only while the button is held. Values remain keyed by page, so the page that leaves the screen keeps its values. Alternative: replace those controls with `page_left` / `page_right` and drop the lamps. Rejected because the spec still requires the lamp while the button is held.

5. **Generic labels are the proof, not the Bitbox pages.** The panels render the loaded page name and each encoder's configured label. `docs/example-config.yaml` keeps the delay, reverb, and filter pages and still assigns no CC action. Tests and the long-term example use two pages labeled `enc 1` through `enc 8` so the panel check does not depend on those three names. Alternative: replace the Bitbox pages with those two generic pages. Rejected because delay, reverb, and filter stay the Bitbox page set.

6. **Panel writes share the lamp submit path.** Output reports only. No feature report. `submit` sends panel reports after the lamp reports when the panel map is matched. A failed write is logged, feedback is marked sent, and note routing continues. Capture and listen still send nothing. Alternative: a second USB claim for the panels. Rejected because this Mac already writes lamps on the open HID interface.

## Risks / Trade-offs

- [NIHardwareAgent repaints a panel during the check] → Stop and store nothing. Do not quit the agent.
- [The left-screen report also changes the right screen or a lamp] → The check does not match. No panel map is stored, and the host sends no LCD report.
- [Encdr's pixel packing is reversed or padded] → The operator rejects the check. The stored map is the packing that produced the commanded picture.
- [Adapted Encdr screen bytes are a derivative of GPL-3.0-or-later work] → Keep Encdr's copyright notice with the map. Do not add the crate.
- [One accepted left panel does not prove the right panel] → The same report shape is used for the right panel with its own screen id. The operator can reject the map if the right panel is wrong when the host runs.

## Migration Plan

`docs/example-config.yaml` keeps the delay, reverb, and filter pages. The generic `enc 1` through `enc 8` labels live in the proof and the long-term example, not in place of those pages. Removing the stored panel map returns panel submit to sending nothing. Stopping the process leaves the panels as they were, aside from whatever the check itself drew.

## Open Questions

- The exact Encdr report id, screen id, and pixel stride are confirmed by the check. They do not change the spec or the task breakdown.
