//! NaliOS development command-line frontend.
//!
//! This crate owns argument parsing and terminal presentation for the
//! `nalios-dev` development binary. Implemented commands are `--help`,
//! `--version` and `config check`; diagnostics and other commands arrive with
//! later implementation tickets.

#![warn(missing_docs)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use nalios_app::control::{ControlAction, ControlRequest, ControlResponse};
use nalios_app::diagnostics::{self, Scope, Status};
use nalios_app::niri::{NiriEvent, NiriObservation};
use nalios_platform::diagnostics::HostDiagnostics;
use nalios_platform::files::FsConfigSource;
use nalios_platform::niri::{self, NiriClient, NiriEventStream};

/// Successful completion.
pub const EXIT_SUCCESS: u8 = 0;

/// A blocking check failed, such as invalid configuration.
pub const EXIT_FAILURE: u8 = 1;

/// The command-line invocation was invalid.
pub const EXIT_USAGE: u8 = 2;

/// An internal failure prevented a trustworthy result.
pub const EXIT_INTERNAL: u8 = 3;

const BIN_NAME: &str = "nalios-dev";

const HELP: &str = "\
NaliOS development CLI

Usage: nalios-dev [OPTIONS]
       nalios-dev config check --file <path>
       nalios-dev doctor
       nalios-dev niri inspect [--socket <path>]
       nalios-dev niri watch [--socket <path>]
       nalios-dev shell <ping|open-launcher|toggle-launcher|toggle-dashboard>
       nalios-dev apps <list|search <query>>

Options:
  -h, --help     Print this help and exit
  -V, --version  Print version information and exit

Commands:
  config check   Validate a configuration document
  doctor         Report passive host and session readiness
  niri inspect   Print a read-only Niri state snapshot
  niri watch     Stream modeled Niri state changes
  shell          Send an allowlisted action to a running shell
  apps           List or search discovered desktop applications

Only help, version, config check, doctor, niri inspect/watch, shell and apps
are implemented. Other commands arrive with later implementation tickets.
";

/// A parsed command-line invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Print usage information.
    Help,
    /// Print the binary name and version.
    Version,
    /// Validate the configuration document at the given path.
    ConfigCheck {
        /// Document to validate.
        path: PathBuf,
    },
    /// Report passive host and session readiness.
    Doctor {
        /// Emit the versioned machine-readable report.
        json: bool,
    },
    /// Print a read-only Niri state snapshot.
    NiriInspect {
        /// Socket path; discovered from `NIRI_SOCKET` when absent.
        socket: Option<PathBuf>,
    },
    /// Stream modeled Niri state changes.
    NiriWatch {
        /// Socket path; discovered from `NIRI_SOCKET` when absent.
        socket: Option<PathBuf>,
    },
    /// Send an allowlisted action to a running shell.
    Shell {
        /// Action to send.
        action: ControlAction,
    },
    /// List discovered desktop applications.
    AppsList,
    /// Search discovered desktop applications.
    AppsSearch {
        /// Query string.
        query: String,
    },
}

/// A command-line parsing failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// A command was not recognized.
    UnknownCommand(String),
    /// An option was not recognized.
    UnknownOption(String),
    /// A command that requires a subcommand was given without one.
    MissingSubcommand(&'static str),
    /// A required option was not supplied.
    MissingOption {
        /// Name of the missing option.
        option: &'static str,
    },
    /// An option was supplied without its value.
    MissingValue {
        /// Name of the option.
        option: &'static str,
    },
    /// An option was supplied more than once.
    DuplicateOption {
        /// Name of the repeated option.
        option: &'static str,
    },
    /// An unexpected extra argument was supplied.
    UnexpectedArgument(String),
}

impl ParseError {
    fn render(&self) -> String {
        let message = match self {
            Self::UnknownCommand(command) => format!("unrecognized command '{command}'"),
            Self::UnknownOption(option) => format!("unrecognized option '{option}'"),
            Self::MissingSubcommand(command) => format!("'{command}' requires a subcommand"),
            Self::MissingOption { option } => format!("missing required option '{option}'"),
            Self::MissingValue { option } => format!("option '{option}' requires a value"),
            Self::DuplicateOption { option } => {
                format!("option '{option}' was given more than once")
            }
            Self::UnexpectedArgument(argument) => format!("unexpected argument '{argument}'"),
        };

        format!("error: {message}\nTry '{BIN_NAME} --help' for more information.")
    }
}

