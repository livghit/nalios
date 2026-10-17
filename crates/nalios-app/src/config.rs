//! Configuration checking workflow.
//!
//! The workflow reads a bounded document through a port, then parses and
//! validates it with the pure core rules. No filesystem or environment access
//! happens in this layer.

use std::path::Path;

use nalios_core::config::{self, Config, ConfigError};

/// A failure while reading a configuration document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ReadError {
    /// No file exists at the requested path.
    #[error("configuration file was not found")]
    NotFound,
    /// The file exists but cannot be read with the current permissions.
    #[error("configuration file could not be read due to permissions")]
    PermissionDenied,
    /// The file exceeds the accepted configuration size limit.
    #[error("configuration file exceeds the accepted size limit")]
    TooLarge,
    /// The file is not valid UTF-8 text.
    #[error("configuration file is not valid UTF-8 text")]
    NotUtf8,
    /// The file could not be read for another reason.
    #[error("configuration file could not be read")]
    Unreadable,
}

/// Reads configuration documents for the application layer.
pub trait ConfigSource {
    /// Read a bounded configuration document from `path`.
    fn read(&self, path: &Path) -> Result<String, ReadError>;
}

/// A configuration check failure.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    /// The document could not be read.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The document could not be parsed or validated.
    #[error(transparent)]
    Config(#[from] ConfigError),
}

/// Read, parse and validate the configuration document at `path`.
pub fn check(source: &impl ConfigSource, path: &Path) -> Result<Config, CheckError> {
    let document = source.read(path).map_err(CheckError::Read)?;
    config::parse(&document).map_err(CheckError::Config)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;

    use super::*;

    struct FakeSource {
        document: Result<String, ReadError>,
    }

    impl ConfigSource for FakeSource {
        fn read(&self, _path: &Path) -> Result<String, ReadError> {
            self.document.clone()
        }
    }

    fn fake(document: Result<&str, ReadError>) -> FakeSource {
        FakeSource {
            document: document.map(str::to_owned),
        }
    }

    #[test]
    fn valid_document_returns_config() {
        let source = fake(Ok("schema_version = 1\n"));

        let config = check(&source, &PathBuf::from("config.toml")).expect("valid document");

        assert_eq!(config.schema_version, 1);
    }

    #[test]
    fn invalid_document_returns_config_error() {
        let source = fake(Ok("schema_version = 2\n"));

        let error = check(&source, &PathBuf::from("config.toml")).expect_err("invalid document");

        assert!(matches!(
            error,
            CheckError::Config(ConfigError::UnsupportedSchemaVersion { found: 2 })
        ));
    }

    #[test]
    fn read_failure_is_propagated() {
        let source = fake(Err(ReadError::NotFound));

        let error = check(&source, &PathBuf::from("missing.toml")).expect_err("unreadable");

        assert!(matches!(error, CheckError::Read(ReadError::NotFound)));
    }
}
