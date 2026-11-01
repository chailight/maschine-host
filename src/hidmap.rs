use crate::model::{ControlAction, ControlId, GroupId, InputEvent, ENCODER_COUNT, PAD_COUNT};
use serde::Deserialize;

/// Confirmed control fields from a labeled session. Bytes that do not appear
/// here produce no events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub stand_in_press: i32,
    fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Field {
    control: Option<ControlId>,
    label: String,
    report: Option<u8>,
    kind: FieldKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FieldKind {
    Bit {
        byte: usize,
        bit: u8,
        pressure_byte: Option<usize>,
    },
    Delta {
        byte: usize,
    },
    /// 12-bit pressure. The high byte keeps a channel nibble in its top half.
    /// Report 0x20 repeats the same 16 values 32 bytes later. A press counts
    /// only when both copies are above zero, so a one-count flicker in the
    /// first copy does not light a pad.
    Adc12 {
        value_byte: usize,
        high_byte: usize,
    },
    /// Little-endian 10-bit position. Clockwise increased it in the capture.
    Pos10 {
        low_byte: usize,
        high_byte: usize,
        scale: i32,
    },
    /// Low 4 bits. Clockwise steps `0x0f` to `0x00`, then upward.
    Nibble4 {
        byte: usize,
    },
}

/// One confirmed edge, named for the terminal listen mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heard {
    pub label: String,
    control: Option<ControlId>,
    pub action: ControlAction,
    pub value: i32,
}

