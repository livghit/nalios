//! Passive host facts for diagnostics.
//!
//! Collection reads environment variables and filesystem metadata only. It
//! never connects to services, starts scans, executes programs, acquires bus
//! names or requests authorization.

use std::ffi::OsStr;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use nalios_app::diagnostics::{
    BusFacts, DiagnosticsSource, EndpointFacts, HostFacts, ProgramFacts, RuntimeDirFacts,
};

/// Collects passive host facts from the current process environment.
#[derive(Debug, Clone, Copy, Default)]
pub struct HostDiagnostics;

impl DiagnosticsSource for HostDiagnostics {
    fn collect(&self) -> HostFacts {
        let runtime = env_value("XDG_RUNTIME_DIR");
        let runtime_path = runtime.as_deref();
        let path_var = std::env::var_os("PATH");

        HostFacts {
            os_family: std::env::consts::OS,
            os_arch: std::env::consts::ARCH,
            runtime_dir: inspect_runtime_dir(runtime_path),
            wayland: inspect_endpoint(env_value("WAYLAND_DISPLAY").as_deref(), runtime_path),
            niri_socket: inspect_endpoint(env_value("NIRI_SOCKET").as_deref(), runtime_path),
            session_bus: inspect_session_bus(
                env_value("DBUS_SESSION_BUS_ADDRESS").is_some(),
                runtime_path,
            ),
            programs: vec![
                inspect_program("program.niri", "niri", true, path_var.as_deref()),
                inspect_program(
                    "program.xwayland-satellite",
                    "xwayland-satellite",
                    false,
                    path_var.as_deref(),
                ),
            ],
        }
    }
}

fn env_value(name: &str) -> Option<String> {
    std::env::var_os(name)
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
}

pub(crate) fn inspect_runtime_dir(value: Option<&str>) -> RuntimeDirFacts {
    let Some(text) = value else {
        return RuntimeDirFacts {
            configured: false,
            path: None,
            exists: false,
            is_dir: false,
            owner_is_user: None,
            mode: None,
        };
    };

    let path = Path::new(text);
    if !path.is_absolute() {
        return RuntimeDirFacts {
            configured: true,
            path: None,
            exists: false,
            is_dir: false,
            owner_is_user: None,
            mode: None,
        };
    }

    let path = path.to_path_buf();
    match std::fs::metadata(&path) {
        Ok(metadata) => RuntimeDirFacts {
            configured: true,
            path: Some(path),
            exists: true,
            is_dir: metadata.is_dir(),
            owner_is_user: current_uid().map(|uid| metadata.uid() == uid),
            mode: Some(metadata.permissions().mode() & 0o777),
        },
        Err(_) => RuntimeDirFacts {
            configured: true,
            path: Some(path),
            exists: false,
            is_dir: false,
            owner_is_user: None,
            mode: None,
        },
    }
}

pub(crate) fn inspect_endpoint(
    value: Option<&str>,
    runtime_dir: Option<&str>,
) -> EndpointFacts {
    let Some(text) = value else {
        return EndpointFacts {
            configured: false,
            path: None,
            exists: false,
            is_socket: false,
        };
    };

    let candidate = Path::new(text);
    let path = if candidate.is_absolute() {
        Some(candidate.to_path_buf())
    } else {
        runtime_dir.and_then(|runtime| {
            let runtime = Path::new(runtime);
            if runtime.is_absolute() {
                Some(runtime.join(candidate))
            } else {
                None
            }
        })
    };

    let Some(path) = path else {
        return EndpointFacts {
            configured: true,
            path: None,
            exists: false,
            is_socket: false,
        };
    };

    match std::fs::symlink_metadata(&path) {
        Ok(metadata) => EndpointFacts {
            configured: true,
            path: Some(path),
            exists: true,
            is_socket: metadata.file_type().is_socket(),
        },
        Err(_) => EndpointFacts {
            configured: true,
            path: Some(path),
            exists: false,
            is_socket: false,
        },
    }
}

pub(crate) fn inspect_session_bus(address_set: bool, runtime_dir: Option<&str>) -> BusFacts {
    let default_socket_exists = runtime_dir
        .map(Path::new)
        .filter(|runtime| runtime.is_absolute())
        .map(|runtime| runtime.join("bus"))
        .map(|bus| {
            std::fs::symlink_metadata(&bus)
                .map(|metadata| metadata.file_type().is_socket())
                .unwrap_or(false)
        })
        .unwrap_or(false);

    BusFacts {
        address_set,
        default_socket_exists,
    }
}

