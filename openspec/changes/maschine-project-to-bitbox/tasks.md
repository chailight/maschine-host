## 1. Read the project

- [x] 1.1 Add `tools/maschine_to_bitbox.py` and a checked-in stand-in document with length-prefixed paths, so tests do not need `MaschineProjects/`.
- [x] 1.2 Read absolute wav paths from the document and resolve them from the sibling `<project name> Samples` folder, including one subdirectory, without requiring the stored absolute path.
- [x] 1.3 Return groups in file order, with sound names, sample files, and a root note when one is stored. Count a repeated file once when it has no distinct root note.
- [x] 1.4 Report plugin sounds that have no wav, and do not return patterns or scenes as sounds.
- [x] 1.5 Run the reader on `america-is-many-things.mxprj` and record which sounds share a group, including `Bright Kira`, the Abbrand sounds, and `Abbrand Kit`.

## 2. Write the preset

- [x] 2.1 Copy `presets/host-template/preset.xml` and keep its learned controllers, full-left pan, and empty pads.
- [x] 2.2 Place sampled groups on row 0 columns 0–3 and row 1 columns 0–3, channels 1–8. Skip groups with no samples. Name any group past the eighth in the report.
- [x] 2.3 Map one-shot sounds to notes 36 and upward, at most 16, and map a multi-sample sound's files onto their root notes. In a mixed group, keep the multi-sample and report the one-shots.
- [x] 2.4 Copy the used wav files into the output folder and point `preset.xml` at them with a backslash path that starts with the folder name.
- [x] 2.5 Add a destination argument that copies the finished folder, and leave the source project and the host template unchanged.

## 3. Check

- [x] 3.1 Test the stand-in document for a missing sample, a duplicate file, a kit on notes 36 and 37, a two-note multi-sample, a ninth group, and a surviving delay learn.
- [x] 3.2 When `MaschineProjects/` is present, export `america-is-many-things` and confirm `Bass G Kira.wav`, `Bright Kira C1.wav`, `Kick Abbrand 1.wav`, and the America Many Things dialogue file are copied, and that `Arp Tired` is reported rather than given a sample.
