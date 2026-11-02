#!/usr/bin/env python3
"""Read a Maschine .mxprj and write a Bitbox Micro preset from the host template.

The project file is a Native Instruments document. Names and sample paths are
length-prefixed text. Sample files are resolved from the sibling
`<project name> Samples` folder, not from the absolute path stored in the file.
"""

from __future__ import annotations

import argparse
import re
import shutil
import struct
import tempfile
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
TEMPLATE = REPO_ROOT / "presets" / "host-template" / "preset.xml"

# Row, column on layer 0 for MIDI channels 1–8.
PAD_SLOTS = (
    (0, 0),
    (0, 1),
    (0, 2),
    (0, 3),
    (1, 0),
    (1, 1),
    (1, 2),
    (1, 3),
)
KIT_NOTE = 36
MAX_KIT_NOTES = 16
# Group C on the host is a natural minor scale from note 48. The other groups
# send notes 36–51, one pad at a time.
SCALE_SLOT = 2
SCALE_ROOT = 48
SCALE_STEPS = (0, 2, 3, 5, 7, 8, 10)
PLUGIN_ID = re.compile(r":(VST3?|AU|MF):")
SKIP_NAME = re.compile(r"^(Pattern \d+|Scene \d+|None|Default)$")
NOTE_IN_NAME = re.compile(r"(?:^|[^A-Za-z])([A-G])([#b])?(-?\d)(?!\d)")
SEMITONE = {
    "C": 0,
    "C#": 1,
    "DB": 1,
    "D": 2,
    "D#": 3,
    "EB": 3,
    "E": 4,
    "F": 5,
    "F#": 6,
    "GB": 6,
    "G": 7,
    "G#": 8,
    "AB": 8,
    "A": 9,
    "A#": 10,
    "BB": 10,
    "B": 11,
}
# Classes that are effects or bookkeeping, not a playable instrument.
EFFECT_CLASS = {
    "Chorus",
    "LoFi",
    "PlateReverb",
    "Flanger",
    "Iceverb",
    "BeatDelay",
    "Compressor",
    "EQ",
    "Maximizer",
    "MullhollandDrive",
    "NoteEvent",
    "MidiEvent",
    "IntModulationEvent",
    "PluginHost",
    "Sampler",
    "FM",
}


@dataclass
class Sample:
    stored_path: str
    root: int | None
    resolved: Path | None = None
    start: int | None = None
    end: int | None = None


@dataclass
class Sound:
    name: str
    samples: list[Sample] = field(default_factory=list)
    unsupported: bool = False


@dataclass
class Group:
    name: str | None
    sounds: list[Sound] = field(default_factory=list)

    def sampled_sounds(self) -> list[Sound]:
        return [sound for sound in self.sounds if sound.samples]


@dataclass
class Project:
    source: Path
    groups: list[Group]
    missing: list[str]


@dataclass
class Placement:
    sound: Sound
    sample: Sample
    root: int
    low: int
    high: int


@dataclass
class ExportReport:
    output: Path
    samples: Path
    placed_groups: list[str]
    skipped_groups: list[str]
    held_sounds: list[str]
    overflow_sounds: list[str]
    unsupported: list[str]
    missing: list[str]


def encode_string(text: str) -> bytes:
    """Length-prefixed text, the same framing the reader expects.

    Paths longer than 255 bytes are split and joined again on read.
    """
    raw = text.encode("utf-8")
    chunks = []
    while raw:
        piece = raw[:255]
        raw = raw[255:]
        chunks.append(bytes((0, 1, len(piece))) + piece)
    return b"".join(chunks)


def encode_root(note: int) -> bytes:
    return bytes((0, 1, note, 1, note, 1, note))


def encode_region(start: int, end: int) -> bytes:
    """Repeated tagged frame counts, matching a Maschine sample zone."""

    def tagged(value: int) -> bytes:
        if value < 65536:
            chunk = bytes((2,)) + value.to_bytes(2, "little")
        else:
            chunk = bytes((3,)) + value.to_bytes(3, "little")
        return chunk + chunk

    return tagged(start) + tagged(end)


