## 1. Pan starts at centre

- [x] 1.1 Set the eight `mix-pan` encoder defaults in `docs/example-config.yaml` to 64, and update any test that expects those pan values to start at 0.
- [x] 1.2 Make `connect` return controller 38 at each pan encoder's stored value on that encoder's channel, and return no level, filter, or pitch controller.
- [x] 1.3 Apply that list from the run loop once MIDI is open and the HID open has succeeded. A refused HID read still disconnects and sends no pan.
- [x] 1.4 Cover the mixer scenarios: connect sends 64 on channels 1–8 only, the pan page shows 64 before a turn, and one step up on pan encoder 3 sends 65 on channel 3 alone.

## 2. Pad pressure floor

- [x] 2.1 Capture one hard hit on the pad reported as pad 16 and one deliberate light hit on that same pad. Read both copies of that pad and of pads 15, 12, and 8 (host pads 15, 14, 11, and 7, unless the capture shows different indexes). Set the floor to the smallest whole count above every coupled reading and at or below the light hit. If the light hit is not above the coupling, stop and report that instead of guessing.
- [x] 2.2 Treat a copy at or below the floor as up, for both copies, before a pad counts as down. Scale velocity from the primary count only after both copies are above the floor. Leave the 36-report release wait in place.
- [x] 2.3 Record the floor and the readings it came from next to the release-wait constant and in `docs/hid-input.md`.
- [x] 2.4 Cover the input scenarios with a stand-in floor: a neighbor above zero and at or below the floor does not press, a reading exactly at the floor does not press, one copy above the floor does not press, and a dip shorter than the release wait does not start a second note.