/// Parse the arguments following the executable name.
///
/// With no arguments this returns [`Command::Help`].
pub fn parse(args: &[String]) -> Result<Command, ParseError> {
    let Some(first) = args.first() else {
        return Ok(Command::Help);
    };

    match first.as_str() {
        "-h" | "--help" => Ok(Command::Help),
        "-V" | "--version" => Ok(Command::Version),
        "config" => parse_config(&args[1..]),
        "doctor" => parse_doctor(&args[1..]),
        "niri" => parse_niri(&args[1..]),
        "shell" => parse_shell(&args[1..]),
        "apps" => parse_apps(&args[1..]),
        other if other.starts_with('-') => Err(ParseError::UnknownOption(other.to_owned())),
        other => Err(ParseError::UnknownCommand(other.to_owned())),
    }
}

fn parse_config(args: &[String]) -> Result<Command, ParseError> {
    match args.first().map(String::as_str) {
        Some("check") => parse_config_check(&args[1..]),
        Some(other) => Err(ParseError::UnknownCommand(format!("config {other}"))),
        None => Err(ParseError::MissingSubcommand("config")),
    }
}

fn parse_config_check(args: &[String]) -> Result<Command, ParseError> {
    let mut path: Option<PathBuf> = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--file" | "-f" => {
                if path.is_some() {
                    return Err(ParseError::DuplicateOption { option: "--file" });
                }
                let value = args
                    .get(index + 1)
                    .ok_or(ParseError::MissingValue { option: "--file" })?;
                path = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(ParseError::UnexpectedArgument(other.to_owned())),
        }
    }

    let path = path.ok_or(ParseError::MissingOption { option: "--file" })?;
    Ok(Command::ConfigCheck { path })
}

fn parse_doctor(args: &[String]) -> Result<Command, ParseError> {
    let mut json = false;

    for argument in args {
        match argument.as_str() {
            "--json" => json = true,
            other => return Err(ParseError::UnexpectedArgument(other.to_owned())),
        }
    }

    Ok(Command::Doctor { json })
}

fn parse_shell(args: &[String]) -> Result<Command, ParseError> {
    let action = match args.first().map(String::as_str) {
        Some("ping") => ControlAction::Ping,
        Some("open-launcher") => ControlAction::OpenLauncher,
        Some("toggle-launcher") => ControlAction::ToggleLauncher,
        Some("toggle-dashboard") => ControlAction::ToggleDashboard,
        Some(other) => return Err(ParseError::UnknownCommand(format!("shell {other}"))),
        None => return Err(ParseError::MissingSubcommand("shell")),
    };

    if let Some(extra) = args.get(1) {
        return Err(ParseError::UnexpectedArgument(extra.to_owned()));
    }

    Ok(Command::Shell { action })
}

fn parse_apps(args: &[String]) -> Result<Command, ParseError> {
    match args.first().map(String::as_str) {
        Some("list") => {
            if let Some(extra) = args.get(1) {
                return Err(ParseError::UnexpectedArgument(extra.to_owned()));
            }
            Ok(Command::AppsList)
        }
        Some("search") => {
            let query = args.get(1..).unwrap_or_default().join(" ");
            if query.is_empty() {
                return Err(ParseError::MissingValue { option: "query" });
            }
            Ok(Command::AppsSearch { query })
        }
        Some(other) => Err(ParseError::UnknownCommand(format!("apps {other}"))),
        None => Err(ParseError::MissingSubcommand("apps")),
    }
}

fn parse_niri(args: &[String]) -> Result<Command, ParseError> {
    match args.first().map(String::as_str) {
        Some("inspect") => parse_niri_inspect(&args[1..]),
        Some("watch") => parse_niri_watch(&args[1..]),
        Some(other) => Err(ParseError::UnknownCommand(format!("niri {other}"))),
        None => Err(ParseError::MissingSubcommand("niri")),
    }
}

