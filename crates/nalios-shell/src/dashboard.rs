//! Full dashboard overlay view.
//!
//! A read-only overview of Niri state, toggled with `Super+Shift+Space` (or the
//! control command) and closed with Escape: workspaces with their windows, the
//! focused window, outputs and session health.

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    App, Bounds, Context, FocusHandle, Render, Size, Window, WindowBackgroundAppearance,
    WindowBounds, WindowHandle, WindowKind, WindowOptions, div, layer_shell::*, point, prelude::*,
    px, rgba,
};

use nalios_app::niri::{NiriConnection, NiriObservation};

use crate::actions::Dismiss;
use crate::theme;

/// Dashboard overlay view.
pub struct Dashboard {
    focus: FocusHandle,
    open: Rc<Cell<bool>>,
    observation: NiriObservation,
}

impl Dashboard {
    /// Replace the displayed snapshot.
    pub fn set_observation(&mut self, observation: NiriObservation) {
        self.observation = observation;
    }
}

fn card(title: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_4()
        .bg(theme::surface())
        .rounded_xl()
        .border_1()
        .border_color(theme::border())
        .child(
            div()
                .text_sm()
                .text_color(theme::text_muted())
                .child(title.to_owned()),
        )
        .child(content)
}

fn datetime_text() -> String {
    match time::OffsetDateTime::now_local() {
        Ok(now) => format!(
            "{:04}-{:02}-{:02}  {:02}:{:02}",
            now.year(),
            u8::from(now.month()),
            now.day(),
            now.hour(),
            now.minute()
        ),
        Err(_) => "time unavailable".to_owned(),
    }
}

impl Render for Dashboard {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open.clone();
        let state = self.observation.state();

        let workspace_rows = state
            .workspaces()
            .iter()
            .map(|workspace| {
                let mut title = format!("{}", workspace.idx);
                if let Some(name) = &workspace.name {
                    title.push_str(&format!("  {name}"));
                }
                if let Some(output) = &workspace.output {
                    title.push_str(&format!("  ·  {output}"));
                }

                let windows: Vec<_> = state
                    .windows()
                    .values()
                    .filter(|window| window.workspace_id == Some(workspace.id))
                    .map(|window| {
                        let name = window
                            .app_id
                            .clone()
                            .or_else(|| window.title.clone())
                            .unwrap_or_else(|| "window".to_owned());
                        div()
                            .text_sm()
                            .text_color(theme::text_muted())
                            .child(format!("• {name}"))
                    })
                    .collect();

                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .px_2()
                    .rounded_md()
                    .when(workspace.is_active, |this| this.bg(theme::elevated()))
                    .child(
                        div()
                            .when(workspace.is_focused, |this| this.text_color(theme::accent()))
                            .child(title),
                    )
                    .children(windows)
            })
            .collect::<Vec<_>>();

        let focused = match state.focused_window() {
            Some(window) => {
                let app = window
                    .app_id
                    .clone()
                    .unwrap_or_else(|| "unknown".to_owned());
                let title = window
                    .title
                    .clone()
                    .unwrap_or_else(|| "(untitled)".to_owned());
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_color(theme::accent()).child(app))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::text_muted())
                            .child(title),
                    )
            }
            None => div()
                .text_color(theme::text_muted())
                .child("No focused window"),
        };

        let outputs = state
            .outputs()
            .values()
            .map(|output| {
                let text = match &output.logical {
                    Some(logical) => format!(
                        "{}  {}x{} @ {:.2}",
                        output.name, logical.width, logical.height, logical.scale
                    ),
                    None => format!("{} (disabled)", output.name),
                };
                div().text_sm().child(text)
            })
            .collect::<Vec<_>>();

        let connection = match self.observation.connection() {
            NiriConnection::Live => "Niri: live",
            NiriConnection::Connecting => "Niri: connecting",
            NiriConnection::Stale { .. } => "Niri: stale",
        };
        let session = div()
            .flex()
            .flex_col()
            .gap_1()
            .child(div().text_sm().child(connection.to_owned()))
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_muted())
                    .child(format!(
                        "{} windows · {} workspaces",
                        state.windows().len(),
                        state.workspaces().len()
                    )),
            );

        div()
            .on_action(move |_: &Dismiss, window, _| {
                open.set(false);
                window.remove_window();
            })
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .bg(rgba(0x0B0F14F2))
            .text_color(theme::text())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(div().text_size(px(24.)).child("NaliOS"))
                    .child(
                        div()
                            .text_color(theme::text_muted())
                            .child(datetime_text()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(
                        div()
                            .w(px(460.))
                            .child(card(
                                "Workspaces",
                                div().flex().flex_col().gap_2().children(workspace_rows),
                            )),
                    )
                    .child(
                        div()
                            .w(px(320.))
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(card("Focused", focused))
                            .child(card(
                                "Outputs",
                                div().flex().flex_col().gap_1().children(outputs),
                            ))
                            .child(card("Session", session)),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_muted())
                    .child("Escape closes · Super+Shift+Space toggles"),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datetime_text_is_never_empty() {
        assert!(!datetime_text().is_empty());
    }
}

/// Layer-shell options for the full dashboard.
pub fn dashboard_options() -> WindowOptions {
    WindowOptions {
        titlebar: None,
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(0.), px(0.)),
            size: Size::new(px(1920.), px(1080.)),
        })),
        app_id: Some("nalios-shell-dashboard".to_string()),
        window_background: WindowBackgroundAppearance::Transparent,
        kind: WindowKind::LayerShell(LayerShellOptions {
            namespace: "nalios-dashboard".to_string(),
            layer: Layer::Overlay,
            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Open the dashboard if it is not already open.
pub fn open_dashboard(
    cx: &mut App,
    open: Rc<Cell<bool>>,
    observation: NiriObservation,
) -> Option<WindowHandle<Dashboard>> {
    if open.get() {
        return None;
    }
    open.set(true);

    cx.open_window(dashboard_options(), move |window, cx| {
        let open = open.clone();
        cx.new(move |cx| {
            let focus = cx.focus_handle();
            focus.focus(window, cx);
            Dashboard {
                focus,
                open,
                observation,
            }
        })
    })
    .ok()
}
