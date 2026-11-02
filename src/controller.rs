use crate::model::{
    clamp_note, ButtonGesture, ConnectionStatus, ControlAction, ControlId, Destination,
    EncoderKind, EncoderMapping, GroupId, HeldNote, HostConfig, InputEvent, LogLevel, MidiMessage,
    OutputAction, PadMode, ParamScope, ParameterState, RadioSet, SamplerCommand, SamplerCommandKind,
    SilkButton, TriggerPolicy, ValueFormat, ENCODER_COUNT, PAD_COUNT, SILK_BUTTON_COUNT,
};
use crate::sampler::select_zone;
use std::collections::BTreeMap;

const OPENING_FEEDBACK_WINDOW_MS: u64 = 1600;
const OPENING_FEEDBACK_GAP_MS: u64 = 200;
const RESET_FLASH_MS: u64 = 250;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogRecord {
    pub level: LogLevel,
    pub message: String,
}

pub struct Controller {
    pub(crate) config: HostConfig,
    pub(crate) active_group: GroupId,
    pub(crate) active_page: usize,
    pub(crate) octave: [i32; 8],
    pub(crate) parameters: BTreeMap<(usize, u8), ParameterState>,
    pub(crate) shared: BTreeMap<(String, u8), ParameterState>,
    pub(crate) category_page: Vec<usize>,
    pub(crate) clip_layer: bool,
    pub(crate) clip_pads: [bool; PAD_COUNT],
    pub(crate) held: Vec<HeldNote>,
    pub(crate) local_voices: u32,
    pub(crate) connection: ConnectionStatus,
    pub(crate) next_voice: u64,
    pub(crate) button_on: [bool; SILK_BUTTON_COUNT],
    pub(crate) feedback_dirty: bool,
    pub(crate) force_refresh: bool,
    pub(crate) first_feedback_ms: Option<u64>,
    pub(crate) last_feedback_ms: Option<u64>,
    reset_flash_until: [Option<u64>; ENCODER_COUNT],
    lfo_phase: f64,
    lfo_last_ms: Option<u64>,
    lfo_last_sent: Option<(u8, u8)>,
    pub(crate) logs: Vec<LogRecord>,
}

impl Controller {
    pub fn new(config: HostConfig) -> Self {
        let mut parameters = BTreeMap::new();
        for (page_index, page) in config.encoder_pages.iter().enumerate() {
            for (encoder_index, encoder) in page.encoders.iter().enumerate() {
                parameters.insert(
                    (page_index, encoder_index as u8),
                    ParameterState {
                        value: encoder.default_value,
                        source: "default",
                        time_ms: 0,
                    },
                );
            }
        }
        let mut shared = BTreeMap::new();
        for page in &config.encoder_pages {
            for (index, encoder) in page.encoders.iter().enumerate() {
                seed_shared(&mut shared, encoder, index as u8);
            }
            for profile in &page.profiles {
                for (index, encoder) in profile.encoders.iter().enumerate() {
                    seed_shared(&mut shared, encoder, index as u8 + 4);
                }
            }
        }
        let category_page: Vec<usize> = config
            .categories
            .iter()
            .filter_map(|category| category.pages.first().copied())
            .collect();
        let mut button_on = [false; SILK_BUTTON_COUNT];
        let active_page = category_page.first().copied().unwrap_or(0);
        if !category_page.is_empty() {
            button_on[SilkButton::Display1.index()] = true;
            if let Some(pages) = config.categories.first().map(|category| &category.pages) {
                let position = pages.iter().position(|page| *page == active_page).unwrap_or(0);
                let last = pages.len().saturating_sub(1);
                button_on[SilkButton::DisplayLeft.index()] = position > 0;
                button_on[SilkButton::DisplayRight.index()] = position < last;
            }
        }
        Self {
            config,
            active_group: GroupId::A,
            active_page,
            octave: [0; 8],
            parameters,
            shared,
            category_page,
            clip_layer: false,
            clip_pads: [false; PAD_COUNT],
            held: Vec::new(),
            local_voices: 0,
            connection: ConnectionStatus::Connected,
            next_voice: 1,
            button_on,
            feedback_dirty: true,
            force_refresh: true,
            first_feedback_ms: None,
            last_feedback_ms: None,
            reset_flash_until: [None; ENCODER_COUNT],
            lfo_phase: 0.0,
            lfo_last_ms: None,
            lfo_last_sent: None,
            logs: Vec::new(),
        }
    }

    pub fn config(&self) -> &HostConfig {
        &self.config
    }

    pub fn active_group(&self) -> GroupId {
        self.active_group
    }

    pub fn active_page(&self) -> usize {
        self.active_page
    }

    pub fn connection(&self) -> ConnectionStatus {
        self.connection
    }

    pub fn held(&self) -> &[HeldNote] {
        &self.held
    }

    pub fn local_voices(&self) -> u32 {
        self.local_voices
    }

    pub fn octave(&self, group: GroupId) -> i32 {
        self.octave[group.index()]
    }

    pub fn parameter(&self, page: usize, encoder: u8) -> Option<&ParameterState> {
        let mapping = self.resolved_mapping(page, encoder as usize);
        if let Some(name) = &mapping.parameter {
            let slot = self.slot_for(&mapping, encoder);
            return self.shared.get(&(name.clone(), slot));
        }
        self.parameters.get(&(page, encoder))
    }

    pub fn encoder_text(&self, page: usize, index: usize) -> (String, String, String) {
        let mapping = self.resolved_mapping(page, index);
        let scope = character_slot_label(mapping.parameter.as_deref())
            .map(str::to_string)
            .unwrap_or_else(|| match mapping.scope {
                ParamScope::Global => "global".to_string(),
                ParamScope::Selected => "selected".to_string(),
                ParamScope::Slot => format!("slot {}", index + 1),
            });
        if mapping.inactive || self.blocked(&mapping) {
            if self.eq_tilt_gap(mapping.parameter.as_deref()) {
                return (String::new(), "N/A".to_string(), "N/A".to_string());
            }
            return (abbreviate_label(&mapping.label), "N/A".to_string(), scope);
        }
        let value = self.stored_value(page, index);
        let shown = match mapping.format {
            ValueFormat::OnOff if value >= 64 => "On".to_string(),
            ValueFormat::OnOff => "Off".to_string(),
            ValueFormat::Number => self.named_value(&mapping, value),
        };
        if let Some(pick) = tone_pick_parameter(mapping.parameter.as_deref()) {
            let selected = self
                .shared
                .get(&(pick.to_string(), 0))
                .map(|state| state.value)
                .unwrap_or(0);
            let (line, more) = character_tone_note(pick, selected);
            return (more.to_string(), shown, line.to_string());
        }
        if let Some((line, more)) = self.eq_caption(mapping.parameter.as_deref()) {
            let shown = if line == "RESET" {
                String::new()
            } else {
                shown
            };
            return (more.to_string(), shown, line.to_string());
        }
        (abbreviate_label(&self.dynamic_label(&mapping)), shown, scope)
    }

