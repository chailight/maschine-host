## ADDED Requirements

### Requirement: Per-group velocity curve
The system SHALL apply each group's velocity curve to the pad velocity before sending the note. An omitted curve SHALL leave the velocity unchanged. A curve of 1 SHALL leave the velocity unchanged. A curve below 1 SHALL raise a softer hit relative to that straight mapping, using the nearest integer of `127 * (velocity / 127) ^ curve`, with a halfway case rounding away from zero, then clamped to 1 through 127. A curve above 1 SHALL lower a softer hit by the same rule. The Pad Mode button SHALL NOT change the curve.

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
A scale-mode group that sets a root and a named scale SHALL send that scale across its sixteen pads. The first pad SHALL send the root. Each following pad SHALL send the next degree, continuing into higher octaves. The names SHALL be `major` (0, 2, 4, 5, 7, 9, 11), `natural-minor` (0, 2, 3, 5, 7, 8, 10), `major-pentatonic` (0, 2, 4, 7, 9), `minor-pentatonic` (0, 3, 5, 7, 10), and `chromatic` (semitone steps). A scale-mode group that omits the scale SHALL send its note list. A one-shot or chromatic group SHALL send its note list. An existing octave offset for scale mode SHALL still be added after the scale degree. This requirement SHALL NOT add a control for that offset. The Pad Mode button SHALL NOT change the scale.

#### Scenario: Natural minor from the root
- **WHEN** a scale-mode group sets root 48 and `natural-minor`, and the octave offset is zero
- **THEN** the first three pads send notes 48, 50, and 51

#### Scenario: Omitted scale keeps the note list
- **WHEN** a scale-mode group omits the scale and its note list starts at 36
- **THEN** the first pad sends note 36

#### Scenario: One-shot keeps its drum notes
- **WHEN** a one-shot group has a note list and a pad is pressed
- **THEN** the pad sends the note from that list

#### Scenario: Pad Mode does not edit the scale
- **WHEN** the Pad Mode button is pressed
- **THEN** the selected group's root and scale stay as configured