fn parse_niri_inspect(args: &[String]) -> Result<Command, ParseError> {
    let mut socket: Option<PathBuf> = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--socket" | "-s" => {
                if socket.is_some() {
                    return Err(ParseError::DuplicateOption { option: "--socket" });
                }
                let value = args
                    .get(index + 1)
                    .ok_or(ParseError::MissingValue { option: "--socket" })?;
                socket = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(ParseError::UnexpectedArgument(other.to_owned())),
        }
    }

    Ok(Command::NiriInspect { socket })
}

fn parse_niri_watch(args: &[String]) -> Result<Command, ParseError> {
    let mut socket: Option<PathBuf> = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--socket" | "-s" => {
                if socket.is_some() {
                    return Err(ParseError::DuplicateOption { option: "--socket" });
                }
                let value = args
                    .get(index + 1)
                    .ok_or(ParseError::MissingValue { option: "--socket" })?;
                socket = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(ParseError::UnexpectedArgument(other.to_owned())),
        }
    }

    Ok(Command::NiriWatch { socket })
}

/// Execute the requested command against the supplied output streams.
///
/// Returns a process exit code. Write failures are ignored so a closed pipe
/// does not panic the process.
pub fn execute<W, E>(args: &[String], out: &mut W, err: &mut E) -> u8
where
    W: Write,
    E: Write,
{
    match parse(args) {
        Ok(Command::Help) => {
            let _ = write!(out, "{HELP}");
            EXIT_SUCCESS
        }
        Ok(Command::Version) => {
            let _ = writeln!(out, "{BIN_NAME} {}", env!("CARGO_PKG_VERSION"));
            EXIT_SUCCESS
        }
        Ok(Command::ConfigCheck { path }) => match nalios_app::config::check(&FsConfigSource, &path)
        {
            Ok(config) => {
                let _ = writeln!(
                    out,
                    "{}: configuration is valid (schema version {})",
                    path.display(),
                    config.schema_version
                );
                EXIT_SUCCESS
            }
            Err(error) => {
                let _ = writeln!(err, "{}: error: {error}", path.display());
                EXIT_FAILURE
            }
        },
        Ok(Command::Doctor { json }) => run_doctor(out, err, json),
        Ok(Command::NiriInspect { socket }) => match socket.or_else(niri::socket_from_env) {
            Some(path) => run_niri_inspect(&path, out, err),
            None => {
                let _ = writeln!(err, "error: NIRI_SOCKET is not set; pass --socket <path>");
                EXIT_FAILURE
            }
        },
        Ok(Command::NiriWatch { socket }) => match socket.or_else(niri::socket_from_env) {
            Some(path) => run_niri_watch(&path, out, err),
            None => {
                let _ = writeln!(err, "error: NIRI_SOCKET is not set; pass --socket <path>");
                EXIT_FAILURE
            }
        },
        Ok(Command::Shell { action }) => {
            match nalios_platform::control::send(&ControlRequest::new(action)) {
                Ok(ControlResponse::Ok) => {
                    let _ = writeln!(out, "ok");
                    EXIT_SUCCESS
                }
                Ok(ControlResponse::Error { message }) => {
                    let _ = writeln!(err, "error: {message}");
                    EXIT_FAILURE
                }
                Err(error) => {
                    let _ = writeln!(err, "error: {error}");
                    EXIT_FAILURE
                }
            }
        }
        Ok(Command::AppsList) => {
            let catalog = nalios_platform::apps::AppCatalog::discover();
            for app in catalog.apps() {
                let _ = writeln!(out, "{}\t{}", app.id, app.name);
            }
            EXIT_SUCCESS
        }
        Ok(Command::AppsSearch { query }) => {
            let catalog = nalios_platform::apps::AppCatalog::discover();
            let apps = catalog.apps();
            for index in nalios_app::apps::rank(apps, &[], &query, 20) {
                let app = &apps[index];
                let _ = writeln!(out, "{}\t{}", app.id, app.name);
            }
            EXIT_SUCCESS
        }
        Err(error) => {
            let _ = writeln!(err, "{}", error.render());
            EXIT_USAGE
        }
    }
}

