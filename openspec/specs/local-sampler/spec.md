# Local sample playback

## Purpose

Plays assigned 16-bit WAV samples for one-shot pads and pitch-shifts a selected zone for chromatic notes. Controller input comes from the full MK2 HID interface when it is present. Panel and LED writes are outside the behavior this capability fulfills.

## Requirements

### Requirement: Local sample playback primary behavior
The system SHALL play an assigned local WAV sample for a one-shot pad independently of an external MIDI instrument.

#### Scenario: Primary operation
- **WHEN** a one-shot pad is triggered
- **THEN** the system plays its assigned local WAV sample independently of an external MIDI instrument

### Requirement: Safety and secondary behavior
The system SHALL select a configured sample zone for a chromatic note and play that note at the requested pitch, subject to declared supported formats and polyphony.

#### Scenario: Secondary operation
- **WHEN** a chromatic note is triggered
- **THEN** the system selects a configured sample zone and plays the note at the requested pitch, subject to declared supported formats and polyphony
