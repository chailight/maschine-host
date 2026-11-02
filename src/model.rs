use serde::{Deserialize, Serialize};
use std::fmt;

pub const GROUP_COUNT: usize = 8;
pub const PAD_COUNT: usize = 16;
pub const ENCODER_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroupId {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl GroupId {
    pub const ALL: [GroupId; GROUP_COUNT] = [
        GroupId::A,
        GroupId::B,
        GroupId::C,
        GroupId::D,
        GroupId::E,
        GroupId::F,
        GroupId::G,
        GroupId::H,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Option<GroupId> {
        GroupId::ALL.get(index).copied()
    }

    pub fn parse(text: &str) -> Option<GroupId> {
        match text.trim().to_ascii_uppercase().as_str() {
            "A" => Some(GroupId::A),
            "B" => Some(GroupId::B),
            "C" => Some(GroupId::C),
            "D" => Some(GroupId::D),
            "E" => Some(GroupId::E),
            "F" => Some(GroupId::F),
            "G" => Some(GroupId::G),
            "H" => Some(GroupId::H),
            _ => None,
        }
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadMode {
    OneShot,
    Chromatic,
    Scale,
}

impl PadMode {
    pub fn parse(text: &str) -> Option<PadMode> {
        match text.trim() {
            "one_shot" => Some(PadMode::OneShot),
            "chromatic" => Some(PadMode::Chromatic),
            "scale" => Some(PadMode::Scale),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PadMode::OneShot => "one-shot",
            PadMode::Chromatic => "chromatic",
            PadMode::Scale => "scale",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerPolicy {
    NoteOff,
    IgnoreRelease,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    Midi { port: String, channel: u8 },
    Sampler { instrument: String },
}

impl Destination {
    pub fn summary(&self) -> String {
        match self {
            Destination::Midi { port, channel } => format!("MIDI {port} ch {channel}"),
            Destination::Sampler { instrument } => format!("sampler {instrument}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleAssignment {
    pub path: String,
    pub choke: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleZone {
    pub path: String,
    pub root: u8,
    pub lo: u8,
    pub hi: u8,
}

/// Exponent applied to a 1–127 pad velocity. 1 leaves the velocity unchanged.
#[derive(Clone, Copy, Debug)]
pub struct VelocityCurve(f64);

impl PartialEq for VelocityCurve {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for VelocityCurve {}

impl VelocityCurve {
    pub const STRAIGHT: Self = Self(1.0);

    pub fn new(value: f64) -> Option<Self> {
        if value.is_finite() && value > 0.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }

    pub fn apply(self, velocity: u8) -> u8 {
        let velocity = velocity.clamp(1, 127);
        if self.0 == 1.0 {
            return velocity;
        }
        let curved = 127.0 * (f64::from(velocity) / 127.0).powf(self.0);
        curved.round().clamp(1.0, 127.0) as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleName {
    Major,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    NaturalMinor,
    Locrian,
    MajorPentatonic,
    MinorPentatonic,
    Chromatic,
}

impl ScaleName {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "major" | "ionian" => Some(Self::Major),
            "dorian" => Some(Self::Dorian),
            "phrygian" => Some(Self::Phrygian),
            "lydian" => Some(Self::Lydian),
            "mixolydian" => Some(Self::Mixolydian),
            "natural-minor" | "aeolian" => Some(Self::NaturalMinor),
            "locrian" => Some(Self::Locrian),
            "major-pentatonic" => Some(Self::MajorPentatonic),
            "minor-pentatonic" => Some(Self::MinorPentatonic),
            "chromatic" => Some(Self::Chromatic),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Dorian => "dorian",
            Self::Phrygian => "phrygian",
            Self::Lydian => "lydian",
            Self::Mixolydian => "mixolydian",
            Self::NaturalMinor => "natural-minor",
            Self::Locrian => "locrian",
            Self::MajorPentatonic => "major-pentatonic",
            Self::MinorPentatonic => "minor-pentatonic",
            Self::Chromatic => "chromatic",
        }
    }

    pub fn screen_label(self) -> &'static str {
        match self {
            Self::Major => "ionian",
            Self::Dorian => "dorian",
            Self::Phrygian => "phrygian",
            Self::Lydian => "lydian",
            Self::Mixolydian => "mixolydian",
            Self::NaturalMinor => "aeolian",
            Self::Locrian => "locrian",
            Self::MajorPentatonic => "pent maj",
            Self::MinorPentatonic => "pent min",
            Self::Chromatic => "chromatic",
        }
    }

    fn steps(self) -> &'static [i32] {
        match self {
            Self::Major => &[0, 2, 4, 5, 7, 9, 11],
            Self::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Self::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Self::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Self::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Self::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            Self::Locrian => &[0, 1, 3, 5, 6, 8, 10],
            Self::MajorPentatonic => &[0, 2, 4, 7, 9],
            Self::MinorPentatonic => &[0, 3, 5, 7, 10],
            Self::Chromatic => &[],
        }
    }

    pub fn interval(self, pad: u8) -> i32 {
        let pad = i32::from(pad);
        if self == Self::Chromatic {
            return pad;
        }
        let steps = self.steps();
        let span = steps.len() as i32;
        let octave = pad.div_euclid(span);
        steps[pad.rem_euclid(span) as usize] + octave * 12
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteScale {
    pub root: u8,
    pub name: ScaleName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub id: GroupId,
    pub label: String,
    pub mode: PadMode,
    pub destination: Destination,
    pub notes: [u8; PAD_COUNT],
    pub trigger: TriggerPolicy,
    pub samples: [Option<SampleAssignment>; PAD_COUNT],
    pub zones: Vec<SampleZone>,
    /// Full LED color. Idle lamps use a dimmer shade of this color.
    pub color: [u8; 3],
    pub velocity_curve: VelocityCurve,
    pub note_scale: Option<NoteScale>,
    /// Scale root kept while the pads are on their note list.
    pub root: u8,
    pub pad_type: Option<PadType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadType {
    Sample,
    Granular,
    Clip,
    Slicer,
}

impl PadType {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "sample" => Some(Self::Sample),
            "granular" => Some(Self::Granular),
            "clip" => Some(Self::Clip),
            "slicer" => Some(Self::Slicer),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Sample => "sample",
            Self::Granular => "granular",
            Self::Clip => "clip",
            Self::Slicer => "slicer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamScope {
    Global,
    Selected,
    Slot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueFormat {
    Number,
    OnOff,
}

pub fn default_group_color(id: GroupId) -> [u8; 3] {
    match id {
        GroupId::A => [255, 32, 16],
        GroupId::B => [255, 96, 0],
        GroupId::C => [255, 196, 0],
        GroupId::D => [0, 176, 48],
        GroupId::E => [0, 176, 176],
        GroupId::F => [32, 64, 255],
        GroupId::G => [160, 32, 255],
        GroupId::H => [255, 0, 144],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CcTarget {
    pub port: String,
    pub channel: u8,
    pub cc: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncoderKind {
    /// Commanded CC. `None` is an explicit placeholder and sends nothing.
    MidiCc {
        port: String,
        channel: u8,
        cc: Option<u8>,
    },
    Local {
        target: String,
    },
    Macro {
        targets: Vec<CcTarget>,
    },
    Unbound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncoderMapping {
    pub label: String,
    pub kind: EncoderKind,
    pub min: i32,
    pub max: i32,
    pub default_value: i32,
    pub parameter: Option<String>,
    pub scope: ParamScope,
    pub format: ValueFormat,
    pub pad_types: Vec<PadType>,
    pub inactive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncoderPage {
    pub id: String,
    pub label: String,
    pub category: Option<String>,
    pub encoders: [EncoderMapping; ENCODER_COUNT],
    pub profiles: Vec<PadProfile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PadProfile {
    pub pad_type: PadType,
    pub encoders: [EncoderMapping; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlCategory {
    pub id: String,
    pub pages: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MidiPortConfig {
    pub id: String,
    pub name: String,
    pub virtual_port: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Info,
    Debug,
}

impl LogLevel {
    pub fn parse(text: &str) -> Option<LogLevel> {
        match text.trim().to_ascii_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            _ => None,
        }
    }

    pub fn allows(self, event: LogLevel) -> bool {
        self as u8 >= event as u8
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostConfig {
    pub schema_version: u32,
    pub midi_outputs: Vec<MidiPortConfig>,
    pub groups: [Group; GROUP_COUNT],
    pub encoder_pages: Vec<EncoderPage>,
    pub categories: Vec<ControlCategory>,
    pub log_level: LogLevel,
    pub display_interval_ms: u64,
    pub audio_device: Option<String>,
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub polyphony: usize,
}

impl HostConfig {
    pub fn port(&self, id: &str) -> Option<&MidiPortConfig> {
        self.midi_outputs.iter().find(|port| port.id == id)
    }
}

pub const SILK_BUTTON_COUNT: usize = 39;

/// How a silk-screen button holds its lamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonGesture {
    Momentary,
    Toggle,
    Radio(RadioSet),
}

/// Buttons that light one at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioSet {
    /// Scene, Pattern, Navigate, Duplicate, Select, Solo, Mute.
    PadModes,
    /// Volume, Swing, Tempo, and Note Repeat.
    Perform,
}

/// Where the lamp byte sits. Offsets are wire indexes, adapted from Encdr.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonLamp {
    /// Monochrome report `0x82`.
    Mono(usize),
    /// Single byte on the group report `0x81`.
    Group(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SilkButton {
    Display1,
    Display2,
    Display3,
    Display4,
    Display5,
    Display6,
    Display7,
    Display8,
    Control,
    Step,
    Browse,
    Sampling,
    DisplayLeft,
    DisplayRight,
    All,
    AutoWrite,
    Volume,
    Swing,
    Tempo,
    NoteRepeat,
    MasterLeft,
    MasterRight,
    Enter,
    Scene,
    Pattern,
    PadMode,
    Navigate,
    Duplicate,
    Select,
    Solo,
    Mute,
    Restart,
    TransportLeft,
    TransportRight,
    Grid,
    Play,
    Rec,
    Erase,
    Shift,
}

impl SilkButton {
    pub const ALL: [SilkButton; SILK_BUTTON_COUNT] = [
        SilkButton::Display1,
        SilkButton::Display2,
        SilkButton::Display3,
        SilkButton::Display4,
        SilkButton::Display5,
        SilkButton::Display6,
        SilkButton::Display7,
        SilkButton::Display8,
        SilkButton::Control,
        SilkButton::Step,
        SilkButton::Browse,
        SilkButton::Sampling,
        SilkButton::DisplayLeft,
        SilkButton::DisplayRight,
        SilkButton::All,
        SilkButton::AutoWrite,
        SilkButton::Volume,
        SilkButton::Swing,
        SilkButton::Tempo,
        SilkButton::NoteRepeat,
        SilkButton::MasterLeft,
        SilkButton::MasterRight,
        SilkButton::Enter,
        SilkButton::Scene,
        SilkButton::Pattern,
        SilkButton::PadMode,
        SilkButton::Navigate,
        SilkButton::Duplicate,
        SilkButton::Select,
        SilkButton::Solo,
        SilkButton::Mute,
        SilkButton::Restart,
        SilkButton::TransportLeft,
        SilkButton::TransportRight,
        SilkButton::Grid,
        SilkButton::Play,
        SilkButton::Rec,
        SilkButton::Erase,
        SilkButton::Shift,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn parse(text: &str) -> Option<SilkButton> {
        Self::ALL.into_iter().find(|button| button.name() == text)
    }

    pub fn name(self) -> &'static str {
        match self {
            SilkButton::Display1 => "display-button-1",
            SilkButton::Display2 => "display-button-2",
            SilkButton::Display3 => "display-button-3",
            SilkButton::Display4 => "display-button-4",
            SilkButton::Display5 => "display-button-5",
            SilkButton::Display6 => "display-button-6",
            SilkButton::Display7 => "display-button-7",
            SilkButton::Display8 => "display-button-8",
            SilkButton::Control => "control",
            SilkButton::Step => "step",
            SilkButton::Browse => "browse",
            SilkButton::Sampling => "sampling",
            SilkButton::DisplayLeft => "display-left",
            SilkButton::DisplayRight => "display-right",
            SilkButton::All => "all",
            SilkButton::AutoWrite => "auto-write",
            SilkButton::Volume => "volume",
            SilkButton::Swing => "swing",
            SilkButton::Tempo => "tempo",
            SilkButton::NoteRepeat => "note-repeat",
            SilkButton::MasterLeft => "master-left",
            SilkButton::MasterRight => "master-right",
            SilkButton::Enter => "enter",
            SilkButton::Scene => "scene",
            SilkButton::Pattern => "pattern",
            SilkButton::PadMode => "pad-mode",
            SilkButton::Navigate => "navigate",
            SilkButton::Duplicate => "duplicate",
            SilkButton::Select => "select",
            SilkButton::Solo => "solo",
            SilkButton::Mute => "mute",
            SilkButton::Restart => "restart",
            SilkButton::TransportLeft => "transport-left",
            SilkButton::TransportRight => "transport-right",
            SilkButton::Grid => "grid",
            SilkButton::Play => "play",
            SilkButton::Rec => "rec",
            SilkButton::Erase => "erase",
            SilkButton::Shift => "shift",
        }
    }

    pub fn gesture(self) -> ButtonGesture {
        match self {
            SilkButton::Display1
            | SilkButton::Display2
            | SilkButton::Display3
            | SilkButton::Display4
            | SilkButton::Display5
            | SilkButton::Display6
            | SilkButton::Display7
            | SilkButton::Display8 => ButtonGesture::Toggle,
            SilkButton::Volume
            | SilkButton::Swing
            | SilkButton::Tempo
            | SilkButton::NoteRepeat => ButtonGesture::Radio(RadioSet::Perform),
            SilkButton::PadMode => ButtonGesture::Toggle,
            SilkButton::Scene
            | SilkButton::Pattern
            | SilkButton::Navigate
            | SilkButton::Duplicate
            | SilkButton::Select
            | SilkButton::Solo
            | SilkButton::Mute => ButtonGesture::Radio(RadioSet::PadModes),
            _ => ButtonGesture::Momentary,
        }
    }

    pub fn lamp(self) -> ButtonLamp {
        match self {
            SilkButton::Display1 => ButtonLamp::Mono(9),
            SilkButton::Display2 => ButtonLamp::Mono(10),
            SilkButton::Display3 => ButtonLamp::Mono(11),
            SilkButton::Display4 => ButtonLamp::Mono(12),
            SilkButton::Display5 => ButtonLamp::Mono(13),
            SilkButton::Display6 => ButtonLamp::Mono(14),
            SilkButton::Display7 => ButtonLamp::Mono(15),
            SilkButton::Display8 => ButtonLamp::Mono(16),
            SilkButton::Control => ButtonLamp::Mono(1),
            SilkButton::Step => ButtonLamp::Mono(2),
            SilkButton::Browse => ButtonLamp::Mono(3),
            SilkButton::Sampling => ButtonLamp::Mono(4),
            SilkButton::DisplayLeft => ButtonLamp::Mono(5),
            SilkButton::DisplayRight => ButtonLamp::Mono(6),
            SilkButton::All => ButtonLamp::Mono(7),
            SilkButton::AutoWrite => ButtonLamp::Mono(8),
            // Bytes 17–31 are the reverse of Encdr's names. A Scene press was
            // lighting Note Repeat, and that row ran through to Volume.
            SilkButton::Scene => ButtonLamp::Mono(17),
            SilkButton::Pattern => ButtonLamp::Mono(18),
            SilkButton::PadMode => ButtonLamp::Mono(19),
            SilkButton::Navigate => ButtonLamp::Mono(20),
            SilkButton::Duplicate => ButtonLamp::Mono(21),
            SilkButton::Select => ButtonLamp::Mono(22),
            SilkButton::Solo => ButtonLamp::Mono(23),
            SilkButton::Mute => ButtonLamp::Mono(24),
            SilkButton::Volume => ButtonLamp::Mono(25),
            SilkButton::Swing => ButtonLamp::Mono(26),
            SilkButton::Tempo => ButtonLamp::Mono(27),
            SilkButton::MasterLeft => ButtonLamp::Mono(28),
            SilkButton::MasterRight => ButtonLamp::Mono(29),
            SilkButton::Enter => ButtonLamp::Mono(30),
            SilkButton::NoteRepeat => ButtonLamp::Mono(31),
            SilkButton::Restart => ButtonLamp::Group(49),
            SilkButton::TransportLeft => ButtonLamp::Group(50),
            SilkButton::TransportRight => ButtonLamp::Group(51),
            SilkButton::Grid => ButtonLamp::Group(52),
            SilkButton::Play => ButtonLamp::Group(53),
            SilkButton::Rec => ButtonLamp::Group(54),
            SilkButton::Erase => ButtonLamp::Group(55),
            SilkButton::Shift => ButtonLamp::Group(56),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlId {
    Pad(u8),
    Group(GroupId),
    Encoder(u8),
    PageLeft,
    PageRight,
    Panic,
    OctaveUp,
    OctaveDown,
    Button(SilkButton),
}

impl fmt::Display for ControlId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControlId::Pad(index) => write!(f, "pad:{index}"),
            ControlId::Group(id) => write!(f, "group:{id}"),
            ControlId::Encoder(index) => write!(f, "encoder:{index}"),
            ControlId::PageLeft => write!(f, "page_left"),
            ControlId::PageRight => write!(f, "page_right"),
            ControlId::Panic => write!(f, "panic"),
            ControlId::OctaveUp => write!(f, "octave_up"),
            ControlId::OctaveDown => write!(f, "octave_down"),
            ControlId::Button(button) => write!(f, "{}", button.name()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlAction {
    Press,
    Release,
    Delta,
}

/// Hardware-independent input. `value` is velocity for a pad press or a step
/// count for an encoder. The adapter, not the core, normalizes device reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub control: ControlId,
    pub action: ControlAction,
    pub value: i32,
    pub time_ms: u64,
}

impl InputEvent {
    pub fn press(control: ControlId, value: i32, time_ms: u64) -> Self {
        Self {
            control,
            action: ControlAction::Press,
            value,
            time_ms,
        }
    }

    pub fn release(control: ControlId, time_ms: u64) -> Self {
        Self {
            control,
            action: ControlAction::Release,
            value: 0,
            time_ms,
        }
    }

    pub fn delta(control: ControlId, value: i32, time_ms: u64) -> Self {
        Self {
            control,
            action: ControlAction::Delta,
            value,
            time_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterState {
    pub value: i32,
    pub source: &'static str,
    pub time_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MidiMessage {
    pub port: String,
    pub bytes: [u8; 3],
    /// Set for note-on so a failed send can drop the unconfirmed hold.
    pub confirm_pad: Option<u8>,
}

impl MidiMessage {
    pub fn note_on(port: &str, channel: u8, note: u8, velocity: u8, pad: u8) -> Self {
        Self {
            port: port.to_string(),
            bytes: [0x90 | channel_nibble(channel), note, velocity],
            confirm_pad: Some(pad),
        }
    }

    pub fn note_off(port: &str, channel: u8, note: u8) -> Self {
        Self {
            port: port.to_string(),
            bytes: [0x80 | channel_nibble(channel), note, 0],
            confirm_pad: None,
        }
    }

    pub fn cc(port: &str, channel: u8, controller: u8, value: u8) -> Self {
        Self {
            port: port.to_string(),
            bytes: [0xB0 | channel_nibble(channel), controller, value],
            confirm_pad: None,
        }
    }

    pub fn all_notes_off(port: &str, channel: u8) -> Self {
        Self::cc(port, channel, 123, 0)
    }
}

pub fn channel_nibble(channel: u8) -> u8 {
    channel.saturating_sub(1) & 0x0F
}

#[derive(Clone, Debug, PartialEq)]
pub struct SamplerCommand {
    pub kind: SamplerCommandKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SamplerCommandKind {
    Trigger {
        voice_id: u64,
        path: String,
        rate: f64,
        choke: Option<u8>,
    },
    Stop {
        voice_id: u64,
    },
    StopAll,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutputAction {
    Midi(MidiMessage),
    Sampler(SamplerCommand),
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeldNote {
    pub pad: u8,
    pub group: GroupId,
    pub destination: Destination,
    pub note: u8,
    pub voice_id: Option<u64>,
    pub release: TriggerPolicy,
}

pub fn clamp_note(value: i32) -> u8 {
    value.clamp(0, 127) as u8
}

pub fn nominal_latency_ms(buffer_frames: u32, sample_rate: u32) -> f64 {
    if sample_rate == 0 {
        return 0.0;
    }
    f64::from(buffer_frames) * 1000.0 / f64::from(sample_rate)
}
