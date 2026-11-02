## Context

See proposal.md for why this tool exists. The example project is `MaschineProjects/america-is-many-things.mxprj`, with samples in the sibling folder `america-is-many-things Samples`. That folder is local reference material and is listed in `.gitignore`.

The project file begins with the bytes for an NI document (`hsin`, then `DSINv`). It is not a zip and not one XML tree. Readable text is length-prefixed. The path `Bass G Kira.wav` is preceded by a single length byte whose value is 84, the length of the absolute path. Type names such as `NI::MASCHINE::DATA::Sampler` appear once, as a type table, not once per sound. Sample paths in this file point at `/Users/markp/Dropbox/MaschineProjects/america-is-many-things Samples/`.

Sounds found by name, in file order, include `Meat Kira` (one wav), `Bright Kira` (twelve note-named wavs from C1 and F#1 through C6 and F#6), the Abbrand one-shots (`Kick Abbrand 1`, `Clap Abbrand`, hats, rim, perc, sfx, synths, chord, noise, basses), the name `Abbrand Kit`, and the movie-dialogue wavs. `Perc[128] Abbrand.wav` is stored many times on one sound. Plugin sounds such as `Arp Tired` and `Creepy Chip Chords` have no wav. `Pattern 1` and `Scene 1` are present and are not sounds.

The host template's eight pads, MIDI learns, and full-left pan are the preset the exporter fills. A multi-sample in an older preset is a `sample` cell plus `asset` cells. Each asset has `rootnote`, `keyrangebottom`, `keyrangetop`, and `asssrcrow` / `asssrccol` pointing at the pad.

## Goals / Non-Goals

**Goals:**

- One command reads a project and writes a card layout: wavs only in `<name>/`, and only `preset.xml` in `PRESETS/<name>/`. `--dest` is the card root. An existing `Presets` or `PRESETS` directory is reused.
- Group order and the kit-versus-multi-sample rule come from the specs, checked against this example.

**Non-Goals:**

- Decoding every NI object, effect preset, or pattern event.
- Changing the Maschine host or the host template file.

## Decisions

1. **A Python script, separate from the host binary.** The host reads HID and sends MIDI. This tool reads a document and writes XML and files. A script under `tools/` can be run without building the host. Alternative: a Rust binary in this workspace. Rejected because it would share a crate with the hardware host for a file conversion.
2. **Resolve samples from the sibling folder, by file name.** The stored absolute path is a hint. The folder Maschine writes is `<project name> Samples`, including one subdirectory level, which is where the nested dialogue wavs sit. When the same name exists in the folder and in a subdirectory, the stored path's longer suffix wins. The frame offsets belong to that file.
3. **Collapse duplicate files on one sound only when no distinct root note is stored.** `Perc[128] Abbrand.wav` is the case to check. If the repeats carry different root notes or slice points, they stay separate. If they do not, the sound lists the file once.
4. **Root notes.** Prefer the note stored next to the path in the project. The implementation has to locate that field using Bright Kira, whose file names already say the note. If the field is not recovered, parse a note name in the file name with C4 as MIDI note 60. One-shot kits do not use those names. They take the notes that host pad sends: 36 and upward, except the channel 3 pad, which is Group C's natural minor scale from 48.
5. **A mixed group keeps the multi-sample.** One Bitbox pad cannot be a chromatic instrument and a drum kit at once. If Meat Kira and Bright Kira turn out to be one group, Bright Kira occupies the pad and Meat Kira is named in the report. Splitting them onto two pads would spend a second channel on one group. That stays available later if the recovered groups show it is the wrong call for this project.
6. **Bitbox paths.** Copied wavs live only in `<name>/` at the card root. `preset.xml` lives only in `PRESETS/<name>/` and refers to the wavs as `\Folder Name\file.wav`, using backslashes. A cell's filename is one of those wavs, so the Micro shows the sample or part, not the preset folder. A Maschine start and end become a trimmed wav, because an asset cell cannot store its own region. Kit notes meet each other, and the first note extends down to note 0. The template's `modsource` lines are copied through, not rewritten.
7. **Tests that can run without the example.** Checked-in tests use a small stand-in document with the same length-prefixed paths. The real project is an extra local check, because `MaschineProjects/` is not in git.

## Risks / Trade-offs

- [Group boundaries are not a literal "Group" string] → The reader walks sound order and the names the file does store, including `Abbrand Kit`, and the local example check records which sounds landed in which group.
- [A long path may not fit in one length byte] → The dialogue paths are the check. Join continued pieces rather than dropping the file.
- [C4 = 60 may not match Maschine's on-screen octave] → Confirm against the root stored beside Bright Kira before trusting the file name.
- [The hand-built host template has not been re-saved by the module] → The exporter still starts from that file. A module save of the template can replace it later without changing this tool's rules.

## Migration Plan

Add the script and its small fixture. Nothing already installed changes. Removing the script removes the tool.

## Open Questions

None that change the specs. Group membership in the example is confirmed by the local check in the tasks, using the rules above.
