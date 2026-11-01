# Group and pad performance

## Purpose

Selects one group from A–H without changing the encoder page, and routes a pad to that group's note or sample. Releasing a held pad after a group change sends note-off to the original destination. Controller input comes from the full MK2 HID interface when it is present. Panel and LED writes are outside the behavior this capability fulfills.

## Requirements

### Requirement: Group and pad performance primary behavior
The system SHALL select a Group A-H when its button is pressed and SHALL leave the encoder page unchanged.

#### Scenario: Primary operation
- **WHEN** a Group A-H button is pressed
- **THEN** the system selects that group without changing the encoder page

### Requirement: Safety and secondary behavior
The system SHALL route a pressed pad's configured note or sample with velocity to the selected group destination. When a held pad is released after a group change, the system SHALL send note-off to the original destination and note.

#### Scenario: Pad press
- **WHEN** a pad is pressed
- **THEN** the system routes its configured note or sample with velocity to the selected group destination

#### Scenario: Release after a group change
- **WHEN** a held pad is released after switching groups
- **THEN** the system sends note-off to the original destination and note

### Requirement: Bitbox pad notes and channels
The Bitbox configuration SHALL give Groups A through H MIDI channels 1 through 8. Every Bitbox group SHALL assign pads 1 through 16 the note numbers 36 through 51. A configuration other than the Bitbox file MAY assign different notes or channels. This change SHALL NOT add a per-group octave control.

#### Scenario: Same pad note on each group
- **WHEN** the Bitbox configuration is loaded and the same pad is pressed in two groups
- **THEN** both presses send that pad's note number, and each press uses its group's channel

#### Scenario: Notes stay editable
- **WHEN** a configuration assigns a different note to a pad in one group
- **THEN** that group sends the assigned note and the other groups keep their own assignments
