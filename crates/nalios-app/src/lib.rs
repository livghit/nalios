//! NaliOS shared application workflows.
//!
//! The application layer owns behavior that both the CLI and the shell must
//! share. It defines narrow ports for external effects; `nalios-platform`
//! implements them and tests supply deterministic fakes.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod apps;
pub mod config;
pub mod control;
pub mod diagnostics;
pub mod niri;
