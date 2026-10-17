//! Launcher overlay view.
//!
//! A layer-shell surface with exclusive keyboard focus: type to search
//! installed applications, Up/Down to select, Enter to launch, Escape to close.
//! Text input is key-event based for now; IME/composition is not handled yet.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, Context, FocusHandle, KeyDownEvent, Render, Size, Window,
    WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind, WindowOptions, div, img,
    layer_shell::*, point, prelude::*, px, svg,
};

use nalios_core::apps::DesktopApp;

use crate::actions::Dismiss;
use crate::theme;

/// Launcher overlay view.
pub struct Launcher {
    focus: FocusHandle,
    open: Rc<Cell<bool>>,
    catalog: Rc<Vec<DesktopApp>>,
    favorites: Rc<Vec<String>>,
    icons: Rc<HashMap<String, PathBuf>>,
    max_results: usize,
    query: String,
    results: Vec<usize>,
    selected: usize,
    error: Option<String>,
}

impl Launcher {
    fn new(
        focus: FocusHandle,
        open: Rc<Cell<bool>>,
        catalog: Rc<Vec<DesktopApp>>,
        favorites: Rc<Vec<String>>,
        icons: Rc<HashMap<String, PathBuf>>,
        max_results: usize,
    ) -> Self {
        let results = nalios_app::apps::rank(&catalog, &favorites, "", max_results);

        Self {
            focus,
            open,
            catalog,
            favorites,
            icons,
            max_results,
            query: String::new(),
            results,
            selected: 0,
            error: None,
        }
    }

    fn refresh(&mut self) {
        self.results =
            nalios_app::apps::rank(&self.catalog, &self.favorites, &self.query, self.max_results);
        self.selected = 0;
        self.error = None;
    }

    fn handle_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "return" | "enter" => {
                self.launch_selected(window, cx);
                return;
            }
            "up" => self.selected = self.selected.saturating_sub(1),
            "down" => {
                if self.selected + 1 < self.results.len() {
                    self.selected += 1;
                }
            }
            "backspace" => {
                if self.query.pop().is_some() {
                    self.refresh();
                }
            }
            _ => {
                if !event.keystroke.modifiers.control && !event.keystroke.modifiers.alt {
                    if let Some(text) = &event.keystroke.key_char {
                        self.query.push_str(text);
                        self.refresh();
                    }
                }
            }
        }

        cx.notify();
    }

    fn launch_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.results.get(self.selected).copied() {
            self.launch_index(index, window, cx);
        }
    }

    fn launch_index(&mut self, index: usize, window: &mut Window, _cx: &mut Context<Self>) {
        let Some(app) = self.catalog.get(index) else {
            return;
        };

        match nalios_platform::apps::launch(app) {
            Ok(()) => {
                self.open.set(false);
                window.remove_window();
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }
}

fn icon_element(path: &Path) -> AnyElement {
    let text = path.to_string_lossy().to_string();

    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
    {
        svg()
            .external_path(text)
            .w(px(20.))
            .h(px(20.))
            .into_any_element()
    } else {
        img(path.to_path_buf())
            .w(px(20.))
            .h(px(20.))
            .into_any_element()
    }
}

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open.clone();
        let entity = cx.entity();

        let results: Vec<_> = self
            .results
            .iter()
            .enumerate()
            .map(|(row, &app_index)| {
                let app = &self.catalog[app_index];
                let selected = row == self.selected;
                let entity = entity.clone();
                let icon = self.icons.get(&app.id).map(|path| icon_element(path));

                let mut element = div()
                    .id(format!("result-{row}"))
                    .w_full()
                    .px_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded_md()
                    .when(selected, |this| this.bg(theme::elevated()));

                if let Some(icon) = icon {
                    element = element.child(icon);
                }

                element
                    .child(app.name.clone())
                    .on_click(move |_event, window, cx| {
                        entity.update(cx, |this, cx| this.launch_index(app_index, window, cx));
                    })
            })
            .collect::<Vec<_>>();

        let input = if self.query.is_empty() {
            div()
                .text_color(theme::text_muted())
                .child("Search applications…")
        } else {
            div().child(self.query.clone())
        };

        let detail = self
            .results
            .get(self.selected)
            .and_then(|&index| self.catalog.get(index))
            .map(|app| {
                app.comment
                    .clone()
                    .unwrap_or_else(|| app.exec.join(" "))
            });

        let detail_element = if self.results.is_empty() {
            div()
                .text_color(theme::text_muted())
                .text_sm()
                .child("No applications found")
        } else {
            div()
                .text_color(theme::text_muted())
                .text_sm()
                .child(detail.unwrap_or_default())
        };

        let mut root = div()
            .on_action(move |_: &Dismiss, window, _| {
                open.set(false);
                window.remove_window();
            })
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::handle_key))
            .flex()
            .flex_col()
            .gap_2()
            .w(px(560.))
            .h(px(360.))
            .p_4()
            .bg(theme::surface())
            .border_1()
            .border_color(theme::border())
            .rounded_xl()
            .text_color(theme::text())
            .child(input)
            .child(div().flex().flex_col().gap_2().children(results))
            .child(detail_element);

        if let Some(error) = &self.error {
            root = root.child(div().text_color(theme::error()).child(error.clone()));
        }

        root
    }
}

/// Layer-shell options for the launcher overlay.
pub fn launcher_options() -> WindowOptions {
    WindowOptions {
        titlebar: None,
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(0.), px(0.)),
            size: Size::new(px(560.), px(360.)),
        })),
        app_id: Some("nalios-shell-launcher".to_string()),
        window_background: WindowBackgroundAppearance::Transparent,
        kind: WindowKind::LayerShell(LayerShellOptions {
            namespace: "nalios-launcher".to_string(),
            layer: Layer::Overlay,
            anchor: Anchor::TOP,
            margin: Some((px(80.), px(0.), px(0.), px(0.))),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Open the launcher if it is not already open.
///
/// Returns the window handle so the shell can close it later.
pub fn open_launcher(
    cx: &mut App,
    open: Rc<Cell<bool>>,
    catalog: Rc<Vec<DesktopApp>>,
    favorites: Rc<Vec<String>>,
    icons: Rc<HashMap<String, PathBuf>>,
    max_results: usize,
) -> Option<WindowHandle<Launcher>> {
    if open.get() {
        return None;
    }
    open.set(true);

    cx.open_window(launcher_options(), move |window, cx| {
        let open = open.clone();
        let catalog = catalog.clone();
        let favorites = favorites.clone();
        let icons = icons.clone();

        cx.new(move |cx| {
            let focus = cx.focus_handle();
            focus.focus(window, cx);
            Launcher::new(focus, open, catalog, favorites, icons, max_results)
        })
    })
    .ok()
}
