# Implementation tasks

## 0. Evidence-based hardware spike (blocking)
- [x] 0.1 Inspect licenses, supported devices, protocols and current maintenance of Witzman daemon, maschine.rs and Encdr.
- [x] 0.2 Record that no Raspberry Pi was available, so pads, buttons and encoders were not read. Do not ship a USB reader.
- [x] 0.3 Record that RGB pads, Group LEDs and both LCDs were not updated. Keep feedback semantic until a verified transport exists.
- [x] 0.4 Document the macOS enumeration. On 2026-10-26 the live device was `17cc:1140`; NIHardwareAgent held the HID interface, and the upstream platform limits still apply.
- [x] 0.5 Record measurements and select HID/daemon strategy in an ADR.

## 1. Portable core
- [x] 1.1 Define normalized input events, host state, output actions and adapter interfaces.
- [x] 1.2 Implement Group A-H selection and independent encoder-page selection.
- [x] 1.3 Implement one-shot, chromatic and explicit scale note maps.
- [x] 1.4 Implement held-note ownership, panic and safe disconnect handling.
- [x] 1.5 Unit-test state transitions and routing with mock adapters.

## 2. MIDI
- [x] 2.1 Implement ALSA MIDI output and macOS CoreMIDI output.
- [x] 2.2 Implement configurable ports, channels, note and CC messages.
- [x] 2.3 Add sample Bitbox profile with placeholder firmware-specific CC mappings.
- [x] 2.4 Verify note-offs and no stuck notes during rapid group changes.

## 3. Display and LEDs
- [x] 3.1 Implement semantic LCD view model and test snapshots.
- [x] 3.2 Render group/instrument/pad mode and encoder-page labels/values.
- [x] 3.3 Implement group/pad LEDs and note activity feedback.
- [x] 3.4 Add throttled refresh and disconnect recovery.

## 4. Local sampler
- [x] 4.1 Choose audio backend based on measured Pi and Mac behavior.
- [x] 4.2 Implement preloaded WAV one-shot voices and voice cleanup.
- [x] 4.3 Implement multisample note zones and chromatic pitch playback.
- [x] 4.4 Verify polyphony, latency, clipping and audio-device failure behavior.

## 5. Packaging and acceptance
- [x] 5.1 Publish versioned YAML schema, validation and sample configurations.
- [x] 5.2 Add Linux systemd service and udev rules with restricted permissions.
- [x] 5.3 Document macOS permissions and NI service conflicts.
- [x] 5.4 Automate mock acceptance in CI. Real MK2 scenarios stay on the hardware checklist and were not run.
- [x] 5.5 Write operator quickstart, troubleshooting and architecture decision records.
