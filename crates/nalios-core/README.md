# nalios-core

**Status: first increment implemented: pure namespace path resolution.**

## Boundary

Own pure domain values, validated configuration/theme models, IDs, capability
and error types, parsing and migration rules. Inputs must be supplied by callers.
No files, environment discovery, sockets, processes, global runtime, native
handles or UI toolkit types belong here.

## Dependencies

Reviewed permissive dependencies: `serde` (derive), `toml` and `thiserror`
(see [decision 0004](../../docs/decisions/0004-core-config-dependencies.md)).
Other NaliOS packages may depend on this crate; it must not depend on app,
platform, CLI or shell. No I/O, runtime or toolkit crate belongs here.

## Implemented modules

- `paths`: pure XDG/namespace directory resolution from explicit `XdgInputs`
  values. It rejects relative XDG values, requires `XDG_RUNTIME_DIR` without a
  `/tmp` fallback, validates namespace/session path components and never
  touches the filesystem or environment itself.
- `config`: versioned TOML documents (`schema_version = 1`), typed defaults,
  bounded size, unknown-field rejection, range and cross-field validation.
- `niri`: pure domain values (`WorkspaceId`, `WindowId`, `OutputName`,
  workspaces/windows/outputs and a read-only snapshot) independent of the IPC
  wire format.
- `error`: crate-level error aggregation for callers handling several modules.

Future areas are theme values and shared operation/state models; their
contracts are in the [source map](../../docs/blueprint/implementation.md).

## Tests

`paths` and `config` have unit tests with fake inputs, including missing home,
relative paths, invalid namespaces, malformed/unknown-version documents,
unknown fields, bounds and cross-field constraints. They run without a display,
service, network or the real home directory. Future tests stay equally pure.

See the [architecture](../../docs/blueprint/architecture.md) and
[first milestone](../../docs/blueprint/first-milestone.md).
