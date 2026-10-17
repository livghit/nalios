# NaliOS next-generation build blueprint

**Purpose:** a written implementation brief for an original, Rust-authored
NaliOS desktop layer on Kali Linux and Niri, optimized first for its author's
laptop and potentially shared later.

This is a specification, not a finished implementation. A
[workspace scaffold](../../crates/README.md) exists with a first CLI/paths
increment; most features are unimplemented. The old prototype is retained as
`kalios-deprecated`; no legacy implementation was imported. See the
[repository boundary decision](../decisions/0001-repository-boundary.md) for
locations, the compatibility alias, and the recovery snapshot.

## Direction established by the user

- Target **Kali Linux + upstream Niri**, not a GNOME Shell extension project.
- Build an **original NaliOS codebase and interface**, not a translation or fork
  of Caelestia's GPL implementation.
- Keep **all first-party executable code/tests in Rust**. Prefer Rust-native UI;
  native system/library boundaries are explicit, not hidden behind “Rust-only.”
  See [decision 0002](../decisions/0002-rust-codebase-boundaries.md).
- The user writes the code. The assistant supplies requirements, component
  boundaries, explanations, reviews, and test planning unless asked otherwise.
- Make a reliable and distinctive personal laptop desktop first; sharing and
  polished packaging come later.
- Original Kali-inspired wallpapers, visual design, and themes are a later
  creative phase. Existing prototype artwork is not automatically imported.

Owning our original application code is different from avoiding GPL software
throughout the system. Niri itself is GPL. Read the licensing boundary before
choosing dependencies or promising a license for the new application.

## Start with the complete delivery plan

The [master plan](master-plan.md) connects the whole desktop to usable vertical
slices, open decisions, risks, and release definitions. Detailed companion specs:

| Area | Specification |
|---|---|
| Crate/runtime/state boundaries and code-quality rules | [Rust architecture](architecture.md) |
| Exact next changes and headless CLI acceptance | [First milestone](first-milestone.md) |
| Direct packages, alternatives, optional profiles | [Package inventory](packages.md), [observed versions/source packages](../research/packages-2026-10-16.tsv) |
| Upstream code, protocols, candidate Cargo dependencies | [Source register](sources.md) |
| First-party crates, modules, files and tests | [Implementation source map](implementation.md) |
| Startup, ownership, agents, portals, secure sleep | [Session integration](session.md) |
| Layout, original theme pair, app adapters, safe apply | [Theme and experience specification](themes.md) |
| Building, Debian packaging, migration and release gates | [Delivery contract](delivery.md) |

These remain planning documents. No proposed crate/dependency, package profile,
layout or palette is approved merely by being listed. Read the research's
[method and limitations](../research/README.md) before using the version tables.

## Foundational contracts

1. [Product brief](product.md): who the desktop serves, desired behavior,
   priorities, and what is deliberately out of scope.
2. [Authorship, licensing, and provenance](ownership.md): how to keep the new
   application independent of the old port and record third-party obligations.
3. [Architecture and contracts](architecture.md): processes, modules, service
   boundaries, configuration, IPC, and failure handling.
4. [Component catalog](components.md): exactly what must be written, why it
   exists, what it reuses, and how to know it works.
5. [Roadmap and first tickets](roadmap.md): the implementation order, role
   assignments, and concrete starting tasks.
6. [Acceptance and laptop test plan](acceptance.md): hardware, security,
   integration, upgrade, and recovery checks.
7. [Design, themes, and artwork brief](design.md): the future visual system and
   original asset workflow.

## People and responsibilities

| Role | Responsibility |
|---|---|
| User: product owner | Choose behavior, priorities, appearance, sharing policy, and what constitutes a good laptop experience. |
| User: code author/maintainer | Write Rust, configuration, packaging, and tests; choose dependencies; decide what to merge and release. |
| User: hardware tester | Exercise real Wi-Fi, mouse, dock/display, lid, battery, suspend, and VM behavior; approve any disruptive test. |
| Assistant: specification/review support | Break down work, explain APIs and trade-offs, review the user's code, suggest tests, and maintain the plan. Do not implement application code by default. |
| Upstream projects | Maintain the compositor, kernel, drivers, system services, toolkits, and other dependencies; retain their own copyrights and licenses. |
| Future contributors | Supply independently licensed contributions; ownership does not automatically transfer to the project maintainer. |

The assistant is not a human co-author with a negotiated ownership share. The
project must identify the real author(s)/rightsholder(s), and select a license
explicitly before public distribution.

## Small initial workspace, larger capability catalog

Five application package shells now exist: `nalios-core`, `nalios-app`,
`nalios-platform`, `nalios-cli`, and `nalios-shell`. At the user's request,
[structural setup](../decisions/0003-architecture-only-scaffold.md) precedes code;
there are no APIs, runtimes or binaries. Implement core + CLI first, app/platform
workflows for real probes next, then UI after the spike. The catalog describes
modules/capabilities, **not** one crate/service per row. `xtask` is not created;
VM/helper/tooling splits need an actual reason.

Build and test in this new `nalios` repository at `~/nalios`, using a
`nalios-dev` configuration/runtime namespace until deliberate installation.
The old runtime path `~/.local/share/nalios` is a compatibility alias for
`kalios-deprecated`, not this repository. No old Git tree, vendor source,
templates, or assets have been imported here.

## Still to decide

- Application license: for example, MIT OR Apache-2.0, subject to dependency
  compatibility and the author's preference; no license has been selected here.
- Rust-native UI after T007: Iced + `iced_exwlshell` is the leading experiment,
  not an approved dependency list or proven accessible shell. GTK is no longer
  the default; any fallback to it needs explicit author approval.
- Approval/revision of the proposed top bar, full dashboard and original
  Night/Day tokens in the [theme specification](themes.md).
- Reproducible Niri/satellite packaging: the observed executables are local
  `/usr/local/bin` programs without candidates in the cached Kali indexes.
- Login/agent/keyring selection, laptop power/lid policy and whether VM features
  belong in a later separate profile. See decisions D01–D09 in the [master plan](master-plan.md).

Detailed decisions should get short records: context, options, choice, reason,
consequences, evidence, and review date. Start with the
[repository boundary decision](../decisions/0001-repository-boundary.md).
Earlier discussions remain in the deprecated repository as historical context;
this blueprint is the forward plan.
