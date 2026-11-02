# Implementation tasks

## 0. Evidence and compatibility
- [x] Record patch version, documentation links, stock Bitbox MIDI version and supported CC/action matrix.
- [x] Verify each candidate CC, MIDI channel/scope, 7-bit ranges, momentary values and parameter availability; mark unsupported mappings.
- [x] Confirm existing host group selection and button/encoder/display event APIs.
- [x] Add delta specs for `encoder-pages`, `controller-input`, and `display-feedback` before changing page wrap, category buttons, or the LCD value line. Do not start section 1 until those deltas are approved.

## 1. Core model
- [x] Define parameter registry and schema validation (including fixed CC vs stock MIDI Learn).
- [x] Implement group-to-slot context, global vs selected-slot vs all-slot address resolution.
- [x] Implement commanded/unknown state, enum/toggle/momentary handling and safe note-off ownership.
- [x] Add deterministic mock MIDI sink and test fixtures.

## 2. Navigation and feedback
- [x] Bind eight above-display buttons to categories, with independent subpage navigation.
- [x] Implement declarative LCD model and active group/page LEDs.
- [x] Implement display refresh, unavailable control labels, scope indicators and stale-value indication.

## 3. Live and mixer pages
- [x] Implement PERFORM E1-E4 invariant mapping and per-pad-type E5-E8 profiles.
- [x] Implement MIX delay/reverb send subpages, then optional learned stock mixer subpages.
- [x] Test that category/subpage changes never alter playable group or pad layout.

## 4. Effects and pad pages
- [x] Implement DELAY and REVERB pages with verified global/stock mappings.
- [x] Implement CHAR, DYNAM and EQ mode-aware labels and safe discrete controls.
- [x] Implement PAD contextual pages for sample/granular/clip/slicer.
- [x] Add optional explicit Clip FX momentary pad layer with all release paths.

## 5. Verification and delivery
- [x] Run event-driven tests for group changes during notes, channel routing, duplicated parameter bindings, and unknown state.
- [x] Validate a real Bitbox preset's MIDI Learn setup and record its configuration. `presets/TEST/preset.xml` is the module-saved learn. `presets/host-template/preset.xml` extends it to Groups A–H. The configuration is recorded in the configuration-and-testing spec. The expanded file has not been loaded on the module.
- [ ] Perform live hardware tests: delay-send/feedback gesture; all-eight sends; dynamic compressor/EQ labels; Clip release on exit.
- [ ] Cold-start PERFORM lamp stays dark on the first host launch after the MK2 is powered on. A later launch without power-cycling lights it. See `docs/troubleshooting.md`. Not blocking.
- [x] Document setup, MIDI Learn requirements, mapping overrides and unsupported functions.