fn run_doctor<W: Write, E: Write>(out: &mut W, err: &mut E, json: bool) -> u8 {
    let report = diagnostics::run(&HostDiagnostics);

    if json {
        match serde_json::to_string_pretty(&report.envelope()) {
            Ok(text) => {
                let _ = writeln!(out, "{text}");
            }
            Err(error) => {
                let _ = writeln!(err, "error: could not serialize doctor report: {error}");
                return EXIT_INTERNAL;
            }
        }
    } else {
        let _ = writeln!(out, "NaliOS doctor");
        let _ = writeln!(out);

        for check in &report.checks {
            let scope = match check.scope {
                Scope::Host => "host",
                Scope::Session => "session",
            };
            let status = match check.status {
                Status::Pass => "pass",
                Status::Warn => "warn",
                Status::Fail => "fail",
                Status::Unavailable => "n/a",
            };
            let _ = writeln!(out, "{scope:7} [{status:>4}] {}: {}", check.id, check.detail);
            if let Some(hint) = &check.hint {
                let _ = writeln!(out, "        hint: {hint}");
            }
        }

        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "summary: {} pass, {} warn, {} fail, {} unavailable",
            report.count(Status::Pass),
            report.count(Status::Warn),
            report.count(Status::Fail),
            report.count(Status::Unavailable),
        );
    }

    if report.has_required_failure() {
        EXIT_FAILURE
    } else {
        EXIT_SUCCESS
    }
}

fn run_niri_inspect<W: Write, E: Write>(path: &Path, out: &mut W, err: &mut E) -> u8 {
    let mut client = match NiriClient::connect(path) {
        Ok(client) => client,
        Err(error) => {
            let _ = writeln!(err, "error: {error}");
            return EXIT_FAILURE;
        }
    };

    let snapshot = match client.snapshot() {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let _ = writeln!(err, "error: {error}");
            return EXIT_FAILURE;
        }
    };

    let _ = writeln!(out, "Niri socket: {}", path.display());
    let _ = writeln!(out);

    let _ = writeln!(out, "outputs ({}):", snapshot.outputs.len());
    for output in snapshot.outputs.values() {
        match &output.logical {
            Some(logical) => {
                let _ = writeln!(
                    out,
                    "  {} {}x{} scale {:.2} transform {}",
                    output.name, logical.width, logical.height, logical.scale, logical.transform
                );
            }
            None => {
                let _ = writeln!(out, "  {} (no logical geometry)", output.name);
            }
        }
    }

    let _ = writeln!(out, "workspaces ({}):", snapshot.workspaces.len());
    for workspace in &snapshot.workspaces {
        let mut line = format!("  {}", workspace.idx);
        if let Some(name) = &workspace.name {
            line.push_str(&format!(" \"{name}\""));
        }
        if let Some(output) = &workspace.output {
            line.push_str(&format!(" {output}"));
        }
        if workspace.is_active {
            line.push_str(" active");
        }
        if workspace.is_focused {
            line.push_str(" focused");
        }
        if workspace.is_urgent {
            line.push_str(" urgent");
        }
        let _ = writeln!(out, "{line}");
    }

    let _ = writeln!(out, "windows ({})", snapshot.windows.len());
    match &snapshot.focused_window {
        Some(window) => {
            let app = window.app_id.as_deref().unwrap_or("unknown");
            let title = window.title.as_deref().unwrap_or("(untitled)");
            let _ = writeln!(out, "focused: {app} \"{title}\"");
        }
        None => {
            let _ = writeln!(out, "focused: none");
        }
    }

    EXIT_SUCCESS
}

