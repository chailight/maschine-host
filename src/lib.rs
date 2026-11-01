//! Performance host for a full-size Maschine MK2.
//!
//! The core talks to hardware only through [`adapter::HardwareAdapter`].
//! macOS can read the full MK2 HID interface. Group, pad, and button LED
//! reports are sent only after a stored lamp map says the one-lamp check
//! matched. LCD panel reports are sent only after a stored panel map says
//! the one-panel check matched. See `docs/led-lamps.md` and `docs/lcd-panels.md`.

pub mod adapter;
pub mod config;
pub mod controller;
pub mod feedback;
pub mod hidmap;
pub mod hidopen;
pub mod ledmap;
pub mod panelmap;
pub mod midi;
pub mod model;
pub mod runtime;
pub mod sampler;

pub use config::load_yaml;
pub use controller::Controller;
pub use model::{HostConfig, InputEvent, OutputAction};
pub use runtime::{dispatch, self_check, AcceptanceReport};
