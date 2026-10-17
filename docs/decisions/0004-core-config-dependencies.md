# 0004 — Core configuration dependencies

Status: **selected for private development** on 2026-10-16. The author directed
implementation to proceed after the permissive-license review recorded here.
Revisit before distribution or if a crate fails maintenance/advisory review.

## Context

The versioned configuration engine needs typed TOML deserialization, bounded
parsing and clear error display/source chains. Hand-writing a TOML parser or
error boilerplate would add risk without product benefit. The
[source register](../blueprint/sources.md) already listed `serde`, `toml` and
`thiserror` as candidates; none is GPL/AGPL or an unknown license.

## Decision

Add these normal dependencies to `nalios-core` (workspace-level requirements;
the committed lockfile pins exact versions):

| Crate | Requirement | Locked | Published license | Purpose |
|---|---|---|---|---|
| `serde` (`derive`) | 1.0.228 | 1.0.229 | MIT OR Apache-2.0 | Config model derives |
| `toml` | 1.1.2 | 1.1.8+spec-1.1.0 | MIT OR Apache-2.0 | Versioned TOML parse/serialize |
| `thiserror` | 2.0.18 | 2.0.21 | MIT OR Apache-2.0 | Typed error display/source chains |

`nalios-app` also uses `thiserror` for port error types. `nalios-platform`
adds `tempfile` 3.27.0 (MIT OR Apache-2.0) as a **dev-dependency only** for
disposable filesystem fixtures. No Tokio, zbus, UI toolkit, D-Bus, HTTP or
system-service crate is added.

Resolved normal graph for core: `serde_core`, `serde_derive`, `serde_spanned`,
`toml_datetime`, `toml_parser`, `toml_writer`, `winnow`, `proc-macro2`,
`quote`, `syn`, `unicode-ident` and `thiserror-impl`. Dev-only graph:
`tempfile`, `fastrand`, `getrandom`, `once_cell`, `rustix`, `linux-raw-sys`,
`bitflags`, `libc` and `cfg-if`.

## Evidence

- `cargo check --workspace --all-targets --offline --locked` passes.
- `cargo test --workspace --offline --locked` passes with 49 tests at this
  revision (core paths/config, app workflow, platform file reads, CLI parsing).
- `cargo tree --workspace --offline` was inspected; the lockfile records exact
  versions and is committed.
- Builds resolve offline from the local registry cache.
- `cargo fmt`/`cargo clippy` are unavailable on this machine, so formatting,
  lint and advisory gates remain open (T002/F07 evidence).

## Consequences and required follow-up

- Before distribution: full transitive license/notice/SBOM review, advisory
  checks and a deliberate update policy for every crate above.
- Requirements are minimums; the lockfile pins current resolution. Revisit the
  pinning strategy at release.
- The author may veto or replace any crate. The document format, defaults,
  bounds and validation rules stay in `nalios-core`; swapping a parser must not
  silently change them.
