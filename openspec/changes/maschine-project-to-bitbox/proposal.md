## Why

A Maschine project already arranges the samples for a piece, but the Bitbox Micro preset that the host plays has to be built by hand. Copying those sounds onto the host template, and onto the Micro's card, should be one command.

## What Changes

- A command reads a Maschine `.mxprj` and the sample folder that sits beside it.
- It writes a Bitbox Micro preset that starts from `presets/host-template/preset.xml`, so the learned controllers stay in place.
- Each Maschine group that contains samples becomes one of the eight template pads, on MIDI channels 1–8. One-shot sounds in a group land on notes 36–51. A multi-sample sound stays one pad, with a key zone per sample.
- The command copies only the samples that preset uses, and can copy the finished folder to a path such as the Micro SD card.
- Patterns, scenes, plugin instruments, and Maschine effects are not converted.

## Capabilities

### New Capabilities

- `maschine-project`: Read a Maschine project file and resolve the samples it names.
- `bitbox-preset-export`: Build a host-template preset from those groups and copy it, with its samples, to a destination folder.

### Modified Capabilities

## Impact

- New command-line tool in this repository. The running Maschine host is unchanged.
- Reads `.mxprj` files and WAV paths. Writes `preset.xml` and copies WAV files.
- Depends on `presets/host-template/preset.xml` for the MIDI learns, channels, and pan.
- The example project `MaschineProjects/` is local reference material and is not part of the tool's output.
