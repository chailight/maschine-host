use crate::adapter::{AdapterError, HardwareAdapter};
use crate::feedback::FeedbackFrame;
use crate::hidmap::{events_from_reports_held, Assignment, PadHold};
use crate::ledmap::{LampMap, BUTTON_LAMP_REPORT};
use crate::panelmap::PanelMap;
use crate::model::InputEvent;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MK2_VENDOR: u16 = 0x17cc;
pub const MK2_PRODUCT: u16 = 0x1140;
pub const MK2_USAGE_PAGE: u16 = 0xFF01;

pub enum ControllerLink {
    Absent,
    Open(LiveInput),
    Refused(String),
}

pub struct LiveInput {
    #[cfg(target_os = "macos")]
    device: hidapi::HidDevice,
    assignment: Assignment,
    previous: HashMap<u8, Vec<u8>>,
    pad_hold: PadHold,
    lamps: LampMap,
    panels: PanelMap,
    /// Button lamps, sent again on the next poll. The cold controller applies
    /// pad, group, and screen reports first and drops a button report that
    /// arrives in that same burst.
    pending_buttons: Option<Vec<u8>>,
}

impl HardwareAdapter for LiveInput {
    fn poll(&mut self) -> Vec<InputEvent> {
        self.flush_button_lamps();
        // Pad report 0x20 streams faster than this loop. hidapi keeps about 30
        // reports and drops the oldest. Reading one report each pass drops the
        // group report, so a Group button press never changes the lamp.
        let mut queued = Vec::new();
        loop {
            match self.read_report() {
                Ok(Some(report)) => queued.push(report),
                Ok(None) | Err(()) => break,
            }
        }
        ingest_reports(
            &self.assignment,
            &mut self.previous,
            &mut self.pad_hold,
            &queued,
            now_ms(),
        )
    }

    fn submit(&mut self, frame: &FeedbackFrame) -> Result<(), AdapterError> {
        let writes = feedback_writes(&self.lamps, &self.panels, frame)?;
        #[cfg(target_os = "macos")]
        {
            for report in &writes.color {
                self.device
                    .write(report)
                    .map_err(|error| AdapterError::LampWrite(error.to_string()))?;
            }
            for report in &writes.panels {
                self.device
                    .write(report)
                    .map_err(|error| AdapterError::PanelWrite(error.to_string()))?;
            }
            if let Some(report) = &writes.buttons {
                self.device
                    .write(report)
                    .map_err(|error| AdapterError::LampWrite(error.to_string()))?;
            }
            self.pending_buttons = writes.buttons;
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = writes;
        }
        Ok(())
    }
}

impl LiveInput {
    pub fn confirmed_controls(&self) -> usize {
        self.assignment.len()
    }

    pub fn stand_in_press(&self) -> i32 {
        self.assignment.stand_in_press()
    }

    pub fn lamps_matched(&self) -> bool {
        self.lamps.matched()
    }

    pub fn panels_matched(&self) -> bool {
        self.panels.matched()
    }

    fn flush_button_lamps(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(report) = self.pending_buttons.take() {
            let _ = self.device.write(&report);
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.pending_buttons = None;
        }
    }