    pub fn character_type_help(&self) -> [(&'static str, &'static str, &'static str); 4] {
        let value = |name: &str| {
            self.shared
                .get(&(name.to_string(), 0))
                .map(|state| state.value)
                .unwrap_or(0)
        };
        [
            character_help_row("char-pick-1", value("char-pick-1")),
            character_help_row("char-pick-2", value("char-pick-2")),
            character_help_row("char-pick-3", value("char-pick-3")),
            character_help_row("char-pick-4", value("char-pick-4")),
        ]
    }

    pub fn logs(&self) -> &[LogRecord] {
        &self.logs
    }

    pub fn reload_if_idle(&mut self, config: HostConfig) -> Result<(), String> {
        if !self.held.is_empty() || self.local_voices > 0 {
            return Err(
                "configuration reload is allowed only while no notes are held and no local voices are active"
                    .to_string(),
            );
        }
        *self = Controller::new(config);
        Ok(())
    }

    pub fn handle(&mut self, event: InputEvent) -> Vec<OutputAction> {
        self.log(
            LogLevel::Debug,
            format!(
                "input control={} action={:?} value={} time_ms={}",
                event.control, event.action, event.value, event.time_ms
            ),
        );
        match event.control {
            ControlId::Group(id) if event.action == ControlAction::Press => self.select_group(id),
            ControlId::Pad(pad) => self.pad(pad, event),
            ControlId::Encoder(index) if event.action == ControlAction::Delta => {
                self.encoder(index, event.value, event.time_ms)
            }
            ControlId::PageLeft if event.action == ControlAction::Press => self.shift_page(-1),
            ControlId::PageRight if event.action == ControlAction::Press => self.shift_page(1),
            ControlId::Panic if event.action == ControlAction::Press => self.panic(),
            ControlId::OctaveUp if event.action == ControlAction::Press => self.shift_octave(1),
            ControlId::OctaveDown if event.action == ControlAction::Press => self.shift_octave(-1),
            ControlId::Button(button) => self.silk_button(button, event.action),
            _ => Vec::new(),
        }
    }

    pub fn disconnect(&mut self) -> Vec<OutputAction> {
        let actions = self.panic();
        self.connection = ConnectionStatus::Disconnected;
        self.feedback_dirty = true;
        self.force_refresh = true;
        self.log(
            LogLevel::Error,
            "controller disconnected; notes cleared".to_string(),
        );
        actions
    }

    pub fn connect(&mut self) -> Vec<OutputAction> {
        self.connection = ConnectionStatus::Connected;
        self.feedback_dirty = true;
        self.force_refresh = true;
        self.first_feedback_ms = None;
        self.log(LogLevel::Info, "controller connected".to_string());
        self.opening_pans()
    }

    /// Centre pan on each slot. Level, filter, and pitch stay where the module saved them.
    fn opening_pans(&mut self) -> Vec<OutputAction> {
        let found: Vec<(String, u8, u8, i32)> = self
            .config
            .encoder_pages
            .iter()
            .flat_map(|page| page.encoders.iter().enumerate())
            .filter_map(|(index, encoder)| {
                if encoder.parameter.as_deref() != Some("pan") {
                    return None;
                }
                let EncoderKind::MidiCc {
                    port,
                    channel,
                    cc: Some(cc),
                } = &encoder.kind
                else {
                    return None;
                };
                if *cc != 38 {
                    return None;
                }
                let value = self
                    .shared
                    .get(&("pan".to_string(), index as u8))
                    .map(|state| state.value)
                    .unwrap_or(encoder.default_value);
                Some((port.clone(), *channel, *cc, value))
            })
            .collect();
        found
            .into_iter()
            .map(|(port, channel, cc, value)| {
                let sent = self.voiced(cc, value);
                OutputAction::Midi(MidiMessage::cc(&port, channel, cc, sent))
            })
            .collect()
    }

    pub fn shutdown(&mut self) -> Vec<OutputAction> {
        let actions = self.panic();
        self.log(LogLevel::Info, "shutdown".to_string());
        actions
    }

    pub fn abandon_unconfirmed(&mut self, pad: u8) {
        let before = self.held.len();
        self.held.retain(|note| note.pad != pad);
        if self.held.len() != before {
            self.feedback_dirty = true;
            self.log(
                LogLevel::Error,
                format!("note-on for pad {pad} was not confirmed; hold dropped"),
            );
        }
    }

    pub fn feedback_due(&mut self, now_ms: u64) -> bool {
        if self.expire_reset_flashes(now_ms) {
            self.feedback_dirty = true;
        }
        let opening = self.first_feedback_ms.is_some_and(|start| {
            now_ms.saturating_sub(start) < OPENING_FEEDBACK_WINDOW_MS
                && self
                    .last_feedback_ms
                    .is_some_and(|last| now_ms.saturating_sub(last) >= OPENING_FEEDBACK_GAP_MS)
        });
        if !self.feedback_dirty && !opening {
            return false;
        }
        if self.force_refresh {
            return true;
        }
        match self.last_feedback_ms {
            None => true,
            Some(_) if opening => true,
            Some(last) => now_ms.saturating_sub(last) >= self.config.display_interval_ms,
        }
    }

    pub fn mark_feedback_sent(&mut self, now_ms: u64) {
        self.feedback_dirty = false;
        self.force_refresh = false;
        if self.first_feedback_ms.is_none() {
            self.first_feedback_ms = Some(now_ms);
        }
        self.last_feedback_ms = Some(now_ms);
    }

    fn silk_button(&mut self, button: SilkButton, action: ControlAction) -> Vec<OutputAction> {
        if !self.config.categories.is_empty() {
            if let Some(category) = display_category(button) {
                if action == ControlAction::Press {
                    return self.select_category(category);
                }
                return Vec::new();
            }
            if button == SilkButton::Sampling && action == ControlAction::Press {
                return self.toggle_clip_layer();
            }
            if matches!(button, SilkButton::DisplayLeft | SilkButton::DisplayRight) {
                if action == ControlAction::Press {
                    let delta = if button == SilkButton::DisplayLeft { -1 } else { 1 };
                    return self.shift_page(delta);
                }
                return Vec::new();
            }
        }
        if self.disconnected(button.name()) {
            return vec![OutputAction::Error(
                "button ignored while the controller is disconnected".to_string(),
            )];
        }
        let changed = match button.gesture() {
            ButtonGesture::Momentary => match action {
                ControlAction::Press => self.set_button(button, true),
                ControlAction::Release => self.set_button(button, false),
                ControlAction::Delta => false,
            },
            ButtonGesture::Toggle => {
                if action == ControlAction::Press {
                    self.set_button(button, !self.button_on[button.index()])
                } else {
                    false
                }
            }
            ButtonGesture::Radio(set) => {
                if action == ControlAction::Press {
                    self.select_radio(set, button)
                } else {
                    false
                }
            }
        };
        if changed {
            self.feedback_dirty = true;
        }
        if action == ControlAction::Press {
            match button {
                SilkButton::DisplayLeft => return self.shift_page(-1),
                SilkButton::DisplayRight => return self.shift_page(1),
                _ => {}
            }
        }
        Vec::new()
    }

    fn set_button(&mut self, button: SilkButton, on: bool) -> bool {
        let slot = &mut self.button_on[button.index()];
        if *slot == on {
            false
        } else {
            *slot = on;
            true
        }
    }

    fn select_radio(&mut self, set: RadioSet, chosen: SilkButton) -> bool {
        let mut changed = false;
        for button in SilkButton::ALL {
            if button.gesture() != ButtonGesture::Radio(set) {
                continue;
            }
            let on = button == chosen;
            if self.button_on[button.index()] != on {
                self.button_on[button.index()] = on;
                changed = true;
            }
        }
        changed
    }

    fn select_group(&mut self, id: GroupId) -> Vec<OutputAction> {
        if self.disconnected("group") {
            return vec![OutputAction::Error(
                "group change ignored while the controller is disconnected".to_string(),
            )];
        }
        let page = self.active_page;
        let release = self.release_clip_hold();
        self.active_group = id;
        self.feedback_dirty = true;
        self.log(
            LogLevel::Info,
            format!("group {id} selected; encoder page unchanged"),
        );
        debug_assert_eq!(self.active_page, page);
        release
    }

    fn shift_page(&mut self, delta: i32) -> Vec<OutputAction> {
        if self.disconnected("encoder page") {
            return vec![OutputAction::Error(
                "encoder page change ignored while the controller is disconnected".to_string(),
            )];
        }
        let count = self.config.encoder_pages.len() as i32;
        if count == 0 {
            return vec![OutputAction::Error(
                "no encoder pages are loaded".to_string(),
            )];
        }
        if let Some(category) = self.category_of(self.active_page) {
            let pages = self.config.categories[category].pages.clone();
            let position = pages
                .iter()
                .position(|page| *page == self.active_page)
                .unwrap_or(0);
            let next = pages[(position as i32 + delta).rem_euclid(pages.len() as i32) as usize];
            return self.show_page(next, category);
        }
        let next = (self.active_page as i32 + delta).rem_euclid(count) as usize;
        self.active_page = next;
        self.feedback_dirty = true;
        self.log(
            LogLevel::Info,
            format!("encoder page {}", self.config.encoder_pages[next].id),
        );
        Vec::new()
    }

    fn shift_octave(&mut self, delta: i32) -> Vec<OutputAction> {
        if self.disconnected("octave") {
            return vec![OutputAction::Error(
                "octave change ignored while the controller is disconnected".to_string(),
            )];
        }
        let group = self.active_group;
        if self.config.groups[group.index()].mode == PadMode::OneShot {
            return vec![OutputAction::Error(
                "octave shift does not apply to one-shot groups".to_string(),
            )];
        }
        self.octave[group.index()] = (self.octave[group.index()] + delta).clamp(-4, 4);
        self.feedback_dirty = true;
        Vec::new()
    }

    fn pad(&mut self, pad: u8, event: InputEvent) -> Vec<OutputAction> {
        if pad >= PAD_COUNT as u8 {
            return vec![OutputAction::Error(format!("pad {pad} is outside 0-15"))];
        }
        if self.connection == ConnectionStatus::Disconnected {
            return vec![OutputAction::Error(
                "pad ignored while the controller is disconnected".to_string(),
            )];
        }
        match event.action {
            ControlAction::Press => {
                if event.value <= 0 {
                    return vec![OutputAction::Error(
                        "zero velocity press ignored so it cannot become a stuck note-on"
                            .to_string(),
                    )];
                }
                if self.clip_layer {
                    return self.clip_press(pad);
                }
                self.note_on(pad, event.value.clamp(1, 127) as u8, event.time_ms)
            }
            ControlAction::Release => {
                if self.clip_pads[pad as usize] {
                    return self.clip_release(pad);
                }
                self.note_off(pad)
            }
            ControlAction::Delta => Vec::new(),
        }
    }

    fn note_on(&mut self, pad: u8, velocity: u8, time_ms: u64) -> Vec<OutputAction> {
        let mut actions = self.note_off(pad);
        let group = self.config.groups[self.active_group.index()].clone();
        let note = self.mapped_note(&group, pad);
        match &group.destination {
            Destination::Midi { port, channel } => {
                let velocity = group.velocity_curve.apply(velocity);
                self.held.push(HeldNote {
                    pad,
                    group: group.id,
                    destination: group.destination.clone(),
                    note,
                    voice_id: None,
                    release: group.trigger,
                });
                actions.push(OutputAction::Midi(MidiMessage::note_on(
                    port, *channel, note, velocity, pad,
                )));
                self.log(
                    LogLevel::Info,
                    format!(
                        "note-on group={} pad={pad} note={note} velocity={velocity} channel={channel} port={port} time_ms={time_ms}",
                        group.id
                    ),
                );
            }
            Destination::Sampler { instrument } => match self.sample_trigger(&group, pad, note) {
                Ok(command) => {
                    let voice_id = match &command.kind {
                        SamplerCommandKind::Trigger { voice_id, .. } => *voice_id,
                        _ => 0,
                    };
                    if group.trigger == TriggerPolicy::IgnoreRelease {
                        self.local_voices = self.local_voices.saturating_add(1);
                    } else {
                        self.held.push(HeldNote {
                            pad,
                            group: group.id,
                            destination: group.destination.clone(),
                            note,
                            voice_id: Some(voice_id),
                            release: group.trigger,
                        });
                    }
                    actions.push(OutputAction::Sampler(command));
                    self.log(
                        LogLevel::Info,
                        format!(
                            "voice group={} pad={pad} note={note} instrument={instrument} time_ms={time_ms}",
                            group.id
                        ),
                    );
                }
                Err(message) => actions.push(OutputAction::Error(message)),
            },
        }
        self.feedback_dirty = true;
        actions
    }

    fn sample_trigger(
        &mut self,
        group: &crate::model::Group,
        pad: u8,
        note: u8,
    ) -> Result<SamplerCommand, String> {
        let (path, rate, choke) = match group.mode {
            PadMode::OneShot => {
                let Some(sample) = &group.samples[pad as usize] else {
                    return Err(format!(
                        "group {} pad {pad} has no sample assigned",
                        group.id
                    ));
                };
                (sample.path.clone(), 1.0, sample.choke)
            }
            PadMode::Chromatic | PadMode::Scale => {
                let Some(zone) = select_zone(&group.zones, note) else {
                    return Err(format!(
                        "group {} has no sample zone for note {note}",
                        group.id
                    ));
                };
                let rate = 2_f64.powf(f64::from(i32::from(note) - i32::from(zone.root)) / 12.0);
                (zone.path.clone(), rate, None)
            }
        };
        let voice_id = self.next_voice;
        self.next_voice = self.next_voice.saturating_add(1);
        Ok(SamplerCommand {
            kind: SamplerCommandKind::Trigger {
                voice_id,
                path,
                rate,
                choke,
            },
        })
    }

    fn note_off(&mut self, pad: u8) -> Vec<OutputAction> {
        let Some(index) = self.held.iter().position(|note| note.pad == pad) else {
            return Vec::new();
        };
        let held = self.held.remove(index);
        self.feedback_dirty = true;
        if held.release == TriggerPolicy::IgnoreRelease {
            return Vec::new();
        }
        match held.destination {
            Destination::Midi { port, channel } => {
                self.log(
                    LogLevel::Info,
                    format!(
                        "note-off group={} pad={pad} note={} channel={channel} port={port}",
                        held.group, held.note
                    ),
                );
                vec![OutputAction::Midi(MidiMessage::note_off(
                    &port, channel, held.note,
                ))]
            }
            Destination::Sampler { .. } => {
                let Some(voice_id) = held.voice_id else {
                    return Vec::new();
                };
                vec![OutputAction::Sampler(SamplerCommand {
                    kind: SamplerCommandKind::Stop { voice_id },
                })]
            }
        }
    }

    fn mapped_note(&self, group: &crate::model::Group, pad: u8) -> u8 {
        let base = match (group.mode, group.note_scale) {
            (PadMode::Scale, Some(scale)) => {
                i32::from(scale.root) + scale.name.interval(pad)
            }
            _ => i32::from(group.notes[pad as usize]),
        };
        let shift = if group.mode == PadMode::OneShot {
            0
        } else {
            self.octave[group.id.index()] * 12
        };
        clamp_note(base + shift)
    }

    fn encoder(&mut self, index: u8, delta: i32, time_ms: u64) -> Vec<OutputAction> {
        if self.disconnected("encoder") {
            return vec![OutputAction::Error(
                "encoder ignored while the controller is disconnected".to_string(),
            )];
        }
        if index >= ENCODER_COUNT as u8 {
            return vec![OutputAction::Error(format!(
                "encoder {index} is outside 0-7"
            ))];
        }
        let page_index = self.active_page;
        let mapping = self.resolved_mapping(page_index, index as usize);
        if mapping.inactive || self.blocked(&mapping) {
            self.feedback_dirty = true;
            return Vec::new();
        }
        let current = self.stored_value(page_index, index as usize);
        let value = (current + delta).clamp(mapping.min, mapping.max);
        self.store_value(page_index, index, &mapping, value, time_ms);
        self.feedback_dirty = true;
        let midi_value = clamp_note(value);
        let channel = self.midi_channel(&mapping);
        let mut actions = match mapping.kind {
            EncoderKind::MidiCc {
                port,
                cc: Some(cc),
                ..
            } => {
                let Some(channel) = channel else {
                    return Vec::new();
                };
                let sent = self.voiced(cc, value);
                vec![OutputAction::Midi(MidiMessage::cc(
                    &port, channel, cc, sent,
                ))]
            }
            EncoderKind::MidiCc { cc: None, .. }
            | EncoderKind::Unbound
            | EncoderKind::Local { .. } => Vec::new(),
            EncoderKind::Macro { targets } => targets
                .into_iter()
                .map(|target| {
                    OutputAction::Midi(MidiMessage::cc(
                        &target.port,
                        target.channel,
                        target.cc,
                        midi_value,
                    ))
                })
                .collect(),
        };
        if let Some(name) = self.reset_target(mapping.parameter.as_deref()) {
            self.reset_flash_until[index as usize] =
                Some(time_ms.saturating_add(RESET_FLASH_MS));
            actions.extend(self.centre_named(name, time_ms));
        }
        actions
    }

    pub fn reset_flashing(&self, index: usize) -> bool {
        self.reset_flash_until
            .get(index)
            .is_some_and(|until| until.is_some())
    }

    fn expire_reset_flashes(&mut self, now_ms: u64) -> bool {
        let mut expired = false;
        for slot in &mut self.reset_flash_until {
            if slot.is_some_and(|until| now_ms >= until) {
                *slot = None;
                expired = true;
            }
        }
        expired
    }

    /// A reset push is unread by the module. Centre the matching play
    /// controller here so the next turn of that encoder does not put the old
    /// value back. Kill, Open, and Slam leave the stored value alone.
    fn reset_target(&self, parameter: Option<&str>) -> Option<&'static str> {
        let target = match parameter {
            Some("eq-push-1") => "eq-a",
            Some("eq-push-2") => "eq-b",
            Some("eq-push-3") => "eq-c",
            _ => return None,
        };
        let (gesture, _) = self.eq_caption(parameter)?;
        if gesture == "RESET" {
            Some(target)
        } else {
            None
        }
    }

