use crate::model::{
    CcTarget, ControlCategory, Destination, EncoderKind, EncoderMapping, EncoderPage, Group,
    GroupId, HostConfig, LogLevel, MidiPortConfig, NoteScale, PadMode, PadProfile, PadType,
    ParamScope, SampleAssignment, SampleZone, ScaleName, TriggerPolicy, ValueFormat, VelocityCurve,
    ENCODER_COUNT, default_group_color, GROUP_COUNT, PAD_COUNT,
};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigIssue {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ConfigIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    schema_version: u32,
    midi_outputs: std::collections::BTreeMap<String, RawPort>,
    groups: Vec<RawGroup>,
    encoder_pages: Vec<RawPage>,
    #[serde(default)]
    log_level: Option<String>,
    #[serde(default)]
    display: Option<RawDisplay>,
    #[serde(default)]
    audio: Option<RawAudio>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawPort {
    Name(String),
    Detail {
        name: String,
        #[serde(default)]
        virtual_port: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGroup {
    id: String,
    label: String,
    mode: String,
    destination: RawDestination,
    notes: Vec<u8>,
    #[serde(default)]
    trigger: Option<String>,
    #[serde(default)]
    samples: Option<Vec<Option<RawSample>>>,
    #[serde(default)]
    zones: Option<Vec<RawZone>>,
    #[serde(default)]
    color: Option<[u16; 3]>,
    #[serde(default)]
    velocity_curve: Option<f64>,
    #[serde(default)]
    note_scale: Option<RawNoteScale>,
    #[serde(default)]
    pad_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNoteScale {
    root: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum RawDestination {
    Midi { port: String, channel: u8 },
    Sampler { instrument: String },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSample {
    path: String,
    #[serde(default)]
    choke: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawZone {
    path: String,
    root: u8,
    lo: u8,
    hi: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPage {
    id: String,
    label: String,
    encoders: Vec<RawEncoder>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    profiles: std::collections::BTreeMap<String, Vec<RawEncoder>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEncoder {
    label: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    port: Option<String>,
    #[serde(default)]
    channel: Option<u8>,
    #[serde(default)]
    cc: Option<u8>,
    #[serde(default)]
    min: Option<i32>,
    #[serde(default)]
    max: Option<i32>,
    #[serde(default)]
    default: Option<i32>,
    #[serde(default)]
    parameter: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    pad_types: Vec<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    targets: Option<Vec<RawTarget>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTarget {
    port: String,
    channel: u8,
    cc: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDisplay {
    #[serde(default)]
    min_interval_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAudio {
    #[serde(default)]
    device: Option<String>,
    #[serde(default)]
    sample_rate: Option<u32>,
    #[serde(default)]
    buffer_frames: Option<u32>,
    #[serde(default)]
    polyphony: Option<usize>,
}

pub fn load_yaml(text: &str) -> Result<HostConfig, Vec<ConfigIssue>> {
    let raw: RawConfig = serde_yaml::from_str(text).map_err(|error| {
        vec![ConfigIssue {
            path: "document".to_string(),
            message: format!("YAML is not a host configuration: {error}"),
        }]
    })?;
    validate(raw)
}

fn validate(raw: RawConfig) -> Result<HostConfig, Vec<ConfigIssue>> {
    let mut issues = Vec::new();
    if raw.schema_version != 1 {
        issues.push(issue(
            "schema_version",
            format!(
                "unsupported schema version {}; expected 1",
                raw.schema_version
            ),
        ));
    }

    let mut midi_outputs = Vec::new();
    for (id, port) in &raw.midi_outputs {
        if id.trim().is_empty() {
            issues.push(issue("midi_outputs", "port id must not be empty"));
            continue;
        }
        let (name, virtual_port) = match port {
            RawPort::Name(name) => (name.clone(), false),
            RawPort::Detail { name, virtual_port } => (name.clone(), *virtual_port),
        };
        if name.trim().is_empty() {
            issues.push(issue(
                &format!("midi_outputs.{id}"),
                "system port name must not be empty",
            ));
        }
        midi_outputs.push(MidiPortConfig {
            id: id.clone(),
            name,
            virtual_port,
        });
    }

    if raw.groups.len() != GROUP_COUNT {
        issues.push(issue(
            "groups",
            format!(
                "expected {GROUP_COUNT} groups A-H, found {}",
                raw.groups.len()
            ),
        ));
    }

    let mut groups: [Option<Group>; GROUP_COUNT] = Default::default();
    for (index, raw_group) in raw.groups.iter().enumerate() {
        let path = format!("groups[{index}]");
        let Some(id) = GroupId::parse(&raw_group.id) else {
            issues.push(issue(&path, format!("unknown group id '{}'", raw_group.id)));
            continue;
        };
        if groups[id.index()].is_some() {
            issues.push(issue(&path, format!("duplicate group {id}")));
            continue;
        }
        if raw_group.label.trim().is_empty() {
            issues.push(issue(&format!("{path}.label"), "label must not be empty"));
        }
        let Some(mode) = PadMode::parse(&raw_group.mode) else {
            issues.push(issue(
                &format!("{path}.mode"),
                format!("unknown mode '{}'", raw_group.mode),
            ));
            continue;
        };
        if raw_group.notes.len() != PAD_COUNT {
            issues.push(issue(
                &format!("{path}.notes"),
                format!(
                    "expected {PAD_COUNT} notes, found {}",
                    raw_group.notes.len()
                ),
            ));
            continue;
        }
        if raw_group.notes.iter().any(|note| *note > 127) {
            issues.push(issue(
                &format!("{path}.notes"),
                "notes must be MIDI note numbers 0-127",
            ));
            continue;
        }
        let mut notes = [0; PAD_COUNT];
        notes.copy_from_slice(&raw_group.notes);

        let destination = match &raw_group.destination {
            RawDestination::Midi { port, channel } => {
                if !midi_outputs.iter().any(|item| item.id == *port) {
                    issues.push(issue(
                        &format!("{path}.destination.port"),
                        format!("unknown MIDI port '{port}'"),
                    ));
                }
                if !(1..=16).contains(channel) {
                    issues.push(issue(
                        &format!("{path}.destination.channel"),
                        format!("channel {channel} is outside 1-16"),
                    ));
                }
                Destination::Midi {
                    port: port.clone(),
                    channel: *channel,
                }
            }
            RawDestination::Sampler { instrument } => {
                if instrument.trim().is_empty() {
                    issues.push(issue(
                        &format!("{path}.destination.instrument"),
                        "sampler instrument name must not be empty",
                    ));
                }
                Destination::Sampler {
                    instrument: instrument.clone(),
                }
            }
        };

        let trigger = match raw_group.trigger.as_deref() {
            None => default_trigger(mode, &destination),
            Some("note_off") => TriggerPolicy::NoteOff,
            Some("ignore_release") => TriggerPolicy::IgnoreRelease,
            Some(other) => {
                issues.push(issue(
                    &format!("{path}.trigger"),
                    format!("unknown trigger policy '{other}'"),
                ));
                TriggerPolicy::NoteOff
            }
        };

        let mut samples: [Option<SampleAssignment>; PAD_COUNT] = Default::default();
        if let Some(raw_samples) = &raw_group.samples {
            if raw_samples.len() != PAD_COUNT {
                issues.push(issue(
                    &format!("{path}.samples"),
                    format!(
                        "expected {PAD_COUNT} sample slots, found {}",
                        raw_samples.len()
                    ),
                ));
            } else {
                for (pad, slot) in raw_samples.iter().enumerate() {
                    if let Some(sample) = slot {
                        if sample.path.trim().is_empty() {
                            issues.push(issue(
                                &format!("{path}.samples[{pad}].path"),
                                "sample path must not be empty",
                            ));
                        }
                        samples[pad] = Some(SampleAssignment {
                            path: sample.path.clone(),
                            choke: sample.choke,
                        });
                    }
                }
            }
        }

        let mut zones = Vec::new();
        if let Some(raw_zones) = &raw_group.zones {
            for (zone_index, zone) in raw_zones.iter().enumerate() {
                let zone_path = format!("{path}.zones[{zone_index}]");
                if zone.path.trim().is_empty() {
                    issues.push(issue(
                        &format!("{zone_path}.path"),
                        "path must not be empty",
                    ));
                }
                if zone.root > 127 || zone.lo > 127 || zone.hi > 127 {
                    issues.push(issue(&zone_path, "root, lo and hi must be 0-127"));
                }
                if zone.lo > zone.hi {
                    issues.push(issue(&zone_path, "lo must be less than or equal to hi"));
                }
                zones.push(SampleZone {
                    path: zone.path.clone(),
                    root: zone.root,
                    lo: zone.lo,
                    hi: zone.hi,
                });
            }
        }

        if matches!(destination, Destination::Sampler { .. }) {
            match mode {
                PadMode::OneShot => {
                    if samples.iter().all(|slot| slot.is_none()) {
                        issues.push(issue(
                            &format!("{path}.samples"),
                            "one-shot sampler group needs at least one sample slot",
                        ));
                    }
                }
                PadMode::Chromatic | PadMode::Scale => {
                    if zones.is_empty() {
                        issues.push(issue(
                            &format!("{path}.zones"),
                            "chromatic and scale sampler groups need at least one zone",
                        ));
                    }
                }
            }
        }

        let velocity_curve = match raw_group.velocity_curve {
            None => VelocityCurve::STRAIGHT,
            Some(value) => match VelocityCurve::new(value) {
                Some(curve) => curve,
                None => {
                    issues.push(issue(
                        &format!("{path}.velocity_curve"),
                        "velocity curve must be a positive number",
                    ));
                    VelocityCurve::STRAIGHT
                }
            },
        };
        let note_scale = match &raw_group.note_scale {
            None => None,
            Some(scale) => match note_scale_for(mode, scale) {
                Ok(scale) => Some(scale),
                Err(message) => {
                    issues.push(issue(&format!("{path}.note_scale"), message));
                    None
                }
            },
        };

        let color = match raw_group.color {
            None => default_group_color(id),
            Some(channels) => {
                if channels.iter().any(|channel| *channel > 255) {
                    issues.push(issue(
                        &format!("{path}.color"),
                        "color channels must be 0-255",
                    ));
                    default_group_color(id)
                } else {
                    [channels[0] as u8, channels[1] as u8, channels[2] as u8]
                }
            }
        };

        groups[id.index()] = Some(Group {
            id,
            label: raw_group.label.clone(),
            mode,
            destination,
            notes,
            trigger,
            samples,
            zones,
            color,
            velocity_curve,
            note_scale,
            pad_type: match &raw_group.pad_type {
                None => None,
                Some(name) => match PadType::parse(name) {
                    Some(pad_type) => Some(pad_type),
                    None => {
                        issues.push(issue(
                            &format!("{path}.pad_type"),
                            format!("unknown pad type '{name}'"),
                        ));
                        None
                    }
                },
            },
        });
    }

    for id in GroupId::ALL {
        if groups[id.index()].is_none() && raw.groups.len() == GROUP_COUNT {
            issues.push(issue("groups", format!("missing group {id}")));
        }
    }

    if raw.encoder_pages.is_empty() {
        issues.push(issue(
            "encoder_pages",
            "at least one encoder page is required",
        ));
    }
    let mut pages = Vec::new();
    let mut page_ids = Vec::new();
    for (index, page) in raw.encoder_pages.iter().enumerate() {
        let path = format!("encoder_pages[{index}]");
        if page.id.trim().is_empty() {
            issues.push(issue(&format!("{path}.id"), "id must not be empty"));
        }
        if page_ids.iter().any(|id: &String| id == &page.id) {
            issues.push(issue(&path, format!("duplicate page id '{}'", page.id)));
        }
        page_ids.push(page.id.clone());
        if page.label.trim().is_empty() {
            issues.push(issue(&format!("{path}.label"), "label must not be empty"));
        }
        if page.encoders.len() != ENCODER_COUNT {
            issues.push(issue(
                &format!("{path}.encoders"),
                format!(
                    "expected {ENCODER_COUNT} encoders, found {}",
                    page.encoders.len()
                ),
            ));
            continue;
        }
        let mut encoders: [Option<EncoderMapping>; ENCODER_COUNT] = Default::default();
        for (encoder_index, encoder) in page.encoders.iter().enumerate() {
            let encoder_path = format!("{path}.encoders[{encoder_index}]");
            if encoder.label.trim().is_empty() {
                issues.push(issue(
                    &format!("{encoder_path}.label"),
                    "label must not be empty",
                ));
            }
            let min = encoder.min.unwrap_or(0);
            let max = encoder.max.unwrap_or(127);
            let default_value = encoder.default.unwrap_or(min);
            if min > max {
                issues.push(issue(
                    &encoder_path,
                    "min must be less than or equal to max",
                ));
            }
            if default_value < min || default_value > max {
                issues.push(issue(&encoder_path, "default must lie inside min and max"));
            }
            let kind = match encoder.kind.as_str() {
                "midi_cc" => {
                    let Some(port) = encoder.port.clone() else {
                        issues.push(issue(
                            &format!("{encoder_path}.port"),
                            "midi_cc requires port",
                        ));
                        continue;
                    };
                    if midi_outputs.iter().all(|item| item.id != port) {
                        issues.push(issue(
                            &format!("{encoder_path}.port"),
                            format!("unknown MIDI port '{port}'"),
                        ));
                    }
                    let channel = match encoder.channel {
                        Some(channel) => channel,
                        None => match group_midi_channel(&groups, encoder_index) {
                            Some(channel) => channel,
                            None => {
                                issues.push(issue(
                                    &format!("{encoder_path}.channel"),
                                    "omitted channel needs the group at the same index to use a MIDI channel",
                                ));
                                continue;
                            }
                        },
                    };
                    if !(1..=16).contains(&channel) {
                        issues.push(issue(
                            &format!("{encoder_path}.channel"),
                            format!("channel {channel} is outside 1-16"),
                        ));
                    }
                    if let Some(cc) = encoder.cc {
                        if cc > 127 {
                            issues.push(issue(
                                &format!("{encoder_path}.cc"),
                                format!("controller {cc} is outside 0-127"),
                            ));
                        }
                    }
                    EncoderKind::MidiCc {
                        port,
                        channel,
                        cc: encoder.cc,
                    }
                }
                "local" => {
                    let Some(target) = encoder
                        .target
                        .clone()
                        .filter(|target| !target.trim().is_empty())
                    else {
                        issues.push(issue(
                            &format!("{encoder_path}.target"),
                            "local encoder requires a target name",
                        ));
                        continue;
                    };
                    EncoderKind::Local { target }
                }
                "macro" => {
                    let Some(targets) = &encoder.targets else {
                        issues.push(issue(
                            &format!("{encoder_path}.targets"),
                            "macro encoder requires targets",
                        ));
                        continue;
                    };
                    if targets.is_empty() {
                        issues.push(issue(
                            &format!("{encoder_path}.targets"),
                            "macro encoder requires at least one target",
                        ));
                    }
                    let mut mapped = Vec::new();
                    for (target_index, target) in targets.iter().enumerate() {
                        let target_path = format!("{encoder_path}.targets[{target_index}]");
                        if midi_outputs.iter().all(|item| item.id != target.port) {
                            issues.push(issue(
                                &target_path,
                                format!("unknown MIDI port '{}'", target.port),
                            ));
                        }
                        if !(1..=16).contains(&target.channel) {
                            issues.push(issue(&target_path, "channel is outside 1-16"));
                        }
                        if target.cc > 127 {
                            issues.push(issue(&target_path, "controller is outside 0-127"));
                        }
                        mapped.push(CcTarget {
                            port: target.port.clone(),
                            channel: target.channel,
                            cc: target.cc,
                        });
                    }
                    EncoderKind::Macro { targets: mapped }
                }
                "unbound" => EncoderKind::Unbound,
                other => {
                    issues.push(issue(
                        &format!("{encoder_path}.type"),
                        format!("unknown encoder type '{other}'"),
                    ));
                    EncoderKind::Unbound
                }
            };
            let scope = match encoder.scope.as_deref() {
                None | Some("slot") => ParamScope::Slot,
                Some("global") => ParamScope::Global,
                Some("selected") => ParamScope::Selected,
                Some(other) => {
                    issues.push(issue(
                        &format!("{encoder_path}.scope"),
                        format!("unknown scope '{other}'"),
                    ));
                    ParamScope::Slot
                }
            };
            let format = match encoder.format.as_deref() {
                None | Some("number") => ValueFormat::Number,
                Some("on-off") => ValueFormat::OnOff,
                Some(other) => {
                    issues.push(issue(
                        &format!("{encoder_path}.format"),
                        format!("unknown format '{other}'"),
                    ));
                    ValueFormat::Number
                }
            };
            let mut pad_types = Vec::new();
            for name in &encoder.pad_types {
                match PadType::parse(name) {
                    Some(pad_type) => pad_types.push(pad_type),
                    None => issues.push(issue(
                        &format!("{encoder_path}.pad_types"),
                        format!("unknown pad type '{name}'"),
                    )),
                }
            }
            let parameter = encoder
                .parameter
                .as_ref()
                .filter(|name| !name.trim().is_empty())
                .cloned();
            encoders[encoder_index] = Some(EncoderMapping {
                label: encoder.label.clone(),
                kind,
                min,
                max,
                default_value,
                parameter,
                scope,
                format,
                pad_types,
                inactive: false,
            });
        }
        if encoders.iter().all(|item| item.is_some()) {
            let encoders = encoders.map(|item| item.expect("checked"));
            let mut profiles = Vec::new();
            for (name, raw_encoders) in &page.profiles {
                let Some(pad_type) = PadType::parse(name) else {
                    issues.push(issue(
                        &format!("{path}.profiles.{name}"),
                        format!("unknown pad type '{name}'"),
                    ));
                    continue;
                };
                if raw_encoders.len() != 4 {
                    issues.push(issue(
                        &format!("{path}.profiles.{name}"),
                        "a pad profile needs four encoders",
                    ));
                    continue;
                }
                let mut built = [None, None, None, None];
                for (profile_index, raw_encoder) in raw_encoders.iter().enumerate() {
                    built[profile_index] = profile_encoder(
                        raw_encoder,
                        &format!("{path}.profiles.{name}[{profile_index}]"),
                        &midi_outputs,
                        &mut issues,
                    );
                }
                if built.iter().all(Option::is_some) {
                    profiles.push(PadProfile {
                        pad_type,
                        encoders: built.map(|item| item.expect("checked")),
                    });
                }
            }
            pages.push(EncoderPage {
                id: page.id.clone(),
                label: page.label.clone(),
                category: page.category.clone().filter(|name| !name.trim().is_empty()),
                encoders,
                profiles,
            });
        }
    }

    let mut categories = Vec::new();
    for (index, page) in pages.iter().enumerate() {
        let Some(name) = page.category.clone() else {
            continue;
        };
        if let Some(category) = categories
            .iter_mut()
            .find(|category: &&mut ControlCategory| category.id == name)
        {
            category.pages.push(index);
        } else {
            categories.push(ControlCategory {
                id: name,
                pages: vec![index],
            });
        }
    }
    if !categories.is_empty() && categories.len() != 8 {
        issues.push(issue(
            "encoder_pages",
            format!("expected 8 control categories, found {}", categories.len()),
        ));
    }

    let log_level = match raw.log_level.as_deref() {
        None => LogLevel::Info,
        Some(text) => match LogLevel::parse(text) {
            Some(level) => level,
            None => {
                issues.push(issue("log_level", format!("unknown log level '{text}'")));
                LogLevel::Info
            }
        },
    };
    let display_interval_ms = raw
        .display
        .as_ref()
        .and_then(|display| display.min_interval_ms)
        .unwrap_or(50);

    let audio = raw.audio.as_ref();
    let sample_rate = audio.and_then(|audio| audio.sample_rate).unwrap_or(48_000);
    let buffer_frames = audio.and_then(|audio| audio.buffer_frames).unwrap_or(256);
    let polyphony = audio.and_then(|audio| audio.polyphony).unwrap_or(8);
    if sample_rate == 0 {
        issues.push(issue(
            "audio.sample_rate",
            "sample rate must be greater than zero",
        ));
    }
    if buffer_frames == 0 {
        issues.push(issue(
            "audio.buffer_frames",
            "buffer size must be greater than zero",
        ));
    }
    if polyphony == 0 {
        issues.push(issue(
            "audio.polyphony",
            "polyphony must be greater than zero",
        ));
    }

    if !issues.is_empty() {
        return Err(issues);
    }

    let groups = groups.map(|group| group.expect("eight valid groups"));
    Ok(HostConfig {
        schema_version: raw.schema_version,
        midi_outputs,
        groups,
        encoder_pages: pages,
        categories,
        log_level,
        display_interval_ms,
        audio_device: audio.and_then(|audio| audio.device.clone()),
        sample_rate,
        buffer_frames,
        polyphony,
    })
}

fn note_scale_for(mode: PadMode, scale: &RawNoteScale) -> Result<NoteScale, String> {
    if mode != PadMode::Scale {
        return Err("a note scale is only valid when the group mode is scale".to_string());
    }
    if !(0..=127).contains(&scale.root) {
        return Err(format!("root {} is outside 0-127", scale.root));
    }
    let Some(name) = ScaleName::parse(&scale.name) else {
        return Err(format!("unknown scale '{}'", scale.name));
    };
    Ok(NoteScale {
        root: scale.root as u8,
        name,
    })
}

fn profile_encoder(
    encoder: &RawEncoder,
    path: &str,
    midi_outputs: &[MidiPortConfig],
    issues: &mut Vec<ConfigIssue>,
) -> Option<EncoderMapping> {
    let min = encoder.min.unwrap_or(0);
    let max = encoder.max.unwrap_or(127);
    let default_value = encoder.default.unwrap_or(min);
    let kind = match encoder.kind.as_str() {
        "unbound" => EncoderKind::Unbound,
        "midi_cc" => {
            let Some(port) = encoder.port.clone() else {
                issues.push(issue(&format!("{path}.port"), "midi_cc requires port"));
                return None;
            };
            if midi_outputs.iter().all(|item| item.id != port) {
                issues.push(issue(
                    &format!("{path}.port"),
                    format!("unknown MIDI port '{port}'"),
                ));
            }
            let channel = encoder.channel.unwrap_or(1);
            if !(1..=16).contains(&channel) {
                issues.push(issue(
                    &format!("{path}.channel"),
                    format!("channel {channel} is outside 1-16"),
                ));
            }
            EncoderKind::MidiCc {
                port,
                channel,
                cc: encoder.cc,
            }
        }
        other => {
            issues.push(issue(
                &format!("{path}.type"),
                format!("unknown encoder type '{other}'"),
            ));
            return None;
        }
    };
    let scope = match encoder.scope.as_deref() {
        None | Some("slot") => ParamScope::Slot,
        Some("global") => ParamScope::Global,
        Some("selected") => ParamScope::Selected,
        Some(other) => {
            issues.push(issue(
                &format!("{path}.scope"),
                format!("unknown scope '{other}'"),
            ));
            ParamScope::Slot
        }
    };
    let format = match encoder.format.as_deref() {
        None | Some("number") => ValueFormat::Number,
        Some("on-off") => ValueFormat::OnOff,
        Some(other) => {
            issues.push(issue(
                &format!("{path}.format"),
                format!("unknown format '{other}'"),
            ));
            ValueFormat::Number
        }
    };
    Some(EncoderMapping {
        label: encoder.label.clone(),
        kind,
        min,
        max,
        default_value,
        parameter: encoder
            .parameter
            .as_ref()
            .filter(|name| !name.trim().is_empty())
            .cloned(),
        scope,
        format,
        pad_types: Vec::new(),
        inactive: false,
    })
}

fn group_midi_channel(groups: &[Option<Group>; GROUP_COUNT], index: usize) -> Option<u8> {
    match groups.get(index).and_then(|group| group.as_ref()) {
        Some(group) => match &group.destination {
            Destination::Midi { channel, .. } => Some(*channel),
            Destination::Sampler { .. } => None,
        },
        None => None,
    }
}

fn default_trigger(mode: PadMode, destination: &Destination) -> TriggerPolicy {
    if mode == PadMode::OneShot && matches!(destination, Destination::Sampler { .. }) {
        TriggerPolicy::IgnoreRelease
    } else {
        TriggerPolicy::NoteOff
    }
}

fn issue(path: &str, message: impl Into<String>) -> ConfigIssue {
    ConfigIssue {
        path: path.to_string(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ports() -> &'static str {
        "midi_outputs:\n  bitbox: CONFIGURE_PORT_NAME\n"
    }

    fn notes() -> String {
        (36..52)
            .map(|note| note.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn group_yaml(id: &str, mode: &str, channel: u8) -> String {
        format!(
            "  - id: {id}\n    label: Group {id}\n    mode: {mode}\n    destination: {{type: midi, port: bitbox, channel: {channel}}}\n    notes: [{}]\n",
            notes()
        )
    }

    fn eight_groups() -> String {
        GroupId::ALL
            .iter()
            .enumerate()
            .map(|(index, id)| group_yaml(&id.to_string(), "one_shot", (index as u8) + 1))
            .collect()
    }

    fn page_yaml(id: &str) -> String {
        let mut encoders = String::new();
        for index in 1..=8 {
            encoders.push_str(&format!(
                "      - {{label: Slot {index}, type: midi_cc, port: bitbox, channel: {index}, cc: null}}\n"
            ));
        }
        format!("  - id: {id}\n    label: {id}\n    encoders:\n{encoders}")
    }

    fn document() -> String {
        format!(
            "schema_version: 1\n{}groups:\n{}encoder_pages:\n{}",
            ports(),
            eight_groups(),
            page_yaml("delay")
        )
    }

    #[test]
    fn valid_document_activates_eight_groups() {
        let config = load_yaml(&document()).expect("valid");
        assert_eq!(config.groups[0].id, GroupId::A);
        assert_eq!(config.groups[7].id, GroupId::H);
        assert!(matches!(
            config.encoder_pages[0].encoders[0].kind,
            EncoderKind::MidiCc { cc: None, .. }
        ));
    }

    #[test]
    fn invalid_channel_rejects_the_whole_document() {
        let mut text = document();
        text = text.replacen("channel: 1", "channel: 0", 1);
        let errors = load_yaml(&text).expect_err("channel 0");
        assert!(errors.iter().any(|issue| issue.message.contains("1-16")));
    }

    #[test]
    fn short_group_list_is_rejected() {
        let text = format!(
            "schema_version: 1\n{}groups:\n{}encoder_pages:\n{}",
            ports(),
            group_yaml("A", "one_shot", 1),
            page_yaml("delay")
        );
        let errors = load_yaml(&text).expect_err("one group");
        assert!(errors.iter().any(|issue| issue.path == "groups"));
    }

    #[test]
    fn omitted_encoder_channel_follows_the_group_and_an_explicit_channel_does_not() {
        let mut groups = eight_groups();
        groups = groups.replacen("channel: 1", "channel: 4", 1);
        let text = format!(
            "schema_version: 1\n{}groups:\n{}encoder_pages:\n  - id: delay\n    label: Delay sends\n    encoders:\n      - {{label: Slot 1, type: midi_cc, port: bitbox, cc: 20}}\n      - {{label: Slot 2, type: midi_cc, port: bitbox, channel: 9, cc: 21}}\n{}\n",
            ports(),
            groups,
            (3..=8)
                .map(|index| format!(
                    "      - {{label: Slot {index}, type: midi_cc, port: bitbox, channel: {index}, cc: null}}"
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let config = load_yaml(&text).expect("omitted channel");
        assert!(matches!(
            config.encoder_pages[0].encoders[0].kind,
            EncoderKind::MidiCc { channel: 4, cc: Some(20), .. }
        ));
        assert!(matches!(
            config.encoder_pages[0].encoders[1].kind,
            EncoderKind::MidiCc { channel: 9, cc: Some(21), .. }
        ));
    }

    #[test]
    fn omitted_velocity_curve_is_straight_and_invalid_pad_mode_fields_do_not_activate() {
        let config = load_yaml(&document()).expect("omitted curve");
        assert_eq!(
            config.groups[0].velocity_curve,
            VelocityCurve::STRAIGHT
        );
        assert!(config.groups[0].note_scale.is_none());

        let curved = document().replacen(
            "mode: one_shot",
            "mode: one_shot\n    velocity_curve: 0",
            1,
        );
        let errors = load_yaml(&curved).expect_err("curve");
        assert!(errors.iter().any(|issue| issue.path.ends_with("velocity_curve")));

        let scaled = document().replacen(
            "mode: one_shot",
            "mode: one_shot\n    note_scale: {root: 48, name: natural-minor}",
            1,
        );
        let errors = load_yaml(&scaled).expect_err("scale on one-shot");
        assert!(errors.iter().any(|issue| issue.path.ends_with("note_scale")));
    }

    #[test]
    fn null_cc_is_a_placeholder_not_a_schema_error() {
        let config = load_yaml(&document()).expect("placeholder cc");
        assert!(config.encoder_pages[0]
            .encoders
            .iter()
            .all(|encoder| { matches!(encoder.kind, EncoderKind::MidiCc { cc: None, .. }) }));
        assert_eq!(config.groups[0].color, default_group_color(GroupId::A));
        assert_ne!(config.groups[0].color, config.groups[1].color);
    }

    #[test]
    fn published_examples_validate() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for name in [
            "docs/example-config.yaml",
            "docs/encoder-panel-example.yaml",
            "docs/local-sampler.example.yaml",
        ] {
            let text = std::fs::read_to_string(root.join(name)).expect(name);
            let config = load_yaml(&text).unwrap_or_else(|issues| panic!("{name}: {issues:?}"));
            if name == "docs/example-config.yaml" {
                assert_eq!(config.groups[GroupId::A.index()].color, [255, 32, 16]);
                assert_eq!(config.groups[GroupId::H.index()].color, [255, 0, 144]);
                assert_eq!(
                    config
                        .categories
                        .iter()
                        .map(|category| category.id.as_str())
                        .collect::<Vec<_>>(),
                    vec!["PERFORM", "MIX", "DELAY", "REVERB", "CHAR", "DYNAM", "EQ", "PAD"]
                );
                assert_eq!(
                    config
                        .encoder_pages
                        .iter()
                        .map(|page| page.id.as_str())
                        .collect::<Vec<_>>(),
                    vec![
                        "perform",
                        "mix-delay",
                        "mix-reverb",
                        "mix-level",
                        "mix-filter",
                        "mix-pan",
                        "mix-pitch",
                        "delay",
                        "reverb",
                        "char-amount",
                        "char-pick",
                        "comp",
                        "eq",
                        "eq-slope",
                        "pad-sample",
                        "pad-granular",
                        "pad-clip",
                        "pad-slicer",
                    ]
                );
                let perform = &config.encoder_pages[0];
                assert!(matches!(
                    perform.encoders[0].kind,
                    EncoderKind::MidiCc { channel: 1, cc: Some(84), .. }
                ));
                assert!(matches!(
                    perform.encoders[2].kind,
                    EncoderKind::MidiCc { channel: 1, cc: Some(30), .. }
                ));
                assert_eq!(perform.profiles.len(), 4);
                let delay = config.encoder_pages.iter().find(|page| page.id == "delay").unwrap();
                assert!(matches!(
                    delay.encoders[0].kind,
                    EncoderKind::MidiCc { channel: 1, cc: Some(108), .. }
                ));
                let level = config.encoder_pages.iter().find(|page| page.id == "mix-level").unwrap();
                assert!(matches!(
                    level.encoders[6].kind,
                    EncoderKind::MidiCc { channel: 7, cc: Some(36), .. }
                ));
                let character = config.encoder_pages.iter().find(|page| page.id == "char-amount").unwrap();
                assert!(matches!(
                    character.encoders[1].kind,
                    EncoderKind::MidiCc { channel: 1, cc: Some(15), .. }
                ));
                assert_eq!(character.encoders[1].default_value, 0);
                assert_eq!(character.encoders[4].default_value, 64);
                let comp = config.encoder_pages.iter().find(|page| page.id == "comp").unwrap();
                assert_eq!(comp.encoders[0].default_value, 0);
                assert_eq!(comp.encoders[1].default_value, 90);
                let eq = config.encoder_pages.iter().find(|page| page.id == "eq").unwrap();
                assert_eq!(eq.encoders[0].default_value, 6);
                for page in &config.encoder_pages {
                    assert!(page.encoders.iter().all(|encoder| {
                        !matches!(
                            encoder.kind,
                            EncoderKind::MidiCc { cc: Some(120 | 123), .. }
                        )
                    }));
                }
                let shared_notes: Vec<u8> = (36..=51).collect();
                for group in &config.groups {
                    assert_eq!(group.notes.as_slice(), shared_notes.as_slice());
                    assert_eq!(group.velocity_curve, VelocityCurve::new(0.5).unwrap());
                    if group.id == GroupId::C {
                        assert_eq!(
                            group.note_scale,
                            Some(NoteScale {
                                root: 48,
                                name: ScaleName::NaturalMinor,
                            })
                        );
                    } else {
                        assert!(group.note_scale.is_none());
                    }
                    assert!(matches!(
                        &group.destination,
                        Destination::Midi { channel, .. } if *channel == group.id.index() as u8 + 1
                    ));
                }
            }
            if name == "docs/encoder-panel-example.yaml" {
                assert_eq!(config.encoder_pages.len(), 2);
                for page in &config.encoder_pages {
                    let labels: Vec<_> = page.encoders.iter().map(|encoder| encoder.label.as_str()).collect();
                    assert_eq!(
                        labels,
                        vec!["enc 1", "enc 2", "enc 3", "enc 4", "enc 5", "enc 6", "enc 7", "enc 8"]
                    );
                    assert!(page.encoders.iter().all(|encoder| {
                        matches!(encoder.kind, EncoderKind::MidiCc { cc: None, .. })
                    }));
                }
            }
        }
    }

    #[test]
    fn unknown_schema_version_is_rejected() {
        let text = document().replacen("schema_version: 1", "schema_version: 2", 1);
        let errors = load_yaml(&text).expect_err("version");
        assert!(errors.iter().any(|issue| issue.path == "schema_version"));
    }
}
