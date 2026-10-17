//! Desktop-entry discovery and launching.
//!
//! Uses the reviewed `freedesktop-desktop-entry` parser (MPL-2.0) for INI and
//! `Exec` semantics and maps entries into pure core values. Launching uses
//! explicit argv, never a shell string.

use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use freedesktop_desktop_entry::{
    DesktopEntry, current_desktop, desktop_entries, get_languages_from_env,
};
use freedesktop_icons::lookup;
use nalios_core::apps::DesktopApp;

/// The installed application catalog.
#[derive(Debug, Clone, Default)]
pub struct AppCatalog {
    apps: Vec<DesktopApp>,
}

impl AppCatalog {
    /// Discover applications from the XDG data directories.
    ///
    /// Hidden and `NoDisplay` entries are skipped, as are entries that exclude
    /// the current desktop or whose `TryExec` program is missing. Earlier data
    /// directories win when two entries share an id.
    pub fn discover() -> Self {
        let locales = get_languages_from_env();
        let current = current_desktop().unwrap_or_default();
        let mut seen = HashSet::new();
        let mut apps = Vec::new();

        for entry in desktop_entries(&locales) {
            if entry.desktop_entry("Type") != Some("Application") {
                continue;
            }
            if entry.hidden() || entry.no_display() {
                continue;
            }
            if !shown_in_current_desktop(&entry, &current) {
                continue;
            }
            if !try_exec_available(&entry) {
                continue;
            }
            let Some(app) = app_from_entry(&entry, &locales) else {
                continue;
            };
            if !seen.insert(app.id.clone()) {
                continue;
            }
            apps.push(app);
        }

        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Self { apps }
    }

    /// Discovered applications in name order.
    pub fn apps(&self) -> &[DesktopApp] {
        &self.apps
    }
}

fn app_from_entry(entry: &DesktopEntry, locales: &[String]) -> Option<DesktopApp> {
    let exec = entry.parse_exec().ok()?;
    if exec.is_empty() {
        return None;
    }
    let name = entry.name(locales)?;

    Some(DesktopApp {
        id: entry.id().to_owned(),
        name: name.into_owned(),
        comment: entry.comment(locales).map(|comment| comment.into_owned()),
        icon: entry.icon().map(str::to_owned),
        terminal: entry.terminal(),
        path: entry.path.clone(),
        exec,
        keywords: entry
            .keywords(locales)
            .unwrap_or_default()
            .into_iter()
            .map(|keyword| keyword.into_owned())
            .filter(|keyword| !keyword.is_empty())
            .collect(),
    })
}

fn shown_in_current_desktop(entry: &DesktopEntry, current: &[String]) -> bool {
    if current.is_empty() {
        return true;
    }

    if let Some(only) = entry.only_show_in() {
        if !only.is_empty()
            && !only
                .iter()
                .any(|name| current.iter().any(|current| current.eq_ignore_ascii_case(name)))
        {
            return false;
        }
    }

    if let Some(not) = entry.not_show_in() {
        if not
            .iter()
            .any(|name| current.iter().any(|current| current.eq_ignore_ascii_case(name)))
        {
            return false;
        }
    }

    true
}

fn try_exec_available(entry: &DesktopEntry) -> bool {
    match entry.try_exec() {
        Some(program) => program_in_path(program),
        None => true,
    }
}

fn program_in_path(program: &str) -> bool {
    let path = Path::new(program);
    if path.is_absolute() {
        return is_executable(path);
    }

    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&path_var).any(|directory| is_executable(&directory.join(program)))
}

fn is_executable(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Find an icon file for a desktop icon name.
///
/// Absolute paths are used directly; theme names are resolved through the
/// user's icon theme (detected once) with the crate's fallback lookup.
pub fn find_icon(name: &str) -> Option<PathBuf> {
    let candidate = Path::new(name);
    if candidate.is_absolute() {
        return candidate.is_file().then(|| candidate.to_path_buf());
    }

    if let Some(theme) = current_icon_theme() {
        if let Some(path) = lookup(name)
            .with_theme(theme)
            .with_size(48)
            .with_scale(1)
            .with_cache()
            .find()
        {
            return Some(path);
        }
    }

    lookup(name).with_size(48).with_scale(1).with_cache().find()
}

fn current_icon_theme() -> Option<&'static str> {
    static THEME: OnceLock<Option<String>> = OnceLock::new();
    THEME
        .get_or_init(freedesktop_icons::default_theme_gtk)
        .as_deref()
}

/// A launch failure.
#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    /// The application has no command to run.
    #[error("application command is empty")]
    EmptyCommand,
    /// The command was not found.
    #[error("application command was not found")]
    NotFound,
    /// The process could not be started.
    #[error("could not launch the application: {0}")]
    Spawn(#[source] std::io::Error),
}

