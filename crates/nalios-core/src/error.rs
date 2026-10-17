//! Crate-level error aggregation.
//!
//! Module-specific errors remain the primary API; this enum lets callers that
//! handle several core operations carry one type.

use crate::config::ConfigError;
use crate::paths::PathError;

/// An error returned by a NaliOS core operation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Namespace path resolution failed.
    #[error(transparent)]
    Path(#[from] PathError),
    /// Configuration parsing or validation failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_module_errors() {
        let error = Error::from(PathError::MissingHome);

        assert!(matches!(error, Error::Path(PathError::MissingHome)));
    }
}
