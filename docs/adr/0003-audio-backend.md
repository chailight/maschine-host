# ADR 0003: In-process mixer, optional cpal probe

Date: 2026-10-25

Status: Accepted

## Context

Local one-shots and chromatic zones need playback without making external MIDI depend on an audio device. No Raspberry Pi and no Pisound were available. On this Mac, System Profiler listed Mac mini Speakers at 48000 Hz as the default output, plus an HDMI display, an iConnectMIDI4+ at 44100 Hz, and a virtual renderer. A cpal open of the default device did not complete: CoreAudio returned an error unknown to coreaudio-rs while the process was sandboxed.

## Decision

The tested backend is `Mixer`. It preloads 16-bit integer PCM WAV, mixes in the caller-supplied buffer, drops finished voices, steals the oldest voice past the polyphony cap, replaces voices that share a choke group, and hard-clips the mix while counting clips. Chromatic and scale playback pick the containing zone with the nearest root, then play at `2^((note-root)/12)`.

`nominal_latency_ms` is `buffer_frames * 1000 / sample_rate`. For the example config that is 256 frames at 48000 Hz, about 5.33 ms. It is not a measured round trip.

`cpal` is behind the `cpal-output` feature and is only a probe of the default output device. The running host does not require it. If sample files named by a sampler group cannot be loaded, startup refuses the whole configuration.

One-shot sampler groups ignore pad release unless the config asks for note-off. MIDI groups send note-off. Shutdown, panic, and disconnect send note-offs, all-notes-off on the channels that were used, and stop every local voice.

## Consequences

Audio-device removal is a recoverable `AudioSink` error. Note routing continues. Pi latency, clipping on a speaker, and Pisound behavior remain on the hardware checklist.
