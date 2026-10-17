//! Passive host diagnostics workflow.
//!
//! The application layer owns check policy and report shaping. Platform
//! adapters collect facts without mutating services, starting scans, acquiring
//! bus names or requesting authorization.
//!
//! Exit policy: a report fails only when a [`Severity::Required`] check has
//! [`Status::Fail`]. Warnings and unavailable facts are reported but do not
//! fail a run, so a TTY without an active session is not treated as a broken
//! host.

use std::path::PathBuf;

use serde::Serialize;

/// Version of the machine-readable doctor report contract.
pub const REPORT_SCHEMA_VERSION: u32 = 1;

/// Which readiness area a check belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Baseline host readiness, meaningful even from a TTY.
    Host,
    /// Active-session readiness.
    Session,
}

/// Result of one check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// The check passed.
    Pass,
    /// Something is unusual but not blocking.
    Warn,
    /// A required condition is broken.
    Fail,
    /// The fact could not be observed in this environment.
    Unavailable,
}

/// Whether a failure blocks readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// A failure fails the doctor run.
    Required,
    /// A failure is reported but does not fail the run.
    Optional,
}

/// One diagnostic check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    /// Stable machine-readable identifier.
    pub id: &'static str,
    /// Readiness area.
    pub scope: Scope,
    /// Whether a failure blocks readiness.
    pub severity: Severity,
    /// Observed result.
    pub status: Status,
    /// Human-readable observation.
    pub detail: String,
    /// Optional safe remediation hint.
    pub hint: Option<String>,
}

/// Collected diagnostic report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Checks in display order.
    pub checks: Vec<Check>,
}

impl Report {
    /// Number of checks with the given status.
    pub fn count(&self, status: Status) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == status)
            .count()
    }

    /// Whether any required check failed.
    pub fn has_required_failure(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.severity == Severity::Required && check.status == Status::Fail)
    }

    /// Status counts for display and machine-readable output.
    pub fn summary(&self) -> Summary {
        Summary {
            pass: self.count(Status::Pass),
            warn: self.count(Status::Warn),
            fail: self.count(Status::Fail),
            unavailable: self.count(Status::Unavailable),
        }
    }

    /// Versioned machine-readable envelope.
    pub fn envelope(&self) -> ReportEnvelope<'_> {
        ReportEnvelope {
            schema_version: REPORT_SCHEMA_VERSION,
            checks: &self.checks,
            summary: self.summary(),
        }
    }
}

/// Status counts in a report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// Checks that passed.
    pub pass: usize,
    /// Checks with warnings.
    pub warn: usize,
    /// Checks that failed.
    pub fail: usize,
    /// Checks that could not be observed.
    pub unavailable: usize,
}

/// Versioned machine-readable doctor report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportEnvelope<'a> {
    /// Schema version of this envelope.
    pub schema_version: u32,
    /// Checks in display order.
    pub checks: &'a [Check],
    /// Status counts.
    pub summary: Summary,
}

/// Facts collected passively from the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostFacts {
    /// Operating-system family, for example `linux`.
    pub os_family: &'static str,
    /// Target architecture, for example `x86_64`.
    pub os_arch: &'static str,
    /// `XDG_RUNTIME_DIR` observation.
    pub runtime_dir: RuntimeDirFacts,
    /// `WAYLAND_DISPLAY` observation.
    pub wayland: EndpointFacts,
    /// `NIRI_SOCKET` observation.
    pub niri_socket: EndpointFacts,
    /// Session-bus observation.
    pub session_bus: BusFacts,
    /// Program availability observations.
    pub programs: Vec<ProgramFacts>,
}

/// Observation of `XDG_RUNTIME_DIR`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDirFacts {
    /// Whether the variable is set and non-empty.
    pub configured: bool,
    /// Resolved absolute path, when the value is absolute.
    pub path: Option<PathBuf>,
    /// Whether the path exists.
    pub exists: bool,
    /// Whether the path is a directory.
    pub is_dir: bool,
    /// Whether the path is owned by the current user, when observable.
    pub owner_is_user: Option<bool>,
    /// Permission bits, when the path exists.
    pub mode: Option<u32>,
}