def iter_strings(data: bytes):
    index = 0
    size = len(data)
    while index < size - 3:
        if data[index] == 0 and data[index + 1] == 1:
            length = data[index + 2]
            start = index + 3
            if 1 <= length <= 255 and start + length <= size:
                raw = data[start : start + length]
                if _printable(raw):
                    text = raw.decode("utf-8")
                    end = start + length
                    while length == 255 and end + 3 <= size and data[end] == 0 and data[end + 1] == 1:
                        extra = data[end + 2]
                        more = data[end + 3 : end + 3 + extra]
                        if len(more) != extra or not _printable(more):
                            break
                        text += more.decode("utf-8")
                        end = end + 3 + extra
                        length = extra
                    yield start, end, text
                    index = end
                    continue
        index += 1


def _printable(raw: bytes) -> bool:
    return all(32 <= byte < 127 for byte in raw)


def note_from_filename(name: str) -> int | None:
    """Read a note name with C4 as MIDI note 60."""
    found = list(NOTE_IN_NAME.finditer(Path(name).name))
    if not found:
        return None
    match = found[-1]
    accidental = (match.group(2) or "").upper()
    pitch = SEMITONE[match.group(1).upper() + accidental]
    octave = int(match.group(3))
    return 60 + (octave - 4) * 12 + pitch


def stored_root(data: bytes, end: int) -> int | None:
    """First root stored beside a path.

    The file repeats a root as `00 01 NN 01 NN 01 ..`. An earlier pair of the
    same shape is followed by zero bytes and is not the root.
    """
    window = data[end : end + 450]
    for offset in range(len(window) - 5):
        if (
            window[offset] == 0
            and window[offset + 1] == 1
            and window[offset + 3] == 1
            and window[offset + 5] == 1
            and window[offset + 2] == window[offset + 4]
            and 12 <= window[offset + 2] <= 127
        ):
            return window[offset + 2]
    return None


def play_region(window: bytes) -> tuple[int, int] | None:
    """Start and end frames stored beside a sample path.

    A play region is a repeated tagged integer, optionally with a smaller
    single integer just before it. The file length that follows is not the end
    of the edit. Returns inclusive frame indexes, or None when the zone does
    not store one.
    """
    repeated: list[tuple[int, int]] = []
    single_before: int | None = None
    offset = 0
    size = len(window)
    while offset < size - 3:
        tag = window[offset]
        if tag not in (2, 3):
            offset += 1
            continue
        width = 2 if tag == 2 else 3
        chunk = window[offset : offset + 1 + width]
        if len(chunk) < 1 + width:
            break
        value = int.from_bytes(chunk[1:], "little")
        follows = window[offset + len(chunk) : offset + 2 * len(chunk)]
        if follows == chunk and value < 5_000_000:
            if not repeated or offset - repeated[-1][0] <= 48:
                repeated.append((offset, value))
            offset += 2 * len(chunk)
            continue
        if (
            not repeated
            and tag == 2
            and offset + 1 + width < size
            and window[offset + 1 + width] == 0
            and 0 < value < 5_000_000
            and not _field_id(window, offset, value)
            and not _near_other_tag(window, offset, 2)
        ):
            single_before = value
            offset += 1 + width
            continue
        offset += 1
    if len(repeated) >= 2 and repeated[1][1] > repeated[0][1]:
        return repeated[0][1], repeated[1][1]
    if repeated:
        end = repeated[0][1]
        start = single_before if single_before is not None and single_before < end else 0
        return start, end
    return None


def _near_other_tag(window: bytes, offset: int, tag: int) -> bool:
    """A lone start count does not sit in a run of field ids of the same tag."""
    start = max(0, offset - 16)
    stop = min(len(window), offset + 16)
    for pos in range(start, stop):
        if offset <= pos < offset + 3:
            continue
        if window[pos] == tag:
            return True
    return False