impl Heard {
    pub fn terminal_line(&self) -> String {
        match self.action {
            ControlAction::Press if self.label.starts_with("pad:") => {
                format!("{} press {}", self.label, self.value)
            }
            ControlAction::Press => format!("{} press", self.label),
            ControlAction::Release => format!("{} release", self.label),
            ControlAction::Delta => format!("{} {:+}", self.label, self.value),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AssignmentFile {
    #[serde(default = "default_stand_in")]
    stand_in_press: i32,
    #[serde(default)]
    fields: Vec<FieldFile>,
}

#[derive(Debug, Deserialize)]
struct FieldFile {
    control: String,
    kind: String,
    #[serde(default)]
    byte: usize,
    #[serde(default)]
    bit: u8,
    #[serde(default)]
    pressure_byte: Option<usize>,
    #[serde(default)]
    value_byte: Option<usize>,
    #[serde(default)]
    high_byte: Option<usize>,
    #[serde(default)]
    low_byte: Option<usize>,
    #[serde(default)]
    scale: Option<i32>,
    #[serde(default)]
    report: Option<u8>,
}

fn default_stand_in() -> i32 {
    100
}

impl Assignment {
    pub fn empty() -> Self {
        Self {
            stand_in_press: default_stand_in(),
            fields: Vec::new(),
        }
    }

    pub fn stand_in_press(&self) -> i32 {
        self.stand_in_press
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Bits read as released. Value bytes stay as they are, so the first
    /// report can be a button press without stepping an encoder.
    pub(crate) fn settled_baseline(&self, report: &[u8]) -> Vec<u8> {
        let mut baseline = report.to_vec();
        let report_id = report.first().copied();
        for field in &self.fields {
            if field.report != report_id {
                continue;
            }
            if let FieldKind::Bit { byte, bit, .. } = field.kind {
                if let Some(slot) = baseline.get_mut(byte) {
                    *slot &= !(1 << bit);
                }
            }
        }
        baseline
    }

    pub fn load_yaml(text: &str) -> Result<Self, String> {
        let file: AssignmentFile =
            serde_yaml::from_str(text).map_err(|error| format!("hid assignment: {error}"))?;
        Self::from_file(file, true)
    }

    /// Load a labeled button map. Names that are not host controls stay printable
    /// and do not become controller events.
    pub fn load_named_yaml(text: &str) -> Result<Self, String> {
        let file: AssignmentFile =
            serde_yaml::from_str(text).map_err(|error| format!("hid assignment: {error}"))?;
        Self::from_file(file, false)
    }

    fn from_file(file: AssignmentFile, require_host_control: bool) -> Result<Self, String> {
        if !(1..=127).contains(&file.stand_in_press) {
            return Err("hid assignment stand_in_press must be 1-127".to_string());
        }
        let mut fields = Vec::new();
        for (index, field) in file.fields.iter().enumerate() {
            let label = field.control.trim().to_string();
            if label.is_empty() {
                return Err(format!("hid assignment field {index}: control is empty"));
            }
            let control = parse_control(&label);
            if require_host_control && control.is_none() {
                return Err(format!("hid assignment field {index}: unknown control"));
            }
            let kind = match field.kind.as_str() {
                "bit" => {
                    if field.bit > 7 {
                        return Err(format!("hid assignment field {index}: bit must be 0-7"));
                    }
                    FieldKind::Bit {
                        byte: field.byte,
                        bit: field.bit,
                        pressure_byte: field.pressure_byte,
                    }
                }
                "delta" => FieldKind::Delta { byte: field.byte },
                "adc12" => {
                    let (Some(value_byte), Some(high_byte)) = (field.value_byte, field.high_byte)
                    else {
                        return Err(format!(
                            "hid assignment field {index}: adc12 needs value_byte and high_byte"
                        ));
                    };
                    FieldKind::Adc12 {
                        value_byte,
                        high_byte,
                    }
                }
                "pos10" => {
                    let (Some(low_byte), Some(high_byte)) = (field.low_byte, field.high_byte)
                    else {
                        return Err(format!(
                            "hid assignment field {index}: pos10 needs low_byte and high_byte"
                        ));
                    };
                    let scale = field.scale.unwrap_or(4);
                    if scale <= 0 {
                        return Err(format!(
                            "hid assignment field {index}: scale must be positive"
                        ));
                    }
                    FieldKind::Pos10 {
                        low_byte,
                        high_byte,
                        scale,
                    }
                }
                "nibble4" => FieldKind::Nibble4 { byte: field.byte },
                other => {
                    return Err(format!(
                        "hid assignment field {index}: kind '{other}' is not known"
                    ));
                }
            };
            fields.push(Field {
                control,
                label,
                report: field.report,
                kind,
            });
        }
        Ok(Self {
            stand_in_press: file.stand_in_press,
            fields,
        })
    }

    pub fn extend(&mut self, other: Assignment) {
        self.fields.extend(other.fields);
    }
}

pub fn parse_control(text: &str) -> Option<ControlId> {
    let text = text.trim();
    if let Some(index) = text.strip_prefix("pad:") {
        let index: u8 = index.parse().ok()?;
        (index < PAD_COUNT as u8).then_some(ControlId::Pad(index))
    } else if let Some(index) = text.strip_prefix("encoder:") {
        let index: u8 = index.parse().ok()?;
        (index < ENCODER_COUNT as u8).then_some(ControlId::Encoder(index))
    } else if let Some(name) = text.strip_prefix("group:") {
        GroupId::parse(name).map(ControlId::Group)
    } else {
        match text {
            "page_left" => Some(ControlId::PageLeft),
            "page_right" => Some(ControlId::PageRight),
            "panic" => Some(ControlId::Panic),
            "octave_up" => Some(ControlId::OctaveUp),
            "octave_down" => Some(ControlId::OctaveDown),
            _ => crate::model::SilkButton::parse(text).map(ControlId::Button),
        }
    }
}

/// Consecutive zero-pressure reports required before a pad note ends.
/// A short return to pressure in this window is the same hit. Some pads
/// bounce longer than others, so the wait covers that mechanical spread.
pub const PAD_RELEASE_REPORTS: u8 = 36;

/// Remembers which pads are down so a short return to zero does not retrigger.
#[derive(Clone, Debug)]
pub struct PadHold {
    down: [bool; PAD_COUNT],
    zeros: [u8; PAD_COUNT],
}

impl Default for PadHold {
    fn default() -> Self {
        Self {
            down: [false; PAD_COUNT],
            zeros: [0; PAD_COUNT],
        }
    }
}

impl PadHold {
    fn edge(&mut self, index: u8, before: i32, now: i32) -> Option<ControlAction> {
        let index = index as usize;
        if index >= PAD_COUNT {
            return None;
        }
        if self.down[index] {
            if now > 0 {
                self.zeros[index] = 0;
                None
            } else {
                self.zeros[index] = self.zeros[index].saturating_add(1);
                if self.zeros[index] >= PAD_RELEASE_REPORTS {
                    self.down[index] = false;
                    self.zeros[index] = 0;
                    Some(ControlAction::Release)
                } else {
                    None
                }
            }
        } else if now > 0 {
            self.down[index] = true;
            self.zeros[index] = 0;
            if before == 0 {
                Some(ControlAction::Press)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Compare two input reports. The first report only becomes the baseline.
/// A zero pressure on a rising edge produces no press. Named buttons that are
/// not host controls are omitted here.
pub fn events_from_reports(
    assignment: &Assignment,
    previous: Option<&[u8]>,
    current: &[u8],
    time_ms: u64,
) -> Vec<InputEvent> {
    events_holding(assignment, previous, current, time_ms, None)
}

/// Same comparison, but a pad stays held until pressure has been zero for
/// [`PAD_RELEASE_REPORTS`] reports. A rise during that wait is the same hit.
pub fn events_from_reports_held(
    assignment: &Assignment,
    previous: Option<&[u8]>,
    current: &[u8],
    time_ms: u64,
    hold: &mut PadHold,
) -> Vec<InputEvent> {
    events_holding(assignment, previous, current, time_ms, Some(hold))
}

fn events_holding(
    assignment: &Assignment,
    previous: Option<&[u8]>,
    current: &[u8],
    time_ms: u64,
    hold: Option<&mut PadHold>,
) -> Vec<InputEvent> {
    heard_holding(assignment, previous, current, hold)
        .into_iter()
        .filter_map(|heard| {
            let control = heard.control?;
            Some(match heard.action {
                ControlAction::Press => InputEvent::press(control, heard.value, time_ms),
                ControlAction::Release => InputEvent::release(control, time_ms),
                ControlAction::Delta => InputEvent::delta(control, heard.value, time_ms),
            })
        })
        .collect()
}

/// Every confirmed edge, including buttons that the host does not route yet.
pub fn heard_from_reports(
    assignment: &Assignment,
    previous: Option<&[u8]>,
    current: &[u8],
) -> Vec<Heard> {
    heard_holding(assignment, previous, current, None)
}

fn heard_holding(
    assignment: &Assignment,
    previous: Option<&[u8]>,
    current: &[u8],
    mut hold: Option<&mut PadHold>,
) -> Vec<Heard> {
    let Some(previous) = previous else {
        return Vec::new();
    };
    let mut heard = Vec::new();
    for field in &assignment.fields {
        if let Some(report) = field.report {
            if current.first().copied() != Some(report) {
                continue;
            }
        }
        let mut push = |action, value| {
            heard.push(Heard {
                label: field.label.clone(),
                control: field.control,
                action,
                value,
            });
        };
        match field.kind {
            FieldKind::Bit {
                byte,
                bit,
                pressure_byte,
            } => {
                let was = bit_at(previous, byte, bit);
                let now = bit_at(current, byte, bit);
                if !was && now {
                    let value = match pressure_byte {
                        Some(index) => current.get(index).copied().unwrap_or(0) as i32,
                        None => assignment.stand_in_press,
                    };
                    if value > 0 {
                        push(ControlAction::Press, value.clamp(1, 127));
                    }
                } else if was && !now {
                    push(ControlAction::Release, 0);
                }
            }
            FieldKind::Delta { byte } => {
                let (Some(&before), Some(&after)) = (previous.get(byte), current.get(byte)) else {
                    continue;
                };
                let delta = after.wrapping_sub(before) as i8;
                if delta != 0 {
                    push(ControlAction::Delta, delta as i32);
                }
            }
            FieldKind::Adc12 {
                value_byte,
                high_byte,
            } => {
                let before = pad_pressure(previous, value_byte, high_byte);
                let now = pad_pressure(current, value_byte, high_byte);
                if let (Some(ControlId::Pad(index)), Some(hold)) = (field.control, hold.as_mut())
                {
                    if let Some(action) = hold.edge(index, before, now) {
                        let value = match action {
                            ControlAction::Press => (now * 127 / 4095).clamp(1, 127),
                            _ => 0,
                        };
                        push(action, value);
                    }
                } else if before == 0 && now > 0 {
                    let value = (now * 127 / 4095).clamp(1, 127);
                    push(ControlAction::Press, value);
                } else if before > 0 && now == 0 {
                    push(ControlAction::Release, 0);
                }
            }
            FieldKind::Pos10 {
                low_byte,
                high_byte,
                scale,
            } => {
                let delta = pos10_delta(previous, current, low_byte, high_byte) / scale;
                if delta != 0 {
                    push(ControlAction::Delta, delta);
                }
            }
            FieldKind::Nibble4 { byte } => {
                let (Some(&before), Some(&after)) = (previous.get(byte), current.get(byte)) else {
                    continue;
                };
                let delta = nibble4_delta(before, after);
                if delta != 0 {
                    push(ControlAction::Delta, delta);
                }
            }
        }
    }
    heard
}

fn nibble4_delta(before: u8, after: u8) -> i32 {
    let mut delta = i32::from(after & 0x0f) - i32::from(before & 0x0f);
    if delta > 8 {
        delta -= 16;
    } else if delta < -8 {
        delta += 16;
    }
    delta
}

fn adc12(report: &[u8], value_byte: usize, high_byte: usize) -> i32 {
    let low = report.get(value_byte).copied().unwrap_or(0) as i32;
    let high = report.get(high_byte).copied().unwrap_or(0) as i32;
    ((high & 0x0F) << 8) | low
}

/// Pressure from the first copy. When the report also carries the repeated
/// copy 32 bytes later, either copy at zero means the pad is up.
fn pad_pressure(report: &[u8], value_byte: usize, high_byte: usize) -> i32 {
    let primary = adc12(report, value_byte, high_byte);
    let mirror_high = high_byte.saturating_add(32);
    if mirror_high >= report.len() {
        return primary;
    }
    let mirror = adc12(report, value_byte.saturating_add(32), mirror_high);
    if primary == 0 || mirror == 0 {
        0
    } else {
        primary
    }
}

fn pos10_delta(previous: &[u8], current: &[u8], low_byte: usize, high_byte: usize) -> i32 {
    fn position(report: &[u8], low_byte: usize, high_byte: usize) -> i32 {
        let low = report.get(low_byte).copied().unwrap_or(0) as i32;
        let high = report.get(high_byte).copied().unwrap_or(0) as i32;
        low | ((high & 0x03) << 8)
    }
    let mut delta =
        position(current, low_byte, high_byte) - position(previous, low_byte, high_byte);
    if delta > 512 {
        delta -= 1024;
    } else if delta < -512 {
        delta += 1024;
    }
    delta
}

fn bit_at(report: &[u8], byte: usize, bit: u8) -> bool {
    report
        .get(byte)
        .is_some_and(|value| value & (1 << bit) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ControlAction;

    fn assignment() -> Assignment {
        Assignment::load_yaml(
            "stand_in_press: 100\nfields:\n  - {control: pad:0, kind: bit, byte: 1, bit: 0}\n  - {control: encoder:1, kind: delta, byte: 3}\n",
        )
        .unwrap()
    }

    #[test]
    fn rising_bit_presses_and_falling_bit_releases() {
        let map = assignment();
        let first = events_from_reports(&map, None, &[0, 0x01, 0, 0], 1);
        assert!(first.is_empty());
        let press = events_from_reports(&map, Some(&[0, 0x00, 0, 0]), &[0, 0x01, 0, 0], 2);
        assert_eq!(press, vec![InputEvent::press(ControlId::Pad(0), 100, 2)]);
        let release = events_from_reports(&map, Some(&[0, 0x01, 0, 0]), &[0, 0x00, 0, 0], 3);
        assert_eq!(release, vec![InputEvent::release(ControlId::Pad(0), 3)]);
    }

    #[test]
    fn zero_pressure_press_is_ignored() {
        let map = Assignment::load_yaml(
            "fields:\n  - {control: pad:3, kind: bit, byte: 0, bit: 1, pressure_byte: 2}\n",
        )
        .unwrap();
        let events = events_from_reports(&map, Some(&[0x00, 0, 0]), &[0x02, 0, 0], 5);
        assert!(events.is_empty());
        let pressed = events_from_reports(&map, Some(&[0x00, 0, 0]), &[0x02, 0, 90], 6);
        assert_eq!(pressed[0].action, ControlAction::Press);
        assert_eq!(pressed[0].value, 90);
    }

    #[test]
    fn unconfirmed_byte_produces_no_event() {
        let map = assignment();
        let events = events_from_reports(&map, Some(&[0, 0, 0, 0]), &[0, 0, 0xFF, 0], 4);
        assert!(events.is_empty());
    }

    #[test]
    fn encoder_delta_uses_signed_byte_change() {
        let map = assignment();
        let events = events_from_reports(&map, Some(&[0, 0, 0, 1]), &[0, 0, 0, 4], 7);
        assert_eq!(events, vec![InputEvent::delta(ControlId::Encoder(1), 3, 7)]);
    }

    #[test]
    fn captured_layout_routes_pad_group_release_and_encoder() {
        let map = Assignment::load_yaml(include_str!("../docs/hid-assignment.yaml")).unwrap();
        assert_eq!(map.len(), 16 + 8 + 8);
        let mut idle = vec![0x20];
        let markers = [
            0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xA0, 0xB0, 0xC0, 0xD0,
            0xE0, 0xF0,
        ];
        for _bank in 0..2 {
            for marker in markers {
                idle.push(0);
                idle.push(marker);
            }
        }
        let mut pressed = idle.clone();
        pressed[25] = 0xFC;
        pressed[26] = 0xC3;
        pressed[57] = 0xF9;
        pressed[58] = 0xC3;
        let pad = events_from_reports(&map, Some(&idle), &pressed, 10);
        assert_eq!(
            pad.len(),
            1,
            "the mirrored pad bytes must not emit a second event"
        );
        assert_eq!(pad[0].control, ControlId::Pad(0));
        assert_eq!(pad[0].value, 1020 * 127 / 4095);
        let mut flicker = idle.clone();
        flicker[31] = 0x01;
        let glitch = events_from_reports(&map, Some(&idle), &flicker, 11);
        assert!(
            glitch.is_empty(),
            "a one-count change in only the first pad copy is not a press"
        );

        let mut host = crate::controller::Controller::new(
            crate::config::load_yaml(include_str!("../docs/example-config.yaml")).unwrap(),
        );
        let page = host.active_page();
        let velocity = crate::model::VelocityCurve::new(0.5)
            .unwrap()
            .apply(pad[0].value as u8);
        let note_on = host.handle(pad[0]);
        assert!(note_on.iter().any(|action| matches!(
            action,
            crate::model::OutputAction::Midi(message) if message.bytes == [0x90, 36, velocity]
        )));
        let group =
            events_from_reports(&map, Some(&[0x01, 0, 0, 0, 0]), &[0x01, 0, 0, 0, 0x02], 11);
        host.handle(group[0]);
        assert_eq!(host.active_group(), GroupId::B);
        assert_eq!(host.active_page(), page);
        let release = events_from_reports(&map, Some(&pressed), &idle, 12);
        let note_off = host.handle(release[0]);
        assert!(note_off.iter().any(|action| matches!(
            action,
            crate::model::OutputAction::Midi(message) if message.bytes == [0x80, 36, 0]
        )));
        let before = host.parameter(0, 0).unwrap().value;
        let turned = events_from_reports(
            &map,
            Some(&[0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0x4D, 0x03]),
            &[0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0x51, 0x03],
            13,
        );
        assert_eq!(
            turned,
            vec![InputEvent::delta(ControlId::Encoder(0), 1, 13)]
        );
        host.handle(turned[0]);
        assert_eq!(host.parameter(0, 0).unwrap().value, before + 1);
    }

    #[test]
    fn named_buttons_print_and_the_shared_encoder_steps() {
        let buttons =
            Assignment::load_named_yaml(include_str!("../docs/hid-buttons.yaml")).unwrap();
        let quiet = [0x01u8, 0, 0, 0, 0, 0, 0, 0, 0x09];
        let restart = [0x01, 0, 0, 0, 0, 0x01, 0, 0, 0x09];
        let click = [0x01, 0, 0, 0x80, 0, 0, 0, 0, 0x09];
        assert_eq!(
            heard_from_reports(&buttons, Some(&quiet), &restart)[0].terminal_line(),
            "restart press"
        );
        assert_eq!(
            heard_from_reports(&buttons, Some(&quiet), &click)[0].terminal_line(),
            "encoder-click press"
        );
        let map = Assignment::load_named_yaml(
            "fields:\n  - {control: volume, kind: bit, report: 1, byte: 3, bit: 0}\n  - {control: encoder, kind: nibble4, report: 1, byte: 8}\n",
        )
        .unwrap();
        let idle = [0x01, 0, 0, 0, 0, 0, 0, 0, 0x0f];
        let pressed = [0x01, 0, 0, 0x01, 0, 0, 0, 0, 0x0f];
        let lines: Vec<_> = heard_from_reports(&map, Some(&idle), &pressed)
            .iter()
            .map(Heard::terminal_line)
            .collect();
        assert_eq!(lines, vec!["volume press".to_string()]);
        assert_eq!(
            events_from_reports(&map, Some(&idle), &pressed, 1),
            vec![InputEvent::press(
                ControlId::Button(crate::model::SilkButton::Volume),
                100,
                1
            )]
        );
        let turned = [0x01, 0, 0, 0, 0, 0, 0, 0, 0x00];
        let back = [0x01, 0, 0, 0, 0, 0, 0, 0, 0x0f];
        assert_eq!(
            heard_from_reports(&map, Some(&idle), &turned)[0].terminal_line(),
            "encoder +1"
        );
        assert_eq!(
            heard_from_reports(&map, Some(&turned), &back)[0].terminal_line(),
            "encoder -1"
        );
    }

    #[test]
    fn a_short_return_to_zero_does_not_retrigger_the_pad() {
        let map = Assignment::load_yaml(
            "fields:\n  - {control: pad:0, kind: adc12, report: 32, value_byte: 25, high_byte: 26}\n",
        )
        .unwrap();
        let down = pad_level(0x00F0);
        let up = pad_level(0);
        let mut reports = vec![down.clone(), up.clone(), down.clone()];
        reports.extend(std::iter::repeat(up.clone()).take(PAD_RELEASE_REPORTS as usize - 1));
        reports.push(down.clone());
        assert_eq!(count_pad_edges(&map, &reports), (1, 0));
        let mut released = reports;
        released.extend(std::iter::repeat(up).take(PAD_RELEASE_REPORTS as usize));
        assert_eq!(count_pad_edges(&map, &released), (1, 1));
        released.push(down);
        assert_eq!(count_pad_edges(&map, &released), (2, 1));
    }

    fn count_pad_edges(map: &Assignment, reports: &[Vec<u8>]) -> (i32, i32) {
        let mut previous = pad_level(0);
        let mut hold = PadHold::default();
        let mut presses = 0;
        let mut releases = 0;
        for report in reports {
            for event in events_from_reports_held(map, Some(&previous), report, 1, &mut hold) {
                match event.action {
                    ControlAction::Press => presses += 1,
                    ControlAction::Release => releases += 1,
                    ControlAction::Delta => {}
                }
            }
            previous = report.clone();
        }
        (presses, releases)
    }

    fn pad_level(level: u16) -> Vec<u8> {
        let mut report = vec![0x20];
        report.resize(64, 0);
        report[25] = (level & 0xFF) as u8;
        report[26] = ((level >> 8) & 0x0F) as u8;
        report[57] = report[25];
        report[58] = report[26];
        report
    }
}
