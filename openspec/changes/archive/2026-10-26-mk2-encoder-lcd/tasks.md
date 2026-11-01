## 1. Encoder page pixels

- [x] 1.1 Draw the left panel as the page name plus encoders 1–4, and the right panel as encoders 5–8. Each row uses the configured label and the value stored for that encoder on the active page.
- [x] 1.2 Test that turning an encoder changes only the active page, selecting the other page shows that page's stored values, and a disconnect clears both panels.

## 2. Page buttons

- [x] 2.1 A press of the Left button beside the displays selects the previous encoder page and wraps. A press of the Right button selects the next page and wraps. Release does not change the page. Each lamp stays on only while that button is held.
- [x] 2.2 Test the wrap and that the page which is no longer shown keeps its stored value.

## 3. Example pages

- [x] 3.1 Keep the Bitbox config's delay, reverb, and filter pages, with no encoder action assigned. Use two pages labeled `enc 1` through `enc 8` only for the panel proof and the long-term example.

## 4. Panel map gate

- [x] 4.1 Keep a panel map unmatched until a check accepts it. An unmatched map sends no LCD report and still sends a matched lamp report. A matched map adds the two panel reports, and a lamp report contains no panel bytes.
- [x] 4.2 Test that gate. Capture and listen still send no output or feature report.

## 5. One-panel check

- [x] 5.1 Add a macOS command that paints the left panel only, using Encdr 0.7.8's left-screen report as the hypothesis. Do not send a feature report. Do not quit `NIHardwareAgent`.
- [x] 5.2 Run that check on the attached MK2. Store the map only if the commanded left panel matches and the right panel and the lamps stay unchanged. Keep Encdr's copyright notice with the adapted map. Do not add the Encdr crate.

## 6. Live writer

- [x] 6.1 After the panel map matches, send both panel reports from the semantic frame on the existing display interval.
- [x] 6.2 On disconnect, while the device is still open, send blank panels. A failed write is logged, feedback is marked sent, and note routing continues.
- [x] 6.3 Test a failed panel write with a mock adapter and show that a following note still routes.

## 7. Spec purpose and notes

- [x] 7.1 Update the `display-feedback` Purpose so a matched panel map writes the active encoder page and an unmatched map leaves the panels unwritten.
- [x] 7.2 Document the check, that generic `enc 1` through `enc 8` labels are the proof and the long-term example, and that the Bitbox config keeps delay, reverb, and filter until real encoder actions are assigned.
