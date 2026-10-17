//! Wire types and reply/event parsing for the documented Niri IPC protocol.
//!
//! Wire structs mirror JSON shapes and convert into pure core domain values.
//! Unknown fields are ignored. Unknown events and reply acknowledgements are
//! filtered out so a newer compositor cannot break the client.

use std::collections::BTreeMap;

use nalios_app::niri::NiriEvent;
use nalios_core::niri::{
    LogicalOutput, Output, OutputName, Window, WindowId, Workspace, WorkspaceId,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WireWorkspace {
    id: u64,
    idx: u32,
    name: Option<String>,
    output: Option<String>,
    is_urgent: bool,
    is_active: bool,
    is_focused: bool,
    active_window_id: Option<u64>,
}

impl From<WireWorkspace> for Workspace {
    fn from(wire: WireWorkspace) -> Self {
        Self {
            id: WorkspaceId(wire.id),
            idx: wire.idx,
            name: wire.name,
            output: wire.output.map(OutputName),
            is_urgent: wire.is_urgent,
            is_active: wire.is_active,
            is_focused: wire.is_focused,
            active_window_id: wire.active_window_id.map(WindowId),
        }
    }
}

#[derive(Debug, Deserialize)]
struct WireWindow {
    id: u64,
    title: Option<String>,
    app_id: Option<String>,
    workspace_id: Option<u64>,
    is_focused: bool,
    is_floating: bool,
    is_urgent: bool,
}

impl From<WireWindow> for Window {
    fn from(wire: WireWindow) -> Self {
        Self {
            id: WindowId(wire.id),
            title: wire.title,
            app_id: wire.app_id,
            workspace_id: wire.workspace_id.map(WorkspaceId),
            is_focused: wire.is_focused,
            is_floating: wire.is_floating,
            is_urgent: wire.is_urgent,
        }
    }
}

#[derive(Debug, Deserialize)]
struct WireOutput {
    name: String,
    logical: Option<WireLogicalOutput>,
}

impl From<WireOutput> for Output {
    fn from(wire: WireOutput) -> Self {
        Self {
            name: OutputName(wire.name),
            logical: wire.logical.map(Into::into),
        }
    }
}

#[derive(Debug, Deserialize)]
struct WireLogicalOutput {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale: f64,
    transform: String,
}

impl From<WireLogicalOutput> for LogicalOutput {
    fn from(wire: WireLogicalOutput) -> Self {
        Self {
            x: wire.x,
            y: wire.y,
            width: wire.width,
            height: wire.height,
            scale: wire.scale,
            transform: wire.transform,
        }
    }
}

/// A wire-level failure.
#[derive(Debug, thiserror::Error)]
pub enum WireError {
    /// Niri returned an error reply.
    #[error("Niri returned an error: {0}")]
    Remote(String),
    /// The reply or event was not valid JSON for the expected shape.
    #[error("unexpected Niri message: {0}")]
    Malformed(#[source] serde_json::Error),
    /// The message was not a JSON object or carried no event.
    #[error("unexpected Niri message shape")]
    MalformedEvent,
    /// The message was not valid UTF-8.
    #[error("Niri message was not valid UTF-8")]
    InvalidUtf8,
    /// The message exceeded the accepted size limit.
    #[error("Niri message exceeded the {0}-byte limit")]
    TooLarge(usize),
}

#[derive(Debug, Deserialize)]
enum Reply<T> {
    Ok(T),
    Err(String),
}

#[derive(Debug, Deserialize)]
struct WorkspacesReply {
    #[serde(rename = "Workspaces")]
    workspaces: Vec<WireWorkspace>,
}

#[derive(Debug, Deserialize)]
struct WindowsReply {
    #[serde(rename = "Windows")]
    windows: Vec<WireWindow>,
}

#[derive(Debug, Deserialize)]
struct OutputsReply {
    #[serde(rename = "Outputs")]
    outputs: BTreeMap<String, WireOutput>,
}

#[derive(Debug, Deserialize)]
struct FocusedWindowReply {
    #[serde(rename = "FocusedWindow")]
    window: Option<WireWindow>,
}

#[derive(Debug, Deserialize)]
struct WorkspacesChanged {
    workspaces: Vec<WireWorkspace>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceUrgencyChanged {
    id: u64,
    urgent: bool,
}

#[derive(Debug, Deserialize)]
struct WorkspaceActivated {
    id: u64,
    focused: bool,
}

#[derive(Debug, Deserialize)]
struct WorkspaceActiveWindowChanged {
    workspace_id: u64,
    active_window_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct WindowsChanged {
    windows: Vec<WireWindow>,
}

#[derive(Debug, Deserialize)]
struct WindowOpenedOrChanged {
    window: WireWindow,
}

#[derive(Debug, Deserialize)]
struct WindowClosed {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct WindowFocusChanged {
    id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct WindowUrgencyChanged {
    id: u64,
    urgent: bool,
}

fn parse_reply<T>(json: &str) -> Result<T, WireError>
where
    T: for<'de> Deserialize<'de>,
{
    match serde_json::from_str::<Reply<T>>(json).map_err(WireError::Malformed)? {
        Reply::Ok(value) => Ok(value),
        Reply::Err(message) => Err(WireError::Remote(message)),
    }
}

fn from_value<T>(value: &serde_json::Value) -> Result<T, WireError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(value.clone()).map_err(WireError::Malformed)
}

pub(crate) fn parse_workspaces(json: &str) -> Result<Vec<Workspace>, WireError> {
    Ok(parse_reply::<WorkspacesReply>(json)?
        .workspaces
        .into_iter()
        .map(Into::into)
        .collect())
}

pub(crate) fn parse_windows(json: &str) -> Result<Vec<Window>, WireError> {
    Ok(parse_reply::<WindowsReply>(json)?
        .windows
        .into_iter()
        .map(Into::into)
        .collect())
}

pub(crate) fn parse_outputs(json: &str) -> Result<BTreeMap<OutputName, Output>, WireError> {
    Ok(parse_reply::<OutputsReply>(json)?
        .outputs
        .into_iter()
        .map(|(_, wire)| {
            let output = Output::from(wire);
            (output.name.clone(), output)
        })
        .collect())
}

pub(crate) fn parse_focused_window(json: &str) -> Result<Option<Window>, WireError> {
    Ok(parse_reply::<FocusedWindowReply>(json)?
        .window
        .map(Into::into))
}

/// Parse one event-stream line.
///
/// Returns `Ok(None)` for reply acknowledgements and events this build does
/// not model, so callers can keep reading without failing.
pub(crate) fn parse_event(json: &str) -> Result<Option<NiriEvent>, WireError> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(WireError::Malformed)?;
    let Some(object) = value.as_object() else {
        return Err(WireError::MalformedEvent);
    };

    if object.contains_key("Ok") {
        return Ok(None);
    }
    if let Some(message) = object.get("Err") {
        let message = message
            .as_str()
            .unwrap_or("unknown error")
            .to_owned();
        return Err(WireError::Remote(message));
    }

    let Some((name, payload)) = object.iter().next() else {
        return Err(WireError::MalformedEvent);
    };

    let event = match name.as_str() {
        "WorkspacesChanged" => {
            let data: WorkspacesChanged = from_value(payload)?;
            NiriEvent::WorkspacesChanged {
                workspaces: data.workspaces.into_iter().map(Into::into).collect(),
            }
        }
        "WorkspaceUrgencyChanged" => {
            let data: WorkspaceUrgencyChanged = from_value(payload)?;
            NiriEvent::WorkspaceUrgencyChanged {
                id: WorkspaceId(data.id),
                urgent: data.urgent,
            }
        }
        "WorkspaceActivated" => {
            let data: WorkspaceActivated = from_value(payload)?;
            NiriEvent::WorkspaceActivated {
                id: WorkspaceId(data.id),
                focused: data.focused,
            }
        }
        "WorkspaceActiveWindowChanged" => {
            let data: WorkspaceActiveWindowChanged = from_value(payload)?;
            NiriEvent::WorkspaceActiveWindowChanged {
                workspace_id: WorkspaceId(data.workspace_id),
                active_window_id: data.active_window_id.map(WindowId),
            }
        }
        "WindowsChanged" => {
            let data: WindowsChanged = from_value(payload)?;
            NiriEvent::WindowsChanged {
                windows: data.windows.into_iter().map(Into::into).collect(),
            }
        }
        "WindowOpenedOrChanged" => {
            let data: WindowOpenedOrChanged = from_value(payload)?;
            NiriEvent::WindowOpenedOrChanged {
                window: data.window.into(),
            }
        }
        "WindowClosed" => {
            let data: WindowClosed = from_value(payload)?;
            NiriEvent::WindowClosed {
                id: WindowId(data.id),
            }
        }
        "WindowFocusChanged" => {
            let data: WindowFocusChanged = from_value(payload)?;
            NiriEvent::WindowFocusChanged {
                id: data.id.map(WindowId),
            }
        }
        "WindowUrgencyChanged" => {
            let data: WindowUrgencyChanged = from_value(payload)?;
            NiriEvent::WindowUrgencyChanged {
                id: WindowId(data.id),
                urgent: data.urgent,
            }
        }
        _ => return Ok(None),
    };

    Ok(Some(event))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn parses_workspaces_and_ignores_unknown_fields() {
        let json = r#"{"Ok":{"Workspaces":[{"id":1,"idx":1,"name":null,"output":"eDP-1","is_urgent":false,"is_active":true,"is_focused":true,"active_window_id":null,"future_field":42}]}}"#;

        let workspaces = parse_workspaces(json).expect("parses");

        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].id, WorkspaceId(1));
        assert_eq!(workspaces[0].output.as_ref().map(OutputName::as_str), Some("eDP-1"));
        assert!(workspaces[0].is_focused);
    }

    #[test]
    fn parses_windows_with_optional_fields_missing() {
        let json = r#"{"Ok":{"Windows":[{"id":5,"is_focused":false,"is_floating":true,"is_urgent":false}]}}"#;

        let windows = parse_windows(json).expect("parses");

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].id, WindowId(5));
        assert!(windows[0].title.is_none());
        assert!(windows[0].app_id.is_none());
        assert!(windows[0].is_floating);
    }

    #[test]
    fn parses_outputs_map() {
        let json = r#"{"Ok":{"Outputs":{"eDP-1":{"name":"eDP-1","logical":{"x":0,"y":0,"width":1920,"height":1080,"scale":1.0,"transform":"normal"}}}}}"#;

        let outputs = parse_outputs(json).expect("parses");

        assert_eq!(outputs.len(), 1);
        let output = outputs.values().next().expect("output");
        assert_eq!(output.name.as_str(), "eDP-1");
        let logical = output.logical.as_ref().expect("logical geometry");
        assert_eq!((logical.width, logical.height), (1920, 1080));
    }

    #[test]
    fn parses_focused_window_replies() {
        assert!(
            parse_focused_window(r#"{"Ok":{"FocusedWindow":null}}"#)
                .expect("parses")
                .is_none()
        );

        let json = r#"{"Ok":{"FocusedWindow":{"id":5,"title":"Terminal","app_id":"alacritty","workspace_id":1,"is_focused":true,"is_floating":false,"is_urgent":false}}}"#;
        let window = parse_focused_window(json).expect("parses").expect("window");
        assert_eq!(window.id, WindowId(5));
        assert_eq!(window.title.as_deref(), Some("Terminal"));
    }

    #[test]
    fn remote_errors_are_reported() {
        let error = parse_workspaces(r#"{"Err":"no such workspace"}"#).expect_err("remote error");

        assert!(matches!(error, WireError::Remote(message) if message == "no such workspace"));
    }

    #[test]
    fn malformed_replies_are_reported() {
        assert!(matches!(
            parse_workspaces("not json"),
            Err(WireError::Malformed(_))
        ));
        assert!(matches!(
            parse_workspaces(r#"{"Ok":{"SomethingElse":[]}}"#),
            Err(WireError::Malformed(_))
        ));
        assert!(matches!(
            parse_workspaces(r#"{"Ok":{"Workspaces":[{"id":1}]}}"#),
            Err(WireError::Malformed(_))
        ));
    }

    #[test]
    fn acknowledgements_and_unknown_events_are_filtered() {
        assert!(parse_event(r#"{"Ok":"Handled"}"#).expect("ack").is_none());
        assert!(
            parse_event(r#"{"KeyboardLayoutsChanged":{"keyboard_layouts":{"names":["us"],"current_idx":0}}}"#)
                .expect("unknown event")
                .is_none()
        );
    }

    #[test]
    fn parses_workspace_events() {
        let json = r#"{"WorkspaceActivated":{"id":2,"focused":true}}"#;

        let event = parse_event(json).expect("parses").expect("event");

        assert_eq!(
            event,
            NiriEvent::WorkspaceActivated {
                id: WorkspaceId(2),
                focused: true
            }
        );
    }

    #[test]
    fn parses_window_events() {
        let closed = parse_event(r#"{"WindowClosed":{"id":7}}"#)
            .expect("parses")
            .expect("event");
        assert_eq!(closed, NiriEvent::WindowClosed { id: WindowId(7) });

        let focus = parse_event(r#"{"WindowFocusChanged":{"id":null}}"#)
            .expect("parses")
            .expect("event");
        assert_eq!(focus, NiriEvent::WindowFocusChanged { id: None });

        let changed = parse_event(
            r#"{"WindowOpenedOrChanged":{"window":{"id":5,"title":"T","app_id":"a","workspace_id":1,"is_focused":true,"is_floating":false,"is_urgent":false}}}"#,
        )
        .expect("parses")
        .expect("event");
        assert!(matches!(
            changed,
            NiriEvent::WindowOpenedOrChanged { window } if window.id == WindowId(5)
        ));
    }

    #[test]
    fn event_errors_are_reported() {
        assert!(matches!(
            parse_event(r#"{"Err":"stream failed"}"#),
            Err(WireError::Remote(message)) if message == "stream failed"
        ));
        assert!(matches!(
            parse_event("[]"),
            Err(WireError::MalformedEvent)
        ));
    }
}
