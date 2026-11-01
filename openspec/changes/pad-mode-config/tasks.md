## 1. Configuration fields

- [x] 1.1 Accept an optional positive velocity curve on a group. An omitted curve is 1. Reject a curve that is not positive.
- [x] 1.2 Accept an optional root and scale name on a scale-mode group. Reject an unknown name, a root outside 0 through 127, and a scale set on a one-shot or chromatic group.
- [x] 1.3 Test that an omitted curve loads as 1, and that an invalid curve or scale does not activate the configuration.

## 2. Pad messages

- [x] 2.1 Apply the group curve to the pad velocity before the MIDI note is sent. A curve of 1 leaves the velocity unchanged.
- [x] 2.2 When a scale-mode group sets a root and a named scale, send that scale from the first pad, then add the existing scale-mode octave offset. Otherwise keep the group's note list.
- [x] 2.3 Test that velocity 16 with curve 0.5 is sent as 45, that root 48 and natural-minor send 48, 50, and 51 on the first three pads, and that pressing Pad Mode does not change either setting.

## 3. Bitbox file

- [x] 3.1 Set velocity curve 0.5 on every group in `docs/example-config.yaml`. Leave the named scale unset and keep notes 36 through 51.
- [x] 3.2 Test that the published Bitbox file loads with curve 0.5, no named scale, and notes 36 through 51.
