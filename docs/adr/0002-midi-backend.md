# ADR 0002: ALSA and CoreMIDI through midir

Date: 2026-10-25

Status: Accepted

## Context

External instruments need channel note and CC output on Linux and macOS. The MK2's class-compliant USB MIDI port is not a pad source: the Zynthian Maschine notes and the Witzman documentation both say pad traffic comes from a HID daemon, not from that port. MIDI notes used by the Witzman daemon to paint pad LEDs are a daemon feature, not device feedback this host implements.

## Decision

`midir` 0.10 is the platform backend. On Linux it uses ALSA. On macOS it uses CoreMIDI. Message bytes are built in `MidiMessage` and covered by tests on both platforms.

A configured port is opened only when its system name matches exactly. `virtual_port: true` creates a virtual output instead of attaching to an existing device. A missing name is a recoverable error. A failed note-on drops the unconfirmed hold so the core does not believe a note is sounding. Note-off and CC failures are logged and do not stop the process.

The sample Bitbox profile uses the port name `CONFIGURE_PORT_NAME` and null CC numbers. Null CC updates the commanded value and the display, and it sends nothing. The development Mac has an iConnectMIDI4+ attached. The host does not open it unless the config names that port.

## Consequences

CI can test routing with `MockMidi`. A live port check has to be done by the operator after the config names a real output. Virtual ports were exercised only as "created, or the backend returned an error"; the sandboxed probe could not initialize MIDI.
