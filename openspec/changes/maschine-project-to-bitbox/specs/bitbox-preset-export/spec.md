## Purpose

Builds a Bitbox Micro preset from Maschine groups by filling the host template and copying the samples that preset needs.

## ADDED Requirements

### Requirement: Host template is the starting preset
The exporter SHALL start from `presets/host-template/preset.xml`. It SHALL keep that file's learned controllers, including delay feedback on controller 30, both delay times on controller 31, delay cutoff on 32, reverb decay, damping, and pre-delay on 33, 34, and 35, and level, filter, pan, and pitch on 36–39. It SHALL keep each group pad's pan at full left. It SHALL NOT add a learn for controller 111. Pads that receive no Maschine group SHALL stay empty, on no MIDI channel, and without a learned controller.

#### Scenario: Delay learn survives export
- **WHEN** a project with one sampled group is exported
- **THEN** the delay cell still learns feedback to controller 30 and both delay times to controller 31, and filter width has no learned controller

### Requirement: One pad per sampled group
Sampled groups SHALL fill the template pads in order: row 0 columns 0–3 are channels 1–4, and row 1 columns 0–3 are channels 5–8. `midimode` SHALL be that channel, counting from 1. A group with no samples SHALL be named in the report and SHALL NOT take a pad. When more than eight groups have samples, the first eight SHALL be written and the rest SHALL be named in the report.

A group whose sampled sounds each have one file SHALL place those sounds on the notes that host pad sends, one note each, in sound order. Pads on channels 1, 2, and 4–8 send notes 36, 37, and upward. The channel 3 pad sends Group C's natural minor scale from note 48. At most 16 sounds SHALL be placed, and further sounds SHALL be named in the report. A sound with more than one sample file SHALL be a multi-sample on that pad: each file keeps its root note, and the key zones cover those roots without overlap. A root note stored in the project SHALL be used. When the project has no root note, a note name in the file name SHALL be read with C4 as MIDI note 60. A group that contains both one-shot sounds and a multi-sample sound SHALL keep the multi-sample on the pad and SHALL name the one-shot sounds in the report instead of placing them on notes 36 and upward.

#### Scenario: Kit sounds use the host pad notes
- **WHEN** a group has two sounds, `Kick` and `Clap`, and each has one wav file
- **THEN** the channel 1 pad places `Kick` on note 36 and `Clap` on note 37

#### Scenario: A kit on channel 3 follows the scale
- **WHEN** the third sampled group is a kit
- **THEN** its first two sounds are on notes 48 and 50

#### Scenario: Multi-sample keeps its own notes
- **WHEN** a sound has separate files for C1 and F#1 and those root notes are known
- **THEN** that pad's zones use those two root notes and do not place the files on notes 36 and 37

#### Scenario: Ninth group is reported
- **WHEN** nine groups contain samples
- **THEN** eight pads are filled and the report names the ninth group

### Requirement: Preset folder and card copy
The exporter SHALL write a card layout. Wav files go only in a top-level folder named for the project. That folder SHALL NOT contain `preset.xml`. `preset.xml` goes only in `PRESETS/<project name>/`, and that folder SHALL NOT contain wav files. An existing preset directory named `Presets` or `PRESETS` SHALL be reused. Each sample path in `preset.xml` SHALL be a Bitbox path, using backslashes, beginning with the sample folder's name and ending with the wav basename. A sample cell's filename SHALL be one of those wav paths, so the Micro shows that basename, not the preset folder on its own. A destination path, when given, SHALL receive that layout at its root and SHALL leave every other file there untouched. The command SHALL NOT modify the source project or the host template.

A stored start and end inside a sample SHALL be written out as its own wav containing that region. A kit note's key range SHALL meet the next kit note, and the first kit note SHALL extend down to note 0.

#### Scenario: Samples travel with the preset
- **WHEN** a group using `Kick Abbrand 1.wav` is exported
- **THEN** the sample folder contains that wav and no `preset.xml`, and `PRESETS/<project>/preset.xml` points at the wav with a backslash path that starts with the sample folder name and ends with the wav basename

#### Scenario: Destination mirrors the card
- **WHEN** a destination directory is given
- **THEN** that directory contains `<project>/` with the wavs only and `PRESETS/<project>/preset.xml` only, and the source project is unchanged
