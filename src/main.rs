use maschine2_host::adapter::{HardwareAdapter, UnavailableAdapter};
use maschine2_host::config::load_yaml;
use maschine2_host::controller::Controller;
use maschine2_host::hidmap::Assignment;
use maschine2_host::hidopen::{self, ControllerLink};
use maschine2_host::ledmap::{self, LampMap};
use maschine2_host::panelmap::{self, PanelMap};
use maschine2_host::midi::PlatformMidi;
use maschine2_host::runtime::{self, apply_actions};
use maschine2_host::sampler::{preload, Mixer};
use std::env;
use std::fs;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help") {
        println!(
            "maschine2-host --config <file>\n  --self-check   run the mock acceptance scenario\n  --probe-audio  print the audio backend probe and MIDI port names\n  --capture-hid [seconds]  log raw MK2 HID input reports and emit no control events\n  --listen  print each confirmed pad, button, and encoder step\n  --light-check  light the bottom-left pad, then Group A, and store nothing\n  --panel-check  paint the left LCD only and store nothing\n  --hid-assignment <file>  confirmed control map, default docs/hid-assignment.yaml"
        );
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|arg| arg == "--self-check") {
        return match runtime::example_config_text().and_then(|text| runtime::self_check(&text)) {
            Ok(_) => {
                println!("self-check ok");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        };
    }
    if args.iter().any(|arg| arg == "--listen") {
        let mut assignment = match load_assignment(argument_value(&args, "--hid-assignment")) {
            Ok(assignment) => assignment,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::from(1);
            }
        };
        if let Err(error) = merge_silk_buttons(&mut assignment) {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
        return match hidopen::listen(assignment) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        };
    }
    if args.iter().any(|arg| arg == "--panel-check") {
        return match hidopen::panel_check() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        };
    }
    if args.iter().any(|arg| arg == "--light-check") {
        return match hidopen::light_check() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        };
    }
    if args.iter().any(|arg| arg == "--capture-hid") {
        let seconds = argument_value(&args, "--capture-hid")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(5);
        return match hidopen::capture(seconds) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        };
    }
    if args.iter().any(|arg| arg == "--probe-audio") {
        match maschine2_host::sampler::probe_default_output() {
            Ok(report) => println!("audio {report}"),
            Err(error) => println!("audio probe: {error}"),
        }
        match PlatformMidi::list_port_names() {
            Ok(names) => println!(
                "midi backend {} ports: {}",
                PlatformMidi::backend_name(),
                names.join(" | ")
            ),
            Err(error) => println!("midi probe: {error}"),
        }
        return ExitCode::SUCCESS;
    }

    let Some(config_path) = argument_value(&args, "--config") else {
        eprintln!("pass --config <file>, or --self-check");
        return ExitCode::from(2);
    };
    let text = match fs::read_to_string(&config_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("{config_path}: {error}");
            return ExitCode::from(1);
        }
    };
    let config = match load_yaml(&text) {
        Ok(config) => config,
        Err(issues) => {
            for issue in issues {
                eprintln!("{issue}");
            }
            return ExitCode::from(1);
        }
    };
    let midi = PlatformMidi::open(&config.midi_outputs);
    if let Some(error) = &midi.backend_error {
        eprintln!("MIDI backend {} error: {error}", midi.backend);
    }
    for port in &midi.unavailable {
        eprintln!("MIDI port '{port}' is unavailable; sends to it will fail and recover");
    }
    let mixer = match preload(&config) {
        Ok(mixer) => mixer,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            eprintln!("sample preload failed; configuration was not activated");
            return ExitCode::from(1);
        }
    };
    let mut assignment = match load_assignment(argument_value(&args, "--hid-assignment")) {
        Ok(assignment) => assignment,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };
    if let Err(error) = merge_silk_buttons(&mut assignment) {
        eprintln!("{error}");
        eprintln!("silk-screen button lamps stay off");
    }
    eprintln!(
        "Nominal mixer latency is {:.2} ms at {} Hz / {} frames; that figure is not a measured round trip.",
        mixer.nominal_latency_ms(),
        config.sample_rate,
        config.buffer_frames
    );
    run(config, midi, mixer, assignment)
}

