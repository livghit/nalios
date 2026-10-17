# NaliOS documentation

These documents specify the **new, original Rust project**, not the deprecated
Caelestia-based prototype. The [workspace scaffold](../crates/README.md) and a
first CLI/paths increment exist; most application behavior is unimplemented and
there is no runnable desktop shell yet.

## Build blueprint

Start with the [Rust architecture](blueprint/architecture.md) and
[first implementation milestone](blueprint/first-milestone.md) for the next work.
The [complete delivery plan](blueprint/master-plan.md) and
[blueprint index](blueprint/README.md) cover the whole desktop. Project contracts:

1. [Product brief](blueprint/product.md)
2. [Authorship, licensing, and provenance](blueprint/ownership.md)
3. [Architecture and contracts](blueprint/architecture.md)
4. [Component catalog](blueprint/components.md)
5. [Roadmap and first tickets](blueprint/roadmap.md)
6. [Acceptance and laptop tests](blueprint/acceptance.md)
7. [Original design, themes, and wallpapers](blueprint/design.md)

## Detailed implementation planning

- [Packages and runtime profiles](blueprint/packages.md)
- [Upstream code, APIs, and Rust dependency register](blueprint/sources.md)
- [First-party source/module map](blueprint/implementation.md)
- [Niri session, agents, portals, locking, and recovery](blueprint/session.md)
- [Original themes, layout, application styling, and safe apply/revert](blueprint/themes.md)
- [Build, Debian packaging, installation, updates, and release gates](blueprint/delivery.md)
- [Read-only research and package-version snapshot](research/README.md)

Proposed choices are explicitly distinguished from approved dependencies and
working code. The package snapshot is cached host evidence, not an install
script, package lock, or a compatibility guarantee.

The user writes application code and tests; the assistant helps specify,
explain, and review them. Work starts with the core/CLI and small, testable
increments rather than copying or translating the old shell implementation.

## Decisions

- [0001 — Repository identity and prototype boundary](decisions/0001-repository-boundary.md)
- [0002 — Rust-only code and five-crate boundaries](decisions/0002-rust-codebase-boundaries.md)
- [0003 — Architecture-only workspace scaffold](decisions/0003-architecture-only-scaffold.md)
- [0004 — Core configuration dependencies](decisions/0004-core-config-dependencies.md)
- [0005 — Niri protocol JSON dependency](decisions/0005-niri-json-dependency.md)
- [0006 — Rust-native UI toolkit spike](decisions/0006-ui-toolkit-spike.md)
- [0007 — Private shell-control channel](decisions/0007-shell-control-channel.md)
- [0008 — Desktop-entry parser dependency](decisions/0008-desktop-entry-dependency.md)
- [0009 — Icon lookup dependency](decisions/0009-icon-lookup-dependency.md)
- [0010 — Local-time formatting dependency](decisions/0010-local-time-dependency.md)

The Rust-only requirement and five-package structural setup are explicit.
Application interfaces, runtime/native dependencies and Iced/Wayland UI remain
to be reviewed/implemented; GTK-first planning is superseded. This does not
claim every native OS library is written in Rust.

Record future decisions with context, alternatives, choice, consequences,
evidence, and review date. License and UI-toolkit selection are still open.

## Project locations

- `~/nalios`: this new source repository.
- `~/.local/share/kalios-deprecated`: historical prototype and its handbook.
- `~/.local/share/nalios`: compatibility alias for that deprecated prototype,
  not this source repository.

Nothing here installs software, switches sessions, changes existing commands,
removes packages, or touches VM vaults.
