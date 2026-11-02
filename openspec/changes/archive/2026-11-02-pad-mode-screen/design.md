## Context

See proposal.md for why. Each group already has a velocity curve, an optional named scale, a runtime octave offset from −4 to +4, and a pad type. The Pad Mode button is its own toggle, and a press does not change those group fields. PERFORM encoders 5–8 and the PAD pages follow the pad type. An omitted type loads as sample.

## Goals / Non-Goals

**Goals:**

- One toggle on the Pad Mode button that swaps the LCDs to a titled setup screen for the selected group and back to the current category page.
- Session edits to the curve, scale, root note, octave offset, and pad type from encoders 1–5, using the pad types the host already has.

**Non-Goals:**

- A program change, a local preset library, or reading `preset.xml` to discover the pad type. That later step is recorded in the top-level README.
- Writing the session edits back to the YAML file or to a preset.
- A new octave field, a multisample pad type, or a pad type named slice.
- Changing a group's one-shot, chromatic, or scale mode from this screen.

## Decisions

1. Pad Mode leaves the Scene/Pattern radio and becomes its own toggle. The other seven buttons stay one-at-a-time. The alternative, keeping it in the radio, would turn the setup screen off whenever Scene or Pattern is pressed, and would clear those lamps whenever the player opens the screen.
2. The setup screen is a flag on the controller, not another encoder page. The category and subpage indexes stay put, so leaving the screen returns to the page the player had. Left and Right do not move that subpage while the flag is set.
3. Encoders 1–5 edit velocity, scale, root, octave, and pad type only while the flag is set. A value changes once every twenty encoder clicks, and the clicks accumulate, so one physical click does not jump to the next value. Encoders 6–8 send nothing for that time, and the category page's stored values are left alone. A button press still does not itself change a field.
4. Velocity steps through 0.5, 1, and 2 and stops at either end. The Bitbox file uses 0.5, 1 is the straight mapping, and 2 makes a softer hit quieter. A loaded curve outside that list is shown until the first completed step, which sets 0.5.
5. Scale cycling is offered on every group, in the order `notes`, ionian (`major`), dorian, phrygian, lydian, mixolydian, aeolian (`natural-minor`), locrian, major pentatonic, minor pentatonic, and chromatic. Choosing `notes` clears the named scale and keeps the root. The first named scale uses that root, which starts as the loaded scale root or else the group's first note. The configuration file still accepts a scale only on a scale-mode group.
6. The root is a MIDI note from 0 to 127, shown as a note name with C4 at 60. Editing it updates the named scale's root when a scale is set. Pads on `notes` keep sending the note list.
7. Octave reuses the existing per-group offset and its −4 to +4 clamp, and it applies to every group mode, including one-shot. The column shows `0` or a signed integer.
8. An omitted pad type loads as `sample`. The cycle is `none`, `sample`, `granular`, `clip`, `slicer`. `sample` is the choice for a sample cell with multisample mode on. The host does not read that mode from the preset. `slicer` is the existing name for a sliced pad. `none` clears the type so PERFORM encoders 5–8 and PAD go inactive.
9. Both panels are titled PAD MODE and show the group letter and label. The left panel holds Vel, Scale, Root, and Oct. The right panel's first column holds Type.
10. A chosen type is stored on the group in memory. PERFORM and PAD already consult that field, so they follow it on the next visit. Nothing is written to disk and no message is sent to retarget the module.

## Risks / Trade-offs

- [A test that presses Pad Mode and expects the curve and scale to stay] → The press still leaves them. Only the setup encoders change them.
- [Scene used to clear the Pad Mode lamp] → The lamp now follows the setup screen alone. Tests of that radio need to expect Scene and Pad Mode to light independently.
- [The example file already sets a named scale on Group C, while the configuration spec says the Bitbox file omits one] → This screen shows the loaded scale. It does not rewrite the file to settle that difference.
- [The player can choose a type the module is not actually using] → That is the manual alignment. The host has no read-back from the module.
