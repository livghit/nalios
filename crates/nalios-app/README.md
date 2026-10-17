# nalios-app

**Status: config checking and passive diagnostics implemented.**

## Boundary

Own shared user workflows, feature state/reducers, command policy, narrow
external-operation ports and task lifetime. CLI and shell use the same behavior
through this boundary. Platform implements its ports; tests can supply fakes.

Keep concrete Linux/D-Bus/Wayland/file/process calls, toolkit types and native
handles out. Define abstractions for actual external effects, not a trait or
framework for every struct. Runtime construction belongs to frontend startup,
not a hidden library singleton.

## Dependencies

Declared: `nalios-core`, `serde` and `thiserror`. Never depend on
`nalios-platform`, CLI or shell. No runtime, D-Bus or toolkit dependency is
selected.

## Implemented modules

- `config`: the `ConfigSource` port, `ReadError`, `CheckError` and `check`,
  which reads a bounded document through the port and parses/validates it with
  the pure core rules.
- `diagnostics`: the passive fact model, check policy/report, the
  `DiagnosticsSource` port and a versioned JSON envelope
  (`REPORT_SCHEMA_VERSION`). A report fails only when a required check has
  status `Fail`; warnings and unavailable facts are reported but not fatal.
- `control`: the versioned CLI↔shell protocol with an allowlisted
  `ControlAction` enum and `ControlResponse`. No command strings or secrets.
- `apps`: pure ranking over the discovered application catalog (exact, prefix,
  word-prefix, substring, keyword and comment scores).
- `niri`: the `NiriEvent` model, the canonical `NiriState` reducer shared by
  every view, and `NiriObservation` with stale-state marking and bounded
  exponential reconnect backoff.

## Future source and tests

Diagnostics and other workflows are added as their tickets begin; see the
[source map](../../docs/blueprint/implementation.md). Fake-port tests now cover
valid, invalid and unreadable documents without touching the filesystem. The
reserved `tests/` directory is empty.
