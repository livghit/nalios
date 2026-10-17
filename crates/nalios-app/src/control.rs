//! Versioned shell-control protocol.
//!
//! The CLI sends allowlisted actions to a running shell over a private Unix
//! socket. Actions are named; there is no arbitrary command execution and no
//! credential path.

use serde::{Deserialize, Serialize};

/// Version of the control protocol.
pub const CONTROL_SCHEMA_VERSION: u32 = 1;

/// Allowlisted shell actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControlAction {
    /// Check that a shell is listening.
    Ping,
    /// Open the launcher if it is closed.
    OpenLauncher,
    /// Toggle the launcher.
    ToggleLauncher,
    /// Toggle the full dashboard.
    ToggleDashboard,
}

/// A control request sent by the CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlRequest {
    /// Protocol version.
    pub schema_version: u32,
    /// Requested action.
    pub action: ControlAction,
}

impl ControlRequest {
    /// Build a request with the current protocol version.
    pub fn new(action: ControlAction) -> Self {
        Self {
            schema_version: CONTROL_SCHEMA_VERSION,
            action,
        }
    }

    /// Validate the protocol version.
    pub fn validate(&self) -> Result<(), ControlVersionError> {
        if self.schema_version != CONTROL_SCHEMA_VERSION {
            return Err(ControlVersionError {
                found: self.schema_version,
            });
        }
        Ok(())
    }
}

/// A control protocol version failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("control protocol version {found} is not supported (expected {CONTROL_SCHEMA_VERSION})")]
pub struct ControlVersionError {
    /// Version found in the request.
    pub found: u32,
}

/// A control response from the shell.
///
/// A success means the action was accepted, not necessarily completed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ControlResponse {
    /// The action was accepted.
    Ok,
    /// The action was rejected.
    Error {
        /// User-safe rejection reason.
        message: String,
    },
}
