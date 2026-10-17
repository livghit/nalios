//! Shared Niri observation state.
//!
//! The application layer owns the canonical reducer. Platform adapters convert
//! wire events into [`NiriEvent`] values; every view consumes the same
//! [`NiriState`] projections instead of keeping a rival copy.

use std::collections::BTreeMap;
use std::time::Duration;

use nalios_core::niri::{Output, OutputName, Window, WindowId, Workspace, WorkspaceId};

/// A modeled Niri state change delivered by the event stream.
///
/// Unknown events are filtered by the platform adapter, so this enum stays
/// exhaustive for the behavior NaliOS implements.
#[derive(Debug, Clone, PartialEq)]
pub enum NiriEvent {
    /// The complete workspace list changed.
    WorkspacesChanged {
        /// Full replacement list.
        workspaces: Vec<Workspace>,
    },
    /// A workspace's urgency changed.
    WorkspaceUrgencyChanged {
        /// Workspace identifier.
        id: WorkspaceId,
        /// New urgency.
        urgent: bool,
    },
    /// A workspace was activated on its output.
    WorkspaceActivated {
        /// Workspace identifier.
        id: WorkspaceId,
        /// Whether the workspace also became focused.
        focused: bool,
    },
    /// The active window of a workspace changed.
    WorkspaceActiveWindowChanged {
        /// Workspace identifier.
        workspace_id: WorkspaceId,
        /// New active window, if any.
        active_window_id: Option<WindowId>,
    },
    /// The complete window list changed.
    WindowsChanged {
        /// Full replacement list.
        windows: Vec<Window>,
    },
    /// A window opened or its properties changed.
    WindowOpenedOrChanged {
        /// The window.
        window: Window,
    },
    /// A window closed.
    WindowClosed {
        /// Window identifier.
        id: WindowId,
    },
    /// Focus moved to another window or was cleared.
    WindowFocusChanged {
        /// Focused window, if any.
        id: Option<WindowId>,
    },
    /// A window's urgency changed.
    WindowUrgencyChanged {
        /// Window identifier.
        id: WindowId,
        /// New urgency.
        urgent: bool,
    },
}

/// Canonical application state derived from Niri events.
#[derive(Debug, Clone, PartialEq)]
pub struct NiriState {
    revision: u64,
    workspaces: Vec<Workspace>,
    windows: BTreeMap<WindowId, Window>,
    outputs: BTreeMap<OutputName, Output>,
    focused_window: Option<WindowId>,
}

impl Default for NiriState {
    fn default() -> Self {
        Self::new()
    }
}

