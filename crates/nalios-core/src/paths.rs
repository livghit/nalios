//! Pure namespace path resolution.
//!
//! Callers supply XDG environment values and a home directory; this module
//! validates them and derives the config, state, cache and runtime directories
//! for one NaliOS namespace. It performs no environment or filesystem access
//! and does not create directories.

use std::path::{Path, PathBuf};

/// Development namespace used until deliberate installation.
pub const DEVELOPMENT_NAMESPACE: &str = "nalios-dev";

/// Production namespace planned for packaged releases.
pub const PRODUCTION_NAMESPACE: &str = "nalios";

/// Explicit inputs used to resolve XDG-style directories.
///
/// Values are supplied by the caller rather than read from the process
/// environment so tests can use fake paths and production code controls when
/// environment access happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct XdgInputs<'a> {
    /// Absolute home directory, used for XDG defaults.
    pub home: Option<&'a str>,
    /// Absolute `XDG_CONFIG_HOME`, when set.
    pub config_home: Option<&'a str>,
    /// Absolute `XDG_STATE_HOME`, when set.
    pub state_home: Option<&'a str>,
    /// Absolute `XDG_CACHE_HOME`, when set.
    pub cache_home: Option<&'a str>,
    /// Absolute `XDG_RUNTIME_DIR`; required, with no fallback.
    pub runtime_dir: Option<&'a str>,
}

/// Directories belonging to one NaliOS namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespacePaths {
    /// User configuration directory.
    pub config: PathBuf,
    /// Persistent state directory; never credentials.
    pub state: PathBuf,
    /// Regenerable cache directory.
    pub cache: PathBuf,
    /// Namespace directory below `XDG_RUNTIME_DIR`.
    pub runtime_base: PathBuf,
}

/// A path-resolution failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathError {
    /// The namespace was empty or not a simple path component.
    #[error("namespace must be a single path component")]
    InvalidNamespace,
    /// A default path was required but no home directory was supplied.
    #[error("a home directory is required for default paths")]
    MissingHome,
    /// The supplied home directory was not absolute.
    #[error("the home directory must be an absolute path")]
    RelativeHome,
    /// An XDG variable was set to a relative path.
    #[error("{variable} must be an absolute path")]
    RelativeXdg {
        /// Name of the offending variable.
        variable: &'static str,
    },
    /// `XDG_RUNTIME_DIR` was unset or empty.
    #[error("XDG_RUNTIME_DIR is not set")]
    MissingRuntimeDir,
    /// `XDG_RUNTIME_DIR` was set to a relative path.
    #[error("XDG_RUNTIME_DIR must be an absolute path")]
    RelativeRuntimeDir,
    /// The session identifier was empty or not a simple path component.
    #[error("session identifier must be a single path component")]
    InvalidSessionId,
}

impl NamespacePaths {
    /// Resolve the runtime directory for one validated session identifier.
    ///
    /// The identifier must be a single path component: non-empty, at most 64
    /// ASCII alphanumeric, `-` or `_` characters.
    pub fn session_runtime(&self, session_id: &str) -> Result<PathBuf, PathError> {
        if !is_component(session_id) {
            return Err(PathError::InvalidSessionId);
        }
        Ok(self.runtime_base.join(session_id))
    }
}

/// Resolve the namespace directories from explicit inputs.
///
/// Empty XDG values are treated as unset, matching the XDG basedir
/// specification. Relative XDG values are rejected instead of silently
/// falling back, so callers can report a real environment problem.
pub fn resolve(inputs: &XdgInputs<'_>, namespace: &str) -> Result<NamespacePaths, PathError> {
    if !is_component(namespace) {
        return Err(PathError::InvalidNamespace);
    }

    let config_base = resolve_dir(inputs.config_home, "XDG_CONFIG_HOME", || {
        Ok(home_dir(inputs)?.join(".config"))
    })?;
    let state_base = resolve_dir(inputs.state_home, "XDG_STATE_HOME", || {
        Ok(home_dir(inputs)?.join(".local/state"))
    })?;
    let cache_base = resolve_dir(inputs.cache_home, "XDG_CACHE_HOME", || {
        Ok(home_dir(inputs)?.join(".cache"))
    })?;

    let runtime_base = match inputs.runtime_dir.filter(|value| !value.is_empty()) {
        Some(value) => {
            let path = Path::new(value);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                return Err(PathError::RelativeRuntimeDir);
            }
        }
        None => return Err(PathError::MissingRuntimeDir),
    };

    Ok(NamespacePaths {
        config: config_base.join(namespace),
        state: state_base.join(namespace),
        cache: cache_base.join(namespace),
        runtime_base: runtime_base.join(namespace),
    })
}

fn resolve_dir(
    value: Option<&str>,
    variable: &'static str,
    default: impl FnOnce() -> Result<PathBuf, PathError>,
) -> Result<PathBuf, PathError> {
    match value.filter(|value| !value.is_empty()) {
        Some(text) => {
            let path = Path::new(text);
            if path.is_absolute() {
                Ok(path.to_path_buf())
            } else {
                Err(PathError::RelativeXdg { variable })
            }
        }
        None => default(),
    }
}

