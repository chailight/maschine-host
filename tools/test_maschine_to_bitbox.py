"""Tests for the Maschine project reader and Bitbox preset export.

The checked-in stand-in is a length-prefixed document. It does not need
MaschineProjects/. The example project, when that folder is present, is an
extra local check.

Groups recovered from america-is-many-things, in file order:

1. Meat Kira — one sample, Bass G Kira.wav
2. Bright Kira — twelve note files. C1 is stored as root 36 and F#1 as 42
   (Maschine's C3 = MIDI 60), not as the C4 = 60 reading of the file name
3. Arp Tired — plugin, no sample
4. Creepy Chip Chords — plugin, no sample
5. Low Smash — plugin, no sample
6. Abbrand Kit — Kick, Clap, hats, Rim, Perc[128] (one wav, roots 60–71),
   SFX, Crackle, two Synths, Chord, Noise, two Basses
7. Fifteen movie-dialogue clips, with no separate group name
"""

from __future__ import annotations

import hashlib
import shutil
import struct
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import maschine_to_bitbox as tool

REPO = Path(__file__).resolve().parents[1]
STANDIN = Path(__file__).resolve().parent / "testdata" / "standin.mxprj"
EXAMPLE = REPO / "MaschineProjects" / "america-is-many-things.mxprj"
TEMPLATE = tool.TEMPLATE


def sound_names(project: tool.Project) -> list[str]:
    return [sound.name for group in project.groups for sound in group.sounds]


class StandInTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.project_path = self.root / "standin.mxprj"
        shutil.copy(STANDIN, self.project_path)
        samples = self.root / "standin Samples"
        nested = samples / "takes"
        nested.mkdir(parents=True)
        for name in ("Clap.wav", "Bright C1.wav", "Bright F#1.wav", "Perc.wav", "Present.wav"):
            (samples / name).write_bytes(b"RIFF")
        (nested / "Kick.wav").write_bytes(b"RIFF")

    def test_reader_resolves_sibling_folder_and_one_subdirectory(self):
        project = tool.read_project(self.project_path)
        kick = next(sound for sound in sound_names(project) if sound == "Kick")
        self.assertEqual(kick, "Kick")
        kick_sample = project.groups[0].sounds[0].samples[0]
        self.assertEqual(kick_sample.resolved, self.root / "standin Samples" / "takes" / "Kick.wav")
        self.assertTrue(str(kick_sample.stored_path).startswith("/archive/"))

    def test_same_name_in_a_subdirectory_follows_the_stored_path(self):
        body = b"".join(
            [
                tool.encode_string("Nested"),
                tool.encode_string("/archive/takes/Take.wav"),
                tool.encode_string("Flat"),
                tool.encode_string("/archive/Take.wav"),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.root / "dup.mxprj"
        path.write_bytes(body)
        folder = self.root / "dup Samples"
        (folder / "takes").mkdir(parents=True)
        (folder / "Take.wav").write_bytes(b"SHORT")
        (folder / "takes" / "Take.wav").write_bytes(b"LONG")
        project = tool.read_project(path)
        nested, flat = (sound.samples[0] for sound in project.groups[0].sounds)
        self.assertEqual(nested.resolved.read_bytes(), b"LONG")
        self.assertEqual(flat.resolved.read_bytes(), b"SHORT")

    def test_missing_sample_is_reported_and_others_remain(self):
        project = tool.read_project(self.project_path)
        self.assertTrue(any(path.endswith("Missing.wav") for path in project.missing))
        self.assertIn("Present", sound_names(project))
        self.assertIn("Gone", sound_names(project))

    def test_duplicate_file_without_a_distinct_root_counts_once(self):
        project = tool.read_project(self.project_path)
        perc = next(sound for group in project.groups for sound in group.sounds if sound.name == "Perc")
        self.assertEqual(len(perc.samples), 1)

    def test_plugin_pattern_and_scene(self):
        project = tool.read_project(self.project_path)
        names = sound_names(project)
        self.assertIn("Arp Tired", names)
        arp = next(sound for group in project.groups for sound in group.sounds if sound.name == "Arp Tired")
        self.assertTrue(arp.unsupported)
        self.assertEqual(arp.samples, [])
        self.assertFalse(any(name.startswith("Pattern") or name.startswith("Scene") for name in names))


class ExportTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.template_hash = hashlib.sha256(TEMPLATE.read_bytes()).hexdigest()

    def write_project(self, name: str, body: bytes, files: dict[str, bytes]) -> Path:
        path = self.root / f"{name}.mxprj"
        path.write_bytes(body)
        folder = self.root / f"{name} Samples"
        folder.mkdir()
        for relative, data in files.items():
            target = folder / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        return path

    def test_kit_notes_and_multisample_roots(self):
        body = b"".join(
            [
                tool.encode_string("Kick"),
                tool.encode_string("/old/Kick.wav"),
                tool.encode_string("Clap"),
                tool.encode_string("/old/Clap.wav"),
                tool.encode_string("Pattern 1"),
                tool.encode_string("Bright"),
                tool.encode_string("/old/Bright C1.wav"),
                tool.encode_root(24),
                tool.encode_string("/old/Bright F#1.wav"),
                tool.encode_root(30),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project(
            "kit",
            body,
            {"Kick.wav": b"K", "Clap.wav": b"C", "Bright C1.wav": b"1", "Bright F#1.wav": b"2"},
        )
        before = path.read_bytes()
        report = tool.export_preset(tool.read_project(path), self.root / "out")
        self.assertEqual(path.read_bytes(), before)
        self.assertEqual(hashlib.sha256(TEMPLATE.read_bytes()).hexdigest(), self.template_hash)
        preset = (report.output / "preset.xml").read_text()
        self.assertNotIn("Supersaw", preset)
        self.assertIn("ccnum=\"30\"", preset)
        self.assertEqual(preset.count("ccnum=\"31\""), 2)
        self.assertIn('dest="delay"', preset)
        self.assertIn('dest="delaymustime"', preset)
        self.assertNotIn("ccnum=\"111\"", preset)
        self.assertIn("\\kit\\Kick.wav", preset)
        self.assertIn("\\kit\\Clap.wav", preset)
        roots = [
            int(cell.find("params").get("rootnote"))
            for cell in ET.parse(report.output / "preset.xml").getroot().iter("cell")
            if cell.get("type") == "asset"
        ]
        self.assertEqual(roots[:2], [36, 37])
        self.assertIn(24, roots)
        self.assertIn(30, roots)
        self.assertNotEqual(roots[2:4], [36, 37])
        pad = _pad(report.output / "preset.xml", 0, 0)
        self.assertEqual(pad.find("params").get("panpos"), "-1000")
        self.assertEqual(pad.find("params").get("midimode"), "1")
        unused = _pad(report.output / "preset.xml", 0, 2)
        self.assertEqual(unused.find("params").get("midimode"), "0")
        self.assertFalse(any(mod.get("src") == "midicc" for mod in unused))
        self.assertTrue((report.samples / "Kick.wav").is_file())
        kit_pad = _pad(report.output / "preset.xml", 0, 0)
        self.assertTrue(kit_pad.get("filename", "").endswith(".wav"))
        self.assertNotEqual(kit_pad.get("filename"), r"\kit")
        ranges = [
            (
                cell.find("params").get("keyrangebottom"),
                cell.find("params").get("keyrangetop"),
            )
            for cell in ET.parse(report.output / "preset.xml").getroot().iter("cell")
            if cell.get("type") == "asset"
        ]
        self.assertEqual(ranges[0], ("0", "36"))
        self.assertEqual(ranges[1], ("37", "127"))

    def test_third_pad_kit_follows_the_group_c_scale(self):
        parts = []
        files = {}
        for name in ("One", "Two", "Kick", "Clap"):
            parts.append(tool.encode_string(name))
            parts.append(tool.encode_string(f"/old/{name}.wav"))
            if name in ("One", "Two"):
                parts.append(tool.encode_string("Pattern 1"))
            files[f"{name}.wav"] = name.encode()
        parts.append(tool.encode_string("Pattern 1"))
        path = self.write_project("scale", b"".join(parts), files)
        report = tool.export_preset(tool.read_project(path), self.root / "out")
        assets = [
            cell
            for cell in ET.parse(report.output / "preset.xml").getroot().iter("cell")
            if cell.get("type") == "asset" and cell.find("params").get("asssrccol") == "2"
        ]
        roots = [cell.find("params").get("rootnote") for cell in assets]
        self.assertEqual(roots[:2], ["48", "50"])
        self.assertEqual(assets[0].find("params").get("keyrangetop"), "49")
        self.assertEqual(assets[1].find("params").get("keyrangebottom"), "50")

    def test_mixed_group_keeps_the_multisample(self):
        body = b"".join(
            [
                tool.encode_string("Kick"),
                tool.encode_string("/old/Kick.wav"),
                tool.encode_string("Bright"),
                tool.encode_string("/old/Bright C1.wav"),
                tool.encode_root(24),
                tool.encode_string("/old/Bright F#1.wav"),
                tool.encode_root(30),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project(
            "mixed",
            body,
            {"Kick.wav": b"K", "Bright C1.wav": b"1", "Bright F#1.wav": b"2"},
        )
        report = tool.export_preset(tool.read_project(path), self.root / "out")
        preset = (report.output / "preset.xml").read_text()
        self.assertNotIn("Kick.wav", preset)
        self.assertIn("Bright C1.wav", preset)
        self.assertTrue(any("Kick" in name for name in report.held_sounds))

    def test_filename_roots_when_the_project_stores_none(self):
        body = b"".join(
            [
                tool.encode_string("Keys"),
                tool.encode_string("/old/Pad C4.wav"),
                tool.encode_string("/old/Pad C1.wav"),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project("keys", body, {"Pad C4.wav": b"a", "Pad C1.wav": b"b"})
        report = tool.export_preset(tool.read_project(path), self.root / "out")
        roots = sorted(
            int(cell.find("params").get("rootnote"))
            for cell in ET.parse(report.output / "preset.xml").getroot().iter("cell")
            if cell.get("type") == "asset"
        )
        self.assertEqual(roots, [24, 60])

    def test_same_file_with_two_roots_stays_separate(self):
        body = b"".join(
            [
                tool.encode_string("Perc"),
                tool.encode_string("/old/Perc.wav"),
                tool.encode_root(60),
                tool.encode_string("/old/Perc.wav"),
                tool.encode_root(61),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project("perc", body, {"Perc.wav": b"P"})
        project = tool.read_project(path)
        self.assertEqual([sample.root for sample in project.groups[0].sounds[0].samples], [60, 61])

    def test_ninth_group_is_named_and_not_written(self):
        parts = []
        files = {}
        for index in range(1, 10):
            parts.append(tool.encode_string(f"Group{index}"))
            parts.append(tool.encode_string(f"/old/G{index}.wav"))
            parts.append(tool.encode_string("Pattern 1"))
            files[f"G{index}.wav"] = b"G"
        path = self.write_project("nine", b"".join(parts), files)
        report = tool.export_preset(tool.read_project(path), self.root / "out")
        self.assertEqual(len(report.placed_groups), 8)
        self.assertIn("Group9", report.skipped_groups)
        preset = (report.output / "preset.xml").read_text()
        self.assertNotIn("G9.wav", preset)

    def test_destination_copy_leaves_the_source_project(self):
        body = b"".join(
            [
                tool.encode_string("Kick"),
                tool.encode_string("/old/Kick.wav"),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project("one", body, {"Kick.wav": b"K"})
        before = path.read_bytes()
        dest = self.root / "card"
        report = tool.export_preset(tool.read_project(path), self.root / "out", dest)
        self.assertEqual(path.read_bytes(), before)
        self.assertTrue((dest / "PRESETS" / "one" / "preset.xml").is_file())
        self.assertEqual([path.name for path in (dest / "PRESETS" / "one").iterdir()], ["preset.xml"])
        self.assertTrue((dest / "one" / "Kick.wav").is_file())
        self.assertFalse(list((dest / "one").glob("*.xml")))
        self.assertEqual((dest / "one" / "Kick.wav").read_bytes(), (report.samples / "Kick.wav").read_bytes())
        (dest / "keep.txt").write_text("stay")
        tool.export_preset(tool.read_project(path), self.root / "out", dest)
        self.assertEqual((dest / "keep.txt").read_text(), "stay")

    def test_dest_named_presets_keeps_samples_out_of_the_preset_folder(self):
        body = b"".join(
            [
                tool.encode_string("Kick"),
                tool.encode_string("/old/Kick.wav"),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project("one", body, {"Kick.wav": b"K"})
        card = self.root / "presets"
        (card / "PRESETS").mkdir(parents=True)
        (card / "keep.txt").write_text("stay")
        tool.export_preset(tool.read_project(path), None, card)
        preset = card / "PRESETS" / "one"
        self.assertEqual([item.name for item in preset.iterdir()], ["preset.xml"])
        self.assertTrue((card / "one" / "Kick.wav").is_file())
        self.assertFalse((card / "one" / "preset.xml").exists())
        self.assertEqual((card / "keep.txt").read_text(), "stay")
        self.assertFalse((self.root / "one").exists())

    def test_long_path_is_joined_and_resolved_by_name(self):
        filename = "N" * 240 + " C4.wav"
        stored = "/Users/old/projects/" + ("samples/" * 5) + filename
        self.assertGreater(len(stored), 255)
        body = b"".join(
            [
                tool.encode_string("Long"),
                tool.encode_string(stored),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project("long", body, {filename: b"L"})
        project = tool.read_project(path)
        sample = project.groups[0].sounds[0].samples[0]
        self.assertEqual(sample.stored_path, stored)
        self.assertEqual(sample.resolved.name, filename)
        self.assertEqual(project.missing, [])

    def test_play_region_trims_a_slice_and_keeps_a_whole_file(self):
        full = _wav(200, 7)
        sliced = _wav(200, 9)
        body = b"".join(
            [
                tool.encode_string("Whole"),
                tool.encode_string("/old/Whole.wav"),
                tool.encode_region(0, 199),
                tool.encode_string("Pattern 1"),
                tool.encode_string("Slice"),
                tool.encode_string("/old/Slice.wav"),
                tool.encode_region(10, 80),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project("cuts", body, {"Whole.wav": full, "Slice.wav": sliced})
        project = tool.read_project(path)
        whole = project.groups[0].sounds[0].samples[0]
        piece = project.groups[1].sounds[0].samples[0]
        self.assertEqual((whole.start, whole.end), (0, 199))
        self.assertEqual((piece.start, piece.end), (10, 80))
        report = tool.export_preset(project, self.root / "out", names={"2": "vocals"})
        self.assertEqual(tool._pcm(report.samples / "Whole.wav")[2], 200)
        self.assertEqual(tool._pcm(report.samples / "vocals.wav")[2], 71)
        preset = (report.output / "preset.xml").read_text()
        self.assertIn("\\cuts\\vocals.wav", preset)
        self.assertNotIn('filename="\\cuts"', preset)

    def test_same_file_with_two_regions_stays_two_files(self):
        source = _wav(1000, 3)
        body = b"".join(
            [
                tool.encode_string("Line"),
                tool.encode_string("/old/Take.wav"),
                tool.encode_region(0, 99),
                tool.encode_string("Line"),
                tool.encode_string("/old/Take.wav"),
                tool.encode_region(200, 499),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project("lines", body, {"Take.wav": source})
        report = tool.export_preset(tool.read_project(path), self.root / "out")
        lengths = sorted(tool._pcm(wav)[2] for wav in report.samples.glob("*.wav"))
        self.assertEqual(lengths, [100, 300])

    def test_bounced_excerpt_keeps_its_opening(self):
        long_frames = 400
        offset = 100
        excerpt_frames = 180
        samples = list(range(long_frames))
        long_wav = _wav_from(samples)
        short_wav = _wav_from(samples[offset : offset + excerpt_frames])
        body = b"".join(
            [
                tool.encode_string("Take"),
                tool.encode_string("/old/takes/Take.wav"),
                tool.encode_string("Pattern 1"),
                tool.encode_string("Phrase"),
                tool.encode_string("/old/Take.wav"),
                tool.encode_region(offset, excerpt_frames - 1),
                tool.encode_string("Pattern 1"),
            ]
        )
        path = self.write_project(
            "excerpt",
            body,
            {"takes/Take.wav": long_wav, "Take.wav": short_wav},
        )
        report = tool.export_preset(tool.read_project(path), self.root / "out")
        phrase = next(item for item in report.samples.glob("*.wav") if tool._pcm(item)[2] != long_frames)
        self.assertEqual(tool._pcm(phrase)[2], excerpt_frames)


def _wav_from(samples: list[int]) -> bytes:
    destination = Path(tempfile.mkstemp(suffix=".wav")[1])
    pcm = struct.pack("<" + "h" * len(samples), *samples)
    tool._write_wav(destination, {"channels": 1, "rate": 44100, "bits": 16}, pcm)
    data = destination.read_bytes()
    destination.unlink()
    return data


def _wav(frames: int, value: int) -> bytes:
    return _wav_from([value] * frames)


def _pad(preset: Path, row: int, column: int):
    for cell in ET.parse(preset).getroot().iter("cell"):
        if cell.get("layer") == "0" and cell.get("row") == str(row) and cell.get("column") == str(column):
            return cell
    raise AssertionError(f"missing pad {row},{column}")


@unittest.skipUnless(EXAMPLE.is_file(), "local example project is not present")
class ExampleProjectTests(unittest.TestCase):
    def test_groups_match_the_project(self):
        project = tool.read_project(EXAMPLE)
        names = sound_names(project)
        self.assertEqual(project.groups[0].sounds[0].name, "Meat Kira")
        self.assertEqual(Path(project.groups[0].sounds[0].samples[0].stored_path).name, "Bass G Kira.wav")
        bright = project.groups[1].sounds[0]
        self.assertEqual(bright.name, "Bright Kira")
        roots = {Path(sample.stored_path).name: sample.root for sample in bright.samples}
        self.assertEqual(roots["Bright Kira C1.wav"], 36)
        nested = next(
            sample
            for sound in project.groups[6].sounds
            for sample in sound.samples
            if sample.stored_path.endswith("96 Rob Freedom/GowlerMusic - Sample Pack Movie Dialogue - Cold War #GM0014 - 96 Rob Freedom.wav")
        )
        self.assertEqual(tool._pcm(nested.resolved)[2], 782775)
        self.assertEqual(roots["Bright Kira F#1.wav"], 42)
        self.assertEqual(project.groups[5].name, "Abbrand Kit")
        kit = {sound.name: sound for sound in project.groups[5].sounds}
        self.assertEqual(len(kit["Kick Abbrand 1"].samples), 1)
        perc = kit["Perc[128]  Abbrand"]
        self.assertGreater(len(perc.samples), 1)
        self.assertEqual({Path(sample.stored_path).name for sample in perc.samples}, {"Perc[128] Abbrand.wav"})
        self.assertGreater(len({sample.root for sample in perc.samples}), 1)
        self.assertIn("Arp Tired", names)
        arp = next(sound for sound in project.groups[2].sounds if sound.name == "Arp Tired")
        self.assertTrue(arp.unsupported)
        self.assertFalse(any(name.startswith("Pattern") or name.startswith("Scene") for name in names))
        for needle in (
            "Bass G Kira.wav",
            "Bright Kira C1.wav",
            "Kick Abbrand 1.wav",
            "GowlerMusic - Sample Pack Movie Dialogue - Cold War #GM0014 - 05 America Many Things.wav",
        ):
            self.assertTrue(
                any(sample.resolved and sample.resolved.name == needle for group in project.groups for sound in group.sounds for sample in sound.samples),
                needle,
            )

    def test_export_copies_the_example_samples(self):
        source = EXAMPLE.read_bytes()
        template = TEMPLATE.read_bytes()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            report = tool.export_preset(tool.read_project(EXAMPLE), root / "out", root / "card")
            self.assertEqual(EXAMPLE.read_bytes(), source)
            self.assertEqual(TEMPLATE.read_bytes(), template)
            preset_text = (report.output / "preset.xml").read_text()
            self.assertTrue((report.samples / "Meat Kira.wav").is_file())
            self.assertTrue((report.samples / "Bright Kira C1.wav").is_file())
            self.assertTrue((report.samples / "Kick Abbrand 1.wav").is_file())
            self.assertIn("\\america-is-many-things\\Bright Kira C1.wav", preset_text)
            self.assertIn("\\america-is-many-things\\Kick Abbrand 1.wav", preset_text)
            for cell in ET.parse(report.output / "preset.xml").getroot().iter("cell"):
                if cell.get("layer") == "0" and cell.get("filename"):
                    self.assertTrue(cell.get("filename").endswith(".wav"), cell.get("filename"))
                    self.assertNotEqual(cell.get("filename"), r"\america-is-many-things")
            america = next(report.samples.glob("*America Many Things.wav"))
            source_america = next((EXAMPLE.parent / "america-is-many-things Samples").rglob("*America Many Things.wav"))
            self.assertLess(tool._pcm(america)[2], tool._pcm(source_america)[2])
            over = report.samples / "81 Over Seas 2.wav"
            self.assertEqual(tool._pcm(over)[2], 34111)
            kick = report.samples / "Kick Abbrand 1.wav"
            source_kick = next((EXAMPLE.parent / "america-is-many-things Samples").rglob("Kick Abbrand 1.wav"))
            self.assertGreater(tool._pcm(kick)[2], tool._pcm(source_kick)[2] - 100)
            bright = report.samples / "Bright Kira C1.wav"
            source_bright = next((EXAMPLE.parent / "america-is-many-things Samples").rglob("Bright Kira C1.wav"))
            self.assertEqual(tool._pcm(bright)[2], tool._pcm(source_bright)[2])
            perc_files = list(report.samples.glob("Perc*.wav"))
            self.assertTrue(perc_files)
            source_perc = next(
                path
                for path in (EXAMPLE.parent / "america-is-many-things Samples").rglob("*.wav")
                if path.name.startswith("Perc")
            )
            self.assertLess(tool._pcm(perc_files[0])[2], tool._pcm(source_perc)[2] // 4)
            rob = sorted(report.samples.glob("*Rob Freedom*.wav"), key=lambda item: item.stat().st_size)
            self.assertGreaterEqual(len(rob), 2)
            self.assertNotEqual(rob[0].stat().st_size, rob[-1].stat().st_size)
            kit = _pad(report.output / "preset.xml", 0, 2)
            self.assertIn("Abbrand Kit", kit.get("filename"))
            self.assertIn("Arp Tired", report.unsupported)
            self.assertIn("ccnum=\"30\"", preset_text)
            self.assertEqual(preset_text.count("ccnum=\"31\""), 2)
            self.assertNotIn("ccnum=\"111\"", preset_text)
            preset_folder = root / "card" / "PRESETS" / "america-is-many-things"
            self.assertEqual([item.name for item in preset_folder.iterdir()], ["preset.xml"])
            self.assertTrue((root / "card" / "america-is-many-things" / "Kick Abbrand 1.wav").is_file())
            self.assertFalse(list((root / "card" / "america-is-many-things").glob("*.xml")))
        reread = tool.read_project(EXAMPLE)
        self.assertFalse(any(sound.name == "Arp Tired" and sound.samples for group in reread.groups for sound in group.sounds))


if __name__ == "__main__":
    unittest.main()
