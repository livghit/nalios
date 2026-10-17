# 0003 — Architecture scaffold without application implementation

Status: **accepted scope**, per the user's request to set up the architecture
without implementing code. Recorded 2026-10-16. This makes the five-package
layout concrete; runtime/toolkit/API/dependency choices still require review.

## Decision

- Create a virtual Cargo workspace and manifests for core, app, platform, CLI
  and shell, following the dependency direction in
  [0002](0002-rust-codebase-boundaries.md).
- Create documentation-only `src/lib.rs` roots so Cargo can validate the layout.
  No Rust declarations, imports, functions, types, traits, modules, stubs,
  `todo!()`, `main()` or application tests are authored.
- CLI and shell temporarily expose library targets only, with automatic binary
  discovery disabled. Explicit development binaries are a future code task.
- Declare only local path dependencies; no toolkit/runtime/system binding or
  other third-party dependency is added, fetched or approved.
- Use Rust 2024 / Cargo resolver 3, placeholder version `0.0.0`, inherited
  `publish = false`, and shared safety/Clippy configuration. Do not choose a
  license or switch/install a Rust toolchain.
- Reserve configuration, artwork, packaging and fixture locations with README
  contracts and empty Git markers. Do not invent usable defaults, services,
  theme schemas, assets or an installer before their implementation tickets.
- Leave development tooling/CI programs, `xtask`, detailed feature modules and
  APIs unimplemented. No architectural framework is generated speculatively.

## Why comment-only libraries?

Cargo members need targets. Empty documented library targets make the workspace
inspectable/checkable without a fake executable that appears to work. This is
explicitly temporary for CLI/shell. Future implementation adds the intended
`nalios-dev` / `nalios-shell-dev` binary targets deliberately.

## Consequences

The earlier recommendation to add crates incrementally is adjusted only for
**structural setup**, at the user's request. Feature implementation still starts
small, per the [first milestone](../blueprint/first-milestone.md). Creating five
empty libraries is not completing F01/T002 or choosing a UI/runtime.

Manifests express current dependency edges and inherit lint settings. They do
not themselves prevent future forbidden imports or std I/O; automated
architecture checks and real tests remain implementation work.

No packages, installed commands, prototype links, sessions, credential stores,
VM data or system configuration are modified. No license is granted, public
release made, or implementation/hardware acceptance claimed. See
[workspace notes](../../crates/README.md) for the exact current structure and
non-disruptive validation commands.

## Follow-up

F01 implementation began after this decision: `nalios-dev` now has an explicit
binary target with `--help`/`--version` and unit tests, and core's pure `paths`
module is implemented. The "no declarations/functions" scope above describes
the scaffold turn only. App/platform/shell remain documentation-only.
