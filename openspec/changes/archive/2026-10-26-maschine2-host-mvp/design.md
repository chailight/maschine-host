# Technical design

## Core architecture

`MK2 USB <-> HardwareAdapter <-> EventBus <-> ControllerCore -> {MidiBackend, SamplerBackend}`

`ControllerCore -> DisplayModel -> HardwareAdapter` and `ControllerCore -> LedModel -> HardwareAdapter`.

HardwareAdapter must be swappable: real MK2 HID backend, mock adapter and future alternatives. The application core must never depend on Zynthian classes or platform-specific USB calls.

## Implementation preference

Rust for the hardware adapter, state engine and MIDI router; use a portable Rust audio engine or a narrow audio-backend interface for local sample playback. Do not commit to a specific HID library until the spike compares the Witzman daemon/maschine.rs lineage and Encdr. A daemon connected over OSC or local IPC is acceptable if it proves more stable than direct library linkage. Linux ALSA MIDI and macOS CoreMIDI are platform backends. Audio device selection should be configurable; Pisound is not mandatory.

## State model

- `HostState`: active_group, active_encoder_page, groups[8], encoder_pages[], held_notes, parameters, connection_status.
- `Group`: id A-H, label, pad_mode, destination, MIDI channel (1-16), note_map[16], optional sample assignments and LED theme.
- `Destination`: external MIDI port or local sampler instrument.
- `EncoderPage`: id, label, 8 encoder mappings with label, target, message/action type, range, scaling and default value.
- `ParameterState`: last commanded value, source and timestamp; not assumed to be remote-device truth.

## Input and routing

Pad press creates a note-on or local voice trigger; release creates note-off when required. One-shot local playback may ignore release according to its trigger policy. Group changes must preserve correct note-off routing to the original destination/channel/note for already-held pads. On disconnect, panic, or shutdown, send note-offs/all-notes-off as appropriate and stop local voices safely.

Encoder actions include MIDI CC, local parameter adjustment and optional bounded multi-target macros. The first version supports absolute MIDI CC; relative controls are an adapter concern and must be normalized to a predictable application value. An encoder's current stored value may differ from the remote instrument's actual value. Explicitly document this on screen/UI where appropriate.

## LCD/LED design

Separate semantic display model from pixel rendering. Both 256x64 monochrome displays should be updated on group/page change and parameter movement, with throttling/coalescing to avoid USB saturation. Display group and instrument on one screen, encoder page and eight parameter names/values across the available screen area (exact arrangement determined by hardware legibility tests). Pads indicate playable/selected/activity states; Group LEDs indicate current selection. Do not rely on display feedback for critical note routing.

## Local sampler

Minimal sample loader supporting WAV PCM and a declared note-zone map. One-shot mode: 16 sample assignments with optional choke group. Chromatic mode: one or more root-key samples with nearest-zone selection and pitch transposition; validate pitch and voice handling in a separate spike. Use a non-blocking audio callback and preload modest sample sets. No effects required in MVP.

## Configuration

YAML loaded and schema-validated at startup, with useful errors and a safe example config. Config defines groups, destinations, note maps, encoder pages and MIDI ports. Config reload only when idle in MVP. Version the schema and supply an example Bitbox profile without claiming firmware-specific CC numbers.

## Reliability and tests

Unit tests for state transitions, note routing, encoder mappings, configuration and display models; mock-HID integration tests; Linux and macOS build CI; hardware smoke-test checklist for pads, encoders, LEDs, LCD, USB reconnect and MIDI port failures. Log structured events at controllable verbosity. Hardware failures must not block application shutdown.

## Key decisions to validate

1. HID library versus OSC daemon after Pi/macOS hardware spike.
2. MIDI library and virtual-port strategy per OS.
3. Local sampler backend and audio latency on Pisound/other interfaces.
4. MK2 button availability for page navigation, shift, panic and octave control.
5. Display update rate and text legibility on physical LCDs.

## Spike resolution (2026-10-25)

Evidence is in `docs/hardware-spike/EVIDENCE.md`. No Raspberry Pi was available, so the physical checks in tasks 0.2 and 0.3 were recorded as not run. On 2026-10-26 this Mac did enumerate a full MK2 at `17cc:1140` behind four USB 2.0 hubs. The HID, MIDI, audio-control, and DFU interfaces were identified, and no report was transferred, so the transport decision below stands.

1. No HID library and no OSC client. The core uses a mock adapter and an unavailable adapter. Encdr is GPL-3.0-or-later with a required wgpu dependency. The Witzman daemon lineage disagrees with Encdr on product id and display size, and its drawing API was not specified in the README that was fetched. See `docs/adr/0001-hid-strategy.md`.
2. midir provides ALSA on Linux and CoreMIDI on macOS. Ports are matched by exact name, with an optional virtual port. See `docs/adr/0002-midi-backend.md`.
3. The tested audio path is an in-process WAV mixer. Nominal latency is buffer time only. cpal is an optional probe. Pisound was not measured. See `docs/adr/0003-audio-backend.md`.
4. Page left, page right, panic, and octave are logical controls. They are not bound to silk-screen buttons.
5. The semantic model targets two 256x64 mono panels because that is the size in the full-MK2 descriptor and in this design. Frames are rate-limited. They are not transmitted, and legibility was not checked on a panel.
