## Why

Pad hits on the MK2 often stay quiet, and a hard hit is required before the MIDI velocity climbs. Each group also needs its own note layout, from a drum map to a musical scale, without a separate host build for each one.

## What Changes

- Each group can set a velocity curve in the configuration file. A curve below 1 raises softer hits. An omitted curve keeps the current straight mapping.
- A group in scale mode can set a root note and a named scale. The sixteen pads then follow that scale. A group that omits the scale keeps its note list.
- One-shot and chromatic groups keep their note lists. One-shot groups can still set a velocity curve.
- The Bitbox file sets the same curve on every group so a normal hit is louder. It does not replace the shared note numbers 36 through 51.
- Selecting the Pad Mode button does not edit these settings. Editing them from the hardware is a later change, using the same per-group fields.

## Capabilities

### New Capabilities

### Modified Capabilities

- `groups-and-pads`: A pad sends the group's curved velocity, and a scale-mode group can send notes from its named scale.
- `configuration`: The configuration file accepts a per-group velocity curve and an optional root-and-scale pair, and rejects an invalid one before activation.

## Impact

- Group configuration parsing and the velocity and note chosen for a pad press.
- `docs/example-config.yaml` gains a velocity curve on each Bitbox group. Existing note lists stay valid when a scale is omitted.
- The Pad Mode button, octave control, encoder pages, and panel layout are unchanged.
