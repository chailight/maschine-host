## Why

The host already keeps an encoder value for each loaded page, and it already draws that page into two in-memory panels. Those panels are never sent to the MK2, and the Left and Right buttons beside the displays do not change the page. The operator cannot see or step through encoder values on the hardware.

## What Changes

- Confirm one LCD panel on the attached full-size MK2 before any panel map is stored. A mismatch stores nothing. Leave `NIHardwareAgent` running. Do not link Encdr.
- After that check matches, write both 256×64 panels from the active encoder page. The left panel shows encoders 1–4 and the right panel shows encoders 5–8, each in the 64-pixel column above that knob. The page name is one line across the top. In each column the configured label sits above the stored value, and the value is on the bottom line.
- Keep one stored value per encoder per page. Turning a knob changes only the active page. Page Left and Page Right are the Left and Right buttons beside the displays. A press moves one page and wraps. The value on the page that is no longer shown stays stored.
- The Bitbox config keeps the delay, reverb, and filter pages. This change does not assign their real encoder actions. Generic labels (`enc 1` through `enc 8`) are only for the panel proof, the tests, and the long-term example. They do not replace those three pages.
- Send panel output reports only after the check matches, on the existing display interval. A failed write is recoverable and does not block note routing. Disconnect clears both panels while the device is still open. Feature reports stay unused. Group, pad, and button lamps keep their current behavior. Lamp reports do not carry panel bytes.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `display-feedback`: The matched panel map writes the active encoder page to both MK2 LCDs. Each page keeps its own encoder values. Until the check matches, the panels stay unwritten.
- `controller-input`: The Left and Right buttons beside the displays select the previous and next encoder page. A matched panel map may send LCD output reports. Feature reports stay forbidden.

## Impact

- The live HID writer gains panel reports beside the existing lamp reports. The semantic frame already has two 256×64 buffers and per-page values; this change sends the encoder page and binds the page buttons.
- The Bitbox config keeps its delay, reverb, and filter pages. The panel proof and the long-term example use two pages of generic `enc 1` through `enc 8` labels and assign no encoder action. A config that lists other pages still shows those pages.
- `NIHardwareAgent` may repaint the panels during the check.
- Linux hidraw, Raspberry Pi checks, and real encoder action mappings are outside this change.
- The `display-feedback` Purpose line is updated when this change is applied so it no longer says the LCD panels stay unwritten.
