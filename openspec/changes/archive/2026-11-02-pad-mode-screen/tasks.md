## 1. Pad Mode toggle

- [x] 1.1 Make Pad Mode its own toggle, out of the Scene, Pattern, Navigate, Duplicate, Select, Solo, and Mute radio. Its lamp is on while the setup screen is engaged. A Scene or Pattern press leaves that lamp as it was.
- [x] 1.2 Test that the first press engages the screen without changing the category, the velocity curve, or the scale, and the second press shows the category page again. Changing group while the screen is up keeps the screen and shows that group's four fields. Left and Right do not move the remembered subpage.

## 2. LCD layout

- [x] 2.1 While the screen is engaged, draw the selected group and the Vel, Scale, Oct, and Type columns on the left panel, and clear the right panel. Hide the category names. Leaving the screen draws the active category page again.
- [x] 2.2 Test that a scale group with curve 0.5, `natural-minor`, octave 0, and no pad type shows 0.5, `natural-minor`, 0, and `none`, and that a one-shot group shows `notes` and `n/a`.

## 3. Setup encoders

- [x] 3.1 Step encoder 1 through curves 0.5, 1, and 2. A loaded curve of 0.75 becomes 0.5 on the first turn. After the curve becomes 1, velocity 16 is sent as 16.
- [x] 3.2 On a scale-mode group, cycle encoder 2 through `notes`, `major`, `natural-minor`, `major-pentatonic`, `minor-pentatonic`, and `chromatic`. The first named scale uses the group's first note as the root. One-shot and chromatic groups keep their note lists.
- [x] 3.3 Step encoder 3 through the existing octave offset, clamped to -4 through 4, for chromatic and scale groups. Root 48 and `natural-minor` at offset +1 send note 60 on the first pad. A one-shot group ignores encoder 3.
- [x] 3.4 Cycle encoder 4 through `none`, `sample`, `granular`, `clip`, and `slicer`. The type remains when the player leaves the group and comes back. Encoders 5–8 send nothing while the screen is up. The edit is not written to the configuration file or to `preset.xml`, and it sends no message that changes the module's pad.
- [x] 3.5 Test that leaving the screen for PERFORM with type `sample` shows reverse, sustain, loop mode, and loop crossfade, that PAD shows granular subpages after type `granular`, and that PAD sends nothing after type `none`.
