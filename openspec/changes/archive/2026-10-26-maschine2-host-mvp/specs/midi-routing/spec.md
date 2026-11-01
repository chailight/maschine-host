# External MIDI

## ADDED Requirements

### Requirement: External MIDI primary behavior
The system SHALL implement the behavior specified below.

#### Scenario: Primary operation
- **WHEN** a group targets an external port
- **THEN** WHEN a group targets an external port, THE SYSTEM SHALL send channel-correct note-on/note-off and velocity.

### Requirement: Safety and secondary behavior
The system SHALL implement the following additional behavior.

#### Scenario: Secondary operation
- **WHEN** the relevant condition occurs
- **THEN** WHEN a mapping targets a MIDI CC, THE SYSTEM SHALL send configured channel, controller number and value; unavailable ports SHALL produce a recoverable error.