    fn read_report(&mut self) -> Result<Option<Vec<u8>>, ()> {
        #[cfg(target_os = "macos")]
        {
            let mut buffer = [0u8; 65];
            match self.device.read_timeout(&mut buffer, 0) {
                Ok(0) => Ok(None),
                Ok(size) => Ok(Some(buffer[..size].to_vec())),
                Err(_) => Err(()),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = &self.assignment;
            Ok(None)
        }
    }
}

pub fn open_input(assignment: Assignment, lamps: LampMap, panels: PanelMap) -> ControllerLink {
    #[cfg(target_os = "macos")]
    {
        open_macos(assignment, lamps, panels)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (assignment, lamps, panels);
        ControllerLink::Absent
    }
}

#[cfg(target_os = "macos")]
fn open_macos(assignment: Assignment, lamps: LampMap, panels: PanelMap) -> ControllerLink {
    let api = match hidapi::HidApi::new() {
        Ok(api) => api,
        Err(error) => return ControllerLink::Refused(error.to_string()),
    };
    let mut saw_mk2 = false;
    for info in api.device_list() {
        if info.vendor_id() != MK2_VENDOR || info.product_id() != MK2_PRODUCT {
            continue;
        }
        saw_mk2 = true;
        if info.usage_page() != MK2_USAGE_PAGE {
            continue;
        }
        return match info.open_device(&api) {
            Ok(device) => {
                let _ = device.set_blocking_mode(false);
                ControllerLink::Open(LiveInput {
                    device,
                    assignment,
                    previous: HashMap::new(),
                    pad_hold: PadHold::default(),
                    lamps,
                    panels,
                    pending_buttons: None,
                })
            }
            Err(error) => ControllerLink::Refused(error.to_string()),
        };
    }
    if saw_mk2 {
        ControllerLink::Refused(
            "full MK2 is present but no HID interface with usage page FF01 could be opened"
                .to_string(),
        )
    } else {
        ControllerLink::Absent
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurveyStep {
    pub name: &'static str,
    pub instruction: &'static str,
}

/// Remaining silk-screen controls, in the order the operator performs them.
/// Knobs are one detent each way. Buttons are one press and release.
pub fn survey_steps() -> &'static [SurveyStep] {
    const PRESS: &str = "press and release once, then wait";
    const CW: &str = "turn one detent clockwise, then wait";
    const CCW: &str = "turn one detent counter-clockwise, then wait";
    &[
        SurveyStep { name: "display-button-1", instruction: "leftmost button above the displays: press and release once, then wait" },
        SurveyStep { name: "display-button-2", instruction: "second button above the displays, from the left: press and release once, then wait" },
        SurveyStep { name: "display-button-3", instruction: "third button above the displays, from the left: press and release once, then wait" },
        SurveyStep { name: "display-button-4", instruction: "fourth button above the displays, from the left: press and release once, then wait" },
        SurveyStep { name: "display-button-5", instruction: "fifth button above the displays, from the left: press and release once, then wait" },
        SurveyStep { name: "display-button-6", instruction: "sixth button above the displays, from the left: press and release once, then wait" },
        SurveyStep { name: "display-button-7", instruction: "seventh button above the displays, from the left: press and release once, then wait" },
        SurveyStep { name: "display-button-8", instruction: "rightmost button above the displays: press and release once, then wait" },
        SurveyStep { name: "control", instruction: PRESS },
        SurveyStep { name: "step", instruction: PRESS },
        SurveyStep { name: "browse", instruction: PRESS },
        SurveyStep { name: "sampling", instruction: PRESS },
        SurveyStep { name: "display-left", instruction: "Left button beside the displays: press and release once, then wait" },
        SurveyStep { name: "display-right", instruction: "Right button beside the displays: press and release once, then wait" },
        SurveyStep { name: "all", instruction: PRESS },
        SurveyStep { name: "auto-write", instruction: PRESS },
        SurveyStep { name: "volume-clockwise", instruction: CW },
        SurveyStep { name: "volume-counter-clockwise", instruction: CCW },
        SurveyStep { name: "swing-clockwise", instruction: CW },
        SurveyStep { name: "swing-counter-clockwise", instruction: CCW },
        SurveyStep { name: "tempo-clockwise", instruction: CW },
        SurveyStep { name: "tempo-counter-clockwise", instruction: CCW },
        SurveyStep { name: "master-left", instruction: "master Left: press and release once, then wait" },
        SurveyStep { name: "master-right", instruction: "master Right: press and release once, then wait" },
        SurveyStep { name: "enter", instruction: PRESS },
        SurveyStep { name: "note-repeat", instruction: PRESS },
        SurveyStep { name: "data-clockwise", instruction: "big data encoder: turn one detent clockwise, then wait" },
        SurveyStep { name: "data-counter-clockwise", instruction: "big data encoder: turn one detent counter-clockwise, then wait" },
        SurveyStep { name: "restart", instruction: PRESS },
        SurveyStep { name: "transport-left", instruction: "transport Left: press and release once, then wait" },
        SurveyStep { name: "transport-right", instruction: "transport Right: press and release once, then wait" },
        SurveyStep { name: "grid", instruction: PRESS },
        SurveyStep { name: "play", instruction: PRESS },
        SurveyStep { name: "rec", instruction: PRESS },
        SurveyStep { name: "erase", instruction: PRESS },
        SurveyStep { name: "shift", instruction: "Shift: press and release once, without holding another control, then wait" },
        SurveyStep { name: "scene", instruction: PRESS },
        SurveyStep { name: "pattern", instruction: PRESS },
        SurveyStep { name: "pad-mode", instruction: PRESS },
        SurveyStep { name: "navigate", instruction: PRESS },
        SurveyStep { name: "duplicate", instruction: PRESS },
        SurveyStep { name: "select", instruction: PRESS },
        SurveyStep { name: "solo", instruction: PRESS },
        SurveyStep { name: "mute", instruction: PRESS },
    ]
}

/// Print each confirmed press, release, and encoder step. This path does not
/// send MIDI, HID output, or feature reports. Restart is not in the map, so it
/// stays silent.
pub fn listen(assignment: Assignment) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        listen_macos(assignment)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = assignment;
        Err("HID listen is implemented for macOS only".to_string())
    }
}