fn run_niri_watch<W: Write, E: Write>(path: &Path, out: &mut W, err: &mut E) -> u8 {
    let mut observation = NiriObservation::new();

    let _ = writeln!(out, "watching Niri at {} (Ctrl+C to stop)", path.display());

    loop {
        match NiriEventStream::connect(path) {
            Ok(mut stream) => {
                observation.on_connected();
                let _ = writeln!(
                    out,
                    "[rev {}] connected",
                    observation.state().revision()
                );

                // Niri 26.04 does not stream output changes, so refresh outputs
                // on every connection. A reconnect therefore rebuilds accurate
                // output state as well.
                if let Ok(mut client) = NiriClient::connect(path) {
                    if let Ok(outputs) = client.outputs() {
                        observation.set_outputs(outputs);
                    }
                }

                loop {
                    match stream.next_event() {
                        Ok(event) => {
                            let summary = describe_event(&event);
                            observation.on_event(event);
                            let _ = writeln!(
                                out,
                                "[rev {}] {summary}",
                                observation.state().revision()
                            );
                        }
                        Err(error) => {
                            observation.on_disconnected(error.to_string());
                            let _ = writeln!(err, "warning: {error}");
                            break;
                        }
                    }
                }
            }
            Err(error) => {
                observation.on_disconnected(error.to_string());
                let _ = writeln!(err, "warning: {error}");
            }
        }

        let delay = observation.retry_delay();
        let _ = writeln!(err, "retrying in {} ms", delay.as_millis());
        std::thread::sleep(delay);
    }
}

fn describe_event(event: &NiriEvent) -> String {
    match event {
        NiriEvent::WorkspacesChanged { workspaces } => {
            format!("workspaces changed ({} total)", workspaces.len())
        }
        NiriEvent::WorkspaceUrgencyChanged { id, urgent } => {
            format!("workspace {} urgency={urgent}", id.0)
        }
        NiriEvent::WorkspaceActivated { id, focused } => {
            format!(
                "workspace {} activated{}",
                id.0,
                if *focused { " (focused)" } else { "" }
            )
        }
        NiriEvent::WorkspaceActiveWindowChanged {
            workspace_id,
            active_window_id,
        } => match active_window_id {
            Some(id) => format!("workspace {} active window={}", workspace_id.0, id.0),
            None => format!("workspace {} active window cleared", workspace_id.0),
        },
        NiriEvent::WindowsChanged { windows } => {
            format!("windows changed ({} total)", windows.len())
        }
        NiriEvent::WindowOpenedOrChanged { window } => {
            format!("window {} opened or changed", window.id.0)
        }
        NiriEvent::WindowClosed { id } => format!("window {} closed", id.0),
        NiriEvent::WindowFocusChanged { id } => match id {
            Some(id) => format!("focus={}", id.0),
            None => "focus cleared".to_owned(),
        },
        NiriEvent::WindowUrgencyChanged { id, urgent } => {
            format!("window {} urgency={urgent}", id.0)
        }
    }
}

