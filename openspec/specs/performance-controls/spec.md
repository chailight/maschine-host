# Performance Controls Specification

## Purpose

Puts delay and reverb sends, feedback, and delay time on the first four PERFORM encoders, and the selected pad type on the last four.

## Requirements

### Requirement: Dub performance layout
PERFORM encoders 1–4 SHALL be the selected group's delay send, the selected group's reverb send, global delay feedback, and active delay time. Delay send SHALL use controller 84 and reverb send SHALL use controller 85, both on the selected group's MIDI channel. Delay feedback SHALL use controller 30 and delay time SHALL use controller 31, both on channel 1. There is one delay-time control. The Bitbox learns 30 and 31. The host SHALL NOT change the selected group when these encoders turn.

#### Scenario: Delay gesture
- **WHEN** a user increases encoder 1 and then encoder 3 while Group C is selected
- **THEN** encoder 1 sends controller 84 on Group C's channel, encoder 3 sends controller 30 on channel 1, and the group stays C

### Requirement: Pad-type performance profile
PERFORM encoders 5–8 SHALL be the performance controls for the selected group's pad type. The granular controls are the example: density, grain size, window, and scatter. The lists for the other pad types are a refinement of that same idea and MAY be replaced without changing the PERFORM layout. The current refinement is: sample uses reverse, sustain, loop mode, and loop crossfade; clip uses Live Flip, jungle cycle, and manual tempo, with encoder 8 unassigned; slicer uses play through, with encoders 6–8 unassigned. A group with no configured pad type SHALL show N/A on encoders 5–8 and SHALL send nothing from them.

#### Scenario: Granular instrument
- **WHEN** a group configured as granular is selected
- **THEN** encoders 5–8 show density, grain size, window, and scatter and send controllers 79, 80, 81, and 82 on that group's channel

#### Scenario: Pad type not configured
- **WHEN** the selected group has no pad type
- **THEN** encoders 5–8 show N/A and send nothing
