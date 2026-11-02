# Mixer Controls Specification

## ADDED Requirements

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
