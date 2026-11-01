# External MIDI

## Purpose

Sends channel-correct note and CC messages to the configured MIDI port, and recovers when that port is unavailable. Controller input comes from the full MK2 HID interface when it is present. Panel and LED writes are outside the behavior this capability fulfills.

## Requirements

### Requirement: External MIDI primary behavior
The system SHALL send channel-correct note-on, note-off, and velocity when a group targets an external port.

#### Scenario: Primary operation
- **WHEN** a group targets an external port
- **THEN** the system sends channel-correct note-on, note-off, and velocity

### Requirement: Safety and secondary behavior
The system SHALL send the configured channel, controller number, and value when a mapping targets a MIDI CC. An unavailable port SHALL produce a recoverable error.

#### Scenario: CC message
- **WHEN** a mapping targets a MIDI CC
- **THEN** the system sends the configured channel, controller number, and value

#### Scenario: Unavailable port
- **WHEN** a configured port is unavailable
- **THEN** the system produces a recoverable error