/// Launch an application with explicit argv.
///
/// Terminal applications are wrapped with `xdg-terminal-exec`.
pub fn launch(app: &DesktopApp) -> Result<(), LaunchError> {
    if app.exec.is_empty() {
        return Err(LaunchError::EmptyCommand);
    }

    let mut command = if app.terminal {
        let mut command = Command::new("xdg-terminal-exec");
        command.args(&app.exec);
        command
    } else {
        let mut command = Command::new(&app.exec[0]);
        command.args(&app.exec[1..]);
        command
    };

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command.spawn().map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => LaunchError::NotFound,
        _ => LaunchError::Spawn(error),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use nalios_core::apps::DesktopApp;
    use tempfile::TempDir;

    use super::*;

    /// Build an entry from a file inside an `applications` directory, which is
    /// what the parser needs to derive a desktop file id.
    fn entry_from(text: &str) -> (TempDir, DesktopEntry) {
        let directory = TempDir::new().expect("temp dir");
        let applications = directory.path().join("applications");
        std::fs::create_dir_all(&applications).expect("mkdir");
        let path = applications.join("test.desktop");
        std::fs::write(&path, text).expect("write");
        let entry = DesktopEntry::from_path(&path, Some(&[] as &[String])).expect("parse");
        (directory, entry)
    }

    #[test]
    fn only_show_in_filters_other_desktops() {
        let (_file, entry) = entry_from(
            "[Desktop Entry]\nType=Application\nName=Test\nExec=test\nOnlyShowIn=GNOME;\n",
        );

        assert!(!shown_in_current_desktop(&entry, &["niri".to_owned()]));
        assert!(shown_in_current_desktop(&entry, &["gnome".to_owned()]));
    }

    #[test]
    fn not_show_in_filters_matching_desktops() {
        let (_file, entry) = entry_from(
            "[Desktop Entry]\nType=Application\nName=Test\nExec=test\nNotShowIn=niri;\n",
        );

        assert!(!shown_in_current_desktop(&entry, &["niri".to_owned()]));
        assert!(shown_in_current_desktop(&entry, &["gnome".to_owned()]));
    }

    #[test]
    fn try_exec_checks_the_program() {
        let (_file, missing) = entry_from(
            "[Desktop Entry]\nType=Application\nName=Test\nExec=test\nTryExec=nalios-missing-tool\n",
        );
        let (_file2, present) = entry_from(
            "[Desktop Entry]\nType=Application\nName=Test\nExec=test\nTryExec=sh\n",
        );

        assert!(!try_exec_available(&missing));
        assert!(try_exec_available(&present));
    }

    #[test]
    fn entries_map_to_core_values() {
        let (_file, entry) = entry_from(
            "[Desktop Entry]\nType=Application\nName=Test App\nComment=Does tests\nExec=sh -c \"echo hi\"\nIcon=test-icon\nKeywords=alpha;beta;\nTerminal=true\n",
        );

        let app = app_from_entry(&entry, &[]).expect("app");

        assert_eq!(app.name, "Test App");
        assert_eq!(app.comment.as_deref(), Some("Does tests"));
        assert_eq!(app.icon.as_deref(), Some("test-icon"));
        assert!(app.terminal);
        assert_eq!(app.keywords, vec!["alpha", "beta"]);
        assert_eq!(app.exec.first().map(String::as_str), Some("sh"));
    }

    #[test]
    #[ignore = "depends on the host icon theme"]
    fn finds_a_theme_icon_on_this_host() {
        let icon = find_icon("utilities-terminal");
        println!("utilities-terminal -> {icon:?}");
        assert!(icon.is_some());
    }

    #[test]
    fn absolute_icon_paths_are_used_directly() {
        let directory = TempDir::new().expect("temp dir");
        let icon = directory.path().join("test.svg");
        std::fs::write(&icon, b"<svg/>").expect("write");

        assert_eq!(find_icon(icon.to_str().expect("utf-8")), Some(icon));
        assert_eq!(find_icon("/definitely/missing/icon.svg"), None);
    }

    #[test]
    fn launch_reports_missing_commands() {
        let app = DesktopApp {
            id: "missing.desktop".to_owned(),
            name: "Missing".to_owned(),
            comment: None,
            icon: None,
            terminal: false,
            path: std::path::PathBuf::from("/tmp/missing.desktop"),
            exec: vec!["nalios-missing-tool".to_owned()],
            keywords: Vec::new(),
        };

        assert!(matches!(launch(&app), Err(LaunchError::NotFound)));
    }

    #[test]
    fn launch_rejects_empty_commands() {
        let app = DesktopApp {
            id: "empty.desktop".to_owned(),
            name: "Empty".to_owned(),
            comment: None,
            icon: None,
            terminal: false,
            path: std::path::PathBuf::from("/tmp/empty.desktop"),
            exec: Vec::new(),
            keywords: Vec::new(),
        };

        assert!(matches!(launch(&app), Err(LaunchError::EmptyCommand)));
    }
}
