# 0007 — Private shell-control channel

Status: **accepted and implemented** on 2026-10-16. Adds one dependency
(`rustix`) and the first CLI↔shell protocol.

## Context

The CLI must be able to ask a running shell to perform allowlisted UI actions
(open/toggle launcher) so a Niri key binding can trigger them. The architecture
requires a private socket with bounded messages, ownership checks and no
arbitrary command execution.

## Decision

- **Protocol (app layer):** versioned, newline-delimited JSON with an
  allowlisted `ControlAction` enum (`ping`, `open-launcher`, `toggle-launcher`)
  and a `ControlResponse`. No command strings, no credentials.
- **Transport (platform layer):** `$XDG_RUNTIME_DIR/nalios-dev/control.sock`,
  mode 0600 inside a 0700 directory; 64 KiB bounded frames; stale-socket
  detection; Linux peer credentials checked with
  `rustix::net::sockopt::socket_peercred`.
- **Dependency:** `rustix` 1.1.5 (Apache-2.0 WITH LLVM-exception OR
  Apache-2.0 OR MIT) with `net`/`process` features, avoiding first-party
  `unsafe` for peer credentials.
- **Shell:** a control worker thread accepts requests, responds with acceptance
  and forwards actions to the UI. **CLI:** `nalios-dev shell <action>`.

## Evidence

- 111 workspace tests pass offline: control round trip, stale socket,
  malformed/oversized messages, version rejection, missing socket and peer
  checks.
- `nalios-dev shell ping` reports a connect error when no shell is running.

## Consequences

- The control socket is same-UID only; it is not a sandbox against processes
  already running as that user.
- Multi-session selection (session IDs) is deferred; one shell per user
  session.
- A success response means the action was accepted, not completed.
