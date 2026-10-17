# NaliOS

An original, user-authored Rust desktop layer for **Kali Linux + upstream
Niri**, built first for the author's laptop.

**Status: early implementation.** `nalios-dev` implements `--help`,
`--version`, `config check`, passive `doctor` diagnostics (text and versioned
JSON), a read-only `niri inspect` snapshot, `niri watch` event streaming,
`shell` control actions and `apps` catalog commands. `nalios-core` provides
pure path/config/Niri domain values; `nalios-app` owns config,
diagnostic and Niri reducer workflows; `nalios-platform` supplies bounded file
reads, passive host probes and an independent Niri IPC client with a bounded
event stream, stale-state marking and reconnect backoff, verified against the
running Niri 26.04 session. A private, peer-checked control socket lets the CLI
toggle the launcher. Device controls, themes and the installer remain
unimplemented. GPUI was accepted after a live spike; the development shell
(`nalios-shell-dev`) builds with a layer-shell bar, an icon-and-search launcher
with config favorites, and a full dashboard on live Niri state. No license has been selected; reviewed
dependencies are recorded in
decisions
[0004](docs/decisions/0004-core-config-dependencies.md),
[0005](docs/decisions/0005-niri-json-dependency.md),
[0006](docs/decisions/0006-ui-toolkit-spike.md) and
[0007](docs/decisions/0007-shell-control-channel.md).

## Start here

- [Workspace setup](crates/README.md): current structure, crate boundaries,
  temporary library targets, and offline validation commands.
- [Scaffold decision](docs/decisions/0003-architecture-only-scaffold.md): what
  was set up and what was deliberately left unimplemented.
- [Rust architecture](docs/blueprint/architecture.md): crate boundaries, shared
  workflows, runtime/state ownership, UI selection, and test rules.
- [First implementation milestone](docs/blueprint/first-milestone.md): the exact
  next changes to build a safe headless CLI before the shell.
- [Complete desktop delivery plan](docs/blueprint/master-plan.md): usable
  milestones, decisions, risks, and what “finished” means.
- [Documentation index](docs/README.md)
- [Build blueprint](docs/blueprint/README.md): product, architecture, 25
  components, exact package/source inventories, proposed Rust modules, session
  integration, original themes, packaging, and acceptance gates.
- [Repository boundary decision](docs/decisions/0001-repository-boundary.md):
  naming, separation from the prototype, and compatibility paths.

Continue with the [first milestone](docs/blueprint/first-milestone.md): F01–F06
are implemented or partially implemented; all UI work remains. Review
dependencies as added; license, UI/runtime and compositor supply decisions
remain open. Nothing here installs or replaces the current desktop, commands,
prototype, configuration or VM data.

## Repository boundary

- New source repository: `~/nalios` — this repository, with independent Git
  history.
- Deprecated prototype: `~/.local/share/kalios-deprecated` — retained with its
  Git history, pending changes, and third-party notices.
- Compatibility alias: `~/.local/share/nalios` still points to the deprecated
  prototype so existing installed commands/config links do not break.

**Develop here, not through the compatibility alias.** Existing installed
`nalios` / `nalios-*` commands still belong to the deprecated prototype; this
repository has not installed or replaced them. Use a separate development
binary/config/runtime namespace when implementation begins.

Only the original planning blueprint was copied here. No legacy application
source, Caelestia/Quickshell vendor tree, themes, images, fonts, build products,
or old Git history was imported.

## Ownership and dependencies

The new application is to be independently authored, not a translation or fork
of the old Caelestia port. Upstream Niri and system services remain external
software with their own licenses. Niri and its `niri-ipc` crate are GPL;
application dependency and IPC choices must respect the
[ownership policy](docs/blueprint/ownership.md).

The author has **not selected a first-party license yet**. No LICENSE file is
added on the author's behalf. Choose the code/documentation/artwork policies
and review dependencies before public distribution. This is not a GPL-free
operating system or a claim of ownership over upstream projects.

## Workspace structure

These package boundaries exist; their source roots contain documentation only:

```text
crates/nalios-core       pure domain/config/theme values and validation
crates/nalios-app        shared workflows, state reducers, ports, task ownership
crates/nalios-platform   independently written Niri/Linux/file adapters
crates/nalios-cli        headless commands, output and application wiring
crates/nalios-shell      original Rust UI, interaction state and application wiring
```

Reserved resource roots are [`config/`](config/README.md),
[`assets/`](assets/README.md), [`packaging/`](packaging/README.md), and
[`tests/`](tests/README.md). They contain guidance and directory markers, not
usable defaults, assets, installers or tests. The CLI now has an explicit
development binary (`nalios-dev`); the shell still has no binary target.

**All first-party executable code and tests stay in Rust.** The revised UI
spike prioritizes Iced + `iced_exwlshell`; GTK is no longer assumed. Native
OS/graphics dependencies remain explicit review boundaries, not a false promise
of zero C anywhere in Kali. See [decision 0002](docs/decisions/0002-rust-codebase-boundaries.md).
No toolkit is approved or installed by this plan. Original artwork needs
editable sources/provenance; never automatically import prototype assets.