    fn centre_named(&mut self, name: &str, time_ms: u64) -> Vec<OutputAction> {
        let page = self.active_page;
        let Some(index) = self.config.encoder_pages[page]
            .encoders
            .iter()
            .position(|encoder| encoder.parameter.as_deref() == Some(name))
        else {
            return Vec::new();
        };
        let mapping = self.resolved_mapping(page, index);
        if self.blocked(&mapping) {
            return Vec::new();
        }
        let value = mapping.default_value.clamp(mapping.min, mapping.max);
        self.store_value(page, index as u8, &mapping, value, time_ms);
        let Some(channel) = self.midi_channel(&mapping) else {
            return Vec::new();
        };
        match mapping.kind {
            EncoderKind::MidiCc {
                port,
                cc: Some(cc),
                ..
            } => vec![OutputAction::Midi(MidiMessage::cc(
                &port,
                channel,
                cc,
                self.voiced(cc, value),
            ))],
            _ => Vec::new(),
        }
    }

    pub fn tick_lfo(&mut self, now_ms: u64) -> Vec<OutputAction> {
        let elapsed = self
            .lfo_last_ms
            .map(|last| now_ms.saturating_sub(last))
            .unwrap_or(0);
        self.lfo_last_ms = Some(now_ms);
        let hz = lfo_hz(self.shared_value("eq-lfo-rate", 0));
        if hz > 0.0 && elapsed > 0 {
            self.lfo_phase = (self.lfo_phase + hz * elapsed as f64 / 1000.0).rem_euclid(1.0);
        }
        let Some(target) = self.lfo_target() else {
            return self.release_lfo();
        };
        let mut actions = Vec::new();
        if let Some((previous, _)) = self.lfo_last_sent {
            if previous != target.cc {
                actions.extend(self.restore_cc(previous));
            }
        }
        let value = self.lfo_mix(target.cc, target.center);
        if self.lfo_last_sent == Some((target.cc, value)) {
            return actions;
        }
        self.lfo_last_sent = Some((target.cc, value));
        actions.push(OutputAction::Midi(MidiMessage::cc(
            &target.port,
            target.channel,
            target.cc,
            value,
        )));
        actions
    }

    fn voiced(&mut self, cc: u8, center: i32) -> u8 {
        let value = self.lfo_mix(cc, center);
        if self.lfo_target().is_some_and(|target| target.cc == cc) {
            self.lfo_last_sent = Some((cc, value));
        }
        value
    }

    fn lfo_mix(&self, cc: u8, center: i32) -> u8 {
        let Some(target) = self.lfo_target() else {
            return clamp_note(center);
        };
        if target.cc != cc {
            return clamp_note(center);
        }
        let depth = self.shared_value("eq-lfo-depth", 0);
        let shape = lfo_shape_index(self.shared_value("eq-lfo-shape", 0));
        let wave = lfo_wave(shape, self.lfo_phase);
        clamp_note((f64::from(center) + wave * f64::from(depth)).round() as i32)
    }

    fn lfo_target(&self) -> Option<LfoTarget> {
        let slot = lfo_dest_slot(self.shared_value("eq-lfo-dest", 0))?;
        if eq_dest_label(self.eq_mode(), slot).is_none() {
            return None;
        }
        let parameter = ["eq-a", "eq-b", "eq-c"][slot];
        let (port, channel, cc) = self.midi_target(parameter)?;
        Some(LfoTarget {
            port,
            channel,
            cc,
            center: self.shared_value(parameter, 64),
        })
    }

    fn lfo_dest_text(&self, value: i32) -> String {
        match lfo_dest_slot(value) {
            None => "None".to_string(),
            Some(slot) => eq_dest_label(self.eq_mode(), slot).unwrap_or_else(|| "N/A".to_string()),
        }
    }

    fn release_lfo(&mut self) -> Vec<OutputAction> {
        let Some((cc, _)) = self.lfo_last_sent.take() else {
            return Vec::new();
        };
        self.restore_cc(cc)
    }

    fn restore_cc(&self, cc: u8) -> Vec<OutputAction> {
        let parameter = match cc {
            51 => "eq-a",
            52 => "eq-b",
            53 => "eq-c",
            _ => return Vec::new(),
        };
        let Some((port, channel, found)) = self.midi_target(parameter) else {
            return Vec::new();
        };
        if found != cc {
            return Vec::new();
        }
        vec![OutputAction::Midi(MidiMessage::cc(
            &port,
            channel,
            cc,
            clamp_note(self.shared_value(parameter, 64)),
        ))]
    }

    fn midi_target(&self, parameter: &str) -> Option<(String, u8, u8)> {
        for page in &self.config.encoder_pages {
            for encoder in &page.encoders {
                if encoder.parameter.as_deref() != Some(parameter) {
                    continue;
                }
                if let EncoderKind::MidiCc {
                    port,
                    channel,
                    cc: Some(cc),
                } = &encoder.kind
                {
                    return Some((port.clone(), *channel, *cc));
                }
            }
        }
        None
    }

    fn shared_value(&self, name: &str, default: i32) -> i32 {
        self.shared
            .get(&(name.to_string(), 0))
            .map(|state| state.value)
            .unwrap_or(default)
    }

    pub fn panic(&mut self) -> Vec<OutputAction> {
        let mut actions = self.release_clip_hold();
        let held = std::mem::take(&mut self.held);
        self.local_voices = 0;
        self.feedback_dirty = true;
        let mut channels: BTreeMap<(String, u8), ()> = BTreeMap::new();
        for note in held {
            if let Destination::Midi { port, channel } = &note.destination {
                channels.insert((port.clone(), *channel), ());
                actions.push(OutputAction::Midi(MidiMessage::note_off(
                    port, *channel, note.note,
                )));
            }
        }
        for (port, channel) in channels.keys() {
            actions.push(OutputAction::Midi(MidiMessage::all_notes_off(
                port, *channel,
            )));
        }
        actions.push(OutputAction::Sampler(SamplerCommand {
            kind: SamplerCommandKind::StopAll,
        }));
        self.log(LogLevel::Info, "panic".to_string());
        actions
    }

    fn disconnected(&self, control: &str) -> bool {
        if self.connection == ConnectionStatus::Disconnected {
            let _ = control;
            true
        } else {
            false
        }
    }

    fn log(&mut self, level: LogLevel, message: String) {
        if self.config.log_level.allows(level) {
            self.logs.push(LogRecord { level, message });
        }
    }

    fn resolved_mapping(&self, page: usize, index: usize) -> EncoderMapping {
        let page_data = &self.config.encoder_pages[page];
        if index >= 4 && !page_data.profiles.is_empty() {
            let pad_type = self.config.groups[self.active_group.index()].pad_type;
            if let Some(pad_type) = pad_type {
                if let Some(profile) = page_data
                    .profiles
                    .iter()
                    .find(|profile| profile.pad_type == pad_type)
                {
                    return profile.encoders[index - 4].clone();
                }
            }
            let mut inactive = page_data.encoders[index].clone();
            inactive.inactive = true;
            inactive.kind = EncoderKind::Unbound;
            return inactive;
        }
        page_data.encoders[index].clone()
    }

    fn slot_for(&self, mapping: &EncoderMapping, encoder: u8) -> u8 {
        match mapping.scope {
            ParamScope::Global => 0,
            ParamScope::Selected => self.active_group.index() as u8,
            ParamScope::Slot => encoder,
        }
    }

    fn stored_value(&self, page: usize, index: usize) -> i32 {
        let mapping = self.resolved_mapping(page, index);
        if let Some(name) = &mapping.parameter {
            let slot = self.slot_for(&mapping, index as u8);
            return self
                .shared
                .get(&(name.clone(), slot))
                .map(|state| state.value)
                .unwrap_or(mapping.default_value);
        }
        self.parameters
            .get(&(page, index as u8))
            .map(|state| state.value)
            .unwrap_or(mapping.default_value)
    }

    fn store_value(
        &mut self,
        page: usize,
        encoder: u8,
        mapping: &EncoderMapping,
        value: i32,
        time_ms: u64,
    ) {
        let state = ParameterState {
            value,
            source: "encoder",
            time_ms,
        };
        if let Some(name) = &mapping.parameter {
            let slot = self.slot_for(mapping, encoder);
            self.shared.insert((name.clone(), slot), state);
        } else {
            self.parameters.insert((page, encoder), state);
        }
    }

    fn midi_channel(&self, mapping: &EncoderMapping) -> Option<u8> {
        if mapping.scope == ParamScope::Selected {
            return match &self.config.groups[self.active_group.index()].destination {
                Destination::Midi { channel, .. } => Some(*channel),
                Destination::Sampler { .. } => None,
            };
        }
        match &mapping.kind {
            EncoderKind::MidiCc { channel, cc: Some(_), .. } => Some(*channel),
            _ => None,
        }
    }

