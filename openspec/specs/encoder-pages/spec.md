# Independent encoder pages

## Purpose

Loads eight encoder mappings for the selected page and keeps each value inside its configured range. Controller input comes from the full MK2 HID interface when it is present. Panel and LED writes are outside the behavior this capability fulfills.

## Requirements

### Requirement: Independent encoder pages primary behavior
The system SHALL load eight independent mappings and update the display when an encoder page is selected.

#### Scenario: Primary operation
- **WHEN** an encoder page is selected
- **THEN** the system loads eight independent mappings and updates the display

### Requirement: Safety and secondary behavior
The system SHALL emit only the configured action or explicit macro targets when an encoder changes, and SHALL preserve a bounded application-side value.

#### Scenario: Secondary operation
- **WHEN** an encoder changes
- **THEN** the system emits only the configured action or explicit macro targets and preserves a bounded application-side value

### Requirement: Bitbox control categories
The Bitbox configuration SHALL group its encoder pages into PERFORM, MIX, DELAY, REVERB, CHAR, DYNAM, EQ, and PAD, in that order. PERFORM encoders 1–4 SHALL be delay send 84 and reverb send 85 on the selected group's channel, then delay feedback 30 and delay time 31 on channel 1. MIX SHALL send delay send 84 and reverb send 85 on channels 1–8, and level, filter, pan, and pitch as controllers 36–39 on those channels. DELAY SHALL send 108, 109, 110, 111, 30, 31, and 32 on channel 1. Global learned controllers SHALL use channel 1 and SHALL NOT reuse a patch-fixed controller on that channel. The panel proof SHALL assign no controller number.

#### Scenario: Delay send from PERFORM
- **WHEN** the Bitbox configuration is loaded and the first PERFORM encoder is turned
- **THEN** controller 84 is sent on channel 1

#### Scenario: Panel proof stays unassigned
- **WHEN** the panel proof configuration is loaded
- **THEN** its encoders assign no controller number
