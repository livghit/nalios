//! Parsing and semantic validation for configuration documents.

use serde::Deserialize;

use super::schema::{
    ClipboardHistory, Config, CURRENT_SCHEMA_VERSION, MAX_CONFIG_BYTES, MAX_FAVORITE_ID_LEN,
    MAX_HISTORY_RETENTION_DAYS, MAX_LAUNCHER_FAVORITES, MAX_LAUNCHER_MAX_RESULTS,
    MIN_LAUNCHER_MAX_RESULTS,
};

/// A configuration parsing or validation failure.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The document exceeds the accepted size limit.
    #[error("configuration document exceeds the {MAX_CONFIG_BYTES}-byte limit")]
    TooLarge,
    /// The document is not valid TOML for the supported schema.
    #[error("configuration is not valid TOML: {0}")]
    Malformed(#[source] toml::de::Error),
    /// The required `schema_version` field is absent.
    #[error("configuration is missing the required `schema_version` field")]
    MissingSchemaVersion,
    /// The document declares a schema version this build does not support.
    #[error("configuration schema version {found} is not supported (expected {CURRENT_SCHEMA_VERSION})")]
    UnsupportedSchemaVersion {
        /// Version found in the document.
        found: u32,
    },
    /// `launcher.max_results` is outside its accepted range.
    #[error("launcher.max_results must be between {MIN_LAUNCHER_MAX_RESULTS} and {MAX_LAUNCHER_MAX_RESULTS} (found {found})")]
    LauncherMaxResultsOutOfRange {
        /// Value found in the document.
        found: u32,
    },
    /// Retention was set although history is not persistent.
    #[error("privacy.history_retention_days must be 0 unless clipboard history is persistent (found {found})")]
    RetentionSetWithoutPersistentHistory {
        /// Value found in the document.
        found: u32,
    },
    /// Persistent history has no bounded retention window.
    #[error("privacy.history_retention_days must be between 1 and {MAX_HISTORY_RETENTION_DAYS} for persistent clipboard history (found {found})")]
    PersistentHistoryRetentionOutOfRange {
        /// Value found in the document.
        found: u32,
    },
    /// Too many launcher favorites were declared.
    #[error("launcher.favorites may contain at most {MAX_LAUNCHER_FAVORITES} entries (found {found})")]
    TooManyFavorites {
        /// Number of entries found.
        found: usize,
    },
    /// A launcher favorite entry is empty or too long.
    #[error("launcher.favorites entry '{value}' is invalid")]
    InvalidFavorite {
        /// Offending entry.
        value: String,
    },
}

/// Parse and validate a configuration document.
///
/// The size limit is checked before parsing, the schema version before field
/// validation, and semantic constraints after deserialization.
pub fn parse(document: &str) -> Result<Config, ConfigError> {
    if document.len() > MAX_CONFIG_BYTES {
        return Err(ConfigError::TooLarge);
    }

    let probe: VersionProbe = toml::from_str(document).map_err(ConfigError::Malformed)?;
    match probe.schema_version {
        None => return Err(ConfigError::MissingSchemaVersion),
        Some(found) if found != CURRENT_SCHEMA_VERSION => {
            return Err(ConfigError::UnsupportedSchemaVersion { found });
        }
        Some(_) => {}
    }

    let config: Config = toml::from_str(document).map_err(ConfigError::Malformed)?;
    validate(&config)?;
    Ok(config)
}

