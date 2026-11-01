## ADDED Requirements

### Requirement: Bitbox send controllers
The Bitbox delay page SHALL send controller 20, the reverb page SHALL send controller 21, and the filter page SHALL send controller 22. Each of the eight encoders on those pages SHALL send on the channel of the group at the same index unless that encoder sets a channel. The panel proof SHALL NOT assign these controller numbers.

#### Scenario: Delay send
- **WHEN** the Bitbox configuration is loaded and the first encoder is turned on the delay page
- **THEN** a controller 20 message is sent on Group A's channel

#### Scenario: Slot follows its group
- **WHEN** the Bitbox configuration is loaded and the encoder for Slot 2 is turned on the reverb page
- **THEN** a controller 21 message is sent on Group B's channel

#### Scenario: Filter send
- **WHEN** the Bitbox configuration is loaded and an encoder is turned on the filter page
- **THEN** a controller 22 message is sent on that slot's group channel

#### Scenario: Explicit channel overrides the group
- **WHEN** an encoder on a Bitbox send page sets a channel
- **THEN** that encoder sends on the set channel

#### Scenario: Panel proof stays unassigned
- **WHEN** the panel proof configuration is loaded
- **THEN** its encoders assign no controller number
