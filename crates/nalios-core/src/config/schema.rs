//! Serde representation of the supported configuration schema.
//!
//! Field names, defaults and bounds are part of the versioned document
//! contract. Changing them incompatibly requires a new
//! [`CURRENT_SCHEMA_VERSION`] and an explicit migration.

use serde::{Deserialize, Serialize};

/// Current supported configuration schema version.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Maximum accepted configuration document size in bytes.
pub const MAX_CONFIG_BYTES: usize = 64 * 1024;

/// Smallest accepted `launcher.max_results` value.
pub const MIN_LAUNCHER_MAX_RESULTS: u32 = 1;

/// Largest accepted `launcher.max_results` value.
pub const MAX_LAUNCHER_MAX_RESULTS: u32 = 50;

/// Largest accepted `privacy.history_retention_days` value.
pub const MAX_HISTORY_RETENTION_DAYS: u32 = 90;

/// Largest accepted number of launcher favorites.
pub const MAX_LAUNCHER_FAVORITES: usize = 32;

/// Largest accepted launcher favorite identifier length.
pub const MAX_FAVORITE_ID_LEN: usize = 128;

/// Fully validated configuration document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Schema version of this document.
    pub schema_version: u32,
    /// Launcher behavior.
    #[serde(default)]
    pub launcher: LauncherConfig,
    /// Privacy-related behavior.
    #[serde(default)]
    pub privacy: PrivacyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            launcher: LauncherConfig::default(),
            privacy: PrivacyConfig::default(),
        }
    }
}

/// Launcher behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LauncherConfig {
    /// Maximum number of search results shown at once.
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    /// Desktop file ids pinned to the top of launcher results.
    #[serde(default)]
    pub favorites: Vec<String>,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
            favorites: Vec::new(),
        }
    }
}

fn default_max_results() -> u32 {
    8
}

/// Privacy-related behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivacyConfig {
    /// Whether clipboard history is recorded and for how long it survives.
    #[serde(default)]
    pub clipboard_history: ClipboardHistory,
    /// Days to keep persistent history; `0` means not applicable.
    #[serde(default)]
    pub history_retention_days: u32,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            clipboard_history: ClipboardHistory::default(),
            history_retention_days: 0,
        }
    }
}

/// Clipboard history mode.
///
/// History is disabled by default and must be opted into explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardHistory {
    /// No clipboard history is recorded.
    #[default]
    Disabled,
    /// History exists only for the current session.
    Session,
    /// History survives restarts for `history_retention_days`.
    Persistent,
}
