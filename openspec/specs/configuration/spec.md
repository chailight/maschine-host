# Declarative configuration

## Purpose

Validates groups, pad maps, destinations, channels, and encoder mappings before a YAML configuration becomes active, and reports every problem instead of activating a partial route. Controller input comes from the full MK2 HID interface when it is present. Panel and LED writes are outside the behavior this capability fulfills.

## Requirements

### Requirement: Declarative configuration primary behavior
The system SHALL validate groups, pad maps, destinations, channels, and encoder mappings before a loaded YAML configuration becomes active. An encoder MIDI CC mapping MAY omit its channel. An omitted channel SHALL be the channel of the group at the same index, with the first encoder matching Group A and the eighth matching Group H. An explicit channel SHALL be used instead of that default. Pad note numbers, group channels, encoder controller numbers, and encoder channels SHALL remain fields in the configuration.

#### Scenario: Primary operation
- **WHEN** a YAML configuration is loaded
- **THEN** the system validates groups, pad maps, destinations, channels, and encoder mappings before activation

#### Scenario: Omitted encoder channel
- **WHEN** an encoder send omits its channel and the group at the same index has a MIDI channel
- **THEN** the encoder sends on that group's channel

#### Scenario: Explicit encoder channel
- **WHEN** an encoder send sets a channel
- **THEN** the encoder sends on that channel

### Requirement: Safety and secondary behavior
The system SHALL report actionable validation errors for an invalid configuration and SHALL avoid partially activating unsafe routes.

#### Scenario: Secondary operation
- **WHEN** configuration is invalid
- **THEN** the system reports actionable validation errors and does not partially activate unsafe routes

### Requirement: Per-group pad mode fields
The configuration SHALL accept an optional velocity curve on each group and an optional root and scale name on a scale-mode group. An omitted curve SHALL mean 1. A curve that is not a positive number SHALL be rejected. A scale name outside `major`, `natural-minor`, `major-pentatonic`, `minor-pentatonic`, and `chromatic` SHALL be rejected. A root outside 0 through 127 SHALL be rejected. A scale set on a one-shot or chromatic group SHALL be rejected. An invalid field SHALL be reported and the configuration SHALL NOT become active. The Bitbox file SHALL set velocity curve 0.5 on every group and SHALL omit the scale, leaving notes 36 through 51.

#### Scenario: Omitted curve is straight
- **WHEN** a group omits its velocity curve
- **THEN** the loaded curve is 1

#### Scenario: Invalid curve
- **WHEN** a group sets a velocity curve of 0 or below
- **THEN** validation fails and the configuration does not become active

#### Scenario: Scale on a drum group
- **WHEN** a one-shot group sets a root and scale name
- **THEN** validation fails and the configuration does not become active

#### Scenario: Bitbox curve
- **WHEN** the Bitbox configuration is loaded
- **THEN** every group uses velocity curve 0.5, omits a named scale, and keeps notes 36 through 51
