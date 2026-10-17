# 0009 — Icon lookup dependency

Status: **accepted and implemented** on 2026-10-16. Adds one dependency
(`freedesktop-icons`) for launcher icons.

## Context

Launcher results should show application icons. Icon themes are non-trivial
(theme inheritance, size/scale directories, SVG and PNG fallbacks). The
[source register](../blueprint/sources.md) listed `freedesktop-icons` (MIT) as
a candidate.

## Decision

- Add `freedesktop-icons` 0.4.0 (MIT) to `nalios-platform`.
- Resolve the current theme once per process via the crate's
  `default_theme_gtk()` (which calls `gsettings get
  org.gnome.desktop.interface icon-theme`) and cache it in a `OnceLock`.
  Explicitly pass the theme to the lookup: the crate's default lookup otherwise
  falls back to `hicolor` and misses theme icons.
- Absolute `Icon=` paths are used directly when they exist.
- The shell precomputes an app-id → icon-path map at startup; rendering uses
  GPUI `svg().external_path(...)` for SVG and `img(path)` otherwise.

## Evidence

- 129 workspace tests pass offline.
- Host check (ignored by default): `utilities-terminal` resolves to
  `/usr/share/icons/Flat-Remix-Blue-Dark/apps/scalable/utilities-terminal.svg`
  under the session's `Flat-Remix-Teal-Dark` theme.

## Consequences

- One `gsettings` subprocess at shell startup; icon lookups are then cached.
- Icons are not yet scaled per monitor/theme setting beyond the 48 px request;
  missing icons simply render no image.
