//! Pure Niri domain values.
//!
//! These types are independent of the IPC wire format and of any toolkit.
//! Platform adapters convert wire data into them; the application layer keeps
//! canonical state using them.

use std::collections::BTreeMap;

/// Stable workspace identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceId(pub u64);

/// Stable window identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub u64);

/// Output name, for example `eDP-1`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputName(pub String);

impl OutputName {
    /// Borrow the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OutputName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A workspace as observed from Niri.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// Stable workspace identifier.
    pub id: WorkspaceId,
    /// Workspace index shown in the bar.
    pub idx: u32,
    /// Optional user-visible name.
    pub name: Option<String>,
    /// Output the workspace currently appears on.
    pub output: Option<OutputName>,
    /// Whether the workspace has an urgent window.
    pub is_urgent: bool,
    /// Whether the workspace is active on its output.
    pub is_active: bool,
    /// Whether the workspace is focused.
    pub is_focused: bool,
    /// Currently active window on the workspace.
    pub active_window_id: Option<WindowId>,
}

/// A window as observed from Niri.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Stable window identifier.
    pub id: WindowId,
    /// Window title, when set.
    pub title: Option<String>,
    /// Application identifier, when set.
    pub app_id: Option<String>,
    /// Workspace the window belongs to.
    pub workspace_id: Option<WorkspaceId>,
    /// Whether the window is focused.
    pub is_focused: bool,
    /// Whether the window is floating.
    pub is_floating: bool,
    /// Whether the window is urgent.
    pub is_urgent: bool,
}

/// An output as observed from Niri.
#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    /// Output name.
    pub name: OutputName,
    /// Logical geometry, absent for disabled outputs.
    pub logical: Option<LogicalOutput>,
}

/// Logical output geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalOutput {
    /// X position in the global layout.
    pub x: i32,
    /// Y position in the global layout.
    pub y: i32,
    /// Width in logical pixels.
    pub width: u32,
    /// Height in logical pixels.
    pub height: u32,
    /// Fractional scale.
    pub scale: f64,
    /// Transform name, for example `normal`.
    pub transform: String,
}

/// A read-only snapshot of Niri state.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    /// Workspaces in Niri order.
    pub workspaces: Vec<Workspace>,
    /// Windows in Niri order.
    pub windows: Vec<Window>,
    /// Outputs keyed by output name.
    pub outputs: BTreeMap<OutputName, Output>,
    /// Focused window, if any.
    pub focused_window: Option<Window>,
}