/// Observation of a session endpoint such as a Wayland or Niri socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointFacts {
    /// Whether the environment variable is set and non-empty.
    pub configured: bool,
    /// Resolved candidate path, when one can be derived.
    pub path: Option<PathBuf>,
    /// Whether the path exists.
    pub exists: bool,
    /// Whether the path is a Unix socket.
    pub is_socket: bool,
}

/// Observation of the session bus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusFacts {
    /// Whether `DBUS_SESSION_BUS_ADDRESS` is set.
    pub address_set: bool,
    /// Whether the default `$XDG_RUNTIME_DIR/bus` socket exists.
    pub default_socket_exists: bool,
}

/// Observation of one program's availability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramFacts {
    /// Stable machine-readable identifier.
    pub id: &'static str,
    /// Program name searched on `PATH`.
    pub name: &'static str,
    /// Whether a missing program is a required failure.
    pub required: bool,
    /// First executable match on `PATH`, if any.
    pub path: Option<PathBuf>,
}

/// Collects passive host facts.
pub trait DiagnosticsSource {
    /// Observe the current host without changing it.
    fn collect(&self) -> HostFacts;
}

/// Run passive diagnostics and produce a report.
pub fn run(source: &impl DiagnosticsSource) -> Report {
    let facts = source.collect();

    let mut checks = vec![make(
        "os",
        Scope::Host,
        Severity::Required,
        Status::Pass,
        format!("{} {}", facts.os_family, facts.os_arch),
        None,
    )];
    checks.push(runtime_dir_check(&facts.runtime_dir));
    checks.push(endpoint_check(
        "wayland-display",
        "WAYLAND_DISPLAY",
        &facts.wayland,
    ));
    checks.push(endpoint_check(
        "niri-socket",
        "NIRI_SOCKET",
        &facts.niri_socket,
    ));
    checks.push(session_bus_check(&facts.session_bus));
    for program in &facts.programs {
        checks.push(program_check(program));
    }

    Report { checks }
}

fn runtime_dir_check(facts: &RuntimeDirFacts) -> Check {
    if !facts.configured {
        return make(
            "runtime-dir",
            Scope::Host,
            Severity::Required,
            Status::Fail,
            "XDG_RUNTIME_DIR is not set",
            Some("Start a login session that provides XDG_RUNTIME_DIR."),
        );
    }

    let Some(path) = &facts.path else {
        return make(
            "runtime-dir",
            Scope::Host,
            Severity::Required,
            Status::Fail,
            "XDG_RUNTIME_DIR is not an absolute path",
            Some("Set XDG_RUNTIME_DIR to an absolute path."),
        );
    };

    if !facts.exists {
        return make(
            "runtime-dir",
            Scope::Host,
            Severity::Required,
            Status::Fail,
            format!("{} does not exist", path.display()),
            None,
        );
    }

    if !facts.is_dir {
        return make(
            "runtime-dir",
            Scope::Host,
            Severity::Required,
            Status::Fail,
            format!("{} is not a directory", path.display()),
            None,
        );
    }

    if facts.owner_is_user == Some(false) {
        return make(
            "runtime-dir",
            Scope::Host,
            Severity::Required,
            Status::Fail,
            format!("{} is not owned by the current user", path.display()),
            Some("Check XDG_RUNTIME_DIR; do not point it at another user's directory."),
        );
    }

    if let Some(mode) = facts.mode {
        if mode != 0o700 {
            return make(
                "runtime-dir",
                Scope::Host,
                Severity::Required,
                Status::Warn,
                format!("{} has mode 0o{mode:03o}; expected 0o700", path.display()),
                Some("Restrict the runtime directory to the current user."),
            );
        }
    }

    make(
        "runtime-dir",
        Scope::Host,
        Severity::Required,
        Status::Pass,
        format!("{} (mode 0700)", path.display()),
        None,
    )
}