impl NiriState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self {
            revision: 0,
            workspaces: Vec::new(),
            windows: BTreeMap::new(),
            outputs: BTreeMap::new(),
            focused_window: None,
        }
    }

    /// Monotonic revision; advances whenever the state may have changed.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Workspaces in Niri order.
    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    /// Windows keyed by identifier.
    pub fn windows(&self) -> &BTreeMap<WindowId, Window> {
        &self.windows
    }

    /// Outputs keyed by name.
    pub fn outputs(&self) -> &BTreeMap<OutputName, Output> {
        &self.outputs
    }

    /// Focused window, if any.
    pub fn focused_window(&self) -> Option<&Window> {
        self.focused_window.and_then(|id| self.windows.get(&id))
    }

    /// Replace the output set.
    ///
    /// Niri 26.04 does not stream output changes, so the initial outputs are
    /// seeded from a snapshot request.
    pub fn set_outputs(&mut self, outputs: BTreeMap<OutputName, Output>) {
        self.outputs = outputs;
        self.revision += 1;
    }

    /// Apply one event to the state.
    pub fn apply(&mut self, event: NiriEvent) {
        match event {
            NiriEvent::WorkspacesChanged { workspaces } => {
                self.workspaces = workspaces;
            }
            NiriEvent::WorkspaceUrgencyChanged { id, urgent } => {
                if let Some(workspace) = self.workspaces.iter_mut().find(|w| w.id == id) {
                    workspace.is_urgent = urgent;
                }
            }
            NiriEvent::WorkspaceActivated { id, focused } => {
                let output = self
                    .workspaces
                    .iter()
                    .find(|workspace| workspace.id == id)
                    .and_then(|workspace| workspace.output.clone());

                for workspace in &mut self.workspaces {
                    if workspace.id == id {
                        workspace.is_active = true;
                    } else if output.is_some() && workspace.output == output {
                        workspace.is_active = false;
                    }
                    if focused {
                        workspace.is_focused = workspace.id == id;
                    }
                }
            }
            NiriEvent::WorkspaceActiveWindowChanged {
                workspace_id,
                active_window_id,
            } => {
                if let Some(workspace) = self
                    .workspaces
                    .iter_mut()
                    .find(|workspace| workspace.id == workspace_id)
                {
                    workspace.active_window_id = active_window_id;
                }
            }
            NiriEvent::WindowsChanged { windows } => {
                self.windows = windows.into_iter().map(|window| (window.id, window)).collect();
                self.focused_window = self
                    .windows
                    .values()
                    .find(|window| window.is_focused)
                    .map(|window| window.id);
            }
            NiriEvent::WindowOpenedOrChanged { window } => {
                if window.is_focused {
                    self.focused_window = Some(window.id);
                }
                self.windows.insert(window.id, window);
            }
            NiriEvent::WindowClosed { id } => {
                self.windows.remove(&id);
                if self.focused_window == Some(id) {
                    self.focused_window = None;
                }
                for workspace in &mut self.workspaces {
                    if workspace.active_window_id == Some(id) {
                        workspace.active_window_id = None;
                    }
                }
            }
            NiriEvent::WindowFocusChanged { id } => {
                self.focused_window = id;
                for window in self.windows.values_mut() {
                    window.is_focused = Some(window.id) == id;
                }
            }
            NiriEvent::WindowUrgencyChanged { id, urgent } => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.is_urgent = urgent;
                }
            }
        }

        self.revision += 1;
    }
}

/// Connection health of the Niri observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NiriConnection {
    /// No connection has been established yet.
    Connecting,
    /// Receiving events; state is live.
    Live,
    /// Disconnected; the last known state may be stale.
    Stale {
        /// User-safe reason for the disconnect.
        reason: String,
        /// Consecutive failures since the last successful connection.
        failures: u32,
    },
}

/// Default delay before the first reconnect attempt.
pub const INITIAL_RECONNECT_BACKOFF: Duration = Duration::from_millis(500);

/// Maximum reconnect delay.
pub const MAX_RECONNECT_BACKOFF: Duration = Duration::from_secs(30);

/// Niri state plus connection and backoff policy.
///
/// The observation keeps the last known state while disconnected and marks it
/// stale instead of pretending it is live. Reconnection is driven by the
/// caller; reconnect rebuilds state from the stream's full-state events and
/// never replays actions.
#[derive(Debug, Clone)]
pub struct NiriObservation {
    state: NiriState,
    connection: NiriConnection,
    failures: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
}

impl Default for NiriObservation {
    fn default() -> Self {
        Self::new()
    }
}

impl NiriObservation {
    /// Create an observation with the default backoff policy.
    pub fn new() -> Self {
        Self::with_backoff(INITIAL_RECONNECT_BACKOFF, MAX_RECONNECT_BACKOFF)
    }

    /// Create an observation with an explicit backoff policy.
    pub fn with_backoff(initial_backoff: Duration, max_backoff: Duration) -> Self {
        Self {
            state: NiriState::new(),
            connection: NiriConnection::Connecting,
            failures: 0,
            initial_backoff,
            max_backoff,
        }
    }

    /// Current canonical state, live or stale.
    pub fn state(&self) -> &NiriState {
        &self.state
    }

    /// Current connection health.
    pub fn connection(&self) -> &NiriConnection {
        &self.connection
    }

    /// Whether events are currently being received.
    pub fn is_live(&self) -> bool {
        self.connection == NiriConnection::Live
    }

    /// Consecutive failures since the last successful connection.
    pub fn failures(&self) -> u32 {
        self.failures
    }