def _field_id(window: bytes, offset: int, value: int) -> bool:
    """Field ids are a run of tagged integers that step by one."""
    start = max(0, offset - 12)
    stop = min(len(window) - 3, offset + 12)
    for pos in range(start, stop):
        if pos == offset or window[pos] != 2:
            continue
        other = int.from_bytes(window[pos + 1 : pos + 3], "little")
        if abs(other - value) == 1:
            return True
    return False


def _candidate(text: str) -> bool:
    if text.startswith("/") or text.startswith("NI::"):
        return False
    if "/" in text or "\\" in text:
        return False
    if SKIP_NAME.match(text) or PLUGIN_ID.search(text):
        return False
    if not any(character.isalpha() for character in text):
        return False
    return 2 <= len(text) <= 160


def _engine_name(text: str) -> str | None:
    prefix = "NI::MASCHINE::DATA::"
    if not text.startswith(prefix):
        return None
    class_name = text[len(prefix) :]
    if class_name in EFFECT_CLASS:
        return None
    return class_name


def read_project(path: Path) -> Project:
    data = path.read_bytes()
    strings = list(iter_strings(data))
    paths = [(start, end, text) for start, end, text in strings if text.startswith("/") and text.endswith(".wav")]
    plugins = [start for start, _, text in strings if PLUGIN_ID.search(text)]
    patterns = [start for start, _, text in strings if re.fullmatch(r"Pattern \d+", text)]
    candidates = [(start, text) for start, _, text in strings if _candidate(text)]
    first_wav = paths[0][0] if paths else len(data)

    def nearest(points: list[int], after: int, before: int) -> int | None:
        for point in points:
            if after < point < before:
                return point
        return None

    anchors: list[tuple[int, str, bool]] = []
    for index, (pos, name) in enumerate(candidates):
        if pos < first_wav - 5000:
            continue
        next_candidate = candidates[index + 1][0] if index + 1 < len(candidates) else len(data)
        limit = min(pos + 500, next_candidate)
        wav = nearest([start for start, _, _ in paths], pos, limit)
        plugin = nearest(plugins, pos, limit)
        engine = None
        for start, _, text in strings:
            if pos < start < limit and _engine_name(text):
                engine = text
                break
        if wav or plugin or engine:
            anchors.append((pos, name, wav is not None))

    sounds: list[tuple[int, Sound]] = []
    for index, (pos, name, has_wav) in enumerate(anchors):
        next_pos = anchors[index + 1][0] if index + 1 < len(anchors) else len(data)
        owned = []
        if has_wav:
            for start, end, text in paths:
                if pos < start < next_pos:
                    region = play_region(data[end : end + 240])
                    owned.append(
                        Sample(
                            text,
                            stored_root(data, end),
                            None,
                            region[0] if region else None,
                            region[1] if region else None,
                        )
                    )
        sounds.append((pos, Sound(name, _collapse(owned), unsupported=not owned)))

    groups = _group_sounds(sounds, patterns)
    missing: list[str] = []
    for group in groups:
        for sound in group.sounds:
            for sample in sound.samples:
                sample.resolved = resolve_sample(path, sample.stored_path)
                if sample.resolved is None:
                    missing.append(sample.stored_path)
    return Project(path, groups, missing)


def _collapse(samples: list[Sample]) -> list[Sample]:
    """Keep one entry when the same file and region have no distinct root note."""
    kept: list[Sample] = []
    seen: dict[tuple, set[int | None]] = {}
    for sample in samples:
        file_key = (Path(sample.stored_path).name, sample.start, sample.end)
        roots = seen.setdefault(file_key, set())
        if sample.root in roots:
            continue
        if roots and sample.root is None:
            continue
        roots.add(sample.root)
        kept.append(sample)
    return kept


def _group_sounds(sounds, patterns) -> list[Group]:
    grouped: list[list[Sound]] = []
    current: list[Sound] = []
    pattern_index = 0
    for pos, sound in sounds:
        while pattern_index < len(patterns) and patterns[pattern_index] < pos:
            if current:
                grouped.append(current)
                current = []
            pattern_index += 1
        current.append(sound)
    if current:
        grouped.append(current)

    groups: list[Group] = []
    for items in grouped:
        name = None
        # A trailing plugin with no sample, after real sounds, is the group name
        # (the example stores "Abbrand Kit" that way).
        if len(items) >= 2 and items[-1].unsupported and any(item.samples for item in items[:-1]):
            name = items[-1].name
            items = items[:-1]
        groups.append(Group(name, items))
    return groups