    fn comp_family(&self) -> u8 {
        let value = self
            .shared
            .get(&("comp-type".to_string(), 0))
            .map(|state| state.value)
            .unwrap_or(0);
        if value < 32 {
            0
        } else if value < 64 {
            1
        } else if value < 96 {
            2
        } else {
            3
        }
    }

    fn blocked(&self, mapping: &EncoderMapping) -> bool {
        if mapping.inactive {
            return true;
        }
        if !mapping.pad_types.is_empty() {
            match self.config.groups[self.active_group.index()].pad_type {
                Some(pad_type) if mapping.pad_types.contains(&pad_type) => {}
                _ => return true,
            }
        }
        let family = self.comp_family();
        matches!(
            (family, mapping.parameter.as_deref()),
            (2, Some("comp-b")) | (3, Some("comp-b" | "comp-c" | "comp-d"))
        ) || self.eq_tilt_gap(mapping.parameter.as_deref())
    }

    fn eq_mode(&self) -> usize {
        let value = self
            .shared
            .get(&("eq-mode".to_string(), 0))
            .map(|state| state.value)
            .unwrap_or(6);
        eq_mode_index(value)
    }

    fn eq_tilt_gap(&self, parameter: Option<&str>) -> bool {
        self.eq_mode() == 5
            && matches!(parameter, Some("eq-c" | "eq-push-3"))
    }

    fn eq_caption(&self, parameter: Option<&str>) -> Option<(&'static str, &'static str)> {
        let mode = self.eq_mode();
        match parameter {
            Some("eq-mode") => Some(("EQ", "TYPE")),
            Some("eq-a") => Some(EQ_PLAY[mode][0]),
            Some("eq-b") => Some(EQ_PLAY[mode][1]),
            Some("eq-c") => Some(EQ_PLAY[mode][2]),
            Some("eq-push-1") => Some(EQ_PUSH[mode][0]),
            Some("eq-push-2") => Some(EQ_PUSH[mode][1]),
            Some("eq-push-3") => Some(EQ_PUSH[mode][2]),
            Some("eq-lfo-rate") => Some(("LFO", "SPEED")),
            Some("eq-lfo-depth") => Some(("LFO", "DEPTH")),
            Some("eq-lfo-shape") => Some(("LFO", "SHAPE")),
            Some("eq-lfo-dest") => Some(("LFO", "DEST")),
            _ => None,
        }
    }

    fn dynamic_label(&self, mapping: &EncoderMapping) -> String {
        let family = self.comp_family();
        let replacement = match (family, mapping.parameter.as_deref()) {
            (0, Some("comp-a")) => Some("Thresh"),
            (0, Some("comp-b")) => Some("Ratio"),
            (0, Some("comp-c")) => Some("Attack"),
            (0, Some("comp-d")) => Some("Rel"),
            (0, Some("comp-e")) => Some("Makeup"),
            (1, Some("comp-a")) => Some("Thresh"),
            (1, Some("comp-b")) => Some("Ratio"),
            (1, Some("comp-c")) => Some("Speed"),
            (1, Some("comp-d")) => Some("Dry"),
            (1, Some("comp-e")) => Some("Makeup"),
            (2, Some("comp-a")) => Some("Intense"),
            (2, Some("comp-c")) => Some("Speed"),
            (2, Some("comp-d")) => Some("Dry"),
            (2, Some("comp-e")) => Some("Output"),
            (3, Some("comp-a")) => Some("Thunder"),
            (3, Some("comp-e")) => Some("Output"),
            (_, Some("comp-b" | "comp-c" | "comp-d")) if family == 2 || family == 3 => Some("--"),
            _ => None,
        };
        replacement.unwrap_or(&mapping.label).to_string()
    }

    fn named_value(&self, mapping: &EncoderMapping, value: i32) -> String {
        if mapping.parameter.as_deref() == Some("comp-type") {
            return match self.comp_family() {
                0 => "Stock",
                1 => "Logical4",
                2 => "VariMu",
                _ => "Thunder",
            }
            .to_string();
        }
        if let Some(name) = character_pick_name(mapping.parameter.as_deref(), value) {
            return name.to_string();
        }
        if mapping.parameter.as_deref() == Some("eq-mode") {
            return EQ_MODE_NAMES[eq_mode_index(value)].to_string();
        }
        if mapping.parameter.as_deref() == Some("eq-lfo-rate") {
            return format_hz(lfo_hz(value));
        }
        if mapping.parameter.as_deref() == Some("eq-lfo-shape") {
            return LFO_SHAPES[lfo_shape_index(value)].to_string();
        }
        if mapping.parameter.as_deref() == Some("eq-lfo-dest") {
            return self.lfo_dest_text(value);
        }
        if mapping.parameter.as_deref() == Some("char-tone-3") {
            let pick = self
                .shared
                .get(&("char-pick-3".to_string(), 0))
                .map(|state| state.value)
                .unwrap_or(0);
            if let Some(name) = channel9_console(pick, value) {
                return name.to_string();
            }
        }
        value.to_string()
    }

    fn category_of(&self, page: usize) -> Option<usize> {
        self.config
            .categories
            .iter()
            .position(|category| category.pages.contains(&page))
    }

    fn show_page(&mut self, page: usize, category: usize) -> Vec<OutputAction> {
        let release = if page == self.active_page {
            Vec::new()
        } else {
            self.release_clip_hold()
        };
        self.active_page = page;
        self.reset_flash_until = [None; ENCODER_COUNT];
        if category < self.category_page.len() {
            self.category_page[category] = page;
        }
        self.feedback_dirty = true;
        self.update_page_arrows();
        self.log(
            LogLevel::Info,
            format!("encoder page {}", self.config.encoder_pages[page].id),
        );
        release
    }

    fn update_page_arrows(&mut self) {
        if self.config.categories.is_empty() {
            return;
        }
        let (left, right) = match self.category_of(self.active_page) {
            Some(category) => {
                let pages = &self.config.categories[category].pages;
                let position = pages
                    .iter()
                    .position(|page| *page == self.active_page)
                    .unwrap_or(0);
                let last = pages.len().saturating_sub(1);
                (position > 0, position < last)
            }
            None => (false, false),
        };
        self.button_on[SilkButton::DisplayLeft.index()] = left;
        self.button_on[SilkButton::DisplayRight.index()] = right;
    }

    fn select_category(&mut self, category: usize) -> Vec<OutputAction> {
        if self.disconnected("category") {
            return vec![OutputAction::Error(
                "category change ignored while the controller is disconnected".to_string(),
            )];
        }
        let Some(page) = self.category_page.get(category).copied() else {
            return Vec::new();
        };
        for index in 0..8 {
            self.button_on[index] = index == category;
        }
        let actions = self.show_page(page, category);
        self.log(
            LogLevel::Info,
            format!("category {}", self.config.categories[category].id),
        );
        actions
    }

    fn toggle_clip_layer(&mut self) -> Vec<OutputAction> {
        if self.clip_layer {
            self.clip_layer = false;
            return self.release_clip_hold();
        }
        let pads: Vec<u8> = self.held.iter().map(|note| note.pad).collect();
        let mut actions = Vec::new();
        for pad in pads {
            actions.extend(self.note_off(pad));
        }
        self.clip_layer = true;
        self.feedback_dirty = true;
        actions
    }

    fn clip_press(&mut self, pad: u8) -> Vec<OutputAction> {
        let fresh = !self.clip_pads.iter().any(|held| *held);
        self.clip_pads[pad as usize] = true;
        self.feedback_dirty = true;
        if fresh {
            self.roll_cc(127)
        } else {
            Vec::new()
        }
    }

    fn clip_release(&mut self, pad: u8) -> Vec<OutputAction> {
        self.clip_pads[pad as usize] = false;
        self.feedback_dirty = true;
        if self.clip_pads.iter().any(|held| *held) {
            Vec::new()
        } else {
            self.roll_cc(0)
        }
    }

    fn release_clip_hold(&mut self) -> Vec<OutputAction> {
        if !self.clip_pads.iter().any(|held| *held) {
            return Vec::new();
        }
        self.clip_pads = [false; PAD_COUNT];
        self.feedback_dirty = true;
        self.roll_cc(0)
    }

    fn roll_cc(&self, value: u8) -> Vec<OutputAction> {
        let Destination::Midi { port, channel } =
            &self.config.groups[self.active_group.index()].destination
        else {
            return Vec::new();
        };
        vec![OutputAction::Midi(MidiMessage::cc(port, *channel, 118, value))]
    }
}

fn seed_shared(
    shared: &mut BTreeMap<(String, u8), ParameterState>,
    encoder: &EncoderMapping,
    index: u8,
) {
    let Some(name) = &encoder.parameter else {
        return;
    };
    let slots: Vec<u8> = match encoder.scope {
        ParamScope::Global => vec![0],
        ParamScope::Slot => vec![index],
        ParamScope::Selected => (0..8).map(|slot| slot as u8).collect(),
    };
    for slot in slots {
        shared.entry((name.clone(), slot)).or_insert(ParameterState {
            value: encoder.default_value,
            source: "default",
            time_ms: 0,
        });
    }
}

fn tone_pick_parameter(parameter: Option<&str>) -> Option<&'static str> {
    match parameter {
        Some("char-tone-1") => Some("char-pick-1"),
        Some("char-tone-2") => Some("char-pick-2"),
        Some("char-tone-3") => Some("char-pick-3"),
        Some("char-tone-4") => Some("char-pick-4"),
        _ => None,
    }
}

fn character_slot_label(parameter: Option<&str>) -> Option<&'static str> {
    match parameter {
        Some("char-amount-1" | "char-tone-1" | "char-pick-1") => Some("Destruct"),
        Some("char-amount-2" | "char-tone-2" | "char-pick-2") => Some("Overdrive"),
        Some("char-amount-3" | "char-tone-3" | "char-pick-3") => Some("Colour"),
        Some("char-amount-4" | "char-tone-4" | "char-pick-4") => Some("Saturation"),
        _ => None,
    }
}

fn character_pick_name(parameter: Option<&str>, value: i32) -> Option<&'static str> {
    let names: &[&str] = match parameter {
        Some("char-pick-1") => &["Pockey", "DeRez", "Ring Mod", "Fracture"],
        Some("char-pick-2") => &["Tube", "Drive", "Mackity", "Huge"],
        Some("char-pick-3") => &["Channel9", "IronOxide", "ToTape", "Coils"],
        Some("char-pick-4") => &["Purest", "Spiral", "Density", "Mojo"],
        _ => return None,
    };
    let index = if value < 32 {
        0
    } else if value < 64 {
        1
    } else if value < 96 {
        2
    } else {
        3
    };
    Some(names[index])
}

const EQ_MODE_NAMES: [&str; 10] = [
    "DJ Filter",
    "Dual Cut",
    "Band Pass",
    "Mixer",
    "Tone+Filt",
    "Tilt",
    "Notch",
    "Peak",
    "Phaser",
    "Formant",
];