fn endpoint_check(id: &'static str, variable: &'static str, facts: &EndpointFacts) -> Check {
    if !facts.configured {
        return make(
            id,
            Scope::Session,
            Severity::Required,
            Status::Unavailable,
            format!("{variable} is not set (no active session?)"),
            None,
        );
    }

    let Some(path) = &facts.path else {
        return make(
            id,
            Scope::Session,
            Severity::Required,
            Status::Fail,
            format!("{variable} could not be resolved to an absolute path"),
            Some("Check the session environment."),
        );
    };

    if !facts.exists {
        return make(
            id,
            Scope::Session,
            Severity::Required,
            Status::Fail,
            format!("{variable} points to {}, which does not exist", path.display()),
            Some("The session may have ended; log in again."),
        );
    }

    if !facts.is_socket {
        return make(
            id,
            Scope::Session,
            Severity::Required,
            Status::Fail,
            format!("{} is not a Unix socket", path.display()),
            None,
        );
    }

    make(
        id,
        Scope::Session,
        Severity::Required,
        Status::Pass,
        path.display().to_string(),
        None,
    )
}

fn session_bus_check(facts: &BusFacts) -> Check {
    if facts.address_set || facts.default_socket_exists {
        return make(
            "session-bus",
            Scope::Session,
            Severity::Optional,
            Status::Pass,
            "session bus is configured",
            None,
        );
    }

    make(
        "session-bus",
        Scope::Session,
        Severity::Optional,
        Status::Unavailable,
        "no session bus detected",
        Some("Start a login session or check DBUS_SESSION_BUS_ADDRESS."),
    )
}

fn program_check(facts: &ProgramFacts) -> Check {
    let severity = if facts.required {
        Severity::Required
    } else {
        Severity::Optional
    };

    match &facts.path {
        Some(path) => make(
            facts.id,
            Scope::Host,
            severity,
            Status::Pass,
            path.display().to_string(),
            None,
        ),
        None if facts.required => make(
            facts.id,
            Scope::Host,
            severity,
            Status::Fail,
            format!("{} was not found on PATH", facts.name),
            Some("Install the program or fix PATH."),
        ),
        None => make(
            facts.id,
            Scope::Host,
            severity,
            Status::Warn,
            format!("{} was not found on PATH", facts.name),
            Some("Optional program; install it for full functionality."),
        ),
    }
}

