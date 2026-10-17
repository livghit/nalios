//! Draft semantic theme tokens for the shell.
//!
//! Values come from the proposed Night palette in `docs/blueprint/themes.md`.
//! They are placeholders until the real theme system (C08) lands; widgets use
//! roles, not palette names, so replacing them later is mechanical.

use gpui::{Rgba, rgb, rgba};

/// Bar height in logical pixels.
pub const BAR_HEIGHT: f32 = 36.0;

/// Window background with slight transparency.
pub fn background() -> Rgba {
    rgba(0x11161CE6)
}

/// Elevated panel surface.
pub fn surface() -> Rgba {
    rgba(0x19222CF2)
}

/// Control background.
pub fn elevated() -> Rgba {
    rgba(0x243140FF)
}

/// Primary text.
pub fn text() -> Rgba {
    rgb(0xE8EEF5)
}

/// Secondary text.
pub fn text_muted() -> Rgba {
    rgb(0xAAB7C6)
}

/// Accent (focused workspace, active states).
pub fn accent() -> Rgba {
    rgb(0x6CD9C8)
}

/// Success/connected state.
pub fn success() -> Rgba {
    rgb(0x8FDFAC)
}

/// Error/disconnected state.
pub fn error() -> Rgba {
    rgb(0xFFB4AB)
}

/// Structural border.
pub fn border() -> Rgba {
    rgb(0x64788D)
}
