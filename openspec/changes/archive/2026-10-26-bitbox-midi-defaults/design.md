## Context

See proposal.md for why this change exists. A pad note and a controller message are already sent when the loaded mapping names them. The Bitbox file leaves every encoder controller empty, and its groups do not share one note list. An encoder mapping currently requires a channel.

## Goals / Non-Goals

**Goals:**

- Let a YAML file set pad notes, group channels, encoder controllers, and encoder channels.
- Default an omitted encoder channel to the group at the same index.
- Put the Bitbox send and pad defaults in `docs/example-config.yaml`.

**Non-Goals:**

- A per-group octave control.
- Changing chromatic or scale mode, or the octave offset that already starts at zero.
- Assigning controller numbers in the panel proof.
- Discovering controller numbers from Bitbox firmware.
- Linux HID, or playing the local sampler through the speakers.

## Decisions

1. **An omitted encoder channel follows group index.** Slot 1 uses Group A's channel and Slot 8 uses Group H's. A channel written on the encoder replaces that default. Alternative: require every encoder to repeat the group channel. Rejected because changing a group's channel would leave the knobs behind unless the file is edited twice.

2. **The Bitbox note list is copied onto each group.** Pads 1 through 16 use notes 36 through 51 in every Bitbox group. The notes stay a field on the group, so one group can be given a different list without a new shared map. Alternative: one note list for the whole file. Rejected because a group must remain adjustable on its own.

3. **Delay, reverb, and filter use controllers 20, 21, and 22.** The same controller is used on every slot, and the slot's channel tells the instruments apart. Controllers 20 through 31 are undefined in General MIDI. Controllers 1, 7, 10, 11, 64, 91, and 93 are the usual modulation, volume, pan, expression, sustain, reverb-send, and chorus-send assignments, so they are not the defaults. Alternative: twenty-four distinct controllers on one channel. Rejected because the slot already has the group's channel.

4. **The published Bitbox file sets each encoder channel explicitly.** The numbers match the groups, so the file shows the channel a reviewer can edit. Omission remains valid for any other file. Alternative: omit the channel in the Bitbox file. Rejected because the channel would be invisible in the config under review.

## Risks / Trade-offs

- [A receiver already uses controller 20, 21, or 22] → The number is a field in the Bitbox file and can be changed without a host change.
- [Group C's scale notes and the other groups' note lists change to 36 through 51] → That is the shared pad map. A group that needs its old notes can set them in the file.
- [Chromatic and scale groups still add the existing octave offset] → The offset starts at zero, so the sent note matches the configured note until a later octave change.

## Migration Plan

Replace the Bitbox note lists and the empty encoder controllers in `docs/example-config.yaml`. Removing a controller number returns that knob to sending no controller message. The panel proof is unchanged.

## Open Questions

- None. The controller numbers and the shared note list are the defaults to review in this proposal.