pub(crate) fn inspect_program(
    id: &'static str,
    name: &'static str,
    required: bool,
    path_var: Option<&OsStr>,
) -> ProgramFacts {
    ProgramFacts {
        id,
        name,
        required,
        path: find_program(name, path_var),
    }
}

pub(crate) fn find_program(name: &str, path_var: Option<&OsStr>) -> Option<PathBuf> {
    let path_var = path_var?;

    for directory in std::env::split_paths(path_var) {
        if directory.as_os_str().is_empty() {
            continue;
        }
        let candidate = directory.join(name);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn current_uid() -> Option<u32> {
    std::fs::metadata("/proc/self")
        .ok()
        .map(|metadata| metadata.uid())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::fs::Permissions;
    use std::os::unix::net::UnixListener;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn unset_runtime_dir_is_not_configured() {
        let facts = inspect_runtime_dir(None);

        assert!(!facts.configured);
        assert!(facts.path.is_none());
    }

    #[test]
    fn relative_runtime_dir_has_no_path() {
        let facts = inspect_runtime_dir(Some("run/user/1000"));

        assert!(facts.configured);
        assert!(facts.path.is_none());
    }

    #[test]
    fn runtime_dir_metadata_is_observed() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::set_permissions(dir.path(), Permissions::from_mode(0o700)).expect("set mode");

        let facts = inspect_runtime_dir(Some(dir.path().to_str().expect("utf-8 path")));

        assert!(facts.configured && facts.exists && facts.is_dir);
        assert_eq!(facts.mode, Some(0o700));
        assert_eq!(facts.owner_is_user, Some(true));
    }

    #[test]
    fn endpoint_socket_is_observed() {
        let dir = TempDir::new().expect("temp dir");
        let socket = dir.path().join("test.sock");
        let _listener = UnixListener::bind(&socket).expect("bind");

        let facts = inspect_endpoint(Some(socket.to_str().expect("utf-8 path")), None);

        assert!(facts.configured && facts.exists && facts.is_socket);
    }

    #[test]
    fn relative_endpoint_uses_absolute_runtime_dir() {
        let dir = TempDir::new().expect("temp dir");
        let runtime = dir.path().to_str().expect("utf-8 path");

        let facts = inspect_endpoint(Some("wayland-1"), Some(runtime));

        assert_eq!(facts.path, Some(dir.path().join("wayland-1")));
        assert!(!facts.exists);
    }

    #[test]
    fn relative_endpoint_with_relative_runtime_has_no_path() {
        let facts = inspect_endpoint(Some("wayland-1"), Some("relative/runtime"));

        assert!(facts.configured);
        assert!(facts.path.is_none());
    }

    #[test]
    fn session_bus_default_socket_is_detected() {
        let dir = TempDir::new().expect("temp dir");
        let bus = dir.path().join("bus");
        let _listener = UnixListener::bind(&bus).expect("bind");
        let runtime = dir.path().to_str().expect("utf-8 path");

        let facts = inspect_session_bus(false, Some(runtime));

        assert!(facts.default_socket_exists);
    }

    #[test]
    fn finds_executable_programs_on_path() {
        let dir = TempDir::new().expect("temp dir");
        let program = dir.path().join("nalios-test-tool");
        std::fs::write(&program, b"#!/bin/sh\n").expect("write");
        std::fs::set_permissions(&program, Permissions::from_mode(0o755)).expect("chmod");
        let path_var = dir.path().as_os_str();

        assert_eq!(find_program("nalios-test-tool", Some(path_var)), Some(program));
    }

    #[test]
    fn ignores_non_executable_and_missing_programs() {
        let dir = TempDir::new().expect("temp dir");
        let file = dir.path().join("not-executable");
        std::fs::write(&file, b"x").expect("write");
        std::fs::set_permissions(&file, Permissions::from_mode(0o644)).expect("chmod");
        let path_var = dir.path().as_os_str();

        assert_eq!(find_program("not-executable", Some(path_var)), None);
        assert_eq!(find_program("missing-program", Some(path_var)), None);
    }

    #[test]
    fn collect_reports_the_host_platform() {
        let facts = HostDiagnostics.collect();

        assert_eq!(facts.os_family, std::env::consts::OS);
        assert_eq!(facts.os_arch, std::env::consts::ARCH);
        assert_eq!(facts.programs.len(), 2);
    }
}
