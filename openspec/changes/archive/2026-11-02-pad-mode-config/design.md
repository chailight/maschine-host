## Context

See proposal.md for why. A pad velocity is already an integer from 1 to 127 before the note is sent. Scale mode and one-shot mode both read the group's note list. Scale mode also adds the existing octave offset. The Pad Mode button only latches its lamp.

## Goals / Non-Goals

**Goals:**

- One velocity curve and one optional musical scale per group, both set in the configuration file.
- The Bitbox file uses a curve that lifts ordinary hits, without replacing notes 36 through 51.

**Non-Goals:**

- Editing the curve or the scale from the Pad Mode button. A later change can edit these same per-group fields.
- A new octave control. The offset already applied to scale mode stays.
- Changing sampler loudness. A sampler trigger still has no velocity.

## Decisions

1. The curve is an exponent on the 1–127 velocity, not a gain. Gain makes every hit louder and then clips the hard ones. An exponent below 1 raises the quiet part of the range and still reaches 127 on a full hit. A curve of 1 is the current mapping, so files that omit it do not change.
2. The curve is applied after the pad pressure has already been scaled to 1–127. The HID report format stays as it is.
3. A named scale is allowed only when the group mode is scale. One-shot groups keep an explicit drum map. Chromatic groups keep their semitone list. The alternative, letting every mode derive notes from a scale name, would replace the Bitbox drum notes.
4. The named scales are major, natural minor, the two pentatonic scales, and chromatic. The first pad is the root. Further pads walk the degrees and continue into the next octave. A missing scale block keeps today's note list, so Group C in the Bitbox file stays on notes 36 through 51.
5. The Bitbox file sets the curve to 0.5 on every group. That is the starting value for this hardware. It is still a field in the file, so a different curve does not require a code change.
6. The scale degree is chosen first. The existing scale-mode octave offset is added after that. No new offset field is introduced.

## Risks / Trade-offs

- [A curve of 0.5 makes a light bounce louder if it still becomes a second note] → The release hold remains the guard against a second edge. The curve does not add a new edge.
- [A later hardware editor needs somewhere to store an edit] → The values live on the group, which is the record a Pad Mode screen can change later.