def resolve_sample(project_path: Path, stored_path: str) -> Path | None:
    """Find a wav under the sibling sample folder.

    The same file name can sit in the folder and again inside a subdirectory.
    The stored path says which one. The offsets beside that path belong to
    that file, so a shorter copy with the same name must not be used instead.
    """
    folder = project_path.parent / f"{project_path.stem} Samples"
    if not folder.is_dir():
        return None
    name = Path(stored_path).name
    matches = []
    direct = folder / name
    if direct.is_file():
        matches.append(direct)
    for child in sorted(folder.iterdir()):
        if child.is_dir() and (child / name).is_file():
            matches.append(child / name)
    if not matches:
        return None
    stored = stored_path.replace("\\", "/")

    def suffix_length(candidate: Path) -> int:
        relative = candidate.relative_to(folder).as_posix()
        return len(relative) if stored.endswith(relative) else 0

    return max(matches, key=suffix_length)


def distinct_files(sound: Sound) -> list[str]:
    seen = []
    for sample in sound.samples:
        name = Path(sample.stored_path).name
        if name not in seen:
            seen.append(name)
    return seen


def is_multisample(sound: Sound) -> bool:
    """A multi-sample has several files and a root for more than one of them.

    The same wav repeated on different roots is still one file, so a drum pad
    keeps that sound as a single kit note. Two files with no recoverable root
    cannot be laid out as key zones, so they stay one shot as well.
    """
    if len(distinct_files(sound)) < 2:
        return False
    roots = {effective_root(sample) for sample in sound.samples}
    roots.discard(None)
    return len(roots) >= 2


def effective_root(sample: Sample) -> int | None:
    if sample.root is not None:
        return sample.root
    return note_from_filename(sample.stored_path)


