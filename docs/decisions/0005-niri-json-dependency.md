# 0005 — Niri protocol JSON dependency

Status: **selected for private development** on 2026-10-16, following the
permissive-license review pattern of [decision 0004](0004-core-config-dependencies.md).
Revisit before distribution.

## Context

The independent Niri IPC client exchanges newline-delimited JSON. It needs
typed parsing of replies with forward-compatible handling of unknown fields
and bounded error reporting. Hand-writing JSON parsing would add risk without
product benefit.

## Decision

Add `serde_json` 1.0.149 (published license **MIT OR Apache-2.0**) as a normal
dependency of `nalios-platform`; it resolves to locked version 1.0.154 with
transitive `itoa` 1.0.18, `memchr` 2.8.3 and `zmij` 1.0.23. `serde` (already
selected) provides the derives.

No GPL/AGPL or unknown-license crate is involved. The GPL `niri-ipc` and
`niri-config` crates remain **not** application dependencies; the client is
authored independently from public documentation and recorded observations.

## Evidence

- Read-only protocol observation against the running Niri 26.04 session:
  requests `"Workspaces"`/`"FocusedWindow"` use the `{"Ok": ...}` / `{"Err": ...}`
  envelope; workspace/window/output field shapes were recorded as sanitized
  type skeletons, not personal values.
- `cargo check --workspace --all-targets --offline` and
  `cargo test --workspace --offline --locked` pass with 85 tests.
- `nalios-dev niri inspect` produced a correct snapshot of the live session
  (one output, three workspaces, two windows, focused Alacritty window).
- Unit tests cover the reply envelope, unknown/optional fields, remote errors,
  malformed replies, a fake Unix-socket server, EOF and oversized frames.

## Consequences

- Transitive license/notice/advisory review of `serde_json`, `itoa`, `memchr`
  and `zmij` is required before distribution.
- Wire types model only fields NaliOS uses; unknown fields are ignored. New
  behavior depending on new protocol capabilities must be gated explicitly.
- The synchronous client serves the headless CLI. The shell must use it through
  an owned worker or a later async adapter; the UI loop must not block on it.
