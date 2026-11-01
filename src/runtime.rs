use crate::adapter::{AdapterError, HardwareAdapter, MockAdapter};
use crate::feedback::FeedbackFrame;
use crate::config::load_yaml;
use crate::controller::Controller;
use crate::feedback::render;
use crate::midi::{MidiError, MidiSink, MockMidi};
use crate::model::{ControlId, GroupId, InputEvent, OutputAction};
use crate::sampler::{pump, FailingSink, Mixer};
use std::fs;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DispatchReport {
    pub errors: Vec<String>,
}

pub fn dispatch(
    controller: &mut Controller,
    event: InputEvent,
    midi: &mut dyn MidiSink,
    mixer: &mut Mixer,
    adapter: &mut dyn HardwareAdapter,
) -> DispatchReport {
    let now = event.time_ms;
    let actions = controller.handle(event);
    apply_actions(controller, actions, midi, mixer, adapter, now)
}

pub fn apply_actions(
    controller: &mut Controller,
    actions: Vec<OutputAction>,
    midi: &mut dyn MidiSink,
    mixer: &mut Mixer,
    adapter: &mut dyn HardwareAdapter,
    now_ms: u64,
) -> DispatchReport {
    let mut report = DispatchReport::default();
    for action in actions {
        match action {
            OutputAction::Midi(message) => {
                if let Err(error) = midi.send(&message.port, &message.bytes) {
                    if let (
                        MidiError::Unavailable(_) | MidiError::Send(_) | MidiError::Backend(_),
                        Some(pad),
                    ) = (&error, message.confirm_pad)
                    {
                        if message.bytes[0] & 0xF0 == 0x90 {
                            controller.abandon_unconfirmed(pad);
                        }
                    }
                    report.errors.push(error.to_string());
                }
            }
            OutputAction::Sampler(command) => {
                if let Err(error) = mixer.apply(&command.kind) {
                    report.errors.push(error);
                }
            }
            OutputAction::Error(message) => report.errors.push(message),
        }
    }
    if controller.feedback_due(now_ms) {
        let frame = render(controller);
        match adapter.submit(&frame) {
            Ok(()) => controller.mark_feedback_sent(now_ms),
            Err(error) => {
                report.errors.push(error.to_string());
                controller.mark_feedback_sent(now_ms);
            }
        }
    }
    report
}

#[derive(Debug, PartialEq, Eq)]
pub struct AcceptanceReport {
    pub midi_messages: Vec<(String, Vec<u8>)>,
    pub display_contains_group_b: bool,
    pub transport_errors: usize,
}

