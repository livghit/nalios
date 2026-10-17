# 0008 — Desktop-entry parser dependency

Status: **accepted and implemented** on 2026-10-16. Adds one dependency
(`freedesktop-desktop-entry`) for the application catalog.

## Context

The launcher needs correct desktop-entry discovery and `Exec` parsing
(quoting, field codes, localization, `TryExec`, `OnlyShowIn`). The
[source register](../blueprint/sources.md) listed
`freedesktop-desktop-entry` (MPL-2.0) as the candidate; the architecture warns
against splitting `Exec` as a shell string.

## Decision

- Add `freedesktop-desktop-entry` 0.8.3 with **default features disabled** so
  the optional native `gettext-rs` dependency is not pulled in.
- Published license: **MPL-2.0** (file-level copyleft). NaliOS does not modify
  the crate; distribution must include its license notice and, if the crate is
  ever modified, the modified files' source.
- Map parsed entries into a pure `nalios-core::apps::DesktopApp`; launching uses
  explicit argv (`std::process::Command`), never a shell string. Terminal
  applications are wrapped with `xdg-terminal-exec`.
- Discovery filters `Type=Application`, `Hidden`, `NoDisplay`, `OnlyShowIn`/
  `NotShowIn` against `XDG_CURRENT_DESKTOP`, and `TryExec` availability;
  earlier XDG data directories win on duplicate ids.

## Evidence

- 124 workspace tests pass offline, including entry mapping, desktop filtering,
  `TryExec` checks, missing/empty launch commands and ranking.
- `nalios-dev apps list` discovers **288 applications** on the author's Kali
  system; `apps search fire` ranks Firefox ESR first.

## Consequences

- The MPL-2.0 notice joins the release notice/SBOM review.
- Text input in the launcher is key-event based; IME/composition and
  `StartupNotify`/D-Bus activation semantics are not implemented yet.
- Icons are not rendered yet (`freedesktop-icons` is a candidate).