fn home_dir<'a>(inputs: &XdgInputs<'a>) -> Result<&'a Path, PathError> {
    let value = inputs
        .home
        .filter(|value| !value.is_empty())
        .ok_or(PathError::MissingHome)?;
    let path = Path::new(value);
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(PathError::RelativeHome)
    }
}

fn is_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    const HOME: &str = "/home/tester";
    const RUNTIME: &str = "/run/user/1000";

    fn resolve_with(
        home: Option<&str>,
        config: Option<&str>,
        state: Option<&str>,
        cache: Option<&str>,
        runtime: Option<&str>,
    ) -> Result<NamespacePaths, PathError> {
        resolve(
            &XdgInputs {
                home,
                config_home: config,
                state_home: state,
                cache_home: cache,
                runtime_dir: runtime,
            },
            DEVELOPMENT_NAMESPACE,
        )
    }

    #[test]
    fn defaults_come_from_home_and_runtime_dir() {
        let paths = resolve_with(Some(HOME), None, None, None, Some(RUNTIME)).expect("resolves");

        assert_eq!(
            paths.config,
            PathBuf::from("/home/tester/.config/nalios-dev")
        );
        assert_eq!(
            paths.state,
            PathBuf::from("/home/tester/.local/state/nalios-dev")
        );
        assert_eq!(paths.cache, PathBuf::from("/home/tester/.cache/nalios-dev"));
        assert_eq!(
            paths.runtime_base,
            PathBuf::from("/run/user/1000/nalios-dev")
        );
    }

    #[test]
    fn absolute_overrides_are_used() {
        let paths = resolve_with(
            Some(HOME),
            Some("/custom/config"),
            Some("/custom/state"),
            Some("/custom/cache"),
            Some("/custom/run"),
        )
        .expect("resolves");

        assert_eq!(paths.config, PathBuf::from("/custom/config/nalios-dev"));
        assert_eq!(paths.state, PathBuf::from("/custom/state/nalios-dev"));
        assert_eq!(paths.cache, PathBuf::from("/custom/cache/nalios-dev"));
        assert_eq!(paths.runtime_base, PathBuf::from("/custom/run/nalios-dev"));
    }

    #[test]
    fn empty_values_are_treated_as_unset() {
        let paths = resolve_with(Some(HOME), Some(""), Some(""), Some(""), Some(RUNTIME))
            .expect("resolves");

        assert_eq!(
            paths.config,
            PathBuf::from("/home/tester/.config/nalios-dev")
        );
    }

    #[test]
    fn relative_config_path_is_rejected() {
        assert_eq!(
            resolve_with(Some(HOME), Some("relative"), None, None, Some(RUNTIME)),
            Err(PathError::RelativeXdg {
                variable: "XDG_CONFIG_HOME"
            })
        );
    }

    #[test]
    fn relative_state_and_cache_paths_are_rejected() {
        assert_eq!(
            resolve_with(Some(HOME), None, Some("state"), None, Some(RUNTIME)),
            Err(PathError::RelativeXdg {
                variable: "XDG_STATE_HOME"
            })
        );
        assert_eq!(
            resolve_with(Some(HOME), None, None, Some("cache"), Some(RUNTIME)),
            Err(PathError::RelativeXdg {
                variable: "XDG_CACHE_HOME"
            })
        );
    }

    #[test]
    fn missing_home_is_an_error() {
        assert_eq!(
            resolve_with(None, None, None, None, Some(RUNTIME)),
            Err(PathError::MissingHome)
        );
    }

    #[test]
    fn relative_home_is_rejected() {
        assert_eq!(
            resolve_with(Some("home"), None, None, None, Some(RUNTIME)),
            Err(PathError::RelativeHome)
        );
    }

    #[test]
    fn runtime_dir_is_required() {
        assert_eq!(
            resolve_with(Some(HOME), None, None, None, None),
            Err(PathError::MissingRuntimeDir)
        );
        assert_eq!(
            resolve_with(Some(HOME), None, None, None, Some("")),
            Err(PathError::MissingRuntimeDir)
        );
    }

    #[test]
    fn relative_runtime_dir_is_rejected() {
        assert_eq!(
            resolve_with(Some(HOME), None, None, None, Some("run")),
            Err(PathError::RelativeRuntimeDir)
        );
    }

    #[test]
    fn invalid_namespaces_are_rejected() {
        for namespace in ["", ".", "..", "with space", "a/b", "a\\b"] {
            assert_eq!(
                resolve(
                    &XdgInputs {
                        home: Some(HOME),
                        runtime_dir: Some(RUNTIME),
                        ..XdgInputs::default()
                    },
                    namespace,
                ),
                Err(PathError::InvalidNamespace),
                "namespace {namespace:?} should be rejected"
            );
        }
    }

    #[test]
    fn session_runtime_validates_the_identifier() {
        let paths = resolve_with(Some(HOME), None, None, None, Some(RUNTIME)).expect("resolves");

        assert_eq!(
            paths.session_runtime("abc-123").expect("valid session"),
            PathBuf::from("/run/user/1000/nalios-dev/abc-123")
        );
        for session in ["", "..", "a/b", "with space"] {
            assert_eq!(
                paths.session_runtime(session),
                Err(PathError::InvalidSessionId),
                "session {session:?} should be rejected"
            );
        }
    }
}
