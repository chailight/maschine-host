# Bitbox control pages

`docs/example-config.yaml` sends to the existing CoreMIDI destination `iConnectMIDI4+ DIN 4`. Restart the host after editing it:

```sh
cargo run -- --config docs/example-config.yaml
```

Groups A–H stay the playable instruments, on MIDI channels 1–8, notes 36–51. Changing a control category does not change the group. The Pad Mode button is not the PAD category. It toggles a setup screen titled PAD MODE for the selected group's velocity curve, scale, root note, octave, and pad type. A press by itself does not change those values. Encoders 1–5 do, one step every twenty clicks, and the values are kept only until the host restarts. The scale list is the western modes on every group. Octave applies to every group. An omitted pad type is sample.

The eight buttons above the displays select, from left to right: PERFORM, MIX, DELAY, REVERB, CHAR, DYNAM, EQ, PAD. One is lit. Left and Right move inside the current category and remember the last page of each category. The Right lamp is on when that category has a later page, and the Left lamp is on when it has an earlier page. A category with one page lights neither arrow. Each column of the LCD shows that button's category name.

## Controllers the Bitbox must learn

The community patch already answers its own controllers. These extra numbers are proposed by the host. Teach them to the module on the channel shown. Do not reuse a controller that the patch, or another row here, already uses on that same channel.

| Control | CC | Channel |
| --- | --- | --- |
| Delay feedback | 30 | 1 |
| Delay time | 31 | 1 |
| Delay cutoff | 32 | 1 |
| Reverb decay | 33 | 1 |
| Reverb damping | 34 | 1 |
| Reverb pre-delay | 35 | 1 |
| Slot level | 36 | that slot, 1–8 |
| Slot filter | 37 | that slot, 1–8 |
| Slot pan | 38 | that slot, 1–8 |
| Slot pitch | 39 | that slot, 1–8 |

PERFORM encoders 1–4 are the selected slot's delay send (84), reverb send (85), then feedback (30) and time (31). Encoders 5–8 follow the group's `pad_type`. An omitted type loads as sample, so those encoders start as reverse, sustain, loop mode, and loop crossfade. Set `pad_type` to `granular`, `clip`, or `slicer` on a group for the other lists, or choose `none` on the Pad Mode screen to turn them off. Granular uses density, grain size, window, and scatter (79–82). Those pad-type controllers belong to the firmware. A preset does not learn them.

## Preset template

`presets/host-template/preset.xml` already contains the learns in the table above, for all eight groups. `presets/TEST/preset.xml` is the copy saved on the module, with the learns proven on delay, reverb, and the Group B pad. Load the host template on the module and save it there once. The expanded file has not been through that save yet.

The module writes a learn as `modsource src="midicc"`. In that attribute the channel counts from 0, so Group A is `0` and Group H is `7`. The pad's MIDI input counts from 1. On the grid, the first row is channels 1–4 and the second row is channels 5–8. The other pads have no channel and no learned controller.

A learned controller moves upward from the value saved in the preset. Full travel is amount `1000`. Pan is saved full left, so 64 is the centre. Level, filter, pitch, delay feedback, both delay times, delay cutoff, and the three reverb parameters are saved at 0. Beat Sync is saved on. Controller 31 is learned to both the percentage time and the musical time, and controller 108 chooses which one is heard. Controller 111 is the firmware width control and is not learned.

Preset Manager 1.0 drops the controller number and channel when it saves, and it replaces the delay and reverb cells. Save this preset on the module.

## Overrides

Page order, labels, controller numbers, and pad type live in the YAML. A parameter name is shared: the selected slot's delay send and that slot's MIX delay-send encoder move the same value. `scope: selected` follows the group's MIDI channel. `scope: global` uses the channel in the file. `scope: slot` uses the encoder's own channel.

`format: on-off` shows On at 64 or above and Off below. The number on the display is the last command the host sent. The module does not report its own value back.

## Not on a default row

These patch controllers step on any value above zero, so an absolute encoder would step while turning back down: character next (26–29), compressor auto-makeup (46), compressor next type (48), clip method (112), clip clock (113), and jungle flavour (116). Controllers 120 and 123 stop a pad and are not encoder rows. EQ mode names its ten bands. LFO wave and Live Flip stay numbers until their band edges are known. Settings such as theme and preset BPM are on the module screen, not on a controller.

The Sampling button toggles a clip-hold layer while these categories are loaded. A pad then sends controller 118 at 127 instead of a note, and the last pad release sends 0. Leaving the page, changing group, disconnect, and shutdown also send 0. Opening PAD does not start that layer.