/// Log raw input reports. This path never builds a control event and does not
/// send an output or feature report.
pub fn capture(seconds: u64) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        capture_macos(seconds)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = seconds;
        Err("HID capture is implemented for macOS only".to_string())
    }
}

#[cfg(target_os = "macos")]
fn capture_macos(seconds: u64) -> Result<(), String> {
    let api = hidapi::HidApi::new().map_err(|error| format!("HID read refused: {error}"))?;
    let mut saw_mk2 = false;
    let mut device = None;
    for info in api.device_list() {
        if info.vendor_id() != MK2_VENDOR || info.product_id() != MK2_PRODUCT {
            continue;
        }
        saw_mk2 = true;
        if info.usage_page() != MK2_USAGE_PAGE {
            continue;
        }
        println!(
            "interface manufacturer={:?} product={:?} serial={:?} usage_page={:#06x} usage={:#06x}",
            info.manufacturer_string(),
            info.product_string(),
            info.serial_number(),
            info.usage_page(),
            info.usage()
        );
        device = Some(
            info.open_device(&api)
                .map_err(|error| format!("HID read refused: {error}"))?,
        );
        break;
    }
    let Some(device) = device else {
        if saw_mk2 {
            return Err(
                "HID read refused: full MK2 is present but no HID interface with usage page FF01 could be opened"
                    .to_string(),
            );
        }
        return Err("no full MK2 HID interface found".to_string());
    };
    let _ = device.set_blocking_mode(false);
    let started = std::time::Instant::now();
    println!("listening");
    let deadline = started + std::time::Duration::from_secs(seconds);
    let mut buffer = [0u8; 65];
    let mut last: Option<Vec<u8>> = None;
    while std::time::Instant::now() < deadline {
        match device.read_timeout(&mut buffer, 50) {
            Ok(0) => {}
            Ok(size) => {
                let report = buffer[..size].to_vec();
                if last.as_deref() != Some(report.as_slice()) {
                    println!("{} {}", started.elapsed().as_millis(), hex(&report));
                    last = Some(report);
                }
            }
            Err(error) => return Err(format!("HID read refused: {error}")),
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn listen_macos(assignment: Assignment) -> Result<(), String> {
    use std::io::Write;
    let api = hidapi::HidApi::new().map_err(|error| format!("HID read refused: {error}"))?;
    let mut saw_mk2 = false;
    let mut device = None;
    for info in api.device_list() {
        if info.vendor_id() != MK2_VENDOR || info.product_id() != MK2_PRODUCT {
            continue;
        }
        saw_mk2 = true;
        if info.usage_page() != MK2_USAGE_PAGE {
            continue;
        }
        device = Some(
            info.open_device(&api)
                .map_err(|error| format!("HID read refused: {error}"))?,
        );
        break;
    }
    let Some(device) = device else {
        if saw_mk2 {
            return Err(
                "HID read refused: full MK2 is present but no HID interface with usage page FF01 could be opened"
                    .to_string(),
            );
        }
        return Err("no full MK2 HID interface found".to_string());
    };
    let _ = device.set_blocking_mode(false);
    println!("listening for pads, groups, screen knobs, buttons, and the encoder");
    let _ = std::io::stdout().flush();
    let mut buffer = [0u8; 65];
    let mut previous: HashMap<u8, Vec<u8>> = HashMap::new();
    let mut held: HashMap<String, (String, std::time::Instant)> = HashMap::new();
    loop {
        match device.read_timeout(&mut buffer, 50) {
            Ok(0) => {}
            Ok(size) => {
                let report = buffer[..size].to_vec();
                let report_id = report.first().copied().unwrap_or(0);
                let prior = previous.get(&report_id).cloned();
                for heard in
                    crate::hidmap::heard_from_reports(&assignment, prior.as_deref(), &report)
                {
                    if heard.action == crate::model::ControlAction::Press
                        && heard.label.starts_with("pad:")
                    {
                        held.insert(
                            heard.label.clone(),
                            (heard.terminal_line(), std::time::Instant::now()),
                        );
                        continue;
                    }
                    if heard.action == crate::model::ControlAction::Release
                        && heard.label.starts_with("pad:")
                        && held.remove(&heard.label).is_some_and(|(_, at)| {
                            at.elapsed() < std::time::Duration::from_millis(20)
                        })
                    {
                        continue;
                    }
                    println!("{}", heard.terminal_line());
                    let _ = std::io::stdout().flush();
                }
                previous.insert(report_id, report);
            }
            Err(error) => return Err(format!("HID read refused: {error}")),
        }
        let ready: Vec<String> = held
            .iter()
            .filter(|(_, (_, at))| at.elapsed() >= std::time::Duration::from_millis(20))
            .map(|(label, _)| label.clone())
            .collect();
        for label in ready {
            if let Some((line, _)) = held.remove(&label) {
                println!("{line}");
                let _ = std::io::stdout().flush();
            }
        }
    }
}

/// Light the bottom-left pad, then Group A, and wait for Enter after each.
/// This uses the Encdr hypothesis and does not store a lamp map. It does not
/// send a feature report or write LCD bytes.
pub fn light_check() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        light_check_macos()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("the lamp check is implemented for macOS only".to_string())
    }
}

#[cfg(target_os = "macos")]
fn light_check_macos() -> Result<(), String> {
    use std::io::Write;
    let api = hidapi::HidApi::new().map_err(|error| format!("HID read refused: {error}"))?;
    let mut saw_mk2 = false;
    let mut device = None;
    for info in api.device_list() {
        if info.vendor_id() != MK2_VENDOR || info.product_id() != MK2_PRODUCT {
            continue;
        }
        saw_mk2 = true;
        if info.usage_page() != MK2_USAGE_PAGE {
            continue;
        }
        device = Some(
            info.open_device(&api)
                .map_err(|error| format!("HID read refused: {error}"))?,
        );
        break;
    }
    let Some(device) = device else {
        if saw_mk2 {
            return Err(
                "HID read refused: full MK2 is present but no HID interface with usage page FF01 could be opened"
                    .to_string(),
            );
        }
        return Err("no full MK2 HID interface found".to_string());
    };
    let map = LampMap::unmatched();
    let pad = map
        .probe_pad(0)
        .map_err(|error| format!("lamp check: {error}"))?;
    let group = map
        .probe_group(0)
        .map_err(|error| format!("lamp check: {error}"))?;
    let pad_off = map
        .blank_pad()
        .map_err(|error| format!("lamp check: {error}"))?;
    let group_off = map
        .blank_group()
        .map_err(|error| format!("lamp check: {error}"))?;
    println!(
        "Lighting the bottom-left pad only. Other bytes in that report stay 0. NIHardwareAgent is left running."
    );
    write_output(&device, &pad)?;
    wait_for_enter("Press Enter after you have looked at the pads.")?;
    write_output(&device, &pad_off)?;
    println!(
        "Lighting Group A only. Button bytes in that report stay 0."
    );
    write_output(&device, &group)?;
    wait_for_enter("Press Enter after you have looked at the groups and the buttons.")?;
    write_output(&device, &group_off)?;
    println!(
        "The bottom-left pad and Group A are the stored match. This command does not change the map."
    );
    let _ = std::io::stdout().flush();
    Ok(())
}

/// Paint the left panel from the generic encoder example and wait for Enter.
/// The right panel and the lamps are not written. This does not store a map
/// and does not send a feature report. `NIHardwareAgent` is left running.
pub fn panel_check() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        panel_check_macos()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("the panel check is implemented for macOS only".to_string())
    }
}

