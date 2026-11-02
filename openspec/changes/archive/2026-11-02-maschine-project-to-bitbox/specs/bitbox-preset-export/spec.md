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

### Requirement: Packed preset folder
The exporter SHALL write one packed preset directory named for the project under `PRESETS/`, or under an existing directory named `Presets`. That directory SHALL contain `preset.xml` and one subdirectory per filled pad. Each subdirectory SHALL be named for that pad and SHALL contain the wav files that pad uses. The exporter SHALL NOT write a sample folder at the card root. A sample cell's filename SHALL be that pad's subdirectory as a preset-relative path beginning with `.\`, and SHALL NOT name a wav file. Each asset filename SHALL be a preset-relative path beginning with `.\`, naming that same subdirectory, and ending with the wav basename. A destination path, when given, SHALL receive that packed directory under its preset folder and SHALL leave every other file there untouched. The command SHALL NOT modify the source project or the host template.

A stored start and end inside a sample SHALL be written out as its own wav containing that region. A start stored once, whose end is the last frame of the wav on disk, is a bounce of an earlier edit: the exporter SHALL copy that wav whole. A kit note's key range SHALL meet the next kit note, and the first kit note SHALL extend down to note 0.

#### Scenario: Samples travel inside the preset
- **WHEN** a group using `Kick Abbrand 1.wav` is exported
- **THEN** `PRESETS/<project>/preset.xml` exists, that wav is in a subdirectory of the preset, the sample cell filename is `.\` plus the subdirectory name, and the asset path begins with `.\`, names that subdirectory, and ends with the wav basename

#### Scenario: A bounced wav keeps its opening
- **WHEN** a sample's start is stored once and its end is the last frame of the wav on disk
- **THEN** the exported wav has the same length as that file

#### Scenario: Destination is a packed preset
- **WHEN** a destination directory is given
- **THEN** that directory contains `PRESETS/<project>/preset.xml` and the pad subdirectories with their wavs, it has no `<project>` sample folder at its root, and the source project is unchanged
