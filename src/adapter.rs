use crate::feedback::FeedbackFrame;
use crate::model::InputEvent;
use std::collections::VecDeque;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AdapterError {
    #[error(
        "MK2 USB, LCD and LED transport is not implemented; the hardware spike did not verify a report map on this machine"
    )]
    TransportUnverified,
    #[error("MK2 lamp write failed: {0}")]
    LampWrite(String),
    #[error("MK2 panel write failed: {0}")]
    PanelWrite(String),
}

pub trait HardwareAdapter {
    fn poll(&mut self) -> Vec<InputEvent>;
    fn submit(&mut self, frame: &FeedbackFrame) -> Result<(), AdapterError>;
}

/// Records normalized events and feedback. It does not speak USB.
#[derive(Default)]
pub struct MockAdapter {
    pub pending: VecDeque<InputEvent>,
    pub submitted: Vec<FeedbackFrame>,
}

impl HardwareAdapter for MockAdapter {
    fn poll(&mut self) -> Vec<InputEvent> {
        self.pending.drain(..).collect()
    }

    fn submit(&mut self, frame: &FeedbackFrame) -> Result<(), AdapterError> {
        self.submitted.push(frame.clone());
        Ok(())
    }
}

/// Refuses device I/O. Note routing must not wait on this adapter.
#[derive(Default)]
pub struct UnavailableAdapter;

impl HardwareAdapter for UnavailableAdapter {
    fn poll(&mut self) -> Vec<InputEvent> {
        Vec::new()
    }

    fn submit(&mut self, _frame: &FeedbackFrame) -> Result<(), AdapterError> {
        Err(AdapterError::TransportUnverified)
    }
}