#[cfg(target_os = "macos")]
fn panel_check_macos() -> Result<(), String> {
    use std::io::Write;
    let device = open_mk2()?;
    let text = std::fs::read_to_string("docs/encoder-panel-example.yaml")
        .map_err(|error| format!("docs/encoder-panel-example.yaml: {error}"))?;
    let config = crate::config::load_yaml(&text).map_err(|issues| {
        issues
            .iter()
            .map(|issue| issue.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let frame = crate::feedback::render(&crate::controller::Controller::new(config));
    let map = PanelMap::unmatched();
    let left = map.probe_left(&frame.screens[0]);
    println!(
        "Painting the left panel only. Page 1 is on the top line, and enc 1 through enc 4 are in the four columns above the knobs. The right panel and the lamps are not written. NIHardwareAgent is left running."
    );
    for strip in &left {
        write_output(&device, strip)?;
    }
    wait_for_enter(
        "Press Enter after you have looked at the left panel. The right panel and the lamps should be unchanged.",
    )?;
    for strip in map.blank_left() {
        write_output(&device, &strip)?;
    }
    println!("This command does not change the stored panel map.");
    let _ = std::io::stdout().flush();
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_mk2() -> Result<hidapi::HidDevice, String> {
    let api = hidapi::HidApi::new().map_err(|error| format!("HID read refused: {error}"))?;
    let mut saw_mk2 = false;
    for info in api.device_list() {
        if info.vendor_id() != MK2_VENDOR || info.product_id() != MK2_PRODUCT {
            continue;
        }
        saw_mk2 = true;
        if info.usage_page() != MK2_USAGE_PAGE {
            continue;
        }
        return info
            .open_device(&api)
            .map_err(|error| format!("HID read refused: {error}"));
    }
    if saw_mk2 {
        return Err(
            "HID read refused: full MK2 is present but no HID interface with usage page FF01 could be opened"
                .to_string(),
        );
    }
    Err("no full MK2 HID interface found".to_string())
}

#[cfg(target_os = "macos")]
fn write_output(device: &hidapi::HidDevice, report: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let written = device
        .write(report)
        .map_err(|error| format!("MK2 lamp write failed: {error}"))?;
    if written != report.len() {
        return Err(format!(
            "MK2 lamp write sent {written} of {} bytes",
            report.len()
        ));
    }
    println!("wrote {written} bytes");
    let _ = std::io::stdout().flush();
    Ok(())
}

#[cfg(target_os = "macos")]
fn wait_for_enter(prompt: &str) -> Result<(), String> {
    use std::io::{BufRead, Write};
    println!("{prompt}");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map(|_| ())
        .map_err(|error| format!("lamp check: {error}"))
}

struct FeedbackWrites {
    color: Vec<Vec<u8>>,
    panels: Vec<Vec<u8>>,
    buttons: Option<Vec<u8>>,
}

/// Pad and group lamps, then the screens, then the silk-button lamps.
/// The button report is also returned so the next poll can send it again
/// after the controller has finished the screen burst.
fn feedback_writes(
    lamps: &LampMap,
    panels: &PanelMap,
    frame: &FeedbackFrame,
) -> Result<FeedbackWrites, AdapterError> {
    let mut color = Vec::new();
    let mut buttons = None;
    for report in lamps
        .output_reports(&frame.leds)
        .map_err(AdapterError::LampWrite)?
    {
        if report.first() == Some(&BUTTON_LAMP_REPORT) {
            buttons = Some(report);
        } else {
            color.push(report);
        }
    }
    Ok(FeedbackWrites {
        color,
        panels: panels
            .output_reports(&frame.screens)
            .map_err(AdapterError::PanelWrite)?,
        buttons,
    })
}

pub fn presents_controller_connected(link: &ControllerLink) -> bool {
    matches!(link, ControllerLink::Open(_))
}

fn ingest_reports(
    assignment: &Assignment,
    previous: &mut HashMap<u8, Vec<u8>>,
    pad_hold: &mut PadHold,
    reports: &[Vec<u8>],
    time_ms: u64,
) -> Vec<InputEvent> {
    let mut events = Vec::new();
    for report in reports {
        let report_id = report.first().copied().unwrap_or(0);
        let prior = match previous.get(&report_id) {
            Some(prior) => prior.clone(),
            None => assignment.settled_baseline(report),
        };
        events.extend(events_from_reports_held(
            assignment,
            Some(&prior),
            report,
            time_ms,
            pad_hold,
        ));
        previous.insert(report_id, report.clone());
    }
    events
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(target_os = "macos")]
fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_press_queued_behind_pad_reports_is_kept() {
        let assignment = Assignment::load_yaml(
            "stand_in_press: 100\nfields:\n  - {control: pad:0, kind: adc12, report: 32, value_byte: 25, high_byte: 26}\n  - {control: group:B, kind: bit, report: 1, byte: 4, bit: 1}\n",
        )
        .unwrap();
        let pad = vec![0x20, 0, 0, 0, 0];
        let mut pressed = pad.clone();
        pressed.resize(27, 0);
        pressed[25] = 0xFF;
        pressed[26] = 0x0F;
        let idle_group = vec![0x01, 0, 0, 0, 0];
        let mut group_b = idle_group.clone();
        group_b[4] = 0x02;
        let reports = vec![
            pad.clone(),
            pressed,
            idle_group,
            vec![0x20, 0, 0, 0, 0],
            group_b,
        ];
        let mut previous = HashMap::new();
        let mut pad_hold = PadHold::default();
        let events = ingest_reports(&assignment, &mut previous, &mut pad_hold, &reports, 10);
        assert!(events
            .iter()
            .any(|event| event.control == crate::model::ControlId::Group(crate::model::GroupId::B)));
    }

    #[test]
    fn the_first_button_report_presses_without_stepping_an_encoder() {
        let assignment = Assignment::load_yaml(
            "stand_in_press: 100\nfields:\n  - {control: display-button-2, kind: bit, report: 1, byte: 1, bit: 1}\n  - {control: encoder:0, kind: pos10, report: 1, low_byte: 9, high_byte: 10, scale: 4}\n  - {control: pad:0, kind: adc12, report: 32, value_byte: 1, high_byte: 2}\n",
        )
        .unwrap();
        let mut button = vec![0x01];
        button.resize(11, 0);
        button[1] = 0x02;
        button[9] = 0x40;
        button[10] = 0x01;
        let pad = vec![0x20, 0xff, 0x0f];
        let mut previous = HashMap::new();
        let mut pad_hold = PadHold::default();
        let events = ingest_reports(
            &assignment,
            &mut previous,
            &mut pad_hold,
            &[button, pad],
            5,
        );
        assert_eq!(
            events,
            vec![InputEvent::press(
                crate::model::ControlId::Button(crate::model::SilkButton::Display2),
                100,
                5
            )]
        );
    }

    #[test]
    fn page_button_lamps_follow_the_pads_groups_and_screens() {
        let lamps = crate::ledmap::parse_lamp_map(include_str!("../docs/led-map.yaml")).unwrap();
        let panels =
            crate::panelmap::parse_panel_map(include_str!("../docs/panel-map.yaml")).unwrap();
        let mut frame = crate::feedback::FeedbackFrame {
            left_text: String::new(),
            right_text: String::new(),
            screens: [
                crate::feedback::MonoScreen::blank(),
                crate::feedback::MonoScreen::blank(),
            ],
            leds: crate::feedback::LedFrame {
                groups: [crate::feedback::GroupLed {
                    level: crate::feedback::LedLevel::Dim,
                    color: crate::feedback::RESTING_COLOR,
                }; 8],
                pads: [crate::feedback::PadLed {
                    level: crate::feedback::LedLevel::Dim,
                    color: crate::feedback::RESTING_COLOR,
                }; crate::model::PAD_COUNT],
                buttons: [false; crate::model::SILK_BUTTON_COUNT],
            },
            full_refresh: true,
            commanded_only: true,
        };
        frame.leds.buttons[crate::model::SilkButton::Display1.index()] = true;
        let writes = super::feedback_writes(&lamps, &panels, &frame).unwrap();
        assert_eq!(writes.color[0][0], 0x80);
        assert_eq!(writes.color[1][0], 0x81);
        assert!(writes.panels.iter().any(|report| report[0] == 0xE0));
        assert!(writes.panels.iter().any(|report| report[0] == 0xE1));
        let buttons = writes.buttons.expect("button lamps");
        assert_eq!(buttons[0], crate::ledmap::BUTTON_LAMP_REPORT);
        assert_eq!(buttons[9], 255);
    }

    #[test]
    fn refused_open_does_not_present_the_controller_as_connected() {
        let refused = ControllerLink::Refused("device busy".to_string());
        assert!(!presents_controller_connected(&refused));
        assert!(!presents_controller_connected(&ControllerLink::Absent));
    }

    #[test]
    fn survey_names_each_remaining_silk_screen_control_once() {
        let steps = survey_steps();
        let names: Vec<&str> = steps.iter().map(|step| step.name).collect();
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len());
        for name in [
            "display-button-1",
            "display-button-8",
            "control",
            "step",
            "browse",
            "sampling",
            "display-left",
            "display-right",
            "all",
            "auto-write",
            "volume-clockwise",
            "volume-counter-clockwise",
            "swing-clockwise",
            "swing-counter-clockwise",
            "tempo-clockwise",
            "tempo-counter-clockwise",
            "master-left",
            "master-right",
            "enter",
            "note-repeat",
            "data-clockwise",
            "data-counter-clockwise",
            "restart",
            "transport-left",
            "transport-right",
            "grid",
            "play",
            "rec",
            "erase",
            "shift",
            "scene",
            "pattern",
            "pad-mode",
            "navigate",
            "duplicate",
            "select",
            "solo",
            "mute",
        ] {
            assert!(names.contains(&name), "{name}");
        }
        assert_eq!(names.len(), 44);
    }

    #[test]
    fn capture_and_listen_do_not_send_output_or_feature_reports() {
        let source = include_str!("hidopen.rs");
        let capture = function_body(source, "fn capture_macos");
        let listen = function_body(source, "fn listen_macos");
        for body in [capture, listen] {
            assert!(!body.contains(".write("));
            assert!(!body.contains("send_feature_report"));
            assert!(!body.contains("send_output_report"));
        }
    }

    fn function_body(source: &str, name: &str) -> String {
        let start = source
            .find(name)
            .unwrap_or_else(|| panic!("{name} missing"));
        let rest = &source[start + name.len()..];
        let end = ["\nfn ", "\npub fn "]
            .into_iter()
            .filter_map(|marker| rest.find(marker))
            .min()
            .unwrap_or(rest.len());
        rest[..end].to_string()
    }
}
