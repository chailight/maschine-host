## MODIFIED Requirements

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
