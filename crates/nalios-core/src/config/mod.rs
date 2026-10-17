//! Versioned NaliOS configuration.
//!
//! This module parses and validates configuration documents supplied as text.
//! It performs no file, environment or service access; adapters read bytes and
//! callers decide what to do with the result.
//!
//! A document starts with `schema_version = 1`. Unknown fields are rejected so
//! a misspelled critical setting cannot be silently ignored. Incompatible
//! schema changes require a new version and an explicit migration.

mod schema;
mod validate;

pub use schema::{
    ClipboardHistory, Config, LauncherConfig, PrivacyConfig, CURRENT_SCHEMA_VERSION,
    MAX_CONFIG_BYTES, MAX_FAVORITE_ID_LEN, MAX_HISTORY_RETENTION_DAYS, MAX_LAUNCHER_FAVORITES,
    MAX_LAUNCHER_MAX_RESULTS, MIN_LAUNCHER_MAX_RESULTS,
};
pub use validate::{parse, validate, ConfigError};
