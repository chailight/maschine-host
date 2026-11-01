## Context

See proposal.md for why. An encoder page already holds eight mappings, and each mapping can name a controller, a channel, and a starting value. The Bitbox file's three pages use controllers 20, 21, and 22. Display left and display right already walk the page list.

## Goals / Non-Goals

**Goals:**

- One published page list that covers the patch's per-pad controllers and the three master devices.
- Master-device encoders forced onto channel 1, where the module listens.

**Non-Goals:**

- A new encoder behaviour. Stepped lists and hold controls still send the stored 0–127 value.
- Controllers the patch reference does not number. Delay time, delay feedback, and LFO rate stay unmapped.
- Pad stop on an encoder. Controllers 120 and 123 remain the host's existing all-notes-off path.
- Knowing which pad on the module is Sample, Granular, Clip, or Slicer. The host sends the controller either way.
- The patch Settings rows. Program change, clock-loss, preset tempo, flip, the pop-up, Keep CCs, and themes are set on the module's Settings screen, not by a controller.

## Decisions

1. Per-pad controls are one page per controller, with encoders 1–8 on channels 1–8. That matches the eight groups already used as pad channels. The alternative, eight parameters for one selected pad, needs a pad selector this host does not have.
2. Master pages set channel 1 on every sending encoder. Omitting the channel would inherit the group channel, and the Character chain, EQ, and compressor ignore every channel except 1. The shared delay controls accept any channel; they use channel 1 as well so the page reads one way.
3. The page order puts the two sends, the shared delay, Character, the EQ, and the compressor first. The remaining per-pad pages follow. Display left and right already wrap, so the mix controls stay a short walk from the first page.
4. Hold controls and on/off controls are ordinary controllers. The patch treats 64 and above as held or on. The starting value is 0, so a control begins released. A stepped list, such as Live Flip or the EQ mode, is also an ordinary 0–127 controller. The module splits that range into bands, which is the fallback the patch describes when a controller has no stepped encoder.
5. Controls that step on any value above 0, such as Character "next effect" and compressor "next type", use the same absolute controller. Each new value above 0 can step the module once, including a turn back toward zero. A one-shot trigger kind would match a button more closely. This change does not add one, because the existing controller already reaches the control.
6. Starting values follow the patch's own centres and glue settings. Character tone and the three EQ play encoders start at 64. EQ mode starts at 6, the middle of the first of the ten bands. Compressor threshold, ratio, attack, release, and makeup start at 90, 7, 38, 37, and 64. The compressor switch starts at 0 so loading the file does not turn the compressor on.
7. The new pages replace the encoder pages in `docs/example-config.yaml`. The MIDI port, groups, notes, and velocity curves in that file stay. The panel proof file is not a Bitbox performance map and keeps empty controllers.
8. Per-pad encoders are labelled Slot 1 through Slot 8. Master encoders use short names that fit a column: for Character, `D Amt` through `S Amt` and `D Tone` through `S Tone`, then `D Pick` through `S Pick` and `D Next` through `S Next`. The EQ page is `Mode`, `Enc 2`, `Enc 3`, `Enc 4`, `Bypass`, `Slope`. The push page is `Push 2`, `Push 3`, `Push 4`. The compressor page is `On`, `Thresh`, `Ratio`, `Attack`, `Rel`, `Makeup`, `Auto`, `Type`, and the following page is `Next`. Delay setup is `Sync`, `Ping`, `Filter`, `Width`.

## Risks / Trade-offs

- [Thirty-five pages is a long walk to a late Clip control] → The mix pages are the first nine, so delay, reverb, Character, EQ, and the compressor are not behind the Clip pages.
- [A hold control latches while the encoder sits at 64 or above, rather than following a finger] → The starting value is 0. Returning the encoder below 64 releases it. A later change can map these to buttons.
- [Turning a "next" control back toward zero still sends values above 0 and can step again] → The page still exposes the control. A trigger kind can replace it later without renumbering the controller.
- [Controllers 20 and 21 no longer mean delay and reverb send] → The file and the encoder-page requirement both move. Delay send is 84 and reverb send is 85.

## Migration Plan

Replace the three encoder pages in the Bitbox file with the page list in the spec. Restart the host. To go back, restore the delay, reverb, and filter pages that send controllers 20, 21, and 22.
