# Maschine2 Host

A portable, headless host for the full-size Native Instruments Maschine MK2, initially targeting Raspberry Pi OS (64-bit) and macOS.

The current program routes groups, pads, and encoder pages, and it can play 16-bit WAV into an in-process mixer. A Maschine Controller MK2 (`17cc:1140`) enumerates on this Mac through the USB 2.0 hub path. The program still does not read or write that device: the HID report descriptor was not decoded, and LCD and LED transport remain unimplemented. Start with `docs/operator-quickstart.md`, `docs/hardware-spike/EVIDENCE.md`, and `docs/adr/0001-hid-strategy.md`.

## Development with OpenSpec

This repository contains an initial OpenSpec change proposal under `openspec/changes/maschine2-host-mvp/`. Install and initialize a compatible version of OpenSpec in the repository, then ask Codex or Cursor to review the proposal, design, specifications, and tasks before implementation. OpenSpec CLI conventions may vary by version; preserve the files if initialization offers to overwrite them.

Suggested agent prompt:

> Read openspec/changes/maschine2-host-mvp/proposal.md, design.md, specs/ and tasks.md. Review the proposed architecture against the upstream MK2 HID implementations and the current OpenSpec conventions. Resolve the hardware-spike gates first, document evidence, then implement the MVP incrementally with tests. Do not assume undocumented USB, LCD, or MIDI feedback capabilities.

## Boundaries

This is not a Zynthian fork, a Maschine software clone, or an autonomous mixing system. The initial product supports MK2 performance control, external MIDI destinations and a small local sampler. Audio listening and Pd are deferred.

## Source projects to audit

- https://github.com/Witzman/Generative-Techno-ZynthianMaschine-MKII
- https://github.com/Witzman/MaschineMK2_linux
- https://github.com/wrl/maschine.rs
- https://github.com/RufusIbiza/Encdr

Do not copy source code before checking license, attribution, dependencies and compatibility.

## License

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. The full text is in `LICENSE`.