def _zones(samples: list[Sample]) -> list[tuple[Sample, int, int, int]]:
    keyed = []
    for sample in samples:
        root = effective_root(sample)
        if root is None:
            continue
        keyed.append((sample, root))
    keyed.sort(key=lambda item: item[1])
    zones = []
    for index, (sample, root) in enumerate(keyed):
        if index == 0:
            low = 0
        else:
            low = zones[-1][3] + 1
        if index == len(keyed) - 1:
            high = 127
        else:
            next_root = keyed[index + 1][1]
            high = min(max(root, (root + next_root) // 2), next_root - 1)
        zones.append((sample, root, low, high))
    return zones


def slot_notes(slot: int) -> list[int]:
    """The sixteen notes this host pad actually sends."""
    if slot == SCALE_SLOT:
        notes = []
        for pad in range(MAX_KIT_NOTES):
            octave, degree = divmod(pad, len(SCALE_STEPS))
            notes.append(SCALE_ROOT + SCALE_STEPS[degree] + octave * 12)
        return notes
    return [KIT_NOTE + index for index in range(MAX_KIT_NOTES)]


def plan_group(group: Group, notes: list[int] | None = None) -> tuple[list[Placement], list[str], list[str]]:
    """Return placements, sounds held back, and sounds past the 16-note kit."""
    pad_notes = list(notes) if notes is not None else slot_notes(0)
    sampled = group.sampled_sounds()
    multis = [sound for sound in sampled if is_multisample(sound)]
    ones = [sound for sound in sampled if not is_multisample(sound)]
    if multis:
        held = [sound.name for sound in ones]
        held.extend(sound.name for sound in multis[1:])
        placements = []
        for sample, root, low, high in _zones(multis[0].samples):
            placements.append(Placement(multis[0], sample, root, low, high))
        return placements, held, []
    placements = []
    overflow = []
    for index, sound in enumerate(ones):
        if index >= len(pad_notes):
            overflow.append(sound.name)
            continue
        sample = next((item for item in sound.samples if item.resolved), sound.samples[0])
        note = pad_notes[index]
        placements.append(Placement(sound, sample, note, note, note))
    for index, placement in enumerate(placements):
        placement.low = 0 if index == 0 else placement.root
        if index + 1 < len(placements):
            placement.high = placements[index + 1].root - 1
        else:
            placement.high = 127
        if placement.high < placement.low:
            placement.high = placement.low
    return placements, [], overflow


def export_preset(
    project: Project,
    output_dir: Path | None = None,
    dest: Path | None = None,
    template: Path = TEMPLATE,
    names: dict[str, str] | None = None,
) -> ExportReport:
    if output_dir is None and dest is None:
        raise SystemExit("Give --out, --dest, or both")
    template_bytes = template.read_bytes()
    folder_name = project.source.stem
    staging = tempfile.TemporaryDirectory()
    stage = Path(staging.name)
    sample_dir = stage / folder_name
    preset_dir = stage / "preset"
    sample_dir.mkdir()
    preset_dir.mkdir()

    root = ET.fromstring(template_bytes)
    session = root.find("session")
    if session is None:
        raise SystemExit(f"{template} has no session")
    for cell in list(session):
        if cell.get("type") == "asset":
            session.remove(cell)

    placed_names: list[str] = []
    skipped: list[str] = []
    held: list[str] = []
    overflow: list[str] = []
    unsupported: list[str] = []
    used: dict[tuple, str] = {}
    stems: set[str] = set()
    asset_row = 0
    slot_index = 0
    labels = names or {}

    for group in project.groups:
        label = group.name or _fallback_label(group)
        for sound in group.sounds:
            if sound.unsupported:
                unsupported.append(sound.name)
        if not group.sampled_sounds():
            skipped.append(label)
            continue
        placements, held_here, overflow_here = plan_group(group, slot_notes(slot_index))
        playable = [item for item in placements if item.sample.resolved is not None]
        if slot_index >= len(PAD_SLOTS):
            skipped.append(label)
            continue
        if not playable:
            skipped.append(label)
            continue
        row, column = PAD_SLOTS[slot_index]
        slot_index += 1
        placed_names.append(label)
        held.extend(f"{label}: {name}" for name in held_here)
        overflow.extend(f"{label}: {name}" for name in overflow_here)
        override = labels.get(str(slot_index)) or labels.get(label)
        written = _fill_pad(
            session,
            row,
            column,
            folder_name,
            playable,
            used,
            stems,
            sample_dir,
            _clip_name(group, playable, override),
        )
        asset_row = _add_assets(session, asset_row, row, column, folder_name, playable, written)

    for index in range(slot_index, len(PAD_SLOTS)):
        row, column = PAD_SLOTS[index]
        _clear_pad(session, row, column)

    tree = ET.ElementTree(root)
    ET.indent(tree, space="    ")
    tree.write(preset_dir / "preset.xml", encoding="UTF-8", xml_declaration=True)
    installed = None
    if output_dir is not None:
        installed = _install_card(sample_dir, preset_dir / "preset.xml", Path(output_dir), folder_name)
    if dest is not None:
        copied = _install_card(sample_dir, preset_dir / "preset.xml", Path(dest), folder_name)
        if installed is None:
            installed = copied
    staging.cleanup()
    preset_out, sample_out = installed
    return ExportReport(
        preset_out,
        sample_out,
        placed_names,
        skipped,
        held,
        overflow,
        unsupported,
        list(project.missing),
    )


def _fallback_label(group: Group) -> str:
    if group.sounds:
        return group.sounds[0].name
    return "unnamed"


def _clip_name(group: Group, placements: list[Placement], override: str | None) -> str:
    if override:
        return _safe_stem(override)
    names = []
    for placement in placements:
        if placement.sound.name not in names:
            names.append(placement.sound.name)
    if len(names) > 1 and group.name:
        return _safe_stem(short_stem(group.name))
    if len(names) == 1 and len(placements) > 1 and is_multisample(placements[0].sound):
        return _safe_stem(short_stem(names[0]))
    return _zone_stem(placements[0], len(placements) > 1)


def _fill_pad(
    session,
    row: int,
    column: int,
    folder_name: str,
    placements: list[Placement],
    used: dict[tuple, str],
    stems: set[str],
    output: Path,
    clip_name: str,
) -> dict[int, str]:
    cell = _cell(session, row, column)
    params = cell.find("params")
    cell.set("type", "sample")
    if params is not None:
        params.set("multisammode", "1")
        params.set("polymode", "2")
        params.set("deftemplate", "0")
        params.set("panpos", "-1000")
        params.set("rootnote", str(placements[0].root))
    written: dict[int, str] = {}
    several = len(placements) > 1
    for placement in placements:
        source = placement.sample.resolved
        if source is None:
            continue
        key = (source, placement.sample.start, placement.sample.end)
        if key not in used:
            stem = _unique_stem(_zone_stem(placement, several), stems)
            filename = stem + ".wav"
            _write_sample(source, output / filename, placement.sample.start, placement.sample.end)
            used[key] = filename
        written[id(placement)] = used[key]
    primary = written[id(placements[0])]
    display = clip_name + ".wav"
    if display != primary:
        display = _unique_stem(Path(display).stem, stems) + ".wav"
        shutil.copy2(output / primary, output / display)
    cell.set("filename", "\\" + folder_name + "\\" + display)
    return written


def _zone_stem(placement: Placement, several: bool) -> str:
    sample_stem = Path(placement.sample.stored_path).stem
    if several and is_multisample(placement.sound):
        return _safe_stem(sample_stem)
    sound = placement.sound.name.strip()
    if len(sound) > 40 or " - " in sound or sound == sample_stem:
        return _safe_stem(short_stem(sound))
    return _safe_stem(sound)


def short_stem(name: str) -> str:
    stem = Path(name).stem
    if " - " in stem:
        stem = stem.split(" - ")[-1].strip()
    return stem or "sample"


def _safe_stem(name: str) -> str:
    cleaned = re.sub(r'[<>:"/\\|?*]', "", name).strip()
    return cleaned[:48] or "sample"


def _unique_stem(stem: str, stems: set[str]) -> str:
    if stem not in stems:
        stems.add(stem)
        return stem
    number = 2
    while f"{stem} {number}" in stems:
        number += 1
    chosen = f"{stem} {number}"
    stems.add(chosen)
    return chosen


def _write_sample(source: Path, dest: Path, start: int | None, end: int | None) -> None:
    parsed = _pcm(source)
    if parsed is None or start is None or end is None:
        shutil.copy2(source, dest)
        return
    fmt, pcm, frames = parsed
    frame = fmt["channels"] * (fmt["bits"] // 8)
    start = max(0, start)
    end = min(frames - 1, end)
    # A repeated count within a few milliseconds of the file length is the
    # whole take or a loop marker, not a slice. A real edit starts later, or
    # ends well before the file.
    if end - start < 32 or (start <= 1 and frames - 1 - end < 200):
        shutil.copy2(source, dest)
        return
    # A bounced excerpt already starts where the project marker points in the
    # longer recording. Trimming it again drops the beginning of the phrase.
    if frames - 1 - end <= 2 and _excerpt_already_starts_at(source, pcm, frame, start):
        shutil.copy2(source, dest)
        return
    if end < start:
        shutil.copy2(source, dest)
        return
    piece = pcm[start * frame : (end + 1) * frame]
    _write_wav(dest, fmt, piece)


def _same_name_files(path: Path) -> list[Path]:
    found = []
    parent = path.parent
    if (parent / path.name).is_file():
        found.append(parent / path.name)
    for child in parent.iterdir():
        if child.is_dir() and (child / path.name).is_file():
            found.append(child / path.name)
    if parent.parent is not None and (parent.parent / path.name).is_file():
        found.append(parent.parent / path.name)
    own = path.resolve()
    return [item for item in found if item.resolve() != own]


def _excerpt_already_starts_at(source: Path, pcm: bytes, frame_bytes: int, start: int) -> bool:
    window = min(2000, len(pcm) // frame_bytes)
    if window < 64:
        return False
    excerpt = pcm[: window * frame_bytes]
    for other in _same_name_files(source):
        parsed = _pcm(other)
        if not parsed:
            continue
        _, other_pcm, other_frames = parsed
        for delta in range(-8, 9):
            at = start + delta
            if at < 0 or at + window > other_frames:
                continue
            piece = other_pcm[at * frame_bytes : (at + window) * frame_bytes]
            if piece == excerpt:
                return True
    return False


def _pcm(path: Path):
    blob = path.read_bytes()
    if blob[:4] != b"RIFF" or blob[8:12] != b"WAVE":
        return None
    position = 12
    fmt = None
    data = None
    while position + 8 <= len(blob):
        chunk = blob[position : position + 4]
        size = int.from_bytes(blob[position + 4 : position + 8], "little")
        body = blob[position + 8 : position + 8 + size]
        if chunk == b"fmt " and len(body) >= 16:
            audio, channels, rate, _, _, bits = struct.unpack_from("<HHIIHH", body)
            fmt = {"audio": audio, "channels": channels, "rate": rate, "bits": bits}
        elif chunk == b"data":
            data = body
            break
        position += 8 + size + (size & 1)
    if not fmt or data is None or fmt["audio"] != 1 or fmt["bits"] not in (16, 24, 32):
        return None
    frame = fmt["channels"] * (fmt["bits"] // 8)
    if frame <= 0:
        return None
    return fmt, data, len(data) // frame


def _write_wav(path: Path, fmt: dict, pcm: bytes) -> None:
    frame = fmt["channels"] * (fmt["bits"] // 8)
    fmt_body = struct.pack(
        "<HHIIHH",
        1,
        fmt["channels"],
        fmt["rate"],
        fmt["rate"] * frame,
        frame,
        fmt["bits"],
    )
    riff_size = 4 + (8 + len(fmt_body)) + (8 + len(pcm))
    blob = b"RIFF" + riff_size.to_bytes(4, "little") + b"WAVE"
    blob += b"fmt " + len(fmt_body).to_bytes(4, "little") + fmt_body
    blob += b"data" + len(pcm).to_bytes(4, "little") + pcm
    path.write_bytes(blob)


def _preset_root(root: Path) -> Path:
    """The card's preset directory. Reuse PRESETS or Presets when one is already there."""
    if root.is_dir():
        for child in root.iterdir():
            if child.is_dir() and child.name.lower() == "presets":
                return child
    return root / "PRESETS"


def _install_card(sample_dir: Path, preset_xml: Path, root: Path, folder_name: str) -> tuple[Path, Path]:
    """Samples go in ``root/<name>/``. Only preset.xml goes in ``root/PRESETS/<name>/``."""
    sample_dest = root / folder_name
    preset_dest = _preset_root(root) / folder_name
    if sample_dest.exists() and preset_dest.exists() and sample_dest.resolve() == preset_dest.resolve():
        raise SystemExit(f"Refusing to mix samples and preset.xml in {sample_dest}")
    if sample_dest.exists():
        shutil.rmtree(sample_dest)
    sample_dest.mkdir(parents=True)
    for wav in sample_dir.iterdir():
        if wav.is_file() and wav.suffix.lower() == ".wav":
            shutil.copy2(wav, sample_dest / wav.name)
    if preset_dest.exists():
        shutil.rmtree(preset_dest)
    preset_dest.mkdir(parents=True)
    shutil.copy2(preset_xml, preset_dest / "preset.xml")
    return preset_dest, sample_dest


def _add_assets(session, asset_row, pad_row, pad_column, folder_name, placements: list[Placement], names: dict[int, str]) -> int:
    insert_at = len(session)
    for index, cell in enumerate(list(session)):
        if cell.get("type") == "ioconnectin":
            insert_at = index
            break
    for placement in placements:
        if placement.sample.resolved is None:
            continue
        filename = "\\" + folder_name + "\\" + names[id(placement)]
        asset = ET.Element("cell", {"row": str(asset_row), "filename": filename, "type": "asset"})
        ET.SubElement(
            asset,
            "params",
            {
                "rootnote": str(placement.root),
                "keyrangebottom": str(placement.low),
                "keyrangetop": str(placement.high),
                "velroot": "0",
                "velrangebottom": "0",
                "velrangetop": "128",
                "asssrcrow": str(pad_row),
                "asssrccol": str(pad_column),
            },
        )
        session.insert(insert_at, asset)
        insert_at += 1
        asset_row += 1
    return asset_row


def _clear_pad(session, row: int, column: int) -> None:
    cell = _cell(session, row, column)
    params = cell.find("params")
    if params is not None:
        params.set("midimode", "0")
    for mod in list(cell):
        if mod.tag == "modsource" and mod.get("src") == "midicc":
            cell.remove(mod)


def _cell(session, row: int, column: int):
    for cell in session:
        if (
            cell.get("layer") == "0"
            and cell.get("row") == str(row)
            and cell.get("column") == str(column)
        ):
            return cell
    raise SystemExit(f"template is missing pad row {row} column {column}")


def load_names(path: Path) -> dict[str, str]:
    mapping: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        label = value.strip()
        if label:
            mapping[key.strip()] = label
    return mapping


def prompt_names(project: Project) -> dict[str, str]:
    """Ask for a clip label per placed pad. An empty line keeps the automatic name."""
    chosen: dict[str, str] = {}
    slot = 0
    print("Clip names. Press enter to keep the suggested name.")
    for group in project.groups:
        if not group.sampled_sounds():
            continue
        slot += 1
        if slot > len(PAD_SLOTS):
            break
        placements, _, _ = plan_group(group)
        playable = [item for item in placements if item.sample.resolved is not None]
        if not playable:
            continue
        suggested = _clip_name(group, playable, None)
        group_label = group.name or _fallback_label(group)
        entered = input(f"  Pad {slot} {group_label} [{suggested}]: ").strip()
        if entered:
            chosen[str(slot)] = entered
    return chosen


def _print_report(report: ExportReport) -> None:
    print(f"Wrote samples {report.samples}")
    print(f"Wrote preset {report.output / 'preset.xml'}")
    print("Pads: " + ", ".join(report.placed_groups))
    if report.skipped_groups:
        print("Not placed: " + ", ".join(report.skipped_groups))
    if report.unsupported:
        print("Unsupported: " + ", ".join(report.unsupported))
    if report.held_sounds:
        print("Held back for a multi-sample pad: " + ", ".join(report.held_sounds))
    if report.overflow_sounds:
        print("Past 16 kit notes: " + ", ".join(report.overflow_sounds))
    if report.missing:
        print("Missing samples:")
        for stored in report.missing:
            print(f"  {stored}")


def main(argv: list[str] | None = None) -> ExportReport:
    parser = argparse.ArgumentParser(description="Convert a Maschine project to a Bitbox Micro preset.")
    parser.add_argument("project", type=Path, help="Path to a .mxprj file")
    parser.add_argument(
        "--out",
        type=Path,
        help="Card-root mirror: wavs in <name>/, and only preset.xml in PRESETS/<name>/",
    )
    parser.add_argument(
        "--dest",
        type=Path,
        help="Card root. Wavs go in <name>/ and only preset.xml goes in PRESETS/<name>/",
    )
    parser.add_argument(
        "--names",
        type=Path,
        help="Clip labels, one per line as a pad number or group name, then =, then the label",
    )
    parser.add_argument(
        "--interactive",
        action="store_true",
        help="Prompt for a clip label on each placed pad",
    )
    parser.add_argument("--template", type=Path, default=TEMPLATE, help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    if args.out is None and args.dest is None:
        args.out = Path.cwd()
    project = read_project(args.project)
    labels = load_names(args.names) if args.names else {}
    if args.interactive:
        labels.update(prompt_names(project))
    report = export_preset(project, args.out, args.dest, args.template, labels)
    _print_report(report)
    return report


if __name__ == "__main__":
    main()
