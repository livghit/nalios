# 0010 — Local-time formatting dependency

Status: **accepted and implemented** on 2026-10-16. Adds `time` as a direct
dependency of `nalios-shell`.

## Context

The bar clock and dashboard need local time. `std` exposes only UTC
(`SystemTime`), and the earlier spike clock was silently UTC. Computing the
local offset by hand would mean parsing TZif or calling `libc` from unsafe
code.

## Decision

- Add `time` 0.3.55 with the `local-offset` feature (MIT OR Apache-2.0) to
  `nalios-shell`. The crate is already in the dependency graph transitively via
  GPUI.
- Fall back to `--:--` / "time unavailable" when the offset cannot be
  determined instead of showing a wrong time.

## Evidence

- 130 workspace tests pass offline; `datetime_text_is_never_empty` covers the
  fallback path.
- The host's `/etc/localtime` resolves, so the bar and dashboard show local
  time.

## Consequences

- The local-offset lookup reads the host timezone database; environments
  without a timezone fall back to the placeholder text.
- Future calendar/date widgets can reuse the same crate.