    /// Delay to wait before the next reconnect attempt.
    pub fn retry_delay(&self) -> Duration {
        backoff_delay(self.initial_backoff, self.max_backoff, self.failures)
    }

    /// Record a successful connection and reset the backoff.
    pub fn on_connected(&mut self) {
        self.connection = NiriConnection::Live;
        self.failures = 0;
    }

    /// Apply an event from the stream.
    pub fn on_event(&mut self, event: NiriEvent) {
        self.state.apply(event);
    }

    /// Record a disconnect and mark the state stale.
    pub fn on_disconnected(&mut self, reason: impl Into<String>) {
        self.failures = self.failures.saturating_add(1);
        self.connection = NiriConnection::Stale {
            reason: reason.into(),
            failures: self.failures,
        };
    }

    /// Replace the output set, for example after a snapshot refresh.
    pub fn set_outputs(&mut self, outputs: BTreeMap<OutputName, Output>) {
        self.state.set_outputs(outputs);
    }
}

fn backoff_delay(initial: Duration, max: Duration, failures: u32) -> Duration {
    if failures == 0 {
        return Duration::ZERO;
    }

    let factor = 2u32.saturating_pow(failures.saturating_sub(1).min(31));
    let millis = u64::try_from(initial.as_millis())
        .unwrap_or(u64::MAX)
        .saturating_mul(u64::from(factor));
    let cap = u64::try_from(max.as_millis()).unwrap_or(u64::MAX);

    Duration::from_millis(millis.min(cap))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn workspace(id: u64, idx: u32, output: Option<&str>, active: bool, focused: bool) -> Workspace {
        Workspace {
            id: WorkspaceId(id),
            idx,
            name: None,
            output: output.map(|name| OutputName(name.to_owned())),
            is_urgent: false,
            is_active: active,
            is_focused: focused,
            active_window_id: None,
        }
    }

    fn window(id: u64, workspace_id: Option<u64>, focused: bool, urgent: bool) -> Window {
        Window {
            id: WindowId(id),
            title: Some(format!("window {id}")),
            app_id: Some("test".to_owned()),
            workspace_id: workspace_id.map(WorkspaceId),
            is_focused: focused,
            is_floating: false,
            is_urgent: urgent,
        }
    }

    fn initial_state() -> NiriState {
        let mut state = NiriState::new();
        state.apply(NiriEvent::WorkspacesChanged {
            workspaces: vec![
                workspace(1, 1, Some("eDP-1"), true, true),
                workspace(2, 2, Some("eDP-1"), false, false),
            ],
        });
        state.apply(NiriEvent::WindowsChanged {
            windows: vec![window(10, Some(1), true, false), window(11, Some(2), false, true)],
        });
        state
    }

    #[test]
    fn full_state_events_replace_state_and_focus() {
        let state = initial_state();

        assert_eq!(state.workspaces().len(), 2);
        assert_eq!(state.windows().len(), 2);
        assert_eq!(state.focused_window().map(|window| window.id), Some(WindowId(10)));
    }

    #[test]
    fn workspace_activation_clears_siblings_on_the_same_output() {
        let mut state = initial_state();

        state.apply(NiriEvent::WorkspaceActivated {
            id: WorkspaceId(2),
            focused: true,
        });

        let workspaces = state.workspaces();
        assert!(!workspaces[0].is_active);
        assert!(workspaces[1].is_active);
        assert!(!workspaces[0].is_focused);
        assert!(workspaces[1].is_focused);
    }

    #[test]
    fn activation_on_another_output_keeps_other_output_active() {
        let mut state = NiriState::new();
        state.apply(NiriEvent::WorkspacesChanged {
            workspaces: vec![
                workspace(1, 1, Some("eDP-1"), true, true),
                workspace(2, 2, Some("HDMI-A-1"), true, false),
            ],
        });

        state.apply(NiriEvent::WorkspaceActivated {
            id: WorkspaceId(1),
            focused: true,
        });

        assert!(state.workspaces()[0].is_active);
        assert!(state.workspaces()[1].is_active);
    }

    #[test]
    fn window_close_clears_focus_and_active_window() {
        let mut state = initial_state();
        state.apply(NiriEvent::WorkspaceActiveWindowChanged {
            workspace_id: WorkspaceId(1),
            active_window_id: Some(WindowId(10)),
        });

        state.apply(NiriEvent::WindowClosed { id: WindowId(10) });

        assert!(state.focused_window().is_none());
        assert_eq!(state.workspaces()[0].active_window_id, None);
        assert_eq!(state.windows().len(), 1);
    }

    #[test]
    fn focus_changes_update_window_flags() {
        let mut state = initial_state();

        state.apply(NiriEvent::WindowFocusChanged {
            id: Some(WindowId(11)),
        });

        assert_eq!(state.focused_window().map(|window| window.id), Some(WindowId(11)));
        assert!(!state.windows()[&WindowId(10)].is_focused);
        assert!(state.windows()[&WindowId(11)].is_focused);

        state.apply(NiriEvent::WindowFocusChanged { id: None });
        assert!(state.focused_window().is_none());
    }

    #[test]
    fn urgency_events_update_state() {
        let mut state = initial_state();

        state.apply(NiriEvent::WindowUrgencyChanged {
            id: WindowId(10),
            urgent: true,
        });
        state.apply(NiriEvent::WorkspaceUrgencyChanged {
            id: WorkspaceId(1),
            urgent: true,
        });

        assert!(state.windows()[&WindowId(10)].is_urgent);
        assert!(state.workspaces()[0].is_urgent);
    }

    #[test]
    fn revision_advances_on_each_applied_event() {
        let mut state = NiriState::new();
        assert_eq!(state.revision(), 0);

        state.apply(NiriEvent::WindowFocusChanged { id: None });
        assert_eq!(state.revision(), 1);

        state.set_outputs(BTreeMap::new());
        assert_eq!(state.revision(), 2);
    }

    #[test]
    fn backoff_doubles_and_caps() {
        let mut observation = NiriObservation::with_backoff(
            Duration::from_millis(500),
            Duration::from_secs(2),
        );

        assert_eq!(observation.retry_delay(), Duration::ZERO);

        observation.on_disconnected("one");
        assert_eq!(observation.retry_delay(), Duration::from_millis(500));
        observation.on_disconnected("two");
        assert_eq!(observation.retry_delay(), Duration::from_millis(1000));
        observation.on_disconnected("three");
        assert_eq!(observation.retry_delay(), Duration::from_millis(2000));
        observation.on_disconnected("four");
        assert_eq!(observation.retry_delay(), Duration::from_millis(2000));

        observation.on_connected();
        assert_eq!(observation.failures(), 0);
        assert_eq!(observation.retry_delay(), Duration::ZERO);
    }

    #[test]
    fn disconnect_marks_state_stale_and_keeps_last_state() {
        let mut observation = NiriObservation::new();
        observation.on_connected();
        observation.on_event(NiriEvent::WindowFocusChanged {
            id: Some(WindowId(3)),
        });

        observation.on_disconnected("connection lost");

        assert!(!observation.is_live());
        assert_eq!(observation.state().revision(), 1);
        assert!(matches!(
            observation.connection(),
            NiriConnection::Stale { failures: 1, .. }
        ));
    }

    #[test]
    fn reconnect_replaces_state_from_full_events() {
        let mut observation = NiriObservation::new();
        observation.on_connected();
        observation.on_event(NiriEvent::WorkspacesChanged {
            workspaces: vec![workspace(1, 1, Some("eDP-1"), true, true)],
        });
        observation.on_disconnected("lost");

        observation.on_connected();
        observation.on_event(NiriEvent::WorkspacesChanged {
            workspaces: vec![workspace(2, 1, Some("eDP-1"), true, true)],
        });

        assert!(observation.is_live());
        assert_eq!(observation.state().workspaces().len(), 1);
        assert_eq!(observation.state().workspaces()[0].id, WorkspaceId(2));
    }
}
