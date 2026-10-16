# NaliOS documentation

These documents specify the **new, original Rust project**, not the deprecated
Caelestia-based prototype. No implementation code is present yet.

## Build blueprint

Read the [blueprint index](blueprint/README.md) first, then:

1. [Product brief](blueprint/product.md)
2. [Authorship, licensing, and provenance](blueprint/ownership.md)
3. [Architecture and contracts](blueprint/architecture.md)
4. [Component catalog](blueprint/components.md)
5. [Roadmap and first tickets](blueprint/roadmap.md)
6. [Acceptance and laptop tests](blueprint/acceptance.md)
7. [Original design, themes, and wallpapers](blueprint/design.md)

The user writes application code and tests; the assistant helps specify,
explain, and review them. Work starts with the core/CLI and small, testable
increments rather than copying or translating the old shell implementation.

## Decisions

- [0001 — Repository identity and prototype boundary](decisions/0001-repository-boundary.md)

Record future decisions with context, alternatives, choice, consequences,
evidence, and review date. License and UI-toolkit selection are still open.

## Project locations

- `~/nalios`: this new source repository.
- `~/.local/share/kalios-deprecated`: historical prototype and its handbook.
- `~/.local/share/nalios`: compatibility alias for that deprecated prototype,
  not this source repository.

Nothing here installs software, switches sessions, changes existing commands,
removes packages, or touches VM vaults.
