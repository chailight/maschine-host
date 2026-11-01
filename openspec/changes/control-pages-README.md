# Bitbox Micro control pages — OpenSpec change

Copy `openspec/changes/bitbox-micro-control-pages/` into the existing Maschine Host repository. This is an additive change: **Group A–H must continue selecting the playable instrument**. Do not replace the current host's architecture with the earlier starter bundle.

Start with `proposal.md`, then `design.md`, then `tasks.md` and the seven capability specifications. **First task is a firmware/stock MIDI mapping audit**; example CCs must be checked against the installed firmware version before implementation.

Suggested agent prompt:

> Read `openspec/changes/bitbox-micro-control-pages/` in full. Inspect the current Maschine Host code and the Bitbox Micro patched firmware documentation. First complete the mapping validation and identify conflicts with existing group/pad behaviour. Implement the change incrementally following tasks.md, using the existing host architecture and mock MIDI/display tests. Preserve Group A–H as playable instrument selection and keep page selection independent. Do not assume stock MIDI Learn CCs are fixed or that Bitbox provides parameter feedback. Report open questions before implementing ambiguous mappings.
