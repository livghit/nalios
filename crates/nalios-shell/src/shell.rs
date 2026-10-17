//! Root shell view: the bar.
//!
//! The bar owns the canonical Niri observation for now and renders workspace
//! indicators, a connection dot, a launcher button and the clock. It polls the
//! worker channel on a timer and never blocks on the socket.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use gpui::{
    App, Bounds, Context, Render, Size, Window, WindowBackgroundAppearance, WindowBounds,
    WindowHandle, WindowKind, WindowOptions, div, layer_shell::*, point, prelude::*, px,
};

use nalios_app::config::ConfigSource;
use nalios_app::control::ControlAction;
use nalios_app::niri::NiriObservation;
use nalios_core::apps::DesktopApp;
use nalios_core::config::Config;

use crate::control::ControlWorker;
use crate::dashboard::{self, Dashboard};
use crate::launcher::{self, Launcher};
use crate::theme;
use crate::worker::{NiriMessage, NiriWorker};

/// Poll interval for worker messages and the clock.
const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Root shell view (the bar).
pub struct Shell {
    observation: NiriObservation,
    worker: NiriWorker,
    control: ControlWorker,
    launcher: Option<WindowHandle<Launcher>>,
    launcher_open: Rc<Cell<bool>>,
    dashboard: Option<WindowHandle<Dashboard>>,
    dashboard_open: Rc<Cell<bool>>,
    catalog: Rc<Vec<DesktopApp>>,
    favorites: Rc<Vec<String>>,
    icons: Rc<HashMap<String, PathBuf>>,
    max_results: usize,
}

impl Shell {
    /// Create the shell, start the Niri worker and schedule UI updates.
    pub fn new(cx: &mut Context<Self>, launcher_open: Rc<Cell<bool>>) -> Self {
        let worker = NiriWorker::spawn();
        let control = ControlWorker::spawn();

        let config = load_config();
        let catalog = Rc::new(nalios_platform::apps::AppCatalog::discover().apps().to_vec());
        let icons = Rc::new(build_icon_map(&catalog));

        cx.spawn(async move |this, cx| {
            loop {
                let _ = this.update(cx, |this, cx| {
                    this.drain_messages();
                    this.drain_actions(cx);
                    this.sync_dashboard(cx);
                    cx.notify();
                });
                cx.background_executor().timer(POLL_INTERVAL).await;
            }
        })
        .detach();

        Self {
            observation: NiriObservation::new(),
            worker,
            control,
            launcher: None,
            launcher_open,
            dashboard: None,
            dashboard_open: Rc::new(Cell::new(false)),
            catalog,
            favorites: Rc::new(config.launcher.favorites),
            icons,
            max_results: config.launcher.max_results as usize,
        }
    }

