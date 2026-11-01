## ADDED Requirements

### Requirement: Bitbox pad notes and channels
The Bitbox configuration SHALL give Groups A through H MIDI channels 1 through 8. Every Bitbox group SHALL assign pads 1 through 16 the note numbers 36 through 51. A configuration other than the Bitbox file MAY assign different notes or channels. This change SHALL NOT add a per-group octave control.

#### Scenario: Same pad note on each group
- **WHEN** the Bitbox configuration is loaded and the same pad is pressed in two groups
- **THEN** both presses send that pad's note number, and each press uses its group's channel

#### Scenario: Notes stay editable
- **WHEN** a configuration assigns a different note to a pad in one group
- **THEN** that group sends the assigned note and the other groups keep their own assignments
