//! Group and pad lamp reports.
//!
//! The byte positions in [`hypothesis`] are adapted from Encdr 0.7.8,
//! `encdr/descriptors/ni_maschine_mk2.json`.
//! Copyright (C) the Encdr authors. SPDX-License-Identifier: GPL-3.0-or-later.
//! <https://github.com/RufusIbiza/Encdr>
//!
//! The stored map records the live check: the bottom-left pad is wire bytes
//! 37–39, and each group button is six bytes. Group A was confirmed as bytes
//! 1–6. The other groups use that same width across bytes 1–48. Button bytes
//! stay 0. On macOS the report id stays inside the output report, so a color offset is the wire
//! index and byte 0 is the report id. The report is one byte longer than the
//! payload because a payload-sized write was ignored. The running host sends
//! a report only when a stored map says the check matched.

use crate::feedback::{LedFrame, Rgb, ACTIVE_COLOR};
use crate::model::{ButtonLamp, GroupId, SilkButton, PAD_COUNT};
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

pub const LAMP_MAP_PATH: &str = "docs/led-map.yaml";
pub const BUTTON_LAMP_REPORT: u8 = 0x82;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RgbAt {
    pub r: usize,
    pub g: usize,
    pub b: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LampReport {
    pub report_id: u8,
    pub length: usize,
    pub slots: Vec<Vec<RgbAt>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LampMap {
    matched: bool,
    pads: LampReport,
    groups: LampReport,
}

impl LampMap {
    pub fn unmatched() -> Self {
        let (pads, groups) = hypothesis();
        Self {
            matched: false,
            pads,
            groups,
        }
    }

    pub fn matched(&self) -> bool {
        self.matched
    }

    pub fn output_reports(&self, leds: &LedFrame) -> Result<Vec<Vec<u8>>, String> {
        if !self.matched {
            return Ok(Vec::new());
        }
        let mut groups = paint(
            &self.groups,
            leds.groups.iter().map(|group| group.color),
            "group",
        )?;
        write_group_buttons(&mut groups, leds)?;
        Ok(vec![
            paint(
                &self.pads,
                leds.pads.iter().map(|pad| pad.color),
                "pad",
            )?,
            groups,
            mono_report(leds)?,
        ])
    }

    pub fn probe_pad(&self, index: usize) -> Result<Vec<u8>, String> {
        probe_one(&self.pads, index, ACTIVE_COLOR, "pad")
    }

    pub fn probe_group(&self, index: usize) -> Result<Vec<u8>, String> {
        probe_one(&self.groups, index, ACTIVE_COLOR, "group")
    }

    pub fn blank_pad(&self) -> Result<Vec<u8>, String> {
        blank(&self.pads)
    }

    pub fn blank_group(&self) -> Result<Vec<u8>, String> {
        blank(&self.groups)
    }
}

pub fn load_lamp_map(path: &Path) -> Result<LampMap, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(LampMap::unmatched()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    parse_lamp_map(&text).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn parse_lamp_map(text: &str) -> Result<LampMap, String> {
    let file: LampFile = serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    if !file.matched {
        return Ok(LampMap::unmatched());
    }
    if file.pads.len() != PAD_COUNT || file.groups.len() != GroupId::ALL.len() {
        return Err(format!(
            "a matched lamp map needs {} pads and {} groups",
            PAD_COUNT,
            GroupId::ALL.len()
        ));
    }
    let (pads, groups) = reports_from_file(&file)?;
    Ok(LampMap {
        matched: true,
        pads,
        groups,
    })
}

fn hypothesis() -> (LampReport, LampReport) {
    let pads = LampReport {
        report_id: 0x80,
        length: 49,
        slots: vec![
            one(37, 38, 39),
            one(40, 41, 42),
            one(43, 44, 45),
            one(46, 47, 48),
            one(25, 26, 27),
            one(28, 29, 30),
            one(31, 32, 33),
            one(34, 35, 36),
            one(13, 14, 15),
            one(16, 17, 18),
            one(19, 20, 21),
            one(22, 23, 24),
            one(1, 2, 3),
            one(4, 5, 6),
            one(7, 8, 9),
            one(10, 11, 12),
        ],
    };
    let groups = LampReport {
        report_id: 0x81,
        length: 57,
        slots: vec![
            vec![at(1, 2, 3), at(4, 5, 6)],
            vec![at(7, 8, 9), at(10, 11, 12)],
            vec![at(13, 14, 15), at(16, 17, 18)],
            vec![at(19, 20, 21), at(22, 23, 24)],
            vec![at(25, 26, 27), at(28, 29, 30)],
            vec![at(31, 32, 33), at(34, 35, 36)],
            vec![at(37, 38, 39), at(40, 41, 42)],
            vec![at(43, 44, 45), at(46, 47, 48)],
        ],
    };
    (pads, groups)
}

fn at(r: usize, g: usize, b: usize) -> RgbAt {
    RgbAt { r, g, b }
}

fn one(r: usize, g: usize, b: usize) -> Vec<RgbAt> {
    vec![at(r, g, b)]
}

fn reports_from_file(file: &LampFile) -> Result<(LampReport, LampReport), String> {
    Ok((
        LampReport {
            report_id: file.pad_report,
            length: file.pad_length,
            slots: single_slots(&file.pads)?,
        },
        LampReport {
            report_id: file.group_report,
            length: file.group_length,
            slots: multi_slots(&file.groups)?,
        },
    ))
}

fn single_slots(values: &[[usize; 3]]) -> Result<Vec<Vec<RgbAt>>, String> {
    Ok(values.iter().map(|slot| one(slot[0], slot[1], slot[2])).collect())
}

fn multi_slots(values: &[Vec<[usize; 3]>]) -> Result<Vec<Vec<RgbAt>>, String> {
    values
        .iter()
        .map(|triples| {
            if triples.is_empty() {
                return Err("a lamp slot needs a color".to_string());
            }
            Ok(triples
                .iter()
                .map(|slot| at(slot[0], slot[1], slot[2]))
                .collect())
        })
        .collect()
}

fn paint(
    report: &LampReport,
    colors: impl Iterator<Item = Rgb>,
    label: &str,
) -> Result<Vec<u8>, String> {
    let mut bytes = wire_buffer(report, label)?;
    for (triples, color) in report.slots.iter().zip(colors) {
        for slot in triples {
            place(&mut bytes, *slot, color, label)?;
        }
    }
    Ok(bytes)
}

fn probe_one(
    report: &LampReport,
    index: usize,
    color: Rgb,
    label: &str,
) -> Result<Vec<u8>, String> {
    let triples = report
        .slots
        .get(index)
        .ok_or_else(|| format!("{label} {index} is outside the lamp map"))?;
    let mut bytes = wire_buffer(report, label)?;
    for slot in triples {
        place(&mut bytes, *slot, color, label)?;
    }
    Ok(bytes)
}

fn blank(report: &LampReport) -> Result<Vec<u8>, String> {
    wire_buffer(report, "lamp")
}

fn wire_buffer(report: &LampReport, label: &str) -> Result<Vec<u8>, String> {
    if report.length == 0 {
        return Err(format!("{label} report is empty"));
    }
    let mut bytes = vec![0u8; report.length + 1];
    bytes[0] = report.report_id;
    Ok(bytes)
}

fn write_group_buttons(bytes: &mut [u8], leds: &LedFrame) -> Result<(), String> {
    for button in SilkButton::ALL {
        if let ButtonLamp::Group(index) = button.lamp() {
            set_lamp_byte(bytes, index, leds.buttons[button.index()], "group button")?;
        }
    }
    Ok(())
}

fn mono_report(leds: &LedFrame) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0u8; 33];
    bytes[0] = BUTTON_LAMP_REPORT;
    for button in SilkButton::ALL {
        if let ButtonLamp::Mono(index) = button.lamp() {
            set_lamp_byte(&mut bytes, index, leds.buttons[button.index()], "button")?;
        }
    }
    Ok(bytes)
}

fn set_lamp_byte(bytes: &mut [u8], index: usize, on: bool, label: &str) -> Result<(), String> {
    if index == 0 || index >= bytes.len() {
        return Err(format!("{label} byte {index} is outside the report"));
    }
    if on {
        bytes[index] = 255;
    }
    Ok(())
}

fn place(bytes: &mut [u8], slot: RgbAt, color: Rgb, label: &str) -> Result<(), String> {
    for (index, value) in [(slot.r, color.r), (slot.g, color.g), (slot.b, color.b)] {
        if index == 0 || index >= bytes.len() {
            return Err(format!("{label} byte {index} is outside the report"));
        }
        bytes[index] = value;
    }
    Ok(())
}

#[derive(Deserialize)]
struct LampFile {
    matched: bool,
    #[serde(default)]
    pad_report: u8,
    #[serde(default)]
    pad_length: usize,
    #[serde(default)]
    pads: Vec<[usize; 3]>,
    #[serde(default)]
    group_report: u8,
    #[serde(default)]
    group_length: usize,
    #[serde(default)]
    groups: Vec<Vec<[usize; 3]>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::{GroupLed, LedLevel, PadLed, ACTIVE_COLOR, OFF_COLOR, RESTING_COLOR};

    fn frame() -> LedFrame {
        let mut leds = LedFrame {
            groups: [GroupLed {
                level: LedLevel::Dim,
                color: RESTING_COLOR,
            }; 8],
            pads: [PadLed {
                level: LedLevel::Dim,
                color: RESTING_COLOR,
            }; PAD_COUNT],
            buttons: [false; crate::model::SILK_BUTTON_COUNT],
        };
        leds.groups[0] = GroupLed {
            level: LedLevel::Bright,
            color: ACTIVE_COLOR,
        };
        leds.pads[0] = PadLed {
            level: LedLevel::Bright,
            color: ACTIVE_COLOR,
        };
        leds
    }

    #[test]
    fn unmatched_map_produces_no_output_report() {
        let reports = LampMap::unmatched().output_reports(&frame()).unwrap();
        assert!(reports.is_empty());
    }

    #[test]
    fn matched_map_paints_active_and_resting_colors() {
        let mut map = LampMap::unmatched();
        map.matched = true;
        let reports = map.output_reports(&frame()).unwrap();
        assert_eq!(reports.len(), 3);
        let pads = &reports[0];
        assert_eq!(pads.len(), 50);
        assert_eq!(pads[0], 0x80);
        assert_eq!(&pads[37..40], &[255, 255, 255]);
        assert_eq!(&pads[40..43], &[0, 80, 40]);
        let groups = &reports[1];
        assert_eq!(groups.len(), 58);
        assert_eq!(groups[0], 0x81);
        assert_eq!(&groups[1..7], &[255, 255, 255, 255, 255, 255]);
        let resting = [0u8, 80, 40];
        for triple in groups[7..49].chunks(3) {
            assert_eq!(triple, resting);
        }
        assert!(groups[49..].iter().all(|byte| *byte == 0));
        let mono = &reports[2];
        assert_eq!(mono.len(), 33);
        assert_eq!(mono[0], 0x82);
        assert!(mono[1..].iter().all(|byte| *byte == 0));
        assert!(reports.iter().all(|report| report[0] != 0xE0 && report[0] != 0xE1));
    }

    #[test]
    fn a_lit_button_sets_only_its_lamp_byte() {
        let mut map = LampMap::unmatched();
        map.matched = true;
        let mut leds = frame();
        leds.buttons[crate::model::SilkButton::Scene.index()] = true;
        leds.buttons[crate::model::SilkButton::Restart.index()] = true;
        let reports = map.output_reports(&leds).unwrap();
        assert_eq!(reports[2][17], 255);
        assert_eq!(reports[1][49], 255);
        assert!(reports[2].iter().enumerate().skip(1).all(|(index, byte)| {
            *byte == if index == 17 { 255 } else { 0 }
        }));
        assert!(reports[1][50..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn probe_lights_one_pad_and_one_group_and_leaves_button_bytes_zero() {
        let map = LampMap::unmatched();
        let pad = map.probe_pad(0).unwrap();
        assert_eq!(pad.len(), 50);
        assert_eq!(pad.iter().filter(|byte| **byte != 0).count(), 4);
        assert_eq!(&pad[37..40], &[255, 255, 255]);
        let group = map.probe_group(0).unwrap();
        assert_eq!(group.len(), 58);
        assert_eq!(&group[1..7], &[255, 255, 255, 255, 255, 255]);
        assert!(group[7..].iter().all(|byte| *byte == 0));
        let off = map.blank_pad().unwrap();
        assert_eq!(off[0], 0x80);
        assert!(off[1..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn a_disconnected_frame_sends_off_colors() {
        let mut map = LampMap::unmatched();
        map.matched = true;
        let leds = LedFrame {
            groups: [GroupLed {
                level: LedLevel::Off,
                color: OFF_COLOR,
            }; 8],
            pads: [PadLed {
                level: LedLevel::Off,
                color: OFF_COLOR,
            }; PAD_COUNT],
            buttons: [false; crate::model::SILK_BUTTON_COUNT],
        };
        let reports = map.output_reports(&leds).unwrap();
        for report in reports {
            assert_ne!(report[0], 0);
            assert!(report[1..].iter().all(|byte| *byte == 0));
        }
    }

    #[test]
    fn stored_map_uses_the_offsets_the_check_accepted() {
        let stored = parse_lamp_map(include_str!("../docs/led-map.yaml")).unwrap();
        let hypothesis = LampMap::unmatched();
        assert!(stored.matched());
        assert_eq!(stored.pads, hypothesis.pads);
        assert_eq!(stored.groups, hypothesis.groups);
        let groups = &stored.output_reports(&frame()).unwrap()[1];
        assert_eq!(&groups[1..7], &[255, 255, 255, 255, 255, 255]);
        assert!(groups[49..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn a_false_matched_flag_stays_unmatched() {
        let map = parse_lamp_map("matched: false\n").unwrap();
        assert!(!map.matched());
        assert!(map.output_reports(&frame()).unwrap().is_empty());
    }
}
