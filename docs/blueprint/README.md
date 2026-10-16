# NaliOS next-generation build blueprint

**Purpose:** a written implementation brief for an original, Rust-authored
NaliOS desktop layer on Kali Linux and Niri, optimized first for its author's
laptop and potentially shared later.

This is a specification, not a working implementation. The old prototype is
retained as `kalios-deprecated`; this new repository contains no legacy
implementation or Rust application code. See the
[repository boundary decision](../decisions/0001-repository-boundary.md) for
locations, the compatibility alias, and the recovery snapshot.

## Direction established by the user

- Target **Kali Linux + upstream Niri**, not a GNOME Shell extension project.
- Build an **original NaliOS codebase and interface**, not a translation or fork
  of Caelestia's GPL implementation.
- The user writes the code. The assistant supplies requirements, component
  boundaries, explanations, reviews, and test planning unless asked otherwise.
- Make a reliable and distinctive personal laptop desktop first; sharing and
  polished packaging come later.
- Original Kali-inspired wallpapers, visual design, and themes are a later
  creative phase. Existing prototype artwork is not automatically imported.

Owning our original application code is different from avoiding GPL software
throughout the system. Niri itself is GPL. Read the licensing boundary before
choosing dependencies or promising a license for the new application.

## Read in this order

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

Start with four crates: `nalios-core`, `nalios-platform`, `nalios-cli`, and
`nalios-shell`. The component catalog describes modules/capabilities, **not**
one crate or service per row. Add a separate VM application or privileged
helper only when its lifecycle and security boundary require it.

Build and test in this new `nalios` repository at `~/nalios`, using a
`nalios-dev` configuration/runtime namespace until deliberate installation.
The old runtime path `~/.local/share/nalios` is a compatibility alias for
`kalios-deprecated`, not this repository. No old Git tree, vendor source,
templates, or assets have been imported here.

## Still to decide

- Application license: for example, MIT OR Apache-2.0, subject to dependency
  compatibility and the author's preference; no license has been selected here.
- UI toolkit after a small technical spike; GTK4 Rust bindings plus
  gtk4-layer-shell are a candidate, not an approved dependency list.
- Bar location/layout, dashboard structure, and first original visual tokens.
- Laptop power/lid policy and which VM features belong in the first release.

Detailed decisions should get short records: context, options, choice, reason,
consequences, evidence, and review date. Start with the
[repository boundary decision](../decisions/0001-repository-boundary.md).
Earlier discussions remain in the deprecated repository as historical context;
this blueprint is the forward plan.
