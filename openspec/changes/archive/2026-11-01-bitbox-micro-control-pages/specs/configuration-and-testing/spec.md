# Configuration And Testing Specification

## ADDED Requirements

### Requirement: Data-driven pages
The system SHALL load categories, subpages, stock-learn assignments, and each group's pad type from configuration. A missing pad type or stock-learn assignment SHALL fail closed as unavailable or UNMAPPED. The Bitbox file SHALL keep Groups A–H on channels 1–8, notes 36–51, and the current velocity curves. The panel proof file SHALL stay without controller numbers.

#### Scenario: Override performance profile
- **WHEN** a granular group assigns encoder 8 to pan random, controller 83
- **THEN** the host sends controller 83 for that encoder and leaves the other granular encoders unchanged

### Requirement: Test without Bitbox hardware
Tests SHALL drive controller events and capture MIDI and the display model. They SHALL check category labels, text values, shared parameter values, unmapped stock controls, and note routing.

#### Scenario: Regression of page selection
- **WHEN** a test plays a note, changes category, then releases the pad
- **THEN** the note-off uses the original note-on destination and the selected group is otherwise unchanged

#### Scenario: Button label and on/off text
- **WHEN** a test selects REVERB and moves an on/off encoder across 64
- **THEN** the display model shows REVERB on that button's column and shows On or Off for the encoder value

### Requirement: Host preset template
The repository SHALL keep `presets/host-template/preset.xml` as the preset that carries the proposed learns for every group. `presets/TEST/preset.xml` SHALL remain the module-saved preset from which those learns were copied. A learned controller SHALL be a `modsource` with `src="midicc"`, `ccnum`, `mchan`, `slot`, and `amount`. `mchan` SHALL count from 0, so MIDI channel 1 is `mchan="0"` and channel 8 is `mchan="7"`. A pad's MIDI input SHALL be `midimode`, counting from 1, and `midimode="0"` SHALL mean no channel. Amount `1000` SHALL be full travel. A learned controller moves the parameter upward from the value saved on that pad or effect.

The eight group pads SHALL be layer 0. Row 0, columns 0–3, SHALL be channels 1–4. Row 1, columns 0–3, SHALL be channels 5–8. Every other layer-0 cell SHALL use `midimode="0"` and SHALL have no `midicc` source. Each group pad SHALL save `panpos="-1000"`, `pitch="0"`, `dualfilcutoff="0"`, and `gaindb="0"`, and SHALL include these sources:

- `gaindb` from `velocity`, slot 0, amount 400.
- `gaindb` from `midivol`, and `panpos` from `midipan`, both slot 2, amount 1000.
- `gaindb` from controller 36, slot 1, amount 1000, on that pad's `mchan`.
- `dualfilcutoff` from controller 37, `panpos` from controller 38, and `pitch` from controller 39, each slot 0, amount 1000, on that pad's `mchan`.

The delay cell SHALL save `delay="0"`, `delaymustime="0"`, `feedback="0"`, `cutoff="0"`, `dealybeatsync="1"`, `filtenable="1"`, and `delaypingpong="1"`. It SHALL learn `feedback` to controller 30, both `delay` and `delaymustime` to controller 31, and `cutoff` to controller 32, each with `mchan="0"`, slot 0, and amount 1000. It SHALL NOT learn `filtquality`. The reverb cell SHALL save `decay`, `predelay`, and `damping` at 0, and SHALL learn them to controllers 33, 35, and 34, with `mchan="0"`, slot 0, and amount 1000. Granular, sample, slicer, and clip controllers SHALL NOT appear as `midicc` sources. The template's `modsource` elements SHALL keep `mchan` and `ccnum`, and the delay and reverb cells SHALL keep their `modsource` children.

#### Scenario: Group A pad learns level on channel 1
- **WHEN** the template pad at row 0, column 0 is read
- **THEN** `midimode` is 1, `panpos` is -1000, and level is controller 36 with `mchan` 0

#### Scenario: Group H pad uses the last channel
- **WHEN** the template pad at row 1, column 3 is read
- **THEN** `midimode` is 8 and pitch is controller 39 with `mchan` 7

#### Scenario: Delay time is learned twice
- **WHEN** the template delay cell is read
- **THEN** both `delay` and `delaymustime` use controller 31, and `filtquality` has no `midicc` source
