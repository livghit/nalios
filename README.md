# NaliOS

An original, user-authored Rust desktop layer for **Kali Linux + upstream
Niri**, built first for the author's laptop.

**Status: planning repository.** No Rust implementation, Cargo workspace,
installer, or runnable desktop is present yet. The author will write the code;
the assistant supports specifications, architecture, reviews, and test planning.

## Start here

- [Documentation index](docs/README.md)
- [Build blueprint](docs/blueprint/README.md): product, roles, ownership,
  architecture, 25 components, roadmap, tests, and original design/artwork.
- [Repository boundary decision](docs/decisions/0001-repository-boundary.md):
  naming, separation from the prototype, and compatibility paths.

The next task is to complete T001's license/dependency decisions, then create
the minimal workspace described by T002. Do not treat the presence of planning
documents as completed software.

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

## Proposed implementation structure

The user will create these as the relevant tickets begin:

```text
crates/nalios-core       configuration, themes, state, shared errors
crates/nalios-platform   independently written Niri/system adapters
crates/nalios-cli        diagnostics, commands, recovery controls
crates/nalios-shell      original bar, launcher, dashboard, settings
```

GTK4 Rust bindings/layer-shell are a UI candidate pending a technical and
license review. No dependencies have been selected or installed by this setup.
Future original artwork should include editable sources and provenance; do not
copy the prototype's asset collection into this repository automatically.
