use crate::controller::Controller;
use crate::model::{
    ConnectionStatus, EncoderKind, GroupId, PadMode, ENCODER_COUNT, PAD_COUNT, SILK_BUTTON_COUNT,
};

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 64;
pub const SCREEN_COUNT: usize = 2;

/// Semantic levels. These are not USB LED packets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedLevel {
    Off,
    Dim,
    Bright,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const ACTIVE_COLOR: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 255,
};
pub const RESTING_COLOR: Rgb = Rgb { r: 0, g: 80, b: 40 };
pub const OFF_COLOR: Rgb = Rgb { r: 0, g: 0, b: 0 };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadLed {
    pub level: LedLevel,
    pub color: Rgb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupLed {
    pub level: LedLevel,
    pub color: Rgb,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedFrame {
    pub groups: [GroupLed; 8],
    pub pads: [PadLed; PAD_COUNT],
    pub buttons: [bool; SILK_BUTTON_COUNT],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonoScreen {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl MonoScreen {
    pub fn blank() -> Self {
        Self {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            pixels: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn ink_count(&self) -> usize {
        self.pixels.iter().filter(|pixel| **pixel != 0).count()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackFrame {
    pub left_text: String,
    pub right_text: String,
    pub screens: [MonoScreen; SCREEN_COUNT],
    pub leds: LedFrame,
    pub full_refresh: bool,
    /// Application-side values are commanded, not read back from an instrument.
    pub commanded_only: bool,
}

pub fn render(controller: &Controller) -> FeedbackFrame {
    let full_refresh = controller.force_refresh;
    let disconnected = controller.connection == ConnectionStatus::Disconnected;
    let (left_text, right_text, leds) = if disconnected {
        (
            "DISCONNECTED\nnotes cleared\ncommanded value is not device truth".to_string(),
            "DISCONNECTED".to_string(),
            leds_off(),
        )
    } else {
        let group = &controller.config.groups[controller.active_group.index()];
        let page = &controller.config.encoder_pages[controller.active_page];
        let octave = if group.mode == PadMode::OneShot {
            String::new()
        } else {
            format!(" oct {:+}", controller.octave[group.id.index()])
        };
        let left = format!(
            "GROUP {}  {}\n{}{octave}\n{}\ncommanded value is not device truth",
            group.id,
            group.label,
            group.mode.label(),
            group.destination.summary()
        );
        let mut right = format!("PAGE {}\n", page.label);
        for index in 0..ENCODER_COUNT {
            let encoder = &page.encoders[index];
            let (label, shown, _) = controller.encoder_text(controller.active_page, index);
            let suffix = match &encoder.kind {
                EncoderKind::MidiCc { cc: None, .. } => "no cc",
                _ => "cmd",
            };
            right.push_str(&format!("{} {label} {shown} {suffix}\n", index + 1));
        }
        (left, right, leds_for(controller))
    };
    let mut screens = [MonoScreen::blank(), MonoScreen::blank()];
    if !disconnected {
        draw_encoder_panels(&mut screens, controller);
    }
    FeedbackFrame {
        left_text,
        right_text,
        screens,
        leds,
        full_refresh,
        commanded_only: true,
    }
}

const DIM_SCALE: u16 = 64;

fn shade(color: [u8; 3], bright: bool) -> Rgb {
    let scale = if bright { 255 } else { DIM_SCALE };
    Rgb {
        r: scale_channel(color[0], scale),
        g: scale_channel(color[1], scale),
        b: scale_channel(color[2], scale),
    }
}

fn scale_channel(value: u8, scale: u16) -> u8 {
    if value == 0 {
        0
    } else {
        (u16::from(value) * scale / 255).max(1) as u8
    }
}

fn leds_off() -> LedFrame {
    LedFrame {
        groups: [GroupLed {
            level: LedLevel::Off,
            color: OFF_COLOR,
        }; 8],
        pads: [PadLed {
            level: LedLevel::Off,
            color: OFF_COLOR,
        }; PAD_COUNT],
        buttons: [false; SILK_BUTTON_COUNT],
    }
}

fn leds_for(controller: &Controller) -> LedFrame {
    let mut frame = leds_off();
    let selected = controller.active_group;
    for id in GroupId::ALL {
        let color = controller.config.groups[id.index()].color;
        let bright = id == selected;
        frame.groups[id.index()] = GroupLed {
            level: if bright {
                LedLevel::Bright
            } else {
                LedLevel::Dim
            },
            color: shade(color, bright),
        };
    }
    let pad_color = controller.config.groups[selected.index()].color;
    for pad in 0..PAD_COUNT {
        let bright = controller.held.iter().any(|note| note.pad as usize == pad);
        frame.pads[pad] = PadLed {
            level: if bright {
                LedLevel::Bright
            } else {
                LedLevel::Dim
            },
            color: shade(pad_color, bright),
        };
    }
    frame.buttons = controller.button_on;
    frame
}

const COLUMN_COUNT: usize = 4;
const COLUMN_WIDTH: usize = SCREEN_WIDTH / COLUMN_COUNT;
const GLYPH_ADVANCE: usize = 6;
const SCOPE_Y: usize = 28;
const LABEL_Y: usize = 40;
const VALUE_Y: usize = 56;
const VALUE_MARK: usize = 8;
const UNIT_PITCH: usize = LABEL_Y - SCOPE_Y;

fn draw_encoder_panels(screens: &mut [MonoScreen; SCREEN_COUNT], controller: &Controller) {
    let page_index = controller.active_page;
    let page = &controller.config.encoder_pages[page_index];
    let names: Vec<&str> = controller
        .config
        .categories
        .iter()
        .map(|category| category.id.as_str())
        .collect();
    for screen_index in 0..SCREEN_COUNT {
        if names.len() != 8 {
            draw_span(&mut screens[screen_index], 0, 0, &page.label, None);
        }
        if page.id == "char-pick" && screen_index == 1 && names.len() == 8 {
            for slot in 0..COLUMN_COUNT {
                let origin = slot * COLUMN_WIDTH;
                let clip = Some((origin, origin + COLUMN_WIDTH));
                draw_centered(
                    &mut screens[screen_index],
                    origin,
                    0,
                    names[screen_index * COLUMN_COUNT + slot],
                    clip,
                );
            }
            draw_character_help(&mut screens[screen_index], controller.character_type_help());
            continue;
        }
        for slot in 0..COLUMN_COUNT {
            let index = screen_index * COLUMN_COUNT + slot;
            let (label, shown, scope) = controller.encoder_text(page_index, index);
            let origin = slot * COLUMN_WIDTH;
            let clip = Some((origin, origin + COLUMN_WIDTH));
            if names.len() == 8 {
                draw_centered(
                    &mut screens[screen_index],
                    origin,
                    0,
                    names[index],
                    clip,
                );
                draw_centered(&mut screens[screen_index], origin, SCOPE_Y, &scope, clip);
            }
            draw_centered(
                &mut screens[screen_index],
                origin,
                LABEL_Y,
                &label,
                clip,
            );
            if controller.reset_flashing(index) {
                draw_value_mark(&mut screens[screen_index], origin);
            } else {
                draw_centered(
                    &mut screens[screen_index],
                    origin,
                    VALUE_Y,
                    &shown,
                    clip,
                );
            }
        }
    }
}

fn draw_character_help(screen: &mut MonoScreen, rows: [(&str, &str, &str); 4]) {
    let line_y = [SCOPE_Y, VALUE_Y - UNIT_PITCH, VALUE_Y];
    for (column, row) in rows.iter().enumerate() {
        let origin = column * COLUMN_WIDTH;
        let clip = Some((origin, origin + COLUMN_WIDTH));
        for (text, y) in [row.0, row.1, row.2].iter().zip(line_y) {
            if text.is_empty() {
                continue;
            }
            draw_centered(screen, origin, y, text, clip);
        }
    }
}

fn draw_value_mark(screen: &mut MonoScreen, column_x: usize) {
    let origin = column_x + (COLUMN_WIDTH - VALUE_MARK) / 2;
    for y in VALUE_Y..VALUE_Y + VALUE_MARK {
        for x in origin..origin + VALUE_MARK {
            paint(screen, x, y, None);
        }
    }
}

fn draw_centered(
    screen: &mut MonoScreen,
    column_x: usize,
    y: usize,
    text: &str,
    clip: Option<(usize, usize)>,
) {
    let columns = text.chars().count();
    let width = if columns == 0 {
        0
    } else {
        columns * GLYPH_ADVANCE - 1
    };
    let x = if width >= COLUMN_WIDTH {
        column_x
    } else {
        column_x + (COLUMN_WIDTH - width) / 2
    };
    draw_span(screen, x, y, text, clip);
}

fn draw_span(
    screen: &mut MonoScreen,
    mut x: usize,
    y: usize,
    text: &str,
    clip: Option<(usize, usize)>,
) {
    for ch in text.chars() {
        if let Some((_, right)) = clip {
            if x >= right {
                break;
            }
        }
        if y + 7 < screen.height {
            blit_glyph(screen, x, y, ch, clip);
        }
        x += GLYPH_ADVANCE;
        if x >= screen.width {
            break;
        }
    }
}

#[cfg(test)]
fn encoder_panel_text(controller: &Controller) -> (String, String) {
    let page_index = controller.active_page;
    let page = &controller.config.encoder_pages[page_index];
    let rows: Vec<String> = page
        .encoders
        .iter()
        .enumerate()
        .map(|(index, encoder)| {
            let (_, shown, _) = controller.encoder_text(page_index, index);
            format!("{} {shown}", encoder.label)
        })
        .collect();
    let left = format!("{}\n{}", page.label, rows[..4].join("\n"));
    let right = rows[4..].join("\n");
    (left, right)
}

fn blit_glyph(
    screen: &mut MonoScreen,
    origin_x: usize,
    origin_y: usize,
    ch: char,
    clip: Option<(usize, usize)>,
) {
    let rows = glyph_rows(ch);
    for (row_index, row) in rows.iter().enumerate() {
        for (column, bit) in row.bytes().enumerate() {
            if bit == b'#' {
                paint(screen, origin_x + column, origin_y + row_index, clip);
            }
        }
    }
}

fn paint(screen: &mut MonoScreen, x: usize, y: usize, clip: Option<(usize, usize)>) {
    if let Some((left, right)) = clip {
        if x < left || x >= right {
            return;
        }
    }
    if x < screen.width && y < screen.height {
        screen.pixels[y * screen.width + x] = 1;
    }
}

fn glyph_rows(ch: char) -> [&'static str; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [
            " ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ],
        'B' => [
            "#### ", "#   #", "#   #", "#### ", "#   #", "#   #", "#### ",
        ],
        'C' => [
            " ### ", "#   #", "#    ", "#    ", "#    ", "#   #", " ### ",
        ],
        'D' => [
            "#### ", "#   #", "#   #", "#   #", "#   #", "#   #", "#### ",
        ],
        'E' => [
            "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#####",
        ],
        'F' => [
            "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#    ",
        ],
        'G' => [
            " ### ", "#   #", "#    ", "# ###", "#   #", "#   #", " ### ",
        ],
        'H' => [
            "#   #", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ],
        'I' => [
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "#####",
        ],
        'J' => [
            "  ###", "   # ", "   # ", "   # ", "#  # ", "#  # ", " ##  ",
        ],
        'K' => [
            "#   #", "#  # ", "# #  ", "##   ", "# #  ", "#  # ", "#   #",
        ],
        'L' => [
            "#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####",
        ],
        'M' => [
            "#   #", "## ##", "# # #", "#   #", "#   #", "#   #", "#   #",
        ],
        'N' => [
            "#   #", "##  #", "# # #", "#  ##", "#   #", "#   #", "#   #",
        ],
        'O' => [
            " ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'P' => [
            "#### ", "#   #", "#   #", "#### ", "#    ", "#    ", "#    ",
        ],
        'Q' => [
            " ### ", "#   #", "#   #", "#   #", "# # #", "#  # ", " ## #",
        ],
        'R' => [
            "#### ", "#   #", "#   #", "#### ", "# #  ", "#  # ", "#   #",
        ],
        'S' => [
            " ####", "#    ", "#    ", " ### ", "    #", "    #", "#### ",
        ],
        'T' => [
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'U' => [
            "#   #", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'V' => [
            "#   #", "#   #", "#   #", "#   #", "#   #", " # # ", "  #  ",
        ],
        'W' => [
            "#   #", "#   #", "#   #", "# # #", "# # #", "## ##", "#   #",
        ],
        'X' => [
            "#   #", "#   #", " # # ", "  #  ", " # # ", "#   #", "#   #",
        ],
        'Y' => [
            "#   #", "#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'Z' => [
            "#####", "    #", "   # ", "  #  ", " #   ", "#    ", "#####",
        ],
        '0' => [
            " ### ", "#   #", "#  ##", "# # #", "##  #", "#   #", " ### ",
        ],
        '1' => [
            "  #  ", " ##  ", "  #  ", "  #  ", "  #  ", "  #  ", " ### ",
        ],
        '2' => [
            " ### ", "#   #", "    #", "  ## ", " #   ", "#    ", "#####",
        ],
        '3' => [
            "#### ", "    #", "    #", " ### ", "    #", "    #", "#### ",
        ],
        '4' => [
            "   # ", "  ## ", " # # ", "#  # ", "#####", "   # ", "   # ",
        ],
        '5' => [
            "#####", "#    ", "#### ", "    #", "    #", "#   #", " ### ",
        ],
        '6' => [
            " ### ", "#    ", "#    ", "#### ", "#   #", "#   #", " ### ",
        ],
        '7' => [
            "#####", "    #", "   # ", "  #  ", " #   ", " #   ", " #   ",
        ],
        '8' => [
            " ### ", "#   #", "#   #", " ### ", "#   #", "#   #", " ### ",
        ],
        '9' => [
            " ### ", "#   #", "#   #", " ####", "    #", "    #", " ### ",
        ],
        '-' => [
            "     ", "     ", "     ", "#####", "     ", "     ", "     ",
        ],
        '+' => [
            "     ", "  #  ", "  #  ", "#####", "  #  ", "  #  ", "     ",
        ],
        '.' => [
            "     ", "     ", "     ", "     ", "     ", " ##  ", " ##  ",
        ],
        ':' => [
            "     ", " ##  ", " ##  ", "     ", " ##  ", " ##  ", "     ",
        ],
        '/' => [
            "    #", "    #", "   # ", "  #  ", " #   ", "#    ", "#    ",
        ],
        ' ' => [
            "     ", "     ", "     ", "     ", "     ", "     ", "     ",
        ],
        _ => [
            "#####", "#   #", "#   #", "#   #", "#   #", "#   #", "#####",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load_yaml;
    use crate::controller::Controller;
    use crate::model::{ControlId, GroupId, InputEvent, SilkButton, ENCODER_COUNT};

    fn column_ink(screen: &MonoScreen, column: usize, y0: usize, y1: usize) -> bool {
        let x0 = column * 64;
        let x1 = x0 + 64;
        (y0..y1).any(|y| {
            (x0..x1).any(|x| screen.pixels[y * screen.width + x] != 0)
        })
    }

    fn first_ink_x(screen: &MonoScreen, x0: usize, x1: usize, y0: usize, y1: usize) -> usize {
        (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .find(|(x, y)| screen.pixels[y * screen.width + x] != 0)
            .map(|(x, _)| x)
            .unwrap_or(usize::MAX)
    }

    fn host() -> Controller {
        let notes = (36..52)
            .map(|note| note.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let mut groups = String::new();
        for (index, id) in GroupId::ALL.iter().enumerate() {
            groups.push_str(&format!(
                "  - id: {id}\n    label: Name {id}\n    mode: one_shot\n    destination: {{type: midi, port: bitbox, channel: {}}}\n    notes: [{notes}]\n",
                index + 1
            ));
        }
        let mut encoders = String::new();
        for index in 1..=ENCODER_COUNT {
            encoders.push_str(&format!(
                "      - {{label: Slot {index}, type: midi_cc, port: bitbox, channel: {index}, cc: null}}\n"
            ));
        }
        let text = format!(
            "schema_version: 1\nmidi_outputs:\n  bitbox: PORT\ngroups:\n{groups}encoder_pages:\n  - id: delay\n    label: Delay sends\n    encoders:\n{encoders}"
        );
        Controller::new(load_yaml(&text).unwrap())
    }

    #[test]
    fn snapshots_name_the_group_and_mark_values_as_commanded() {
        let mut host = host();
        host.handle(InputEvent::press(ControlId::Group(GroupId::B), 1, 1));
        host.handle(InputEvent::delta(ControlId::Encoder(0), 4, 2));
        let frame = render(&host);
        assert!(frame.left_text.contains("GROUP B"));
        assert!(frame.left_text.contains("Name B"));
        assert!(frame.left_text.contains("one-shot"));
        assert!(frame.right_text.contains("Delay sends"));
        assert!(frame.right_text.contains("Slot 1 4 no cc"));
        assert!(frame.commanded_only);
        assert_eq!(frame.screens[0].width, SCREEN_WIDTH);
        assert_eq!(frame.screens[0].height, SCREEN_HEIGHT);
        assert_eq!(frame.screens.len(), SCREEN_COUNT);
        assert!(frame.screens[0].ink_count() > 0);
        assert!(frame.screens[1].ink_count() > 0);
        assert!(column_ink(&frame.screens[0], 0, 0, 8));
        for column in 0..4 {
            assert!(column_ink(&frame.screens[0], column, 40, 48));
            assert!(column_ink(&frame.screens[0], column, 56, 64));
            assert!(column_ink(&frame.screens[1], column, 40, 48));
        }
        assert!(first_ink_x(&frame.screens[0], 0, 64, 40, 48) >= 8);
        let (left_panel, right_panel) = encoder_panel_text(&host);
        assert!(left_panel.starts_with("Delay sends\nSlot 1 4"));
        assert!(right_panel.starts_with("Slot 5"));
        assert_eq!(
            frame.leds.groups[GroupId::B.index()].level,
            LedLevel::Bright
        );
        assert_eq!(frame.leds.groups[GroupId::A.index()].level, LedLevel::Dim);
    }

    #[test]
    fn encoder_values_stay_on_their_page_and_disconnect_clears_the_panels() {
        let mut encoders = String::new();
        for index in 1..=ENCODER_COUNT {
            encoders.push_str(&format!(
                "      - {{label: enc {index}, type: midi_cc, port: bitbox, channel: {index}, cc: null}}\n"
            ));
        }
        let notes = (36..52)
            .map(|note| note.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let mut groups = String::new();
        for (index, id) in GroupId::ALL.iter().enumerate() {
            groups.push_str(&format!(
                "  - id: {id}\n    label: Name {id}\n    mode: one_shot\n    destination: {{type: midi, port: bitbox, channel: {}}}\n    notes: [{notes}]\n",
                index + 1
            ));
        }
        let text = format!(
            "schema_version: 1\nmidi_outputs:\n  bitbox: PORT\ngroups:\n{groups}encoder_pages:\n  - id: page-1\n    label: Page 1\n    encoders:\n{encoders}  - id: page-2\n    label: Page 2\n    encoders:\n{encoders}"
        );
        let mut host = Controller::new(load_yaml(&text).unwrap());
        host.handle(InputEvent::delta(ControlId::Encoder(0), 4, 1));
        host.handle(InputEvent::press(ControlId::PageRight, 1, 2));
        host.handle(InputEvent::delta(ControlId::Encoder(0), 9, 3));
        let (left, right) = encoder_panel_text(&host);
        assert!(left.starts_with("Page 2\nenc 1 9"));
        assert!(right.starts_with("enc 5"));
        host.handle(InputEvent::press(ControlId::PageLeft, 1, 4));
        let (left, _) = encoder_panel_text(&host);
        assert!(left.starts_with("Page 1\nenc 1 4"));
        assert_eq!(host.parameter(1, 0).unwrap().value, 9);
        host.disconnect();
        let frame = render(&host);
        assert_eq!(frame.screens[0].ink_count(), 0);
        assert_eq!(frame.screens[1].ink_count(), 0);
    }

    #[test]
    fn selected_group_and_held_pad_share_that_groups_color() {
        let mut host = host();
        host.handle(InputEvent::press(ControlId::Group(GroupId::B), 1, 1));
        host.handle(InputEvent::press(ControlId::Pad(3), 100, 2));
        let frame = render(&host);
        let group_b = host.config().groups[GroupId::B.index()].color;
        let group_a = host.config().groups[GroupId::A.index()].color;
        let bright = shade(group_b, true);
        let dim = shade(group_b, false);
        assert_ne!(bright, dim);
        assert_eq!(frame.leds.groups[GroupId::B.index()].color, bright);
        assert_eq!(frame.leds.pads[3].color, bright);
        assert_eq!(frame.leds.pads[0].color, dim);
        assert_eq!(frame.leds.groups[GroupId::A.index()].color, shade(group_a, false));
        host.disconnect();
        let blank = render(&host);
        assert!(blank
            .leds
            .groups
            .iter()
            .all(|group| group.color == OFF_COLOR));
        assert!(blank.leds.pads.iter().all(|pad| pad.color == OFF_COLOR));
    }

    #[test]
    fn pad_activity_is_brighter_than_a_resting_pad() {
        let mut host = host();
        host.handle(InputEvent::press(ControlId::Pad(3), 100, 1));
        let frame = render(&host);
        assert_eq!(frame.leds.pads[3].level, LedLevel::Bright);
        assert_eq!(frame.leds.pads[0].level, LedLevel::Dim);
    }

    #[test]
    fn disconnect_blanks_leds_and_reconnect_restores_the_group() {
        let mut host = host();
        host.handle(InputEvent::press(ControlId::Group(GroupId::E), 1, 1));
        host.disconnect();
        let blank = render(&host);
        assert!(blank.left_text.contains("DISCONNECTED"));
        assert!(blank.leds.groups.iter().all(|group| group.level == LedLevel::Off
            && group.color == OFF_COLOR));
        assert!(blank
            .leds
            .pads
            .iter()
            .all(|pad| pad.level == LedLevel::Off && pad.color == OFF_COLOR));
        assert!(blank.full_refresh);
        host.connect();
        let restored = render(&host);
        assert!(restored.left_text.contains("GROUP E"));
        assert_eq!(
            restored.leds.groups[GroupId::E.index()].level,
            LedLevel::Bright
        );
        assert!(restored.full_refresh);
    }

    #[test]
    fn category_names_sit_under_each_display_button() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display2),
            1,
            1,
        ));
        let frame = render(&host);
        assert!(frame.right_text.contains("PAGE Delay send"));
        assert!(frame.leds.buttons[SilkButton::Display2.index()]);
        assert!(!frame.leds.buttons[SilkButton::Display1.index()]);
        for screen in &frame.screens {
            for column in 0..4 {
                assert!(column_ink(screen, column, 0, 8));
                assert!(!column_ink(screen, column, 12, 20));
            }
        }
        let perform = first_ink_x(&frame.screens[0], 0, 64, 0, 8);
        let mix = first_ink_x(&frame.screens[0], 64, 128, 0, 8);
        assert!(perform < 64);
        assert!(mix >= 64);
        let fresh = Controller::new(
            load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox"),
        );
        assert_eq!(fresh.config.encoder_pages[fresh.active_page()].id, "perform");
        let opening = render(&fresh);
        assert!(opening.leds.buttons[SilkButton::Display1.index()]);
        assert!(!opening.leds.buttons[SilkButton::Display2.index()]);
        let (label, _, _) = fresh.encoder_text(fresh.active_page(), 0);
        assert_eq!(label, "Delay snd");
    }

    #[test]
    fn character_type_page_uses_the_right_screen_for_tone_help() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display5),
            1,
            1,
        ));
        let amount = render(&host);
        assert!(column_ink(&amount.screens[1], 0, super::SCOPE_Y, super::SCOPE_Y + 7));
        assert!(column_ink(&amount.screens[1], 0, super::LABEL_Y, super::LABEL_Y + 7));
        assert!(column_ink(&amount.screens[1], 0, super::VALUE_Y, super::VALUE_Y + 7));
        assert!(column_ink(&amount.screens[1], 3, super::SCOPE_Y, super::SCOPE_Y + 7));
        assert!(!column_ink(&amount.screens[1], 3, super::LABEL_Y, super::LABEL_Y + 7));
        assert!(column_ink(&amount.screens[1], 3, super::VALUE_Y, super::VALUE_Y + 7));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            2,
        ));
        let pick = render(&host);
        assert!(column_ink(&pick.screens[0], 0, super::SCOPE_Y, super::SCOPE_Y + 7));
        assert!(column_ink(&pick.screens[0], 0, super::LABEL_Y, super::LABEL_Y + 7));
        assert!(column_ink(&pick.screens[0], 0, super::VALUE_Y, super::VALUE_Y + 7));
        let help_y = super::VALUE_Y - super::UNIT_PITCH;
        for column in 0..4 {
            assert!(column_ink(&pick.screens[1], column, 0, 8));
            assert!(column_ink(
                &pick.screens[1],
                column,
                super::SCOPE_Y,
                super::SCOPE_Y + 7
            ));
            assert!(column_ink(&pick.screens[1], column, help_y, help_y + 7));
            assert!(!column_ink(
                &pick.screens[1],
                column,
                super::SCOPE_Y + 7,
                help_y
            ));
        }
        for column in 0..3 {
            assert!(column_ink(
                &pick.screens[1],
                column,
                super::VALUE_Y,
                super::VALUE_Y + 7
            ));
        }
        assert!(!column_ink(&pick.screens[1], 3, super::VALUE_Y, super::VALUE_Y + 7));
    }
}
