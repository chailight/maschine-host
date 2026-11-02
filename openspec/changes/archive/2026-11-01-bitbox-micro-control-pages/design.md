# Design: Bitbox Micro control surface

## Existing behaviour to preserve
Groups A–H select the playable instrument and its pad notes. Group selection also selects the Bitbox slot for selected-slot controls, using that group's MIDI channel. Group A is channel 1 and Group H is channel 8. Page and category changes do not change the group, and group changes do not change the category. The silk-screen Pad Mode button is not the PAD category. Pads send notes unless an explicit clip-hold layer is active.

The eight buttons above the displays are `display-button-1` through `display-button-8`. Today each one toggles its own lamp and selects nothing. This change makes those eight a category radio. The Left and Right buttons beside the displays wrap the active category's subpages. Their lamps show whether another subpage exists in that direction. A category with one subpage lights neither arrow. Pages without categories still light Left and Right only while held.

Encoder values are stored per page position and drawn as decimal numbers. Shared parameters need one value per parameter and slot. The LCD already draws four columns on each panel. Category names go on the top line of each column, under the button. The encoder label and value stay lower in the column.

## Navigation
Categories, left to right: PERFORM, MIX, DELAY, REVERB, CHAR, DYNAM, EQ, PAD. The host remembers the last subpage of each category. Selecting a category does not send a note.

## Default encoder assignments
Verified against the community patch CC reference for stock 2.3.4. Where the patch publishes no controller, the host proposes one that is free on that channel. The Bitbox learns the proposal. The host does not learn it.

| Category | Subpage | Encoders |
|---|---|---|
| PERFORM | default | selected-slot delay send 84; selected-slot reverb send 85; global delay feedback 30 and delay time 31, both channel 1; then the pad-type row below |
| MIX | delay send | controller 84 on channels 1–8 |
| MIX | reverb send | controller 85 on channels 1–8 |
| MIX | level, filter, pan, pitch | controllers 36, 37, 38, and 39 on channels 1–8 |
| DELAY | global | beat-sync 108, ping-pong 109, filter enable 110, filter width 111, feedback 30, time 31, cutoff 32, all channel 1 |
| REVERB | global | decay 33, damping 34, pre-delay 35, channel 1; selected-slot reverb send 85; selected-slot delay send 84 |
| CHAR | amount / tone | amounts 14–17, tones 18–21, channel 1. Tones start at 64 |
| CHAR | algorithm | pick 22–25, four bands, channel 1. Next-effect 26–29 is not on this row |
| DYNAM | compressor | enable 40, parameters 41–45, type 47, channel 1. Labels follow the commanded type. Next-type 48 is not on this row |
| EQ | mode | mode 50, play encoders 51–53, slope 59, bypass 57, channel 1 |
| EQ | host LFO | second EQ subpage, encoders 2–5. Speed in Hz, depth, shape, and a destination of None or one of the current mode's three play controls. The host moves that play controller and does not send 91–93 |
| EQ | push | holds 54–56, channel 1 |
| PAD | by configured type | only controllers the registry allows for that type |

PERFORM encoders 5–8 are the useful controls for the selected pad type. Granular is the example: density 79, grain size 80, window 81, scatter 82, on the group's channel. The other lists are a refinement and can change without moving the row:

- Sample: reverse 86, sustain 87, loop mode 88, loop crossfade 89.
- Clip: Live Flip 117, jungle cycle 115, manual tempo 114, encoder 8 unassigned.
- Slicer: play through 90, encoders 6–8 unassigned.

No pad type is configured on Groups A–H today, so encoders 5–8 stay inactive until one is set. There is one delay-time controller, 31 on channel 1. The module keeps two time values, and Beat Sync chooses which one is heard. Both values are learned to controller 31. A synced and a free time control are not separate encoders.

## MIDI mapping policy
The registry is the only source of patch-fixed controllers. One parameter shared by PERFORM and MIX uses one stored value for that slot. The host does not read values back from the module. On/off text uses 64. Named bands are shown only after their value ranges are recorded. Controllers that step on any value above 0 stay off these rows: Character next 26–29, compressor auto-makeup 46, compressor next type 48, clip method 112, clip clock 113, and jungle flavour 116.

## Preset template
`presets/TEST/preset.xml` is the preset saved on the module after the learns. It covers delay, reverb, and the Group B pad. `presets/host-template/preset.xml` copies that pad learn onto Groups A–H. The module has not yet reloaded the expanded file.

The file stores a learn as `modsource src="midicc"`. `mchan` counts from 0, so Group A is `0` and Group H is `7`. The pad's own MIDI input, `midimode`, counts from 1, and `0` means no channel. On the grid, row 0 columns 0–3 are channels 1–4, and row 1 columns 0–3 are channels 5–8. The other cells have no channel and no learned controller.

A learned controller moves upward from the saved value. Amount `1000` is the full travel. Pan is saved at `-1000`, full left, so controller 64 is the centre. Level, filter, and pitch are saved at 0. Delay feedback, both delay times, cutoff, and the three reverb parameters are saved at 0. Beat Sync is saved on. Controller 111 is the firmware's width control and has no learn.

Granular, sample, slicer, and clip controllers are firmware controllers. They are not entries in this preset. Preset Manager 1.0 drops `mchan` and `ccnum` on save and replaces the delay and reverb cells, so this file is saved on the module.

## Display
Each column shows the category name at the top, the encoder label, and either the formatted value, the number, UNMAPPED, or unavailable. The value is the last command the host sent.

## Open questions
1. The published reference is the community patch for stock 2.3.4. The module's installed image was not read. Confirm the splash says `community fw` before implementation.
2. LFO wave and Live Flip name ranges are not published as numeric edges. Those values stay numbers. EQ mode is ten bands of 13, from DJ Filter at 0–12 through Formant at 117–127.
3. Whether the step-on-nonzero controllers above should gain a one-shot trigger later. They are omitted from the default rows until then.
4. Which pad type belongs to each of Groups A–H. Until configured, type-specific encoders send nothing.
5. Whether an explicit clip-hold layer is wanted in this change. The specs allow it and keep it off PAD and Pad Mode. The default is that pads keep sending notes.
6. Editing a control on the module leaves the host showing its last command. No reconciliation is proposed.
