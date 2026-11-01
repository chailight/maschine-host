## 1. Bitbox pages

- [x] 1.1 Replace the encoder pages in `docs/example-config.yaml` with the ordered list in the encoder-pages delta. Leave the MIDI port, groups, notes, and velocity curves unchanged.
- [x] 1.2 On each per-pad page, set encoders 1 through 8 to that page's controller on channels 1 through 8. On each master page, set the named controllers on channel 1 and leave the remaining encoders with no controller.
- [x] 1.3 Set the starting values from the spec: 0 unless the spec names 64, 6, 90, 7, 38, 37, or 64. Use the short labels from the design.

## 2. Tests

- [x] 2.1 Update the published Bitbox test for the new page order, controller 84 on `delay-send` channel 1, controller 85 for slot 2 of `reverb-send` on channel 2, controller 15 on `character` channel 1, compressor threshold 90, compressor switch 0, and no assignment of controllers 120 or 123.
- [x] 2.2 Keep the panel proof test that its encoders assign no controller.
- [x] 2.3 Update the Bitbox encoder send test so the delay-send page sends controller 84 and the character page sends controller 15 on channel 1.