pub fn self_check(config_text: &str) -> Result<AcceptanceReport, String> {
    let config = load_yaml(config_text).map_err(|issues| {
        issues
            .iter()
            .map(|issue| issue.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mut controller = Controller::new(config.clone());
    let mut midi = MockMidi {
        open: config
            .midi_outputs
            .iter()
            .map(|port| port.id.clone())
            .collect(),
        sent: Vec::new(),
    };
    let mut mixer = Mixer::empty(config.polyphony, config.sample_rate, config.buffer_frames);
    let mut adapter = MockAdapter::default();
    let page_before = controller.active_page();
    dispatch(
        &mut controller,
        InputEvent::press(ControlId::Group(GroupId::B), 1, 1),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    if controller.active_group() != GroupId::B || controller.active_page() != page_before {
        return Err("group B changed the encoder page".to_string());
    }
    dispatch(
        &mut controller,
        InputEvent::press(ControlId::Pad(0), 110, 2),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    dispatch(
        &mut controller,
        InputEvent::press(ControlId::Group(GroupId::A), 1, 3),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    dispatch(
        &mut controller,
        InputEvent::release(ControlId::Pad(0), 4),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    dispatch(
        &mut controller,
        InputEvent::press(ControlId::PageRight, 1, 60),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    dispatch(
        &mut controller,
        InputEvent::delta(ControlId::Encoder(0), 3, 61),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    apply_actions(
        &mut controller,
        Vec::new(),
        &mut midi,
        &mut mixer,
        &mut adapter,
        200,
    );
    if !adapter
        .submitted
        .iter()
        .any(|frame| frame.right_text.contains("3 cmd"))
    {
        return Err("encoder movement did not reach a display frame".to_string());
    }
    let disconnect = controller.disconnect();
    apply_actions(
        &mut controller,
        disconnect,
        &mut midi,
        &mut mixer,
        &mut adapter,
        70,
    );
    if !controller.held().is_empty() {
        return Err("disconnect left a held note".to_string());
    }
    let display_contains_group_b = adapter
        .submitted
        .iter()
        .any(|frame| frame.left_text.contains("GROUP B"));
    if !display_contains_group_b {
        return Err("display never showed group B".to_string());
    }
    let note_on = midi
        .sent
        .iter()
        .any(|(_, bytes)| bytes.len() == 3 && bytes[0] & 0xF0 == 0x90);
    let note_off = midi.sent.iter().any(|(_, bytes)| {
        bytes.len() == 3 && bytes[0] & 0xF0 == 0x80 && bytes[1] == midi.sent[0].1[1]
    });
    if !note_on || !note_off {
        return Err("acceptance did not pair a note-on with a note-off".to_string());
    }
    Ok(AcceptanceReport {
        midi_messages: midi.sent,
        display_contains_group_b,
        transport_errors: 0,
    })
}

pub fn example_config_text() -> Result<String, String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/example-config.yaml");
    fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn transport_failure_does_not_block_notes() -> Result<(), String> {
    let config = load_yaml(&example_body()).map_err(|issues| {
        issues
            .iter()
            .map(|issue| issue.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mut controller = Controller::new(config);
    let mut midi = MockMidi {
        open: vec!["bitbox".into()],
        sent: Vec::new(),
    };
    let mut mixer = Mixer::empty(8, 48_000, 128);
    let mut adapter = crate::adapter::UnavailableAdapter;
    let report = dispatch(
        &mut controller,
        InputEvent::press(ControlId::Pad(0), 100, 1),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    if midi.sent.is_empty() {
        return Err("note was blocked by the unverified display transport".to_string());
    }
    if report.errors.is_empty() {
        return Err("expected a transport error".to_string());
    }
    let mut buffer = [0.0; 16];
    let audio_error = pump(&mut mixer, &mut FailingSink, &mut buffer).unwrap_err();
    if !audio_error.contains("audio device") {
        return Err(audio_error);
    }
    let follow = dispatch(
        &mut controller,
        InputEvent::release(ControlId::Pad(0), 2),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    if follow.errors.iter().any(|error| error.contains("panic")) {
        return Err("audio failure poisoned note routing".to_string());
    }
    if !midi.sent.iter().any(|(_, bytes)| bytes[0] & 0xF0 == 0x80) {
        return Err("note-off missing after audio device failure".to_string());
    }
    Ok(())
}

pub fn lamp_write_failure_does_not_block_notes() -> Result<(), String> {
    let config = load_yaml(&example_body()).map_err(|issues| {
        issues
            .iter()
            .map(|issue| issue.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mut controller = Controller::new(config);
    let mut midi = MockMidi {
        open: vec!["bitbox".into()],
        sent: Vec::new(),
    };
    let mut mixer = Mixer::empty(8, 48_000, 128);
    let mut adapter = FailingLamps;
    let report = dispatch(
        &mut controller,
        InputEvent::press(ControlId::Pad(0), 100, 1),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    if midi.sent.is_empty() {
        return Err("note was blocked by the lamp write".to_string());
    }
    if !report.errors.iter().any(|error| error.contains("lamp write")) {
        return Err("expected a lamp write error".to_string());
    }
    dispatch(
        &mut controller,
        InputEvent::release(ControlId::Pad(0), 2),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    if !midi.sent.iter().any(|(_, bytes)| bytes[0] & 0xF0 == 0x80) {
        return Err("note-off missing after the lamp write failed".to_string());
    }
    Ok(())
}

pub fn panel_write_failure_does_not_block_notes() -> Result<(), String> {
    let config = load_yaml(&example_body()).map_err(|issues| {
        issues
            .iter()
            .map(|issue| issue.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mut controller = Controller::new(config);
    let mut midi = MockMidi {
        open: vec!["bitbox".into()],
        sent: Vec::new(),
    };
    let mut mixer = Mixer::empty(8, 48_000, 128);
    let mut adapter = FailingPanels;
    let report = dispatch(
        &mut controller,
        InputEvent::press(ControlId::Pad(0), 100, 1),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    if midi.sent.is_empty() {
        return Err("note was blocked by the panel write".to_string());
    }
    if !report.errors.iter().any(|error| error.contains("panel write")) {
        return Err("expected a panel write error".to_string());
    }
    dispatch(
        &mut controller,
        InputEvent::release(ControlId::Pad(0), 2),
        &mut midi,
        &mut mixer,
        &mut adapter,
    );
    if !midi.sent.iter().any(|(_, bytes)| bytes[0] & 0xF0 == 0x80) {
        return Err("note-off missing after the panel write failed".to_string());
    }
    Ok(())
}

struct FailingPanels;

impl HardwareAdapter for FailingPanels {
    fn poll(&mut self) -> Vec<InputEvent> {
        Vec::new()
    }

    fn submit(&mut self, _frame: &FeedbackFrame) -> Result<(), AdapterError> {
        Err(AdapterError::PanelWrite("write failed".to_string()))
    }
}

struct FailingLamps;

impl HardwareAdapter for FailingLamps {
    fn poll(&mut self) -> Vec<InputEvent> {
        Vec::new()
    }

    fn submit(&mut self, _frame: &FeedbackFrame) -> Result<(), AdapterError> {
        Err(AdapterError::LampWrite("write failed".to_string()))
    }
}

fn example_body() -> String {
    let notes = (36..52)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let mut groups = String::new();
    for (index, id) in GroupId::ALL.iter().enumerate() {
        groups.push_str(&format!(
            "  - id: {id}\n    label: G{id}\n    mode: one_shot\n    destination: {{type: midi, port: bitbox, channel: {}}}\n    notes: [{notes}]\n",
            index + 1
        ));
    }
    let mut encoders = String::new();
    for index in 1..=8 {
        encoders.push_str(&format!(
            "      - {{label: Slot {index}, type: midi_cc, port: bitbox, channel: 1, cc: null}}\n"
        ));
    }
    format!(
        "schema_version: 1\nmidi_outputs:\n  bitbox: CONFIGURE_PORT_NAME\ngroups:\n{groups}encoder_pages:\n  - id: delay\n    label: Delay sends\n    encoders:\n{encoders}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_profile_passes_the_mock_acceptance_scenario() {
        let text = example_config_text().expect("example config");
        self_check(&text).expect("self-check");
    }

    #[test]
    fn a_failed_panel_write_still_routes_the_note() {
        panel_write_failure_does_not_block_notes().unwrap();
    }

    #[test]
    fn missing_port_is_recoverable_and_does_not_hold_the_note() {
        let config = load_yaml(&example_body()).unwrap();
        let mut controller = Controller::new(config);
        let mut midi = MockMidi::default();
        let mut mixer = Mixer::empty(4, 48_000, 64);
        let mut adapter = MockAdapter::default();
        let report = dispatch(
            &mut controller,
            InputEvent::press(ControlId::Pad(0), 80, 1),
            &mut midi,
            &mut mixer,
            &mut adapter,
        );
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("unavailable")));
        assert!(controller.held().is_empty());
    }

    #[test]
    fn audio_and_display_failures_leave_note_off_routing_intact() {
        transport_failure_does_not_block_notes().unwrap();
    }

    #[test]
    fn a_failed_lamp_write_still_routes_the_following_note() {
        lamp_write_failure_does_not_block_notes().unwrap();
    }
}
