//! LCD panel reports.
//!
//! The attached MK2's HID report descriptor defines output report `0xE0` for
//! the left panel and `0xE1` for the right. Each report is four 16-bit fields
//! and 256 bytes: x, y, row width in bytes, and row count. A captured frame
//! for product `1140` uses width `32` and height `8`, with y at `0`, `8`,
//! and so on through `56`. Pixels are one bit, MSB first, 32 bytes per row.
//! Width `256` reached the panel and cut single lines through the picture,
//! because that value is a pixel count rather than the byte width. On macOS
//! the report id stays in byte 0. The running host sends these reports only
//! when a stored map says the check matched.

use crate::feedback::{MonoScreen, SCREEN_COUNT, SCREEN_HEIGHT, SCREEN_WIDTH};
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

pub const PANEL_MAP_PATH: &str = "docs/panel-map.yaml";
pub const LEFT_REPORT_ID: u8 = 0xE0;
pub const RIGHT_REPORT_ID: u8 = 0xE1;
const ROW_STRIDE: usize = SCREEN_WIDTH / 8;
const STRIP_ROWS: usize = 8;
const STRIP_BYTES: usize = ROW_STRIDE * STRIP_ROWS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelMap {
    matched: bool,
    left_report_id: u8,
    right_report_id: u8,
}

impl PanelMap {
    pub fn unmatched() -> Self {
        Self {
            matched: false,
            left_report_id: LEFT_REPORT_ID,
            right_report_id: RIGHT_REPORT_ID,
        }
    }

    pub fn matched(&self) -> bool {
        self.matched
    }

    pub fn output_reports(
        &self,
        screens: &[MonoScreen; SCREEN_COUNT],
    ) -> Result<Vec<Vec<u8>>, String> {
        if !self.matched {
            return Ok(Vec::new());
        }
        let mut reports = strips(self.left_report_id, &screens[0]);
        reports.extend(strips(self.right_report_id, &screens[1]));
        Ok(reports)
    }

    pub fn probe_left(&self, screen: &MonoScreen) -> Vec<Vec<u8>> {
        strips(self.left_report_id, screen)
    }

    pub fn blank_left(&self) -> Vec<Vec<u8>> {
        strips(self.left_report_id, &MonoScreen::blank())
    }
}

pub fn load_panel_map(path: &Path) -> Result<PanelMap, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(PanelMap::unmatched()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    parse_panel_map(&text).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn parse_panel_map(text: &str) -> Result<PanelMap, String> {
    let file: PanelFile = serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    if !file.matched {
        return Ok(PanelMap::unmatched());
    }
    Ok(PanelMap {
        matched: true,
        left_report_id: file.left_report_id,
        right_report_id: file.right_report_id,
    })
}

fn strips(report_id: u8, screen: &MonoScreen) -> Vec<Vec<u8>> {
    (0..SCREEN_HEIGHT)
        .step_by(STRIP_ROWS)
        .map(|y| strip(report_id, screen, y))
        .collect()
}

fn strip(report_id: u8, screen: &MonoScreen, y: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; 1 + 8 + STRIP_BYTES];
    bytes[0] = report_id;
    put_u16(&mut bytes, 1, 0);
    put_u16(&mut bytes, 3, y as u16);
    put_u16(&mut bytes, 5, ROW_STRIDE as u16);
    put_u16(&mut bytes, 7, STRIP_ROWS as u16);
    for row in 0..STRIP_ROWS {
        let pixel_y = y + row;
        if pixel_y >= screen.height {
            break;
        }
        for x in 0..SCREEN_WIDTH.min(screen.width) {
            if screen.pixels[pixel_y * screen.width + x] == 0 {
                continue;
            }
            let index = 9 + row * ROW_STRIDE + x / 8;
            bytes[index] |= 0x80 >> (x % 8);
        }
    }
    bytes
}

fn put_u16(bytes: &mut [u8], index: usize, value: u16) {
    bytes[index] = value as u8;
    bytes[index + 1] = (value >> 8) as u8;
}

#[derive(Deserialize)]
struct PanelFile {
    matched: bool,
    #[serde(default = "default_left")]
    left_report_id: u8,
    #[serde(default = "default_right")]
    right_report_id: u8,
}

fn default_left() -> u8 {
    LEFT_REPORT_ID
}

fn default_right() -> u8 {
    RIGHT_REPORT_ID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmatched_map_sends_no_panel_report() {
        let screens = [MonoScreen::blank(), MonoScreen::blank()];
        assert!(PanelMap::unmatched()
            .output_reports(&screens)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn a_matched_map_sends_the_left_and_right_panels() {
        let map =
            parse_panel_map("matched: true\nleft_report_id: 224\nright_report_id: 225\n").unwrap();
        let mut left = MonoScreen::blank();
        left.pixels[0] = 1;
        let reports = map
            .output_reports(&[left, MonoScreen::blank()])
            .unwrap();
        assert_eq!(reports.len(), 16);
        assert_eq!(reports[0][0], 0xE0);
        assert_eq!(reports[0].len(), 265);
        assert_eq!(&reports[0][1..9], &[0, 0, 0, 0, 0x20, 0, 8, 0]);
        assert_eq!(reports[0][9], 0x80);
        assert_eq!(&reports[7][1..9], &[0, 0, 0x38, 0, 0x20, 0, 8, 0]);
        assert_eq!(reports[8][0], 0xE1);
        assert!(reports[8][9..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn the_stored_map_matches_the_live_left_panel() {
        let map = load_panel_map(Path::new(PANEL_MAP_PATH)).unwrap();
        assert!(map.matched());
        let screens = [MonoScreen::blank(), MonoScreen::blank()];
        let reports = map.output_reports(&screens).unwrap();
        assert_eq!(reports.len(), 16);
        assert!(reports[..8].iter().all(|report| report[0] == 0xE0));
        assert!(reports[8..].iter().all(|report| report[0] == 0xE1));
    }
}
