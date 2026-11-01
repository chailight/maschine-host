## 1. Semantic colors

- [x] 1.1 Give group lamps the same active and resting colors the pad lamps already use. Off stays black for both.
- [x] 1.2 Test that the selected group and a held pad use the active color, the other groups and idle pads use the resting color, and a disconnect clears both.

## 2. Matched map gate

- [x] 2.1 Represent a group and pad lamp map that is unmatched until a check accepts it. Tests and `--self-check` stay on the mock adapter and send no HID output.
- [x] 2.2 Test that an unmatched map produces no group or pad LED output report, and that a matched map produces those reports for the active and resting colors.
- [x] 2.3 Keep capture and listen from sending output or feature reports.

## 3. One-lamp check

- [x] 3.1 Add a macOS command that lights one pad, then one group, on the open HID interface. Do not write LCD bytes or feature reports. Do not quit `NIHardwareAgent`.
- [x] 3.2 Run that check on the attached MK2. Store the map only if the commanded pad and the commanded group match and no other lamp changes. Keep Encdr's copyright notice with the adapted map. Do not add the Encdr crate.

## 4. Live writer

- [x] 4.1 After the map matches, send the group and pad LED output reports from the semantic frame on the existing feedback throttle.
- [x] 4.2 On disconnect, while the device is still open, send the off colors. A failed write is logged, feedback is marked sent, and note routing continues.
- [x] 4.3 Test a failed lamp write with a mock adapter and show that a following note still routes.

## 5. Spec purpose and notes

- [x] 5.1 Update the `display-feedback` Purpose so it states that matched group and pad lamps are written and that LCD pixels stay unwritten.
- [x] 5.2 Document the check, the two colors, and that button lamps and both panels stay unchanged.
