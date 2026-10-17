# 0002 — Rust-only application code and explicit architecture boundaries

Status: **Rust-only first-party code is a user requirement.** The five-package
structural layout is now scaffolded at the user's request under
[decision 0003](0003-architecture-only-scaffold.md). Application APIs,
runtime/native-library policy and UI candidates remain design choices requiring
implementation review, not completed software. Recorded 2026-10-16.

Supersedes the earlier four-crate proposal and GTK-first UI assumption in the
planning documents. Does not change the repository/prototype boundary, choose
a first-party license, install dependencies or authorize system changes.

## Context

The user wants a thoroughly planned original Kali/Niri desktop with a Rust-only
codebase and good architecture. Earlier planning identified functionality and
packages, but left workflow ownership between the CLI, shell and platform
adapters too implicit. It also treated GTK Rust bindings as the leading UI
candidate, which is not the same as a Rust-native widget toolkit.

## Language boundary

- All first-party **executable application behavior** is Rust: CLI, shell UI,
  Linux adapters, workflow/state logic, migrations, installer planning, tests
  and repository automation when needed.
- No QML/Quickshell, JavaScript/TypeScript, Python, Lua, Electron/Tauri frontend,
  custom C/C++ modules or Bash programs implementing desktop behavior.
- TOML, JSON fixtures, Niri KDL, systemd/desktop/portal files, Markdown, SVG and
  other artwork are data/resources. They are not reasons to invent executable
  theme files or a scripting engine.
- Prefer Cargo commands and a Rust `xtask` for automation. Conventional Debian
  build/maintainer glue may be unavoidable; it must be minimal declarative
  invocation, explicitly reviewed, and contain no independent application,
  migration or privilege policy. Do not grow an `install.sh` codebase.
- Kali, Niri and external services keep their own implementation languages.
  Rewriting the kernel, drivers, BlueZ, PipeWire, PAM or the locker to meet a
  language slogan is out of scope.

Recommended dependency interpretation: use Rust-native libraries where practical,
with individually reviewed OS/graphics/native-library boundaries. **This is not
“zero C anywhere in the binary/dependency tree.”** GPU drivers, input libraries,
libc and some service bindings can still be native. Any stricter ban on linked
native libraries needs a separate feasibility decision, not a false all-Rust
claim. Rust-only is also not a license/security approval.

## Architecture recommendation

Use a **modular monolith**: libraries with enforced responsibilities, a desktop
shell process and short-lived CLI processes, not a fleet of feature daemons.

| Crate | Owns |
|---|---|
| `nalios-core` | Pure domain/config/theme values, validation, stable IDs and errors; no I/O or async runtime |
| `nalios-app` | Toolkit-independent workflows, feature state/reducers, narrow port contracts, task supervision and command policy |
| `nalios-platform` | Implementations of app-defined ports for Niri, Linux services, files and external tools |
| `nalios-cli` | Arguments/output and wiring the app to real adapters or contacting the running shell |
| `nalios-shell` | Rust UI, local interaction state and wiring the app to adapters; no duplicated backend workflow |

Dependencies: `app → core`; `platform → app/core`; `cli,shell → app/platform/core`.
The app never imports platform. Frontend composition roots may construct
platform implementations; their widgets/commands use app interfaces. No cyclic
crate dependencies and no runtime backend that knows about widget types.

The fifth crate has one concrete purpose: enforce shared application behavior
and allow fake adapters without a GUI or live system. Do not add more crates
for each widget/device/backend until a demonstrated lifecycle/build/security
boundary justifies the split. Feature implementation starts core + CLI, then
app/platform probes, then UI after the spike. The later user request authorizes
five manifests/comment-only roots for structural setup, not empty API/framework
implementations or permission to start all features together.

## UI recommendation and unresolved gate

Evaluate **Iced + `iced_exwlshell`** first for Rust-native widget/layout code and
Wayland shell surfaces. Registry observations and upstream migration notes are
recorded in the [source register](../blueprint/sources.md). This remains a
candidate: layer surfaces, normal settings windows, IME/text input, focus,
accessibility, multi-output scaling, rendering cost and async executor
integration must pass the T007 spike.

GTK4 is **not the default frontend** under this revised plan. Reintroducing it
requires an explicit author-approved exception; a Rust wrapper alone is not a
Rust-native toolkit. GIO is likewise not silently added to the launcher. A
Rust desktop-entry parser must be license-reviewed and must not be mistaken
for a complete correct application launcher.

Do not write a rendering/widget/text engine ourselves merely because a spike
fails. Record the failure, assess maintained Rust alternatives or a scoped
exception, and decide before implementing the full shell. Do not support two
production frontend stacks simultaneously.

## Runtime and quality consequences

Recommend one Tokio backend runtime per process, with the chosen UI toolkit's
own Wayland/event loop integrated through one bounded bridge. Toolkit event
loops/native library loops are not additional general-purpose async runtimes;
verify executor construction rather than accidentally creating two Tokios.

Feature state has a single writer. Views hold projections and local interaction
state, not rival hardware truth. External effects use explicit ports. No global
mutable service locator, generic plugin framework, unbounded event bus,
root-running renderer or broad shell-command endpoint.

Pure/core and application tests must run without GTK, Wayland, D-Bus, network,
root or the real home directory. Architecture/feature-dependency checks join
normal formatting, linting, tests, advisory and license review.

## Alternatives considered

- **Keep four crates and put workflows in platform:** simpler file count, but
  makes orchestration hard to test independently and invites UI/CLI duplication.
- **Dozens of crates/services:** unnecessary build/protocol/lifecycle overhead
  for a single-author desktop. Modules first; process splits need evidence.
- **GTK through Rust:** credible integration alternative, but a native C toolkit;
  not silently equivalent to the newly requested Rust-native direction.
- **Custom engine from raw Wayland:** maximum control, excessive maintenance for
  typography, IME, accessibility, focus and rendering; not the initial project.
- **Literal zero-native-library stack:** different and much larger requirement;
  not promised for Kali + GPU desktop integration.

## Evidence and review trigger

At proposal time no Cargo build or UI spike had run. The later structural
scaffold can be checked offline; this is not an application build or UI-spike
result. See [decision 0003](0003-architecture-only-scaffold.md).
Review at T004 (port/state usefulness), T007 (toolkit/executor feasibility), and
before packaging (actual native/transitive dependencies and source obligations).
The [architecture](../blueprint/architecture.md),
[source map](../blueprint/implementation.md), and
[first milestone](../blueprint/first-milestone.md) specify the implementation work.