/// Validate an already-deserialized configuration document.
pub fn validate(config: &Config) -> Result<(), ConfigError> {
    if !(MIN_LAUNCHER_MAX_RESULTS..=MAX_LAUNCHER_MAX_RESULTS).contains(&config.launcher.max_results)
    {
        return Err(ConfigError::LauncherMaxResultsOutOfRange {
            found: config.launcher.max_results,
        });
    }

    if config.launcher.favorites.len() > MAX_LAUNCHER_FAVORITES {
        return Err(ConfigError::TooManyFavorites {
            found: config.launcher.favorites.len(),
        });
    }
    if let Some(favorite) = config.launcher.favorites.iter().find(|favorite| {
        favorite.trim().is_empty() || favorite.len() > MAX_FAVORITE_ID_LEN
    }) {
        return Err(ConfigError::InvalidFavorite {
            value: favorite.clone(),
        });
    }

    match config.privacy.clipboard_history {
        ClipboardHistory::Disabled | ClipboardHistory::Session => {
            if config.privacy.history_retention_days != 0 {
                return Err(ConfigError::RetentionSetWithoutPersistentHistory {
                    found: config.privacy.history_retention_days,
                });
            }
        }
        ClipboardHistory::Persistent => {
            if !(1..=MAX_HISTORY_RETENTION_DAYS).contains(&config.privacy.history_retention_days) {
                return Err(ConfigError::PersistentHistoryRetentionOutOfRange {
                    found: config.privacy.history_retention_days,
                });
            }
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct VersionProbe {
    schema_version: Option<u32>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn parse_ok(document: &str) -> Config {
        parse(document).expect("document should be valid")
    }

    #[test]
    fn minimal_document_uses_documented_defaults() {
        let config = parse_ok("schema_version = 1\n");

        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(config.launcher.max_results, 8);
        assert_eq!(config.privacy.clipboard_history, ClipboardHistory::Disabled);
        assert_eq!(config.privacy.history_retention_days, 0);
    }

    #[test]
    fn default_config_passes_validation() {
        assert!(validate(&Config::default()).is_ok());
    }

    #[test]
    fn full_document_round_trips() {
        let document = "\
schema_version = 1

[launcher]
max_results = 12

[privacy]
clipboard_history = \"persistent\"
history_retention_days = 30
";

        let config = parse_ok(document);
        assert_eq!(config.launcher.max_results, 12);
        assert_eq!(
            config.privacy.clipboard_history,
            ClipboardHistory::Persistent
        );
        assert_eq!(config.privacy.history_retention_days, 30);

        let serialized = toml::to_string(&config).expect("config serializes");
        assert_eq!(parse_ok(&serialized), config);
    }

    #[test]
    fn empty_document_is_missing_version() {
        assert!(matches!(parse(""), Err(ConfigError::MissingSchemaVersion)));
    }

    #[test]
    fn unsupported_version_is_reported() {
        assert!(matches!(
            parse("schema_version = 2\n"),
            Err(ConfigError::UnsupportedSchemaVersion { found: 2 })
        ));
    }

    #[test]
    fn version_check_precedes_field_validation() {
        assert!(matches!(
            parse("schema_version = 2\nbogus = true\n"),
            Err(ConfigError::UnsupportedSchemaVersion { found: 2 })
        ));
    }

    #[test]
    fn malformed_document_is_reported() {
        assert!(matches!(
            parse("schema_version = [\n"),
            Err(ConfigError::Malformed(_))
        ));
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let error = parse("schema_version = 1\nbogus = true\n").expect_err("should be rejected");

        assert!(matches!(error, ConfigError::Malformed(_)));
        assert!(error.to_string().contains("bogus"), "message was: {error}");
    }

    #[test]
    fn unknown_section_fields_are_rejected() {
        assert!(matches!(
            parse("schema_version = 1\n[launcher]\nmax_results = 8\nbogus = 1\n"),
            Err(ConfigError::Malformed(_))
        ));
    }

    #[test]
    fn favorites_are_parsed_and_validated() {
        let config = parse_ok(
            "schema_version = 1\n[launcher]\nfavorites = [\"firefox-esr.desktop\", \"code.desktop\"]\n",
        );
        assert_eq!(
            config.launcher.favorites,
            vec!["firefox-esr.desktop", "code.desktop"]
        );

        assert!(matches!(
            parse("schema_version = 1\n[launcher]\nfavorites = [\"\"]\n"),
            Err(ConfigError::InvalidFavorite { .. })
        ));

        let too_many = (0..MAX_LAUNCHER_FAVORITES + 1)
            .map(|index| format!("app{index}.desktop"))
            .map(|id| format!("\"{id}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let document = format!("schema_version = 1\n[launcher]\nfavorites = [{too_many}]\n");
        assert!(matches!(
            parse(&document),
            Err(ConfigError::TooManyFavorites { .. })
        ));
    }

    #[test]
    fn launcher_max_results_bounds_are_enforced() {
        assert!(matches!(
            parse("schema_version = 1\n[launcher]\nmax_results = 0\n"),
            Err(ConfigError::LauncherMaxResultsOutOfRange { found: 0 })
        ));
        assert!(matches!(
            parse("schema_version = 1\n[launcher]\nmax_results = 51\n"),
            Err(ConfigError::LauncherMaxResultsOutOfRange { found: 51 })
        ));

        let config = parse_ok("schema_version = 1\n[launcher]\nmax_results = 50\n");
        assert_eq!(config.launcher.max_results, 50);
    }

    #[test]
    fn retention_requires_persistent_history() {
        assert!(matches!(
            parse("schema_version = 1\n[privacy]\nhistory_retention_days = 5\n"),
            Err(ConfigError::RetentionSetWithoutPersistentHistory { found: 5 })
        ));
        assert!(matches!(
            parse(
                "schema_version = 1\n[privacy]\nclipboard_history = \"session\"\nhistory_retention_days = 5\n"
            ),
            Err(ConfigError::RetentionSetWithoutPersistentHistory { found: 5 })
        ));
    }

    #[test]
    fn persistent_history_requires_bounded_retention() {
        assert!(matches!(
            parse("schema_version = 1\n[privacy]\nclipboard_history = \"persistent\"\n"),
            Err(ConfigError::PersistentHistoryRetentionOutOfRange { found: 0 })
        ));
        assert!(matches!(
            parse(
                "schema_version = 1\n[privacy]\nclipboard_history = \"persistent\"\nhistory_retention_days = 91\n"
            ),
            Err(ConfigError::PersistentHistoryRetentionOutOfRange { found: 91 })
        ));

        let config = parse_ok(
            "schema_version = 1\n[privacy]\nclipboard_history = \"persistent\"\nhistory_retention_days = 90\n",
        );
        assert_eq!(config.privacy.history_retention_days, 90);
    }

    #[test]
    fn oversized_document_is_rejected_before_parsing() {
        let document = "x".repeat(MAX_CONFIG_BYTES + 1);

        assert!(matches!(parse(&document), Err(ConfigError::TooLarge)));
    }

    #[test]
    fn invalid_clipboard_mode_is_malformed() {
        assert!(matches!(
            parse("schema_version = 1\n[privacy]\nclipboard_history = \"forever\"\n"),
            Err(ConfigError::Malformed(_))
        ));
    }
}
