# Application workspace

**Early implementation.** `nalios-cli` builds `nalios-dev` with `--help`,
`--version`, `config check`, `doctor`, `niri inspect` and `niri watch`;
`nalios-core` provides pure path/config/Niri domain values; `nalios-app` owns
config, diagnostic and Niri reducer workflows; `nalios-platform` implements
bounded file reads, passive host probes and an independent Niri IPC client plus
bounded event stream, verified against the live session. `nalios-shell` remains
a documentation-only root. No UI, device control, installer or hardware
integration exists yet.

| Package | Responsibility | Direct workspace dependencies |
|---|---|---|
| [nalios-core](nalios-core/README.md) | Pure domain/config/theme values and validation | None |
| [nalios-app](nalios-app/README.md) | Shared workflows, state reducers, ports and task ownership | core |
| [nalios-platform](nalios-platform/README.md) | Linux/Niri/filesystem implementations of app ports | app, core |
| [nalios-cli](nalios-cli/README.md) | Headless presentation and composition root | app, platform, core |
| [nalios-shell](nalios-shell/README.md) | Rust-native presentation and composition root | app, platform, core |

The manifests declare these edges; they do not implement or automatically
police all architectural rules. App must not import platform, core must remain
I/O/runtime/toolkit-free, and CLI must not import shell. Review future changes
against the [architecture](../docs/blueprint/architecture.md).

## Current setup

- One virtual Cargo workspace, Rust 2024 edition, resolver 3.
- Placeholder version `0.0.0`; every package inherits `publish = false`.
- Headless dependencies: `serde`, `toml`, `thiserror` and `serde_json`, plus
  test-only `tempfile`. `nalios-shell` additionally depends on GPUI pinned to
  one Zed revision for the spike; it is the only crate with a renderer graph.
  Cargo.lock records exact versions; see decisions
  [0004](../docs/decisions/0004-core-config-dependencies.md),
  [0005](../docs/decisions/0005-niri-json-dependency.md) and
  [0006](../docs/decisions/0006-ui-toolkit-spike.md).
- Workspace safety/Clippy settings are inherited by every member. Lint
  configuration is not a substitute for actual tests or architecture checks.
- Default members are core + CLI (and their dependency closure), so ordinary
  headless builds need not include the UI. `--workspace` includes all.
- No `rust-toolchain.toml`: use the installed toolchain rather than silently
  downloading/switching one. Edition/resolver require Rust/Cargo 1.85 or newer;
  this is a format requirement, not a tested minimum-support promise.
  Development uses the existing Rust/Cargo 1.95.0 environment.
- `cargo fmt` and `cargo clippy` cannot run yet because this machine does not
  have `rustfmt`/`rust-clippy` installed. They remain part of the T002/F07
  evidence; install them deliberately rather than as an implicit setup step.
- No license is selected and no existing command/session is installed/replaced.

## Frontend targets

`nalios-cli` keeps a library root for parsing/presentation logic and declares
an explicit `[[bin]]` named `nalios-dev` with `autobins = false`.
`nalios-shell` keeps its library placeholder and declares the development
`nalios-shell-dev` binary (GPUI, pinned Zed revision). The spike passed live
evaluation and was promoted into this binary; a production install name is
still reserved for deliberate migration. Do not shadow the prototype's
installed production commands.

## Where future work belongs

Introduce feature modules only when their ticket starts; no empty module tree
or placeholder API has been generated. Unit tests now exist for the CLI and
core paths module; crate-local `tests/` directories remain reserved for
integration targets. Shared sanitized data can live in
[`tests/fixtures/`](../tests/README.md).

Continue with the [first milestone](../docs/blueprint/first-milestone.md) and
[source map](../docs/blueprint/implementation.md). F01–F05 are implemented or
partially implemented; the Niri event stream/reconnect and all UI work remain.

## Non-disruptive validation

```text
cargo metadata --offline --locked --format-version 1
cargo check --workspace --all-targets --offline --locked
cargo test --workspace --offline --locked
cargo run -p nalios-cli --bin nalios-dev -- --help
cargo run -p nalios-cli --bin nalios-dev -- config check --file <path>
cargo run -p nalios-cli --bin nalios-dev -- doctor
cargo run -p nalios-cli --bin nalios-dev -- niri inspect
```

These validate the current workspace and implemented increment, not a
functioning desktop or feature-complete CLI.
