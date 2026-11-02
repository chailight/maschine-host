# Mixer Controls Specification

## Purpose

Sends each of the eight slots its own delay send, reverb send, level, filter, pan, and pitch.

## Requirements

### Requirement: Eight-slot mixer mapping
MIX SHALL provide delay-send and reverb-send subpages. Encoders 1–8 SHALL send controllers 84 and 85 on channels 1–8. That routing SHALL NOT follow the playable group. The selected group SHALL stay the instrument the pads play.

#### Scenario: Delay send balancing
- **WHEN** MIX delay send is active, Group A is selected, and encoder 7 turns
- **THEN** only channel 7 receives controller 84, and a following pad press still uses Group A's channel

### Requirement: Learned stock controls are proposed
Level, filter, pan, and pitch mixer subpages SHALL use controllers 36, 37, 38, and 39. Each encoder SHALL send that controller on its slot channel, channels 1–8. The Bitbox learns those numbers. In the preset template, pan SHALL be saved full left, because the learn travels upward from the saved value and controller 64 is then the centre.

#### Scenario: Level proposal
- **WHEN** MIX level is active and encoder 7 turns
- **THEN** the host sends controller 36 on channel 7 and on no other channel

### Requirement: Pan starts at centre
Each of the eight pan encoders SHALL start at 64. When the controller connects, the host SHALL send controller 38 with value 64 on channels 1 through 8. That connect SHALL NOT send level, filter, or pitch controllers. The preset template SHALL keep pan saved full left, so 64 remains the centre of the learned travel. A later turn of one pan encoder SHALL send controller 38 for that encoder's channel only, starting from 64.

#### Scenario: Connect centres every channel
- **WHEN** the controller connects and no pan encoder has been turned
- **THEN** channels 1 through 8 each receive controller 38 with value 64, and channels 1 through 8 receive no level, filter, or pitch controller from that connect

#### Scenario: The pan page shows centre before a turn
- **WHEN** the pan page is shown and no pan encoder has been turned
- **THEN** each of the eight pan values is 64

#### Scenario: One step leaves centre on that channel
- **WHEN** pan encoder 3 turns one step upward from its starting value
- **THEN** channel 3 receives controller 38 with value 65 and no other channel receives a pan controller
