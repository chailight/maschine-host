## Why

Velocity, scale, octave, and pad type already live on each group, and the Pad Mode button only latches a lamp. The player needs that button to open a setup screen for the selected group, and to choose a pad type by hand so the performance and pad pages show the matching controls.

## What Changes

- Pad Mode becomes a toggle. It leaves the Scene, Pattern, Navigate, Duplicate, Select, Solo, and Mute radio. Its lamp is on while the setup screen is showing.
- While the screen is engaged, both LCDs are titled PAD MODE and show the selected group. Encoders 1–5 edit velocity, scale, root note, octave, and pad type, one step every twenty clicks. Encoders 6–8 send nothing.
- Pressing Pad Mode again restores the category page that was already selected. Pads keep sending notes. Pad Mode does not open the PAD category.
- The pad type choices are none, sample, granular, clip, and slicer. An omitted type loads as sample. A multisample cell uses sample. There is no separate multisample or slice choice.
- Choosing a type enables that type's PERFORM controls and hides the other types on the PAD pages. The choice is kept on the group for the session. It is not written to the configuration file, to `preset.xml`, or to the Bitbox.
- Every group can change its scale through the western modes, its root note, and the existing octave offset. Choosing notes clears the scale name and keeps the root.

## Capabilities

### New Capabilities

### Modified Capabilities

- `navigation`: Pad Mode toggles a setup screen and stays distinct from the PAD category and from the Scene/Pattern radio.
- `display-feedback`: The LCDs and the Pad Mode lamp follow that screen.
- `groups-and-pads`: The setup encoders change the selected group's live curve, scale, octave, and pad type, and a button press alone does not.
- `performance-controls`: PERFORM encoders 5–8 follow the group's current pad type, including one chosen on the setup screen.
- `pad-context`: PAD pages follow that same current type and still do not read it from the module.

## Impact

- The Pad Mode button gesture, the LCD panel drawing, and what encoders 1–4 do while that screen is up.
- Live group fields for velocity curve, named scale, octave offset, and pad type. The YAML file and the Bitbox preset stay as loaded.
- PERFORM encoders 5–8 and the PAD category, which already filter on pad type, now follow a type set during the session.
