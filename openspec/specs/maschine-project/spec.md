# maschine-project Specification

## Purpose

Reads a Maschine project and the sample folder beside it, and reports the groups and sounds that can become a Bitbox preset.

## Requirements

### Requirement: Project file and sample folder
The reader SHALL accept a Maschine `.mxprj` file. The file is a Native Instruments document, not a zip archive and not a single XML document. Sample references inside it SHALL be absolute filesystem paths. The reader SHALL resolve each sample from the sibling directory named `<project name> Samples`, including one level of subdirectory, matching the file name. When that name occurs more than once, the reader SHALL choose the file whose path under the sample folder is the longest suffix of the stored path. It SHALL NOT require the absolute path stored in the project to still exist. A sample that is named and not found SHALL be reported by the path the project stored, and the other samples SHALL still be returned.

#### Scenario: Example project uses the sibling sample folder
- **WHEN** the reader is given `america-is-many-things.mxprj` and the folder `america-is-many-things Samples` beside it
- **THEN** it resolves `Bass G Kira.wav`, `Bright Kira C1.wav`, `Kick Abbrand 1.wav`, and `GowlerMusic - Sample Pack Movie Dialogue - Cold War #GM0014 - 05 America Many Things.wav` from that folder

#### Scenario: Missing sample is reported
- **WHEN** a project names a wav file that is not in the sibling sample folder
- **THEN** the report names that stored path and the sounds whose files are present are still returned

### Requirement: Groups and sounds
The reader SHALL return groups in the order they appear in the project. Each group SHALL have a name when the project stores one, and an ordered list of sounds. A sound SHALL have a name and either no sample or one or more sample files. A sample file SHALL include a root note when the project stores one. The same file referenced more than once on one sound, with no distinct root note, SHALL count once. Plugin instruments and effects that have no sample SHALL be listed by name as unsupported and SHALL NOT appear as sample files. Patterns and scenes SHALL NOT be returned as sounds.

#### Scenario: One-shot and multi-sample sounds
- **WHEN** the example project is read
- **THEN** `Kick Abbrand 1` has one sample file and `Bright Kira` has a separate sample file for each of its recorded notes, including `C1` and `F#1`

#### Scenario: Repeated sample counts once
- **WHEN** one sound refers to the same wav file several times and those references have no distinct root notes
- **THEN** that sound lists the file once

#### Scenario: Plugin sound is unsupported
- **WHEN** the example project contains a sound such as `Arp Tired` that has a plugin and no wav file
- **THEN** the report names that sound as unsupported and does not invent a sample for it

#### Scenario: Patterns are not sounds
- **WHEN** the example project is read
- **THEN** `Pattern 1` and `Scene 1` are not returned as sounds
