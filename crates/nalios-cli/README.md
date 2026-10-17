# nalios-cli

**Status: early implementation.** `nalios-dev` implements `--help`,
`--version`, `config check --file <path>`, passive `doctor` diagnostics (text
or `--json`), read-only `niri inspect`, streaming `niri watch` and
`shell <ping|open-launcher|toggle-launcher>` and `apps <list|search <query>>`
with unit tests. Other commands do not exist yet.

## Boundary

Own argument parsing, output/exit-code presentation and startup wiring. Invoke
shared app workflows or the selected running shell's control endpoint; do not
reimplement theme/device policy or require a GUI for diagnostics.

## Dependencies

Declared: `nalios-app`, `nalios-platform`, `nalios-core`. Never import shell.
No argument-parser, async runtime or other third-party crate has been added;
the CLI uses `std` plus the local application/platform crates.

## Current binary

`Cargo.toml` declares `[[bin]] name = "nalios-dev"` with `autobins = false`, so
the binary is explicit rather than auto-discovered. `src/main.rs` is a thin
entry point over the `nalios_cli::run` library API; `src/lib.rs` owns parsing,
presentation and tests.

```text
cargo run -p nalios-cli --bin nalios-dev -- --help
cargo run -p nalios-cli --bin nalios-dev -- --version
cargo run -p nalios-cli --bin nalios-dev -- config check --file <path>
cargo run -p nalios-cli --bin nalios-dev -- doctor
cargo run -p nalios-cli --bin nalios-dev -- doctor --json
cargo run -p nalios-cli --bin nalios-dev -- niri inspect
cargo run -p nalios-cli --bin nalios-dev -- niri watch
cargo run -p nalios-cli --bin nalios-dev -- shell toggle-launcher
cargo run -p nalios-cli --bin nalios-dev -- shell toggle-dashboard
cargo run -p nalios-cli --bin nalios-dev -- apps list
cargo run -p nalios-cli --bin nalios-dev -- apps search browser
```

`niri inspect` and `niri watch` use `$NIRI_SOCKET` when no `--socket` is given.
`watch` runs until interrupted, reconnecting with bounded backoff and marking
state stale while disconnected. Exit codes: `0` success, `1` failed check
(invalid configuration, required diagnostic failure, unreachable Niri), `2`
invalid invocation, `3` internal failure. Errors go to stderr. `doctor --json`
emits the versioned report envelope (`schema_version`, `checks`, `summary`).
Nothing is installed and the prototype's production commands are untouched.

## Future tests

Unit tests cover command parsing, config-check errors, doctor report shape and
Niri connect errors. The Niri wire/client behavior is tested in
`nalios-platform` with a fake socket server. The [first
milestone](../../docs/blueprint/first-milestone.md) specifies remaining work;
Rust CLI integration tests belong in the reserved but empty `tests/` directory.

See the [architecture](../../docs/blueprint/architecture.md) and
[source map](../../docs/blueprint/implementation.md).
