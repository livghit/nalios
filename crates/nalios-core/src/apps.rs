//! Pure application-catalog values.
//!
//! Platform adapters discover and launch; these types carry the data without
//! depending on a parser or process API.

use std::path::PathBuf;

/// An installed desktop application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopApp {
    /// Desktop file id, for example `firefox.desktop`.
    pub id: String,
    /// Localized display name.
    pub name: String,
    /// Optional localized description.
    pub comment: Option<String>,
    /// Icon name, if declared.
    pub icon: Option<String>,
    /// Whether the application needs a terminal.
    pub terminal: bool,
    /// Desktop file path.
    pub path: PathBuf,
    /// Parsed command line without a terminal wrapper.
    pub exec: Vec<String>,
    /// Search keywords declared by the desktop entry.
    pub keywords: Vec<String>,
}