const EQ_PLAY: [[(&str, &str); 3]; 10] = [
    [("FILTER", ""), ("RESONANCE", ""), ("KNEE", "")],
    [("HIGH CUT", ""), ("LOW CUT", ""), ("RESONANCE", "")],
    [("CENTRE", "FREQ"), ("WIDTH", ""), ("GAIN", "")],
    [("HIGH", ""), ("MID", ""), ("LOW", "")],
    [("HIGH", ""), ("LOW", ""), ("FILTER", "")],
    [("TILT", ""), ("HINGE FREQ", ""), ("", "")],
    [("FREQUENCY", ""), ("Q", ""), ("DEPTH", "")],
    [("FREQUENCY", ""), ("Q", ""), ("GAIN", "")],
    [("SWEEP", ""), ("DEPTH", ""), ("SPREAD", "")],
    [("VOWEL", ""), ("INTENSITY", ""), ("SHIFT", "")],
];

const EQ_PUSH: [[(&str, &str); 3]; 10] = [
    [("RESET", "FILTER"), ("RESET", "RESONANCE"), ("RESET", "KNEE")],
    [("OPEN", "HIGH CUT"), ("OPEN", "LOW CUT"), ("RESET", "RESONANCE")],
    [("RESET", "CTR FREQ"), ("OPEN", "WIDTH"), ("RESET", "GAIN")],
    [("KILL", "HIGH"), ("KILL", "MID"), ("KILL", "LOW")],
    [("KILL", "HIGH"), ("KILL", "LOW"), ("OPEN", "FILTER")],
    [("SLAM", "TILT"), ("RESET", "HINGE FREQ"), ("", "")],
    [("RESET", "FREQUENCY"), ("RESET", "Q"), ("SLAM", "DEPTH")],
    [("RESET", "FREQUENCY"), ("SLAM", "Q"), ("SLAM", "GAIN")],
    [("RESET", "SWEEP"), ("SLAM", "DEPTH"), ("SLAM", "SPREAD")],
    [("RESET", "VOWEL"), ("SLAM", "INTENSITY"), ("RESET", "SHIFT")],
];

fn eq_mode_index(value: i32) -> usize {
    (value.clamp(0, 127) / 13) as usize
}

struct LfoTarget {
    port: String,
    channel: u8,
    cc: u8,
    center: i32,
}

const LFO_SHAPES: [&str; 4] = ["Sine", "Triangle", "Square", "Saw"];

fn lfo_dest_slot(value: i32) -> Option<usize> {
    match value.clamp(0, 127) {
        0..=31 => None,
        32..=63 => Some(0),
        64..=95 => Some(1),
        _ => Some(2),
    }
}

fn lfo_shape_index(value: i32) -> usize {
    (value.clamp(0, 127) as usize) * LFO_SHAPES.len() / 128
}

/// Stopped at 0. Otherwise 0.1 Hz through 10 Hz, with 64 landing on 1 Hz.
fn lfo_hz(value: i32) -> f64 {
    if value <= 0 {
        return 0.0;
    }
    let t = f64::from(value.clamp(1, 127) - 1) / 126.0;
    0.1 * 100.0_f64.powf(t)
}

fn format_hz(hz: f64) -> String {
    if hz <= 0.0 {
        "0 Hz".to_string()
    } else if hz >= 9.95 {
        format!("{hz:.0} Hz")
    } else {
        format!("{hz:.1} Hz")
    }
}

fn lfo_wave(shape: usize, phase: f64) -> f64 {
    let phase = phase.rem_euclid(1.0);
    match shape {
        0 => (phase * std::f64::consts::TAU).sin(),
        1 => {
            if phase < 0.5 {
                phase * 4.0 - 1.0
            } else {
                3.0 - phase * 4.0
            }
        }
        2 => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        _ => phase * 2.0 - 1.0,
    }
}

fn eq_dest_label(mode: usize, slot: usize) -> Option<String> {
    let (top, bottom) = EQ_PLAY[mode][slot];
    if top.is_empty() {
        return None;
    }
    if bottom.is_empty() {
        return Some(top.to_string());
    }
    if top == "CENTRE" && bottom == "FREQ" {
        return Some("CTR FREQ".to_string());
    }
    let joined = format!("{top} {bottom}");
    if joined.chars().count() <= 10 {
        Some(joined)
    } else {
        Some(top.to_string())
    }
}

fn channel9_console(pick: i32, tone: i32) -> Option<&'static str> {
    if !(0..32).contains(&pick) {
        return None;
    }
    // The firmware index is (int)((cc / 127) * 4.999): five consoles, not a sweep.
    Some(match tone {
        ..=25 => "Neve",
        26..=50 => "API",
        51..=76 => "SSL",
        77..=101 => "Teac",
        _ => "Mackie",
    })
}

fn character_tone_note(parameter: &str, value: i32) -> (&'static str, &'static str) {
    let index = if value < 32 {
        0
    } else if value < 64 {
        1
    } else if value < 96 {
        2
    } else {
        3
    };
    match (parameter, index) {
        ("char-pick-1", 0) => ("SAMPLE", "RATE"),
        ("char-pick-1", 1) => ("HARSHNESS", ""),
        ("char-pick-1", 2) => ("PITCH", "SILENCE"),
        ("char-pick-1", 3) => ("WAVE", "FOLDER"),
        ("char-pick-2", 0) => ("VALVE", "DRIVE"),
        ("char-pick-2", 1) => ("CUTS LOWS", "THINS TOP"),
        ("char-pick-2", 2) => ("INPUT", "OVERLOAD"),
        ("char-pick-2", 3) => ("HARSHNESS", ""),
        ("char-pick-3", 0) => ("CONSOLE", "STYLE"),
        ("char-pick-3", 1) => ("TAPE SPEED", ""),
        ("char-pick-3", 2) => ("BRIGHTNESS", ""),
        ("char-pick-3", 3) => ("QUALITY", ""),
        ("char-pick-4", 0) => ("DRIVE", ""),
        ("char-pick-4", 1) => ("PRESENCE", ""),
        ("char-pick-4", 2) => ("LOW CUT", ""),
        ("char-pick-4", 3) => ("FATNESS", ""),
        _ => ("", ""),
    }
}

pub fn character_help_row(
    parameter: &str,
    value: i32,
) -> (&'static str, &'static str, &'static str) {
    (
        character_pick_name(Some(parameter), value).unwrap_or(""),
        character_tone_note(parameter, value).0,
        character_tone_note(parameter, value).1,
    )
}

