## Why

The Bitbox config stores encoder values and pad notes, and it already sends a pad note on the virtual MIDI port. The knobs send nothing, because every send is `cc: null`, and the eight groups do not share one pad note map. A connected instrument needs configurable controller numbers and note numbers, with defaults that avoid the controllers a general MIDI device already uses.

## What Changes

- Keep pad notes, group MIDI channels, encoder controller numbers, and encoder MIDI channels in the YAML configuration. An explicit value overrides the default.
- When an encoder send omits its channel, use the channel of the group at the same index. Slot 1 follows Group A, through Slot 8 and Group H.
- The Bitbox config gives every group the same sixteen pad notes, 36 through 51, and keeps one channel per group: Group A on channel 1 through Group H on channel 8.
- The Bitbox delay, reverb, and filter pages assign a controller on each slot. Delay uses controller 20, reverb uses 21, and filter uses 22. These three numbers are undefined in General MIDI, so they are not the usual modulation, volume, pan, expression, sustain, reverb-send, or chorus-send controllers. Each slot uses its group's channel unless the file sets another channel.
- A per-group octave adjustment is a later change. This change does not add one. Existing chromatic and scale octave behavior stays as it is, and it starts at zero.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `configuration`: An encoder send may omit its MIDI channel and then uses the matching group's channel. Notes, channels, and controller numbers remain editable fields.
- `groups-and-pads`: The Bitbox groups keep distinct channels and use one shared pad note list.
- `encoder-pages`: The Bitbox delay, reverb, and filter sends use controllers 20, 21, and 22 on the matching group's channel unless a channel is set.

## Impact

- `docs/example-config.yaml` gains the shared pad notes and the three send controllers. The virtual port stays `Maschine2 Host`. Group colors, modes, and labels stay.
- The proof file `docs/encoder-panel-example.yaml` stays a panel example and does not gain these controller numbers.
- The host already sends a note or a controller when the mapping names one. This change fills in the Bitbox mappings and the omitted-channel default.
- Linux HID, speaker playback of the local sampler, and a per-group octave control are outside this change.
