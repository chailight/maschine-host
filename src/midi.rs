use crate::model::MidiPortConfig;
use std::collections::HashMap;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use midir::os::unix::VirtualOutput;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MidiError {
    #[error("MIDI port {0} is unavailable")]
    Unavailable(String),
    #[error("MIDI send failed: {0}")]
    Send(String),
    #[error("MIDI backend failed: {0}")]
    Backend(String),
}

pub trait MidiSink {
    fn send(&mut self, port: &str, message: &[u8]) -> Result<(), MidiError>;
}

#[derive(Default)]
pub struct MockMidi {
    pub open: Vec<String>,
    pub sent: Vec<(String, Vec<u8>)>,
}

impl MidiSink for MockMidi {
    fn send(&mut self, port: &str, message: &[u8]) -> Result<(), MidiError> {
        if !self.open.iter().any(|name| name == port) {
            return Err(MidiError::Unavailable(port.to_string()));
        }
        self.sent.push((port.to_string(), message.to_vec()));
        Ok(())
    }
}

pub struct PlatformMidi {
    pub backend: &'static str,
    pub backend_error: Option<String>,
    pub unavailable: Vec<String>,
    connections: HashMap<String, midir::MidiOutputConnection>,
}

impl PlatformMidi {
    pub fn backend_name() -> &'static str {
        #[cfg(target_os = "linux")]
        {
            "alsa"
        }
        #[cfg(target_os = "macos")]
        {
            "coremidi"
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            "unsupported"
        }
    }

    pub fn list_port_names() -> Result<Vec<String>, MidiError> {
        let output = midir::MidiOutput::new("maschine2-host")
            .map_err(|error| MidiError::Backend(error.to_string()))?;
        let mut names = Vec::new();
        for port in output.ports() {
            names.push(
                output
                    .port_name(&port)
                    .map_err(|error| MidiError::Backend(error.to_string()))?,
            );
        }
        Ok(names)
    }

    pub fn open(ports: &[MidiPortConfig]) -> Self {
        let backend = Self::backend_name();
        let mut connections = HashMap::new();
        let mut unavailable = Vec::new();
        let mut backend_error = None;
        for port in ports {
            if port.virtual_port {
                match open_virtual(&port.name) {
                    Ok(connection) => {
                        connections.insert(port.id.clone(), connection);
                    }
                    Err(error) => {
                        unavailable.push(port.id.clone());
                        backend_error.get_or_insert(error.to_string());
                    }
                }
                continue;
            }
            match open_named(&port.name) {
                Ok(Some(connection)) => {
                    connections.insert(port.id.clone(), connection);
                }
                Ok(None) => unavailable.push(port.id.clone()),
                Err(error) => {
                    unavailable.push(port.id.clone());
                    backend_error.get_or_insert(error.to_string());
                }
            }
        }
        Self {
            backend,
            backend_error,
            unavailable,
            connections,
        }
    }
}

impl MidiSink for PlatformMidi {
    fn send(&mut self, port: &str, message: &[u8]) -> Result<(), MidiError> {
        if let Some(error) = &self.backend_error {
            if !self.connections.contains_key(port) {
                return Err(MidiError::Backend(error.clone()));
            }
        }
        match self.connections.get_mut(port) {
            Some(connection) => connection
                .send(message)
                .map_err(|error| MidiError::Send(error.to_string())),
            None => Err(MidiError::Unavailable(port.to_string())),
        }
    }
}

fn open_named(system_name: &str) -> Result<Option<midir::MidiOutputConnection>, MidiError> {
    let output = midir::MidiOutput::new("maschine2-host")
        .map_err(|error| MidiError::Backend(error.to_string()))?;
    let ports = output.ports();
    let Some(port) = ports
        .iter()
        .find(|port| output.port_name(port).ok().as_deref() == Some(system_name))
    else {
        return Ok(None);
    };
    output
        .connect(port, "maschine2-host")
        .map(Some)
        .map_err(|error| MidiError::Backend(error.to_string()))
}

fn open_virtual(name: &str) -> Result<midir::MidiOutputConnection, MidiError> {
    let output = midir::MidiOutput::new("maschine2-host")
        .map_err(|error| MidiError::Backend(error.to_string()))?;
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        output
            .create_virtual(name)
            .map_err(|error| MidiError::Backend(error.to_string()))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = name;
        Err(MidiError::Backend(
            "virtual MIDI ports are not implemented on this operating system".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MidiMessage;

    #[test]
    fn mock_reports_a_missing_port_without_sending() {
        let mut midi = MockMidi {
            open: vec!["present".into()],
            sent: Vec::new(),
        };
        let message = MidiMessage::note_on("missing", 1, 60, 100, 0);
        let error = midi.send("missing", &message.bytes).unwrap_err();
        assert_eq!(error, MidiError::Unavailable("missing".into()));
        assert!(midi.sent.is_empty());
        midi.send("present", &message.bytes).unwrap();
        assert_eq!(midi.sent.len(), 1);
    }

    #[test]
    fn note_and_cc_bytes_use_the_configured_channel() {
        let note = MidiMessage::note_on("bitbox", 2, 60, 99, 3);
        assert_eq!(note.bytes, [0x91, 60, 99]);
        let off = MidiMessage::note_off("bitbox", 2, 60);
        assert_eq!(off.bytes, [0x81, 60, 0]);
        let cc = MidiMessage::cc("bitbox", 8, 22, 10);
        assert_eq!(cc.bytes, [0xB7, 22, 10]);
    }

    #[test]
    fn named_port_that_is_absent_is_recoverable() {
        let midi = PlatformMidi::open(&[MidiPortConfig {
            id: "bitbox".into(),
            name: "CONFIGURE_PORT_NAME".into(),
            virtual_port: false,
        }]);
        assert_eq!(midi.backend, PlatformMidi::backend_name());
        assert!(midi.unavailable.iter().any(|id| id == "bitbox") || midi.backend_error.is_some());
    }

    #[test]
    fn virtual_port_can_be_created_or_reports_a_backend_error() {
        let midi = PlatformMidi::open(&[MidiPortConfig {
            id: "virtual".into(),
            name: "maschine2-host-test".into(),
            virtual_port: true,
        }]);
        if midi.backend_error.is_some() {
            assert!(midi.unavailable.iter().any(|id| id == "virtual"));
            return;
        }
        assert!(midi.connections.contains_key("virtual"));
    }
}
