# Local sample playback

## ADDED Requirements

### Requirement: Local sample playback primary behavior
The system SHALL implement the behavior specified below.

#### Scenario: Primary operation
- **WHEN** a one-shot pad is triggered
- **THEN** WHEN a one-shot pad is triggered, THE SYSTEM SHALL play its assigned local WAV sample independently of an external MIDI instrument.

### Requirement: Safety and secondary behavior
The system SHALL implement the following additional behavior.

#### Scenario: Secondary operation
- **WHEN** the relevant condition occurs
- **THEN** WHEN a chromatic note is triggered, THE SYSTEM SHALL select a configured sample zone and play the note at the requested pitch, subject to declared supported formats and polyphony.
