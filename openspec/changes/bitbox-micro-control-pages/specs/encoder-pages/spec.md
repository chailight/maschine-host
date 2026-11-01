## REMOVED Requirements

### Requirement: Bitbox Micro patch pages
**Reason**: The flat 35-page list is replaced by eight categories. The controller numbers stay.
**Migration**: Choose a category with the buttons above the displays. Left and Right move inside that category.

## ADDED Requirements

### Requirement: Bitbox control categories
The Bitbox configuration SHALL group its encoder pages into PERFORM, MIX, DELAY, REVERB, CHAR, DYNAM, EQ, and PAD, in that order. PERFORM encoders 1–4 SHALL be delay send 84 and reverb send 85 on the selected group's channel, then delay feedback 30 and delay time 31 on channel 1. MIX SHALL send delay send 84 and reverb send 85 on channels 1–8, and level, filter, pan, and pitch as controllers 36–39 on those channels. DELAY SHALL send 108, 109, 110, 111, 30, 31, and 32 on channel 1. Global learned controllers SHALL use channel 1 and SHALL NOT reuse a patch-fixed controller on that channel. The panel proof SHALL assign no controller number.

#### Scenario: Delay send from PERFORM
- **WHEN** the Bitbox configuration is loaded and the first PERFORM encoder is turned
- **THEN** controller 84 is sent on channel 1

#### Scenario: Panel proof stays unassigned
- **WHEN** the panel proof configuration is loaded
- **THEN** its encoders assign no controller number