    fn drain_actions(&mut self, cx: &mut Context<Self>) {
        loop {
            let action = self.control.actions().try_recv();
            match action {
                Ok(ControlAction::Ping) => {}
                Ok(ControlAction::OpenLauncher) => self.open_launcher(cx),
                Ok(ControlAction::ToggleLauncher) => self.toggle_launcher(cx),
                Ok(ControlAction::ToggleDashboard) => self.toggle_dashboard(cx),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
    }

    fn open_launcher(&mut self, cx: &mut Context<Self>) {
        if self.launcher.is_some() {
            return;
        }
        self.launcher = launcher::open_launcher(
            cx,
            self.launcher_open.clone(),
            self.catalog.clone(),
            self.favorites.clone(),
            self.icons.clone(),
            self.max_results,
        );
    }

    fn toggle_launcher(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.launcher.take() {
            if handle
                .update(cx, |_, window, _| window.remove_window())
                .is_ok()
            {
                self.launcher_open.set(false);
                return;
            }
        }
        self.open_launcher(cx);
    }

    fn sync_dashboard(&mut self, cx: &mut Context<Self>) {
        let Some(handle) = &self.dashboard else {
            return;
        };

        let observation = self.observation.clone();
        let _ = handle.update(cx, |dashboard, _, cx| {
            dashboard.set_observation(observation);
            cx.notify();
        });
    }

    fn open_dashboard(&mut self, cx: &mut Context<Self>) {
        if self.dashboard.is_some() {
            return;
        }
        self.dashboard = dashboard::open_dashboard(
            cx,
            self.dashboard_open.clone(),
            self.observation.clone(),
        );
    }

    fn toggle_dashboard(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.dashboard.take() {
            if handle
                .update(cx, |_, window, _| window.remove_window())
                .is_ok()
            {
                self.dashboard_open.set(false);
                return;
            }
        }
        self.open_dashboard(cx);
    }

    fn drain_messages(&mut self) {
        loop {
            let message = self.worker.messages().try_recv();
            match message {
                Ok(NiriMessage::Connected) => self.observation.on_connected(),
                Ok(NiriMessage::Disconnected) => {
                    self.observation.on_disconnected("Niri stream disconnected");
                }
                Ok(NiriMessage::Event(event)) => self.observation.on_event(event),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
    }
}

fn load_config() -> Config {
    let Some(path) = nalios_platform::files::default_config_path() else {
        return Config::default();
    };

    let source = nalios_platform::files::FsConfigSource;
    let Ok(document) = source.read(&path) else {
        return Config::default();
    };

    nalios_core::config::parse(&document).unwrap_or_default()
}

fn build_icon_map(catalog: &[DesktopApp]) -> HashMap<String, PathBuf> {
    catalog
        .iter()
        .filter_map(|app| {
            let icon = app.icon.as_deref()?;
            let path = nalios_platform::apps::find_icon(icon)?;
            Some((app.id.clone(), path))
        })
        .collect()
}

fn clock_text() -> String {
    match time::OffsetDateTime::now_local() {
        Ok(now) => format!("{:02}:{:02}", now.hour(), now.minute()),
        Err(_) => "--:--".to_owned(),
    }
}

fn bar_button(label: &str, on_click: impl Fn(&mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .id(label.to_string())
        .px_2()
        .rounded_md()
        .bg(theme::elevated())
        .cursor_pointer()
        .active(|this| this.opacity(0.8))
        .child(label.to_string())
        .on_click(move |_, window, cx| on_click(window, cx))
}

/// Layer-shell options for the top bar.
pub fn bar_options() -> WindowOptions {
    WindowOptions {
        titlebar: None,
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(0.), px(0.)),
            size: Size::new(px(1920.), px(theme::BAR_HEIGHT)),
        })),
        app_id: Some("nalios-shell-bar".to_string()),
        window_background: WindowBackgroundAppearance::Transparent,
        kind: WindowKind::LayerShell(LayerShellOptions {
            namespace: "nalios-bar".to_string(),
            layer: Layer::Top,
            anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            exclusive_zone: Some(px(theme::BAR_HEIGHT)),
            keyboard_interactivity: KeyboardInteractivity::None,
            ..Default::default()
        }),
        ..Default::default()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspaces = self
            .observation
            .state()
            .workspaces()
            .iter()
            .map(|workspace| {
                div()
                    .px_2()
                    .rounded_md()
                    .when(workspace.is_active, |this| this.bg(theme::elevated()))
                    .when(workspace.is_focused, |this| this.text_color(theme::accent()))
                    .child(format!("{}", workspace.idx))
            })
            .collect::<Vec<_>>();

        let status_color = if self.observation.is_live() {
            theme::success()
        } else {
            theme::error()
        };

        let shell = cx.entity();
        let launcher = bar_button("launcher", move |_window, cx| {
            shell.update(cx, |this, cx| this.toggle_launcher(cx));
        });

        div()
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_2()
            .bg(theme::background())
            .text_color(theme::text())
            .text_sm()
            .child(div().flex().items_center().gap_2().children(workspaces))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().w(px(8.)).h(px(8.)).rounded_md().bg(status_color))
                    .child(launcher)
                    .child(clock_text()),
            )
    }
}
