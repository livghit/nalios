//! NaliOS development shell (GPUI).
//!
//! Development binary; not installed over any production command. It opens the
//! layer-shell bar, launcher and dashboard and streams live Niri state through
//! our own client. See `crates/nalios-shell/README.md`.

mod actions;
mod control;
mod dashboard;
mod launcher;
mod shell;
mod theme;
mod worker;

use std::cell::Cell;
use std::rc::Rc;

use gpui::{App, AppContext as _, KeyBinding};
use gpui_platform::application;

use crate::actions::Dismiss;
use crate::shell::{Shell, bar_options};

fn main() {
    application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("escape", Dismiss, None)]);

        let launcher_open = Rc::new(Cell::new(false));

        let open = launcher_open.clone();
        cx.open_window(bar_options(), move |_window, cx| {
            cx.new(|cx| Shell::new(cx, open.clone()))
        })
        .expect("open bar window");

        cx.activate(true);
    });
}