fn run(
    config: maschine2_host::HostConfig,
    mut midi: PlatformMidi,
    mut mixer: Mixer,
    assignment: Assignment,
) -> ExitCode {
    let mut controller = Controller::new(config);
    let confirmed = assignment.len();
    let lamps = match ledmap::load_lamp_map(std::path::Path::new(ledmap::LAMP_MAP_PATH)) {
        Ok(map) => map,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("group and pad LEDs stay unchanged");
            LampMap::unmatched()
        }
    };
    let panels = match panelmap::load_panel_map(std::path::Path::new(panelmap::PANEL_MAP_PATH)) {
        Ok(map) => map,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("LCD panels stay unwritten");
            PanelMap::unmatched()
        }
    };
    let mut adapter: Box<dyn HardwareAdapter> = match hidopen::open_input(assignment, lamps, panels) {
        ControllerLink::Open(mut input) => {
            if input.lamps_matched() && input.panels_matched() {
                eprintln!(
                    "MK2 HID input is open with {confirmed} confirmed control(s). Group, pad, and button lamps follow the selection. The LCD panels show the active encoder page. A press with no confirmed pressure field uses stand-in value {}.",
                    input.stand_in_press()
                );
            } else if input.lamps_matched() {
                eprintln!(
                    "MK2 HID input is open with {confirmed} confirmed control(s). Group, pad, and button lamps follow the selection. The panel map is not matched, so the LCD panels stay unwritten. A press with no confirmed pressure field uses stand-in value {}.",
                    input.stand_in_press()
                );
            } else {
                eprintln!(
                    "MK2 HID input is open with {confirmed} confirmed control(s). The lamp map is not matched, so group and pad LEDs stay unchanged. A press with no confirmed pressure field uses stand-in value {}.",
                    input.stand_in_press()
                );
            }
            let actions = controller.connect();
            let report = apply_actions(
                &mut controller,
                actions,
                &mut midi,
                &mut mixer,
                &mut input,
                now_ms(),
            );
            for error in report.errors {
                eprintln!("{error}");
            }
            Box::new(input)
        }
        ControllerLink::Refused(error) => {
            eprintln!("HID read refused: {error}");
            eprintln!("controller left disconnected");
            let actions = controller.disconnect();
            let report = apply_actions(
                &mut controller,
                actions,
                &mut midi,
                &mut mixer,
                &mut UnavailableAdapter,
                now_ms(),
            );
            for error in report.errors {
                eprintln!("{error}");
            }
            Box::new(UnavailableAdapter)
        }
        ControllerLink::Absent => {
            eprintln!(
                "No full MK2 HID interface is open. Pads, LEDs and LCDs stay unread and unwritten."
            );
            Box::new(UnavailableAdapter)
        }
    };
    let stop = Arc::new(AtomicBool::new(false));
    if let Err(error) = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop))
    {
        eprintln!("signal handler: {error}");
        return ExitCode::from(1);
    }
    if let Err(error) = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&stop))
    {
        eprintln!("signal handler: {error}");
        return ExitCode::from(1);
    }
    while !stop.load(Ordering::Relaxed) {
        for event in adapter.poll() {
            let report = runtime::dispatch(
                &mut controller,
                event,
                &mut midi,
                &mut mixer,
                adapter.as_mut(),
            );
            for error in report.errors {
                eprintln!("{error}");
            }
        }
        let now = now_ms();
        let actions = controller.tick_lfo(now);
        if !actions.is_empty() || controller.feedback_due(now) {
            let report = apply_actions(
                &mut controller,
                actions,
                &mut midi,
                &mut mixer,
                adapter.as_mut(),
                now,
            );
            for error in report.errors {
                eprintln!("{error}");
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    let actions = controller.shutdown();
    let report = apply_actions(
        &mut controller,
        actions,
        &mut midi,
        &mut mixer,
        adapter.as_mut(),
        now_ms(),
    );
    for error in report.errors {
        eprintln!("{error}");
    }
    ExitCode::SUCCESS
}

fn merge_silk_buttons(assignment: &mut Assignment) -> Result<(), String> {
    let text = fs::read_to_string("docs/hid-buttons.yaml")
        .map_err(|error| format!("docs/hid-buttons.yaml: {error}"))?;
    assignment.extend(Assignment::load_named_yaml(&text)?);
    Ok(())
}

fn load_assignment(path: Option<&str>) -> Result<Assignment, String> {
    let path = path.unwrap_or("docs/hid-assignment.yaml");
    match fs::read_to_string(path) {
        Ok(text) => Assignment::load_yaml(&text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("{path} is missing; no confirmed controls will emit events");
            Ok(Assignment::empty())
        }
        Err(error) => Err(format!("{path}: {error}")),
    }
}

fn argument_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
