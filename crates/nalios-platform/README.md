# nalios-platform

**Status: filesystem reads, passive host probes and an independent Niri client implemented.**

## Boundary

Implement app-defined ports with independently authored Niri/Linux/service,
filesystem and reviewed external-tool adapters. Keep protocol DTOs and native
handles private; normalize results into app/core types. Concrete effects belong
here, but shared workflow/transaction policy belongs in app.

No widgets, root-running shell, copied upstream implementation, arbitrary
shell-command endpoint or automatic service takeover. Diagnostic adapters must
be passive and distinct from mutation/agent initialization.

## Dependencies

Declared: `nalios-app`, `nalios-core`, `freedesktop-desktop-entry`,
`freedesktop-icons`, `rustix`, `serde`, `serde_json` and `thiserror` (see
decisions [0004](../../docs/decisions/0004-core-config-dependencies.md),
[0005](../../docs/decisions/0005-niri-json-dependency.md),
[0007](../../docs/decisions/0007-shell-control-channel.md),
[0008](../../docs/decisions/0008-desktop-entry-dependency.md) and
[0009](../../docs/decisions/0009-icon-lookup-dependency.md)). `tempfile` is a
dev-dependency for disposable fixtures only. No UI toolkit, runtime or
service/native binding is selected; headless diagnosis must not acquire the GUI
dependency graph. Do not link `niri-ipc`/`niri-config` under the current
independent-client policy.

## Implemented modules

- `files`: `FsConfigSource`, which opens a path, reads at most
  `MAX_CONFIG_BYTES + 1` bytes, rejects oversized/non-UTF-8 input and maps
  open errors to the app port's typed `ReadError`.
- `diagnostics`: `HostDiagnostics`, collecting environment and filesystem
  facts (runtime directory, Wayland/Niri endpoints, session bus, programs)
  without connecting to services or executing programs.
- `niri`: wire types, reply/event parsing, a synchronous `NiriClient` and a
  bounded `NiriEventStream` for the documented IPC protocol, with
  unknown-field/event tolerance and a fake-socket test server. Both are
  verified against Niri 26.04.
- `control`: the private control socket server/client: 0600 socket in a 0700
  runtime directory, 64 KiB bounded JSON frames, stale-socket handling and
  Linux peer-UID checks through `rustix`.
- `apps`: XDG desktop-entry discovery (hidden/no-display/desktop/TryExec
  filtering, first-directory-wins dedupe), argv-based launching with
  `xdg-terminal-exec` for terminal applications, and theme icon lookup with a
  once-per-process theme detection.

## Future source and tests

The Niri event stream/reconnect adapter and later backend modules come next;
they remain only in the [source map](../../docs/blueprint/implementation.md).
Tests use disposable temp files, a fake Unix-socket server and synthetic I/O
errors. Physical-device, power and active-session actions never run in the
default test suite. The reserved `tests/` directory is empty.
