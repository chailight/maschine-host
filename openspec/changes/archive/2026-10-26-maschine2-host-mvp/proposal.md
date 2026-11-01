# Change: Maschine2 Host MVP

## Why

The full-size Maschine MK2 remains an expressive performance surface, but its proprietary software is not a suitable dependency for a portable, laptop-free instrument. A Raspberry Pi or Mac host should expose the controller's groups, pads, encoders, LEDs and LCDs for both external MIDI instruments (notably Bitbox Micro) and optional local sample playback.

## Goals

1. Boot into a usable headless performance controller on Raspberry Pi 4/5 (64-bit Linux).
2. Support development and operation on macOS with the same core logic.
3. Provide eight selectable groups (A-H) with independent pad layouts and destinations.
4. Support one-shot pads, chromatic note pads and explicit scale note maps.
5. Support independent encoder pages with eight custom controls per page, including eight per-slot delay sends, reverb sends or filter controls.
6. Display actionable group/page/parameter state on the MK2 LCDs and show selection/activity on RGB LEDs.
7. Allow a group to target external MIDI or a basic local sample engine without changing the interaction model.
8. Make the core testable without MK2 hardware or an audio interface.

## Non-goals (MVP)

- Full Maschine software compatibility, sequencing, plugin hosting or sample editing.
- Automatic detection of Bitbox's actual internal parameter state.
- MIDI/audio machine listening, autonomous dub mixing or Pd integration.
- Touchscreen GUI or browser configuration editor.
- Advanced sample time-stretching, streaming, disk recording, effects or sophisticated voice allocation.
- Arbitrary incoming SysEx editor integration.

## Success criteria

- On physical MK2, A-H selects groups and the 16 pads play correctly routed notes/samples.
- Pad note-off, velocity, group changes and stuck-note prevention are verified.
- Encoder page changes update LCD labels and each encoder sends its configured action.
- Device reconnection and application restart return to a safe state.
- One-shot and chromatic local playback works through an available audio device; external MIDI works without audio.
- Same state-machine tests pass on Linux and macOS.

## Risks and prerequisites

- Verify actual USB HID, display and RGB APIs in upstream projects on target systems.
- Confirm Raspberry Pi ARM64 build and headless display operation before selecting a library.
- Confirm USB permissions, NI service conflicts on macOS and available ALSA/CoreMIDI ports.
- Confirm Bitbox enhanced-firmware CC assignments from its own documentation; use configuration rather than hardcoded assumptions.
- Check upstream licenses and redistribution obligations.