fn make(
    id: &'static str,
    scope: Scope,
    severity: Severity,
    status: Status,
    detail: impl Into<String>,
    hint: Option<&str>,
) -> Check {
    Check {
        id,
        scope,
        severity,
        status,
        detail: detail.into(),
        hint: hint.map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    struct Fake(HostFacts);

    impl DiagnosticsSource for Fake {
        fn collect(&self) -> HostFacts {
            self.0.clone()
        }
    }

    fn healthy() -> HostFacts {
        HostFacts {
            os_family: "linux",
            os_arch: "x86_64",
            runtime_dir: RuntimeDirFacts {
                configured: true,
                path: Some(PathBuf::from("/run/user/1000")),
                exists: true,
                is_dir: true,
                owner_is_user: Some(true),
                mode: Some(0o700),
            },
            wayland: EndpointFacts {
                configured: true,
                path: Some(PathBuf::from("/run/user/1000/wayland-1")),
                exists: true,
                is_socket: true,
            },
            niri_socket: EndpointFacts {
                configured: true,
                path: Some(PathBuf::from("/run/user/1000/niri.sock")),
                exists: true,
                is_socket: true,
            },
            session_bus: BusFacts {
                address_set: true,
                default_socket_exists: false,
            },
            programs: vec![
                ProgramFacts {
                    id: "program.niri",
                    name: "niri",
                    required: true,
                    path: Some(PathBuf::from("/usr/bin/niri")),
                },
                ProgramFacts {
                    id: "program.xwayland-satellite",
                    name: "xwayland-satellite",
                    required: false,
                    path: Some(PathBuf::from("/usr/bin/xwayland-satellite")),
                },
            ],
        }
    }

    fn report(facts: HostFacts) -> Report {
        run(&Fake(facts))
    }

    fn check<'a>(report: &'a Report, id: &str) -> &'a Check {
        report
            .checks
            .iter()
            .find(|check| check.id == id)
            .expect("check present")
    }

    #[test]
    fn healthy_host_passes_without_required_failure() {
        let report = report(healthy());

        assert!(!report.has_required_failure());
        assert_eq!(report.count(Status::Fail), 0);
        assert_eq!(check(&report, "os").status, Status::Pass);
        assert_eq!(check(&report, "runtime-dir").status, Status::Pass);
        assert_eq!(check(&report, "niri-socket").status, Status::Pass);
    }

    #[test]
    fn missing_runtime_dir_fails() {
        let mut facts = healthy();
        facts.runtime_dir = RuntimeDirFacts {
            configured: false,
            path: None,
            exists: false,
            is_dir: false,
            owner_is_user: None,
            mode: None,
        };

        let report = report(facts);

        assert!(report.has_required_failure());
        assert_eq!(check(&report, "runtime-dir").status, Status::Fail);
    }

    #[test]
    fn relative_runtime_dir_fails() {
        let mut facts = healthy();
        facts.runtime_dir.path = None;

        let report = report(facts);

        assert!(report.has_required_failure());
        assert_eq!(check(&report, "runtime-dir").status, Status::Fail);
    }

    #[test]
    fn unusual_runtime_mode_warns_without_failing() {
        let mut facts = healthy();
        facts.runtime_dir.mode = Some(0o755);

        let report = report(facts);

        assert!(!report.has_required_failure());
        assert_eq!(check(&report, "runtime-dir").status, Status::Warn);
    }

    #[test]
    fn unset_wayland_is_unavailable_not_a_failure() {
        let mut facts = healthy();
        facts.wayland = EndpointFacts {
            configured: false,
            path: None,
            exists: false,
            is_socket: false,
        };

        let report = report(facts);

        assert!(!report.has_required_failure());
        assert_eq!(check(&report, "wayland-display").status, Status::Unavailable);
    }

    #[test]
    fn configured_missing_niri_socket_fails() {
        let mut facts = healthy();
        facts.niri_socket.exists = false;

        let report = report(facts);

        assert!(report.has_required_failure());
        assert_eq!(check(&report, "niri-socket").status, Status::Fail);
    }

    #[test]
    fn absent_session_bus_is_unavailable() {
        let mut facts = healthy();
        facts.session_bus = BusFacts {
            address_set: false,
            default_socket_exists: false,
        };

        let report = report(facts);

        assert!(!report.has_required_failure());
        assert_eq!(check(&report, "session-bus").status, Status::Unavailable);
    }

    #[test]
    fn summary_and_envelope_are_consistent() {
        let report = report(healthy());
        let summary = report.summary();

        assert_eq!(summary.pass, report.count(Status::Pass));
        assert_eq!(summary.warn, report.count(Status::Warn));
        assert_eq!(summary.fail, report.count(Status::Fail));
        assert_eq!(summary.unavailable, report.count(Status::Unavailable));

        let envelope = report.envelope();
        assert_eq!(envelope.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(envelope.checks.len(), report.checks.len());
    }

    #[test]
    fn missing_required_program_fails() {
        let mut facts = healthy();
        facts.programs[0].path = None;

        let report = report(facts);

        assert!(report.has_required_failure());
        assert_eq!(check(&report, "program.niri").status, Status::Fail);
    }

    #[test]
    fn missing_optional_program_warns() {
        let mut facts = healthy();
        facts.programs[1].path = None;

        let report = report(facts);

        assert!(!report.has_required_failure());
        assert_eq!(
            check(&report, "program.xwayland-satellite").status,
            Status::Warn
        );
    }
}