/// Run the development CLI with the real process streams.
pub fn run(args: &[String]) -> ExitCode {
    let mut out = std::io::stdout().lock();
    let mut err = std::io::stderr().lock();
    ExitCode::from(execute(args, &mut out, &mut err))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn no_arguments_requests_help() {
        assert_eq!(parse(&[]), Ok(Command::Help));
    }

    #[test]
    fn help_flags_request_help() {
        for flag in ["-h", "--help"] {
            assert_eq!(parse(&args(&[flag])), Ok(Command::Help));
        }
    }

    #[test]
    fn version_flags_request_version() {
        for flag in ["-V", "--version"] {
            assert_eq!(parse(&args(&[flag])), Ok(Command::Version));
        }
    }

    #[test]
    fn first_decisive_argument_wins() {
        assert_eq!(parse(&args(&["--version", "--help"])), Ok(Command::Version));
        assert_eq!(parse(&args(&["--help", "--version"])), Ok(Command::Help));
    }

    #[test]
    fn unknown_commands_and_options_are_rejected() {
        assert_eq!(
            parse(&args(&["frobnicate"])),
            Err(ParseError::UnknownCommand("frobnicate".to_owned()))
        );
        assert_eq!(
            parse(&args(&["--bogus"])),
            Err(ParseError::UnknownOption("--bogus".to_owned()))
        );
    }

    #[test]
    fn config_check_parses_file_option() {
        let command = parse(&args(&["config", "check", "--file", "config.toml"])).expect("parses");

        assert_eq!(
            command,
            Command::ConfigCheck {
                path: PathBuf::from("config.toml")
            }
        );
    }

    #[test]
    fn config_check_requires_file() {
        assert_eq!(
            parse(&args(&["config", "check"])),
            Err(ParseError::MissingOption { option: "--file" })
        );
    }

    #[test]
    fn config_check_rejects_missing_value() {
        assert_eq!(
            parse(&args(&["config", "check", "--file"])),
            Err(ParseError::MissingValue { option: "--file" })
        );
    }

    #[test]
    fn config_check_rejects_duplicate_file() {
        assert_eq!(
            parse(&args(&["config", "check", "--file", "a", "--file", "b"])),
            Err(ParseError::DuplicateOption { option: "--file" })
        );
    }

    #[test]
    fn config_without_subcommand_is_rejected() {
        assert_eq!(
            parse(&args(&["config"])),
            Err(ParseError::MissingSubcommand("config"))
        );
    }

    #[test]
    fn unknown_config_subcommand_is_rejected() {
        assert_eq!(
            parse(&args(&["config", "wat"])),
            Err(ParseError::UnknownCommand("config wat".to_owned()))
        );
    }

    #[test]
    fn doctor_parses() {
        assert_eq!(
            parse(&args(&["doctor"])),
            Ok(Command::Doctor { json: false })
        );
        assert_eq!(
            parse(&args(&["doctor", "--json"])),
            Ok(Command::Doctor { json: true })
        );
    }

    #[test]
    fn doctor_rejects_unknown_arguments() {
        assert_eq!(
            parse(&args(&["doctor", "--bogus"])),
            Err(ParseError::UnexpectedArgument("--bogus".to_owned()))
        );
    }

    #[test]
    fn doctor_json_is_versioned_and_structured() {
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute(&args(&["doctor", "--json"]), &mut out, &mut err);

        assert!(code == EXIT_SUCCESS || code == EXIT_FAILURE);
        assert!(err.is_empty());
        let text = String::from_utf8(out).expect("doctor output is UTF-8");
        let value: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

        assert_eq!(value["schema_version"], 1);
        assert!(value["summary"]["pass"].is_number());
        let checks = value["checks"].as_array().expect("checks array");
        let ids: Vec<&str> = checks.iter().filter_map(|check| check["id"].as_str()).collect();
        assert!(ids.contains(&"os"));
        assert!(ids.contains(&"runtime-dir"));
    }

    #[test]
    fn doctor_writes_report_and_summary() {
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute(&args(&["doctor"]), &mut out, &mut err);

        assert!(code == EXIT_SUCCESS || code == EXIT_FAILURE);
        assert!(err.is_empty());
        let text = String::from_utf8(out).expect("doctor output is UTF-8");
        assert!(text.contains("NaliOS doctor"));
        assert!(text.contains("os: "));
        assert!(text.contains("summary:"));
    }

    #[test]
    fn niri_inspect_parses_socket_option() {
        assert_eq!(
            parse(&args(&["niri", "inspect", "--socket", "/tmp/niri.sock"])),
            Ok(Command::NiriInspect {
                socket: Some(PathBuf::from("/tmp/niri.sock"))
            })
        );
        assert_eq!(
            parse(&args(&["niri", "inspect"])),
            Ok(Command::NiriInspect { socket: None })
        );
    }

    #[test]
    fn niri_inspect_rejects_missing_socket_value() {
        assert_eq!(
            parse(&args(&["niri", "inspect", "--socket"])),
            Err(ParseError::MissingValue { option: "--socket" })
        );
    }

    #[test]
    fn niri_requires_a_subcommand() {
        assert_eq!(
            parse(&args(&["niri"])),
            Err(ParseError::MissingSubcommand("niri"))
        );
        assert_eq!(
            parse(&args(&["niri", "wat"])),
            Err(ParseError::UnknownCommand("niri wat".to_owned()))
        );
    }

    #[test]
    fn niri_inspect_reports_connect_errors() {
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute(
            &args(&[
                "niri",
                "inspect",
                "--socket",
                "/definitely/missing/niri.sock",
            ]),
            &mut out,
            &mut err,
        );

        assert_eq!(code, EXIT_FAILURE);
        assert!(out.is_empty());
        let text = String::from_utf8(err).expect("error output is UTF-8");
        assert!(text.contains("could not connect"));
    }

    #[test]
    fn shell_actions_parse() {
        assert_eq!(
            parse(&args(&["shell", "ping"])),
            Ok(Command::Shell {
                action: ControlAction::Ping
            })
        );
        assert_eq!(
            parse(&args(&["shell", "open-launcher"])),
            Ok(Command::Shell {
                action: ControlAction::OpenLauncher
            })
        );
        assert_eq!(
            parse(&args(&["shell", "toggle-launcher"])),
            Ok(Command::Shell {
                action: ControlAction::ToggleLauncher
            })
        );
        assert_eq!(
            parse(&args(&["shell", "toggle-dashboard"])),
            Ok(Command::Shell {
                action: ControlAction::ToggleDashboard
            })
        );
        assert_eq!(
            parse(&args(&["shell"])),
            Err(ParseError::MissingSubcommand("shell"))
        );
        assert_eq!(
            parse(&args(&["shell", "wat"])),
            Err(ParseError::UnknownCommand("shell wat".to_owned()))
        );
    }

    #[test]
    fn apps_commands_parse() {
        assert_eq!(parse(&args(&["apps", "list"])), Ok(Command::AppsList));
        assert_eq!(
            parse(&args(&["apps", "search", "web", "browser"])),
            Ok(Command::AppsSearch {
                query: "web browser".to_owned()
            })
        );
        assert_eq!(
            parse(&args(&["apps", "search"])),
            Err(ParseError::MissingValue { option: "query" })
        );
        assert_eq!(
            parse(&args(&["apps"])),
            Err(ParseError::MissingSubcommand("apps"))
        );
    }

    #[test]
    fn niri_watch_parses_socket_option() {
        assert_eq!(
            parse(&args(&["niri", "watch"])),
            Ok(Command::NiriWatch { socket: None })
        );
        assert_eq!(
            parse(&args(&["niri", "watch", "--socket", "/tmp/niri.sock"])),
            Ok(Command::NiriWatch {
                socket: Some(PathBuf::from("/tmp/niri.sock"))
            })
        );
    }

    // `niri watch` retries forever by design, so its loop is not exercised
    // here; the backoff/stale policy is unit-tested in `nalios-app` and the
    // stream transport in `nalios-platform`.

    #[test]
    fn help_writes_to_stdout_and_succeeds() {
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute(&args(&["--help"]), &mut out, &mut err);

        assert_eq!(code, EXIT_SUCCESS);
        assert!(err.is_empty());
        let text = String::from_utf8(out).expect("help output is UTF-8");
        assert!(text.contains("Usage: nalios-dev"));
        assert!(text.contains("config check"));
    }

    #[test]
    fn version_writes_name_and_version() {
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute(&args(&["--version"]), &mut out, &mut err);

        assert_eq!(code, EXIT_SUCCESS);
        assert!(err.is_empty());
        let text = String::from_utf8(out).expect("version output is UTF-8");
        assert_eq!(
            text.trim_end(),
            format!("{BIN_NAME} {}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn config_check_reports_missing_file() {
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute(
            &args(&[
                "config",
                "check",
                "--file",
                "/definitely/missing/nalios-config.toml",
            ]),
            &mut out,
            &mut err,
        );

        assert_eq!(code, EXIT_FAILURE);
        assert!(out.is_empty());
        let text = String::from_utf8(err).expect("error output is UTF-8");
        assert!(text.contains("not found"));
    }

    #[test]
    fn invalid_invocation_writes_stderr_and_uses_usage_code() {
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute(&args(&["--bogus"]), &mut out, &mut err);

        assert_eq!(code, EXIT_USAGE);
        assert!(out.is_empty());
        let text = String::from_utf8(err).expect("error output is UTF-8");
        assert!(text.contains("--bogus"));
        assert!(text.contains("nalios-dev --help"));
    }
}