fn abbreviate_label(label: &str) -> String {
    label
        .split(' ')
        .map(|word| {
            if word.eq_ignore_ascii_case("send") {
                "snd"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_category(button: SilkButton) -> Option<usize> {
    match button {
        SilkButton::Display1 => Some(0),
        SilkButton::Display2 => Some(1),
        SilkButton::Display3 => Some(2),
        SilkButton::Display4 => Some(3),
        SilkButton::Display5 => Some(4),
        SilkButton::Display6 => Some(5),
        SilkButton::Display7 => Some(6),
        SilkButton::Display8 => Some(7),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load_yaml;
    use crate::model::{ControlId, SilkButton};

    fn config_text() -> String {
        let notes: Vec<String> = (36..52).map(|note| note.to_string()).collect();
        let notes = notes.join(", ");
        let scale = "[60, 64, 67, 72, 60, 64, 67, 72, 60, 64, 67, 72, 60, 64, 67, 72]";
        let mut groups = String::new();
        for (index, id) in ["A", "B", "C", "D", "E", "F", "G", "H"].iter().enumerate() {
            let (mode, extra, channel) = match *id {
                "B" => ("chromatic", "", 2),
                "C" => ("scale", "", 3),
                "H" => (
                    "one_shot",
                    "\n    destination: {type: sampler, instrument: clicks}\n    samples:\n",
                    8,
                ),
                _ => ("one_shot", "", (index as u8) + 1),
            };
            if *id == "H" {
                let mut samples = String::new();
                for _ in 0..16 {
                    samples.push_str("      - {path: click.wav, choke: 1}\n");
                }
                groups.push_str(&format!(
                    "  - id: H\n    label: Clicks\n    mode: one_shot\n    destination: {{type: sampler, instrument: clicks}}\n    notes: [{notes}]\n    samples:\n{samples}"
                ));
            } else if *id == "C" {
                groups.push_str(&format!(
                    "  - id: C\n    label: Scale\n    mode: {mode}\n    destination: {{type: midi, port: bitbox, channel: {channel}}}\n    notes: {scale}\n"
                ));
            } else {
                let _ = extra;
                groups.push_str(&format!(
                    "  - id: {id}\n    label: Group {id}\n    mode: {mode}\n    destination: {{type: midi, port: bitbox, channel: {channel}}}\n    notes: [{notes}]\n"
                ));
            }
        }
        let mut encoders = String::new();
        for index in 1..=8 {
            let cc = if index == 1 { "20" } else { "null" };
            encoders.push_str(&format!(
                "      - {{label: Slot {index}, type: midi_cc, port: bitbox, channel: {index}, cc: {cc}}}\n"
            ));
        }
        format!(
            "schema_version: 1\nmidi_outputs:\n  bitbox: PORT\nlog_level: debug\ndisplay:\n  min_interval_ms: 50\ngroups:\n{groups}encoder_pages:\n  - id: delay\n    label: Delay sends\n    encoders:\n{encoders}  - id: reverb\n    label: Reverb sends\n    encoders:\n{encoders}"
        )
    }

    fn controller() -> Controller {
        Controller::new(load_yaml(&config_text()).expect("config"))
    }

    fn midi_bytes(actions: &[OutputAction]) -> Vec<[u8; 3]> {
        actions
            .iter()
            .filter_map(|action| match action {
                OutputAction::Midi(message) => Some(message.bytes),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn group_selection_does_not_change_encoder_page() {
        let mut host = controller();
        host.handle(InputEvent::press(ControlId::PageRight, 1, 1));
        host.handle(InputEvent::press(ControlId::Group(GroupId::D), 1, 2));
        assert_eq!(host.active_group(), GroupId::D);
        assert_eq!(host.active_page(), 1);
    }

    #[test]
    fn display_left_wraps_and_keeps_the_other_pages_value() {
        let mut host = controller();
        host.handle(InputEvent::delta(ControlId::Encoder(0), 3, 1));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayLeft),
            1,
            2,
        ));
        assert_eq!(host.active_page(), 1);
        host.handle(InputEvent::delta(ControlId::Encoder(0), 8, 3));
        host.handle(InputEvent::release(
            ControlId::Button(SilkButton::DisplayLeft),
            4,
        ));
        assert_eq!(host.active_page(), 1);
        assert!(!crate::feedback::render(&host).leds.buttons[SilkButton::DisplayLeft.index()]);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            5,
        ));
        assert_eq!(host.active_page(), 0);
        assert_eq!(host.parameter(0, 0).unwrap().value, 3);
        assert_eq!(host.parameter(1, 0).unwrap().value, 8);
    }

    #[test]
    fn release_after_group_change_uses_the_original_note() {
        let mut host = controller();
        let on = host.handle(InputEvent::press(ControlId::Pad(0), 100, 1));
        host.handle(InputEvent::press(ControlId::Group(GroupId::B), 1, 2));
        let off = host.handle(InputEvent::release(ControlId::Pad(0), 3));
        assert_eq!(midi_bytes(&on), vec![[0x90, 36, 100]]);
        assert_eq!(midi_bytes(&off), vec![[0x80, 36, 0]]);
        assert!(host.held().is_empty());
    }

    #[test]
    fn rapid_group_changes_pair_every_note_off() {
        let mut host = controller();
        let mut ons = Vec::new();
        let mut offs = Vec::new();
        for pad in 0..7 {
            let group = GroupId::from_index(pad).unwrap();
            host.handle(InputEvent::press(ControlId::Group(group), 1, pad as u64));
            let actions = host.handle(InputEvent::press(ControlId::Pad(pad as u8), 90, pad as u64));
            ons.extend(midi_bytes(&actions));
        }
        host.handle(InputEvent::press(ControlId::Group(GroupId::H), 1, 8));
        let local = host.handle(InputEvent::press(ControlId::Pad(7), 90, 8));
        assert!(local
            .iter()
            .any(|action| matches!(action, OutputAction::Sampler(_))));
        for pad in 0..8 {
            let actions = host.handle(InputEvent::release(
                ControlId::Pad(pad as u8),
                20 + pad as u64,
            ));
            offs.extend(midi_bytes(&actions));
        }
        assert_eq!(ons.len(), 7);
        assert_eq!(offs.len(), 7);
        for (on, off) in ons.iter().zip(offs.iter()) {
            assert_eq!(on[1], off[1]);
            assert_eq!(on[0] & 0x0F, off[0] & 0x0F);
            assert_eq!(off[0] & 0xF0, 0x80);
        }
        assert!(host.held().is_empty());
        assert_eq!(host.local_voices(), 1);
        let panic = host.handle(InputEvent::press(ControlId::Panic, 1, 40));
        assert!(panic.iter().any(|action| matches!(
            action,
            OutputAction::Sampler(command) if matches!(command.kind, crate::model::SamplerCommandKind::StopAll)
        )));
        assert_eq!(host.local_voices(), 0);
    }

    #[test]
    fn scale_map_is_explicit_and_not_filled_chromatically() {
        let mut host = controller();
        host.handle(InputEvent::press(ControlId::Group(GroupId::C), 1, 1));
        let actions = host.handle(InputEvent::press(ControlId::Pad(1), 80, 2));
        assert_eq!(midi_bytes(&actions), vec![[0x92, 64, 80]]);
    }

    #[test]
    fn chromatic_octave_shifts_by_twelve() {
        let mut host = controller();
        host.handle(InputEvent::press(ControlId::Group(GroupId::B), 1, 1));
        host.handle(InputEvent::press(ControlId::OctaveUp, 1, 2));
        let actions = host.handle(InputEvent::press(ControlId::Pad(0), 70, 3));
        assert_eq!(midi_bytes(&actions)[0][1], 48);
    }

    #[test]
    fn null_cc_updates_the_value_and_sends_nothing() {
        let mut host = controller();
        let actions = host.handle(InputEvent::delta(ControlId::Encoder(1), 15, 5));
        assert!(midi_bytes(&actions).is_empty());
        assert_eq!(host.parameter(0, 1).unwrap().value, 15);
    }

    #[test]
    fn macro_sends_each_explicit_target() {
        let mut text = config_text();
        text.push_str(
            "  - id: macro\n    label: Macro\n    encoders:\n      - {label: Both, type: macro, min: 0, max: 100, default: 0, targets: [{port: bitbox, channel: 1, cc: 10}, {port: bitbox, channel: 2, cc: 11}]}\n",
        );
        for index in 2..=8 {
            text.push_str(&format!(
                "      - {{label: Unused {index}, type: unbound}}\n"
            ));
        }
        let mut host = Controller::new(load_yaml(&text).expect("macro config"));
        host.handle(InputEvent::press(ControlId::PageRight, 1, 1));
        host.handle(InputEvent::press(ControlId::PageRight, 1, 2));
        let actions = host.handle(InputEvent::delta(ControlId::Encoder(0), 40, 3));
        assert_eq!(midi_bytes(&actions), vec![[0xB0, 10, 40], [0xB1, 11, 40]]);
    }

    #[test]
    fn absolute_cc_is_clamped() {
        let mut host = controller();
        let first = host.handle(InputEvent::delta(ControlId::Encoder(0), 10, 1));
        let second = host.handle(InputEvent::delta(ControlId::Encoder(0), 500, 2));
        assert_eq!(midi_bytes(&first), vec![[0xB0, 20, 10]]);
        assert_eq!(midi_bytes(&second), vec![[0xB0, 20, 127]]);
    }

    #[test]
    fn panic_sends_note_off_and_all_notes_off() {
        let mut host = controller();
        host.handle(InputEvent::press(ControlId::Pad(0), 100, 1));
        host.handle(InputEvent::press(ControlId::Group(GroupId::B), 1, 2));
        host.handle(InputEvent::press(ControlId::Pad(1), 100, 3));
        let actions = host.handle(InputEvent::press(ControlId::Panic, 1, 4));
        let bytes = midi_bytes(&actions);
        assert!(bytes.contains(&[0x80, 36, 0]));
        assert!(bytes.contains(&[0x81, 37, 0]));
        assert!(bytes.contains(&[0xB0, 123, 0]));
        assert!(bytes.contains(&[0xB1, 123, 0]));
        assert!(host.held().is_empty());
    }

    #[test]
    fn disconnect_clears_held_notes_and_blocks_new_ones() {
        let mut host = controller();
        host.handle(InputEvent::press(ControlId::Pad(0), 100, 1));
        let actions = host.disconnect();
        assert!(midi_bytes(&actions)
            .iter()
            .any(|bytes| bytes[0] & 0xF0 == 0x80));
        assert_eq!(host.connection(), ConnectionStatus::Disconnected);
        let blocked = host.handle(InputEvent::press(ControlId::Pad(1), 100, 2));
        assert!(matches!(blocked[0], OutputAction::Error(_)));
        assert!(host.held().is_empty());
    }

    #[test]
    fn reload_waits_until_idle() {
        let mut host = controller();
        let config = host.config().clone();
        host.handle(InputEvent::press(ControlId::Pad(0), 40, 1));
        assert!(host.reload_if_idle(config.clone()).is_err());
        host.handle(InputEvent::release(ControlId::Pad(0), 2));
        assert!(host.reload_if_idle(config).is_ok());
        assert!(host.held().is_empty());
    }

    #[test]
    fn feedback_coalesces_until_the_interval_and_reconnect_forces_refresh() {
        let mut host = controller();
        assert!(host.feedback_due(0));
        host.mark_feedback_sent(0);
        host.handle(InputEvent::delta(ControlId::Encoder(1), 1, 10));
        assert!(!host.feedback_due(10));
        assert!(host.feedback_due(50));
        host.disconnect();
        assert!(host.feedback_due(51));
        host.mark_feedback_sent(51);
        host.connect();
        assert!(host.feedback_due(52));
    }

    #[test]
    fn opening_feedback_repeats_while_the_hardware_may_still_be_starting() {
        let mut host = controller();
        assert!(host.feedback_due(0));
        host.mark_feedback_sent(0);
        assert!(!host.feedback_due(199));
        assert!(host.feedback_due(200));
        host.mark_feedback_sent(200);
        assert!(host.feedback_due(1500));
        host.mark_feedback_sent(1500);
        assert!(!host.feedback_due(1800));
    }

    #[test]
    fn silk_buttons_follow_radio_toggle_and_momentary() {
        let mut host = controller();
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Scene),
            1,
            1,
        ));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Pattern),
            1,
            2,
        ));
        let frame = crate::feedback::render(&host);
        assert!(!frame.leds.buttons[SilkButton::Scene.index()]);
        assert!(frame.leds.buttons[SilkButton::Pattern.index()]);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Volume),
            1,
            3,
        ));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::NoteRepeat),
            1,
            4,
        ));
        let frame = crate::feedback::render(&host);
        assert!(!frame.leds.buttons[SilkButton::Volume.index()]);
        assert!(frame.leds.buttons[SilkButton::NoteRepeat.index()]);
        assert!(frame.leds.buttons[SilkButton::Pattern.index()]);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display1),
            1,
            5,
        ));
        let frame = crate::feedback::render(&host);
        assert!(frame.leds.buttons[SilkButton::Display1.index()]);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display1),
            1,
            6,
        ));
        let frame = crate::feedback::render(&host);
        assert!(!frame.leds.buttons[SilkButton::Display1.index()]);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Control),
            1,
            7,
        ));
        assert!(crate::feedback::render(&host).leds.buttons[SilkButton::Control.index()]);
        host.handle(InputEvent::release(
            ControlId::Button(SilkButton::Control),
            8,
        ));
        assert!(!crate::feedback::render(&host).leds.buttons[SilkButton::Control.index()]);
        host.handle(InputEvent::press(ControlId::Button(SilkButton::Enter), 1, 9));
        host.handle(InputEvent::release(
            ControlId::Button(SilkButton::Enter),
            10,
        ));
        assert!(!crate::feedback::render(&host).leds.buttons[SilkButton::Enter.index()]);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Restart),
            1,
            11,
        ));
        host.handle(InputEvent::release(
            ControlId::Button(SilkButton::Restart),
            12,
        ));
        assert!(!crate::feedback::render(&host).leds.buttons[SilkButton::Restart.index()]);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayLeft),
            1,
            13,
        ));
        assert_eq!(host.active_page(), 1);
        assert!(crate::feedback::render(&host).leds.buttons[SilkButton::DisplayLeft.index()]);
        host.handle(InputEvent::release(
            ControlId::Button(SilkButton::DisplayLeft),
            14,
        ));
        assert_eq!(host.active_page(), 1);
        assert!(!crate::feedback::render(&host).leds.buttons[SilkButton::DisplayLeft.index()]);
        let mut seen = [false; SILK_BUTTON_COUNT];
        for button in SilkButton::ALL {
            assert!(!seen[button.index()]);
            seen[button.index()] = true;
        }
        assert!(seen.iter().all(|on| *on));
    }

    #[test]
    fn velocity_curve_and_named_scale_shape_the_note_and_pad_mode_does_not() {
        let notes = (36..52)
            .map(|note| note.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let mut groups = String::new();
        for (index, id) in GroupId::ALL.iter().enumerate() {
            let mode = if *id == GroupId::A { "scale" } else { "one_shot" };
            let pad_mode = if *id == GroupId::A {
                "    velocity_curve: 0.5\n    note_scale: {root: 48, name: natural-minor}\n"
            } else {
                ""
            };
            groups.push_str(&format!(
                "  - id: {id}\n    label: Group {id}\n    mode: {mode}\n    destination: {{type: midi, port: bitbox, channel: {}}}\n{pad_mode}    notes: [{notes}]\n",
                index + 1
            ));
        }
        let mut encoders = String::new();
        for index in 1..=8 {
            encoders.push_str(&format!(
                "      - {{label: Slot {index}, type: unbound}}\n"
            ));
        }
        let text = format!(
            "schema_version: 1\nmidi_outputs:\n  bitbox: PORT\ngroups:\n{groups}encoder_pages:\n  - id: delay\n    label: Delay\n    encoders:\n{encoders}"
        );
        let mut host = Controller::new(load_yaml(&text).expect("scale group"));
        let first = host.handle(InputEvent::press(ControlId::Pad(0), 16, 1));
        assert_eq!(midi_bytes(&first), vec![[0x90, 48, 45]]);
        host.handle(InputEvent::release(ControlId::Pad(0), 2));
        let second = host.handle(InputEvent::press(ControlId::Pad(1), 127, 3));
        assert_eq!(midi_bytes(&second), vec![[0x90, 50, 127]]);
        host.handle(InputEvent::release(ControlId::Pad(1), 4));
        let third = host.handle(InputEvent::press(ControlId::Pad(2), 127, 5));
        assert_eq!(midi_bytes(&third), vec![[0x90, 51, 127]]);
        host.handle(InputEvent::release(ControlId::Pad(2), 6));
        host.handle(InputEvent::press(ControlId::OctaveUp, 1, 7));
        let shifted = host.handle(InputEvent::press(ControlId::Pad(0), 16, 8));
        assert_eq!(midi_bytes(&shifted), vec![[0x90, 60, 45]]);
        let curve = host.config.groups[0].velocity_curve;
        let scale = host.config.groups[0].note_scale;
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::PadMode),
            1,
            9,
        ));
        assert_eq!(host.config.groups[0].velocity_curve, curve);
        assert_eq!(host.config.groups[0].note_scale, scale);
    }

    #[test]
    fn bitbox_pad_sends_the_shared_note_on_the_group_channel() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        let curve = crate::model::VelocityCurve::new(0.5).expect("curve");
        let group_a = host.handle(InputEvent::press(ControlId::Pad(0), 100, 1));
        assert_eq!(midi_bytes(&group_a), vec![[0x90, 36, curve.apply(100)]]);
        host.handle(InputEvent::release(ControlId::Pad(0), 2));
        host.handle(InputEvent::press(ControlId::Group(GroupId::B), 1, 3));
        let group_b = host.handle(InputEvent::press(ControlId::Pad(0), 90, 4));
        assert_eq!(midi_bytes(&group_b), vec![[0x91, 36, curve.apply(90)]]);
    }

    #[test]
    fn bitbox_encoder_sends_the_page_controller_on_the_slot_channel() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        let delay = host.handle(InputEvent::delta(ControlId::Encoder(0), 4, 1));
        assert_eq!(midi_bytes(&delay), vec![[0xB0, 84, 4]]);
        host.handle(InputEvent::press(ControlId::Group(GroupId::C), 1, 2));
        let selected = host.handle(InputEvent::delta(ControlId::Encoder(0), 1, 3));
        assert_eq!(midi_bytes(&selected), vec![[0xB2, 84, 1]]);
        let feedback = host.handle(InputEvent::delta(ControlId::Encoder(2), 1, 4));
        assert_eq!(midi_bytes(&feedback), vec![[0xB0, 30, 1]]);
        assert_eq!(host.active_group(), GroupId::C);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display2),
            1,
            4,
        ));
        let mix = host.handle(InputEvent::delta(ControlId::Encoder(6), 2, 5));
        assert_eq!(midi_bytes(&mix), vec![[0xB6, 84, 2]]);
        assert_eq!(host.active_group(), GroupId::C);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display5),
            1,
            6,
        ));
        let character = host.handle(InputEvent::delta(ControlId::Encoder(1), 3, 7));
        assert_eq!(midi_bytes(&character), vec![[0xB0, 15, 3]]);
    }

    #[test]
    fn pan_starts_at_centre_and_one_step_leaves_that_channel() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        let opening = host.connect();
        assert_eq!(
            midi_bytes(&opening),
            (0..8u8)
                .map(|channel| [0xB0 + channel, 38, 64])
                .collect::<Vec<_>>()
        );
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display2),
            1,
            1,
        ));
        for step in 0..4 {
            host.handle(InputEvent::press(ControlId::PageRight, 1, 2 + step));
        }
        assert_eq!(host.config.encoder_pages[host.active_page()].id, "mix-pan");
        for index in 0..8 {
            assert_eq!(host.encoder_text(host.active_page(), index).1, "64");
        }
        let turned = host.handle(InputEvent::delta(ControlId::Encoder(2), 1, 10));
        assert_eq!(midi_bytes(&turned), vec![[0xB2, 38, 65]]);
    }

    #[test]
    fn bitbox_categories_keep_the_group_and_share_a_parameter() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display2),
            1,
            1,
        ));
        let start = host.active_page();
        let mix = host.handle(InputEvent::delta(ControlId::Encoder(0), 5, 2));
        assert_eq!(midi_bytes(&mix), vec![[0xB0, 84, 5]]);
        for step in 0..6 {
            host.handle(InputEvent::press(ControlId::PageRight, 1, 3 + step));
        }
        assert_eq!(host.active_page(), start);
        assert_eq!(host.active_group(), GroupId::A);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display1),
            1,
            10,
        ));
        let shared = host.handle(InputEvent::delta(ControlId::Encoder(0), 1, 11));
        assert_eq!(midi_bytes(&shared), vec![[0xB0, 84, 6]]);
        let silent = host.handle(InputEvent::delta(ControlId::Encoder(4), 3, 12));
        assert!(midi_bytes(&silent).is_empty());
        let on = host.handle(InputEvent::press(ControlId::Pad(0), 80, 13));
        assert!(on.iter().any(|action| matches!(action, OutputAction::Midi(_))));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display3),
            1,
            14,
        ));
        let off = host.handle(InputEvent::release(ControlId::Pad(0), 15));
        assert_eq!(midi_bytes(&off), vec![[0x80, 36, 0]]);
        let (_, shown, _) = host.encoder_text(host.active_page(), 0);
        host.handle(InputEvent::delta(ControlId::Encoder(0), 64, 16));
        let (_, on_text, _) = host.encoder_text(host.active_page(), 0);
        assert_eq!(shown, "Off");
        assert_eq!(on_text, "On");
    }

    #[test]
    fn granular_profile_and_clip_hold_follow_the_selected_group() {
        let mut config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        config.groups[GroupId::A.index()].pad_type = Some(crate::model::PadType::Granular);
        let mut host = Controller::new(config);
        let density = host.handle(InputEvent::delta(ControlId::Encoder(4), 2, 1));
        assert_eq!(midi_bytes(&density), vec![[0xB0, 79, 2]]);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Sampling),
            1,
            2,
        ));
        let roll = host.handle(InputEvent::press(ControlId::Pad(0), 100, 3));
        assert_eq!(midi_bytes(&roll), vec![[0xB0, 118, 127]]);
        let release = host.handle(InputEvent::release(ControlId::Pad(0), 4));
        assert_eq!(midi_bytes(&release), vec![[0xB0, 118, 0]]);
        host.handle(InputEvent::press(ControlId::Pad(1), 90, 5));
        let cleared = host.handle(InputEvent::press(ControlId::Group(GroupId::B), 1, 6));
        assert_eq!(midi_bytes(&cleared), vec![[0xB0, 118, 0]]);
        assert_eq!(host.active_group(), GroupId::B);
    }

    #[test]
    fn compressor_type_renames_and_disables_the_unused_controls() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display6),
            1,
            1,
        ));
        host.handle(InputEvent::delta(ControlId::Encoder(6), 96, 2));
        let (label, shown, _) = host.encoder_text(host.active_page(), 6);
        assert_eq!(label, "Type");
        assert_eq!(shown, "Thunder");
        let (thunder, _, _) = host.encoder_text(host.active_page(), 1);
        assert_eq!(thunder, "Thunder");
        let blocked = host.handle(InputEvent::delta(ControlId::Encoder(2), 1, 3));
        assert!(midi_bytes(&blocked).is_empty());
        let (_, unavailable, _) = host.encoder_text(host.active_page(), 2);
        assert_eq!(unavailable, "N/A");
    }

    #[test]
    fn character_pick_shows_the_effect_name() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display5),
            1,
            1,
        ));
        let shown = |host: &Controller, index: usize| host.encoder_text(host.active_page(), index).1;
        let label = |host: &Controller, index: usize| host.encoder_text(host.active_page(), index).0;
        let slot = |host: &Controller, index: usize| host.encoder_text(host.active_page(), index).2;
        assert_eq!(label(&host, 0), "Amount");
        assert_eq!(slot(&host, 0), "Destruct");
        assert_eq!(slot(&host, 1), "Overdrive");
        assert_eq!(slot(&host, 2), "Colour");
        assert_eq!(slot(&host, 3), "Saturation");
        assert_eq!(slot(&host, 4), "SAMPLE");
        assert_eq!(label(&host, 4), "RATE");
        assert_eq!(slot(&host, 5), "VALVE");
        assert_eq!(label(&host, 5), "DRIVE");
        assert_eq!(slot(&host, 6), "CONSOLE");
        assert_eq!(label(&host, 6), "STYLE");
        assert_eq!(slot(&host, 7), "DRIVE");
        assert_eq!(label(&host, 7), "");
        let amount = host.active_page();
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            2,
        ));
        let page = host.active_page();
        assert_eq!(label(&host, 0), "Type");
        assert_eq!(slot(&host, 0), "Destruct");
        assert_eq!(slot(&host, 3), "Saturation");
        assert_eq!(shown(&host, 0), "Pockey");
        assert_eq!(host.character_type_help()[0], ("Pockey", "SAMPLE", "RATE"));
        assert_eq!(host.character_type_help()[1], ("Tube", "VALVE", "DRIVE"));
        assert_eq!(host.character_type_help()[2], ("Channel9", "CONSOLE", "STYLE"));
        assert_eq!(host.character_type_help()[3], ("Purest", "DRIVE", ""));
        host.handle(InputEvent::delta(ControlId::Encoder(0), 32, 3));
        assert_eq!(shown(&host, 0), "DeRez");
        assert_eq!(host.character_type_help()[0], ("DeRez", "HARSHNESS", ""));
        assert_eq!(host.encoder_text(amount, 4).2, "HARSHNESS");
        assert_eq!(host.encoder_text(amount, 4).0, "");
        assert_eq!(shown(&host, 1), "Tube");
        assert_eq!(shown(&host, 2), "Channel9");
        assert_eq!(shown(&host, 3), "Purest");
        host.handle(InputEvent::delta(ControlId::Encoder(0), 32, 4));
        assert_eq!(shown(&host, 0), "Ring Mod");
        host.handle(InputEvent::delta(ControlId::Encoder(0), 32, 5));
        assert_eq!(shown(&host, 0), "Fracture");
        assert_eq!(host.parameter(page, 0).unwrap().value, 96);
        host.handle(InputEvent::delta(ControlId::Encoder(2), 96, 6));
        assert_eq!(shown(&host, 2), "Coils");
        assert_eq!(host.character_type_help()[2], ("Coils", "QUALITY", ""));
    }

    #[test]
    fn channel9_tone_shows_the_console() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display5),
            1,
            1,
        ));
        let shown = |host: &Controller, index: usize| host.encoder_text(host.active_page(), index).1;
        assert_eq!(shown(&host, 4), "64");
        assert_eq!(shown(&host, 6), "SSL");
        host.handle(InputEvent::delta(ControlId::Encoder(6), -39, 2));
        assert_eq!(shown(&host, 6), "Neve");
        host.handle(InputEvent::delta(ControlId::Encoder(6), 1, 3));
        assert_eq!(shown(&host, 6), "API");
        host.handle(InputEvent::delta(ControlId::Encoder(6), 25, 4));
        assert_eq!(shown(&host, 6), "SSL");
        host.handle(InputEvent::delta(ControlId::Encoder(6), 26, 5));
        assert_eq!(shown(&host, 6), "Teac");
        host.handle(InputEvent::delta(ControlId::Encoder(6), 25, 6));
        assert_eq!(shown(&host, 6), "Mackie");
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            7,
        ));
        host.handle(InputEvent::delta(ControlId::Encoder(2), 32, 8));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayLeft),
            1,
            9,
        ));
        assert_eq!(shown(&host, 6), "102");
    }

    #[test]
    fn eq_pages_name_the_mode_and_its_controls() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display7),
            1,
            1,
        ));
        let text = |host: &Controller, index: usize| host.encoder_text(host.active_page(), index);
        assert_eq!(text(&host, 0), ("TYPE".to_string(), "DJ Filter".to_string(), "EQ".to_string()));
        assert_eq!(text(&host, 1).2, "FILTER");
        assert_eq!(text(&host, 1).0, "");
        assert_eq!(text(&host, 4), ("FILTER".to_string(), String::new(), "RESET".to_string()));
        assert_eq!(text(&host, 7).0, "Bypass");
        host.handle(InputEvent::delta(ControlId::Encoder(0), 7, 2));
        assert_eq!(text(&host, 0).1, "Dual Cut");
        host.handle(InputEvent::delta(ControlId::Encoder(0), 32, 3));
        assert_eq!(text(&host, 0).1, "Mixer");
        assert_eq!(text(&host, 1).2, "HIGH");
        assert_eq!(text(&host, 2).2, "MID");
        assert_eq!(text(&host, 3).2, "LOW");
        assert_eq!(text(&host, 4), ("HIGH".to_string(), "Off".to_string(), "KILL".to_string()));
        assert_eq!(text(&host, 5).2, "KILL");
        assert_eq!(text(&host, 5).0, "MID");
        assert_eq!(text(&host, 6).0, "LOW");
        host.handle(InputEvent::delta(ControlId::Encoder(0), 26, 4));
        assert_eq!(text(&host, 0).1, "Tilt");
        assert_eq!(text(&host, 3), (String::new(), "N/A".to_string(), "N/A".to_string()));
        assert_eq!(text(&host, 6), (String::new(), "N/A".to_string(), "N/A".to_string()));
        let silent = host.handle(InputEvent::delta(ControlId::Encoder(3), 1, 5));
        assert!(midi_bytes(&silent).is_empty());
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            6,
        ));
        assert_eq!(host.config.encoder_pages[host.active_page()].id, "eq-slope");
        assert_eq!(text(&host, 0).0, "Slope");
        assert_eq!(text(&host, 0).1, "Off");
        assert_eq!(text(&host, 1), ("SPEED".to_string(), "1.0 Hz".to_string(), "LFO".to_string()));
        assert_eq!(text(&host, 2), ("DEPTH".to_string(), "0".to_string(), "LFO".to_string()));
        assert_eq!(text(&host, 3), ("SHAPE".to_string(), "Sine".to_string(), "LFO".to_string()));
        assert_eq!(text(&host, 4), ("DEST".to_string(), "None".to_string(), "LFO".to_string()));
    }

    #[test]
    fn the_eq_lfo_moves_the_named_control_of_the_current_mode() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display7),
            1,
            1,
        ));
        host.handle(InputEvent::delta(ControlId::Encoder(0), 98, 2));
        assert_eq!(host.encoder_text(host.active_page(), 0).1, "Phaser");
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            3,
        ));
        let text = |host: &Controller, index: usize| host.encoder_text(host.active_page(), index);
        assert_eq!(text(&host, 4).1, "None");
        host.handle(InputEvent::delta(ControlId::Encoder(4), 32, 4));
        assert_eq!(text(&host, 4).1, "SWEEP");
        host.handle(InputEvent::delta(ControlId::Encoder(4), 32, 5));
        assert_eq!(text(&host, 4).1, "DEPTH");
        host.handle(InputEvent::delta(ControlId::Encoder(4), 32, 6));
        assert_eq!(text(&host, 4).1, "SPREAD");
        host.handle(InputEvent::delta(ControlId::Encoder(4), -64, 7));
        assert_eq!(text(&host, 4).1, "SWEEP");
        host.handle(InputEvent::delta(ControlId::Encoder(2), 127, 8));
        let held = host.tick_lfo(1_000);
        assert_eq!(midi_bytes(&held), vec![[0xB0, 51, 64]]);
        let peaked = host.tick_lfo(1_250);
        assert_eq!(midi_bytes(&peaked), vec![[0xB0, 51, 127]]);
        host.handle(InputEvent::delta(ControlId::Encoder(4), -32, 9));
        let restored = host.tick_lfo(1_260);
        assert_eq!(midi_bytes(&restored), vec![[0xB0, 51, 64]]);
    }

    #[test]
    fn a_reset_push_centres_the_matching_control() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display7),
            1,
            1,
        ));
        host.handle(InputEvent::delta(ControlId::Encoder(1), 20, 2));
        assert_eq!(host.encoder_text(host.active_page(), 1).1, "84");
        let reset = host.handle(InputEvent::delta(ControlId::Encoder(4), 1, 3));
        assert_eq!(host.encoder_text(host.active_page(), 1).1, "64");
        assert_eq!(
            midi_bytes(&reset),
            vec![[0xB0, 54, 1], [0xB0, 51, 64]]
        );
        host.handle(InputEvent::delta(ControlId::Encoder(0), 39, 4));
        assert_eq!(host.encoder_text(host.active_page(), 0).1, "Mixer");
        host.handle(InputEvent::delta(ControlId::Encoder(1), 10, 5));
        let kill = host.handle(InputEvent::delta(ControlId::Encoder(4), 1, 6));
        assert_eq!(host.encoder_text(host.active_page(), 1).1, "74");
        assert_eq!(midi_bytes(&kill), vec![[0xB0, 54, 2]]);
    }

    #[test]
    fn a_reset_push_flashes_a_square_then_goes_blank() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display7),
            1,
            1,
        ));
        let resting = crate::feedback::render(&host);
        assert!(!value_row_ink(&resting.screens[1], 0));
        host.handle(InputEvent::delta(ControlId::Encoder(4), -1, 10));
        assert!(host.reset_flashing(4));
        let flashed = crate::feedback::render(&host);
        assert!(value_mark_filled(&flashed.screens[1], 0));
        assert!(!value_row_outside_mark(&flashed.screens[1], 0));
        host.mark_feedback_sent(10);
        assert!(host.feedback_due(10 + 250));
        assert!(!host.reset_flashing(4));
        let cleared = crate::feedback::render(&host);
        assert!(!value_row_ink(&cleared.screens[1], 0));
    }

    fn value_row_ink(screen: &crate::feedback::MonoScreen, column: usize) -> bool {
        let x0 = column * 64;
        (56..64).any(|y| (x0..x0 + 64).any(|x| screen.pixels[y * screen.width + x] != 0))
    }

    fn value_mark_filled(screen: &crate::feedback::MonoScreen, column: usize) -> bool {
        let x0 = column * 64 + (64 - 8) / 2;
        (56..64).all(|y| (x0..x0 + 8).all(|x| screen.pixels[y * screen.width + x] != 0))
    }

    fn value_row_outside_mark(screen: &crate::feedback::MonoScreen, column: usize) -> bool {
        let x0 = column * 64;
        let mark = x0 + (64 - 8) / 2;
        (56..64).any(|y| {
            (x0..x0 + 64).any(|x| {
                (x < mark || x >= mark + 8) && screen.pixels[y * screen.width + x] != 0
            })
        })
    }

    #[test]
    fn category_arrow_lamps_show_whether_another_subpage_exists() {
        let config = load_yaml(include_str!("../docs/example-config.yaml")).expect("bitbox");
        let mut host = Controller::new(config);
        let lit = |host: &Controller, button: SilkButton| {
            crate::feedback::render(host).leds.buttons[button.index()]
        };
        assert!(!lit(&host, SilkButton::DisplayLeft));
        assert!(!lit(&host, SilkButton::DisplayRight));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display2),
            1,
            1,
        ));
        assert!(!lit(&host, SilkButton::DisplayLeft));
        assert!(lit(&host, SilkButton::DisplayRight));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            2,
        ));
        host.handle(InputEvent::release(
            ControlId::Button(SilkButton::DisplayRight),
            3,
        ));
        assert!(lit(&host, SilkButton::DisplayLeft));
        assert!(lit(&host, SilkButton::DisplayRight));
        for step in 0..4 {
            host.handle(InputEvent::press(
                ControlId::Button(SilkButton::DisplayRight),
                1,
                4 + step,
            ));
        }
        assert!(lit(&host, SilkButton::DisplayLeft));
        assert!(!lit(&host, SilkButton::DisplayRight));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            9,
        ));
        assert!(!lit(&host, SilkButton::DisplayLeft));
        assert!(lit(&host, SilkButton::DisplayRight));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display3),
            1,
            10,
        ));
        assert!(!lit(&host, SilkButton::DisplayLeft));
        assert!(!lit(&host, SilkButton::DisplayRight));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::Display5),
            1,
            11,
        ));
        assert!(!lit(&host, SilkButton::DisplayLeft));
        assert!(lit(&host, SilkButton::DisplayRight));
        host.handle(InputEvent::press(
            ControlId::Button(SilkButton::DisplayRight),
            1,
            12,
        ));
        assert!(lit(&host, SilkButton::DisplayLeft));
        assert!(!lit(&host, SilkButton::DisplayRight));
    }

    #[test]
    fn hid_assignment_drives_group_pad_release_encoder_and_sampler() {
        let map = crate::hidmap::Assignment::load_yaml(include_str!("../docs/hid-assignment.yaml"))
            .unwrap();
        let mut host = controller();
        let page = host.active_page();
        let mut idle = vec![0x20, 0, 0x00];
        idle.resize(65, 0);
        idle[25] = 0;
        idle[26] = 0xC0;
        let mut pressed = idle.clone();
        pressed[25] = 0xFC;
        pressed[26] = 0xC3;
        pressed[57] = 0xFC;
        pressed[58] = 0xC3;
        let pad = crate::hidmap::events_from_reports(&map, Some(&idle), &pressed, 10);
        let on = host.handle(pad[0]);
        assert_eq!(midi_bytes(&on), vec![[0x90, 36, pad[0].value as u8]]);
        let group = crate::hidmap::events_from_reports(
            &map,
            Some(&[0x01, 0, 0, 0, 0]),
            &[0x01, 0, 0, 0, 0x02],
            11,
        );
        host.handle(group[0]);
        assert_eq!(host.active_group(), GroupId::B);
        assert_eq!(host.active_page(), page);
        let release = crate::hidmap::events_from_reports(&map, Some(&pressed), &idle, 12);
        let off = host.handle(release[0]);
        assert_eq!(midi_bytes(&off), vec![[0x80, 36, 0]]);
        let turned = crate::hidmap::events_from_reports(
            &map,
            Some(&[0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0x4D, 0x03]),
            &[0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0x51, 0x03],
            13,
        );
        let cc = host.handle(turned[0]);
        assert_eq!(midi_bytes(&cc), vec![[0xB0, 20, 1]]);
        assert_eq!(host.parameter(0, 0).unwrap().value, 1);
        host.handle(
            crate::hidmap::events_from_reports(
                &map,
                Some(&[0x01, 0, 0, 0, 0]),
                &[0x01, 0, 0, 0, 0x80],
                14,
            )[0],
        );
        let local =
            host.handle(crate::hidmap::events_from_reports(&map, Some(&idle), &pressed, 15)[0]);
        assert!(local
            .iter()
            .any(|action| matches!(action, OutputAction::Sampler(_))));
    }
}
