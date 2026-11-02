## MODIFIED Requirements

### Requirement: Bitbox pad notes and channels
The Bitbox configuration SHALL give Groups A through H MIDI channels 1 through 8. Every Bitbox group SHALL assign pads 1 through 16 the note numbers 36 through 51. A configuration other than the Bitbox file MAY assign different notes or channels. The configuration file SHALL NOT gain an octave field. The existing per-group octave offset is the value the setup screen adjusts.

#### Scenario: Same pad note on each group
- **WHEN** the Bitbox configuration is loaded and the same pad is pressed in two groups
- **THEN** both presses send that pad's note number, and each press uses its group's channel

#### Scenario: Notes stay editable
- **WHEN** a configuration assigns a different note to a pad in one group
- **THEN** that group sends the assigned note and the other groups keep their own assignments

### Requirement: Per-group velocity curve
The system SHALL apply each group's velocity curve to the pad velocity before sending the note. An omitted curve SHALL leave the velocity unchanged. A curve of 1 SHALL leave the velocity unchanged. A curve below 1 SHALL raise a softer hit relative to that straight mapping, using the nearest integer of `127 * (velocity / 127) ^ curve`, with a halfway case rounding away from zero, then clamped to 1 through 127. A curve above 1 SHALL lower a softer hit by the same rule. Pressing the Pad Mode button SHALL NOT change the curve.

#### Scenario: Soft hit is raised
- **WHEN** a group curve is 0.5 and a pad is pressed at velocity 16
- **THEN** the note is sent at velocity 45

#### Scenario: Omitted curve
- **WHEN** a group omits its velocity curve and a pad is pressed at velocity 16
- **THEN** the note is sent at velocity 16

#### Scenario: Pad Mode does not edit velocity
- **WHEN** the Pad Mode button is pressed
- **THEN** the selected group's velocity curve stays as configured

### Requirement: Per-group note scale
A group that sets a root and a named scale SHALL send that scale across its sixteen pads, whichever group mode it uses. The first pad SHALL send the root. Each following pad SHALL send the next degree, continuing into higher octaves. The names SHALL be `major` (0, 2, 4, 5, 7, 9, 11), `dorian` (0, 2, 3, 5, 7, 9, 10), `phrygian` (0, 1, 3, 5, 7, 8, 10), `lydian` (0, 2, 4, 6, 7, 9, 11), `mixolydian` (0, 2, 4, 5, 7, 9, 10), `natural-minor` (0, 2, 3, 5, 7, 8, 10), `locrian` (0, 1, 3, 5, 6, 8, 10), `major-pentatonic` (0, 2, 4, 7, 9), `minor-pentatonic` (0, 3, 5, 7, 10), and `chromatic` (semitone steps). `ionian` SHALL mean `major` and `aeolian` SHALL mean `natural-minor`. A group that omits the scale SHALL send its note list. The existing octave offset SHALL be added after that note for every group mode. Pressing the Pad Mode button SHALL NOT change the root or the scale. The setup screen adjusts that same offset, as specified for the setup values.

#### Scenario: Natural minor from the root
- **WHEN** a scale-mode group sets root 48 and `natural-minor`, and the octave offset is zero
- **THEN** the first three pads send notes 48, 50, and 51

#### Scenario: Omitted scale keeps the note list
- **WHEN** a scale-mode group omits the scale and its note list starts at 36
- **THEN** the first pad sends note 36

#### Scenario: One-shot keeps its drum notes
- **WHEN** a one-shot group has a note list, no named scale, and a pad is pressed
- **THEN** the pad sends the note from that list

#### Scenario: Pad Mode does not edit the scale
- **WHEN** the Pad Mode button is pressed
- **THEN** the selected group's root and scale stay as configured

## ADDED Requirements

### Requirement: Pad Mode setup values
While the setup screen is engaged, each of encoders 1–5 SHALL change its value once per twenty accumulated encoder clicks. A smaller turn SHALL leave the value unchanged. Encoder 1 SHALL step the selected group's velocity curve through 0.5, 1, and 2, and SHALL stop at 0.5 and at 2. A loaded curve outside that list SHALL be shown until encoder 1 completes a step, and that first step SHALL set the curve to 0.5. Encoder 2 SHALL cycle `notes`, ionian, dorian, phrygian, lydian, mixolydian, aeolian, locrian, major pentatonic, minor pentatonic, and chromatic on every group. Choosing `notes` SHALL clear the named scale and SHALL keep the root, so later pads use the note list. Choosing a name SHALL use the group's root. Encoder 3 SHALL move that root by one semitone per step, clamped to 0 through 127, and SHALL update the named scale's root when a scale is set. Encoder 4 SHALL add one to the existing octave offset per upward step and subtract one per downward step, clamped to -4 through 4, for every group mode. Encoder 5 SHALL cycle `none`, `sample`, `granular`, `clip`, and `slicer`. `none` SHALL clear the pad type. `sample` SHALL be the type used for a sample cell, including one with multisample mode on, and SHALL be the type loaded when the configuration omits `pad_type`. The slice type SHALL be named `slicer`. These edits SHALL apply to later pad presses and SHALL remain on that group when another group is selected and this group is selected again. They SHALL NOT be written to the configuration file or to `preset.xml`, and they SHALL NOT send a message that changes the module's pad. Encoders 6–8 SHALL send nothing while the screen is engaged.

#### Scenario: Velocity step
- **WHEN** the setup screen is engaged, the selected group's curve is 0.5, and encoder 1 turns twenty clicks upward
- **THEN** the curve becomes 1 and a later pad press at velocity 16 is sent at velocity 16

#### Scenario: Velocity stops at the quiet end
- **WHEN** the selected group's curve is 0.5 and encoder 1 turns one step downward on the setup screen
- **THEN** the curve stays 0.5

#### Scenario: Velocity stops at the loud end
- **WHEN** the selected group's curve is 2 and encoder 1 turns one step upward on the setup screen
- **THEN** the curve stays 2

#### Scenario: One click does not change the curve
- **WHEN** the selected group's curve is 0.75 and encoder 1 turns one click on the setup screen
- **THEN** the curve stays 0.75

#### Scenario: Loaded curve outside the steps
- **WHEN** the selected group's curve is 0.75 and encoder 1 completes one step on the setup screen
- **THEN** the curve becomes 0.5

#### Scenario: Scale chosen from the note list
- **WHEN** a group has root 36, no named scale, and encoder 2 on the setup screen selects ionian
- **THEN** the root stays 36, the first pad sends note 36, and the second pad sends note 38

#### Scenario: One-shot octave encoder
- **WHEN** a one-shot group has root 36, scale ionian, and octave offset 0, and encoder 4 turns one step upward on the setup screen
- **THEN** the offset is +1 and the first pad sends note 48

#### Scenario: Octave step
- **WHEN** a group has root 48, scale `natural-minor`, and octave offset 0, and encoder 4 turns one step upward on the setup screen
- **THEN** the offset is +1 and the first pad sends note 60

#### Scenario: Root step
- **WHEN** a group has root 48 and scale `natural-minor`, and encoder 3 turns one step upward on the setup screen
- **THEN** the root is 49 and the first pad sends note 49

#### Scenario: Pad type stays on the group
- **WHEN** encoder 5 on the setup screen selects `sample`, another group is selected, and the first group is selected again
- **THEN** the first group's pad type is `sample`

#### Scenario: Edits stay out of the files
- **WHEN** encoder 5 on the setup screen selects `slicer`
- **THEN** the configuration file and `preset.xml` are unchanged, and no message is sent to change the module's pad
