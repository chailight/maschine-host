# ADR 0001: Keep USB, LCD, and LED transport out of the host

Date: 2026-10-25

Status: Accepted for output. macOS HID input is specified by `openspec/changes/mk2-hid-input` and reads only controls confirmed in `docs/hid-assignment.yaml`. LCD and LED transport stay unimplemented.

## Context

The design refuses a HID library until a Pi and macOS spike compares the Witzman daemon lineage with Encdr. Evidence is in `docs/hardware-spike/EVIDENCE.md`.

## Decision

The host core depends on `HardwareAdapter`. The shipped adapters are a mock used by tests and an unavailable adapter that rejects every submit. There is no USB reader, no LCD blit, and no LED packet writer.

Encdr is the only inspected project whose full-MK2 descriptor is explicit (`17cc:1140`, two 256x64 mono panels, 16 pads, eight group LEDs, eight screen encoders). Linking it would make this program GPL-3.0-or-later and would pull a mandatory `wgpu` dependency. Headless use of that GPU path on a Pi was not measured.

The Witzman daemon is GPL-3.0, still implemented in a file named `mikro.rs`, and its own README disagrees with Encdr on product ID (`1200` versus `1140`) and display size (128x64 or 255x64 versus 256x64). Its OSC drawing API is not specified in the README that was fetched. This host does not speak that protocol.

`maschine.rs` upstream supports the Mikro MK2 on Linux hidraw and has not changed since 2020. It is not a full-MK2 backend.

Semantic feedback is still implemented: two 256x64 monochrome buffers, group and pad levels, and text that says values are commanded rather than read back. Those buffers are not sent to a device. Display failure does not block note routing.

## Consequences

Physical pads, LEDs, and LCDs stay on `docs/hardware-smoke-checklist.md`. A later adapter can map confirmed controls onto `ControlId` without changing the core. Page left/right are logical controls. Encdr's hardware page names `arrow_left` and `arrow_right`; that mapping is not confirmed here.

Velocity is whatever the adapter already normalized to 1-127. A zero-velocity press is ignored so it cannot become a stuck note-on. Pad ADC scaling is not implemented.
