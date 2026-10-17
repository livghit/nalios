# Rust architecture and contracts

**Status: early implementation; most APIs and behavior are unimplemented.**
Rust-only first-party code is the user's requirement. The five package
boundaries and manifest dependencies are set up under
[decision 0003](../decisions/0003-architecture-only-scaffold.md); `nalios-cli`
implements help/version/config-check, core has tested pure paths and config
validation, app defines the config port and platform implements bounded file
reads. [Decision 0002](../decisions/0002-rust-codebase-boundaries.md) supersedes
the four-crate/GTK-first proposal; [0004](../decisions/0004-core-config-dependencies.md)
records the selected permissive config crates. Runtime/toolkit/native
dependencies remain review choices. The author stabilizes APIs through tested
increments.

## 1. Architectural goals and non-goals

Goals: understand one feature without reading the entire shell; test logic
without Linux services or a GUI; keep hardware truth authoritative; share
behavior between CLI and desktop; replace an adapter without rewriting views;
recover from invalid config/backend loss; and keep the dependency graph small.

Use a **modular monolith**, not microservices. Normally one shell process hosts
the application engine and adapters; short-lived CLI processes diagnose or send
commands. Niri and system services remain independent upstream processes.
Library crates do not imply five daemons or a new IPC hop for every call.

Do not build a generic framework, plugin ecosystem, service locator, widget
engine, compositor, package manager, authenticator or cryptographic format.
Later VM/privileged components require separate lifecycle/security justification.

## 2. What Rust-only means

Our UI, workflows, platform code, CLI, tests, migrations and automation are
Rust. No QML, JS/TS, Python, Lua, handwritten C/C++, or Bash desktop logic. Use
Cargo and, when useful, a Rust `xtask`; configuration/theme/art files remain
non-executable data. Conventional package glue is restricted as in decision
0002; it does not become a second implementation language.

Prefer Rust-native UI and protocol crates. Existing native system services and
reviewed OS/graphics/library interfaces are external dependencies, not rewritten
NaliOS code. A Rust binding to a C library is **not** a pure-Rust dependency.
Native linkage, transitive build scripts, licenses and update obligations are
recorded explicitly. No claim of zero C/GPL anywhere in Kali is made.

## 3. Crate boundaries and dependency direction

`A → B` means A may depend on B, not the reverse:

```text
nalios-core                         pure values, validation, schemas
     ^
     |
nalios-app                          workflows, ports, feature state, supervision
     ^                 ^
     |                 |
nalios-platform        |            Niri / Linux / files / tool adapters
     ^                 |
     +-----------------+
     |                 |
nalios-cli        nalios-shell       composition + presentation
```

Both frontends may also import core types. Platform may depend on core and app;
app cannot depend on platform. The CLI never depends on shell. Tooling `xtask`,
if added, is a non-shipping workspace package, not another desktop service.

| Crate | Owns | Must not contain |
|---|---|---|
| `nalios-core` | Config/theme/domain values, validation, stable IDs, capability/error types, pure parsing/migration rules | Environment reads, files/sockets/processes, clocks/randomness without inputs, Tokio, zbus, UI/renderer/native handles |
| `nalios-app` | User workflows, reducers, commands/results, narrow external-effect ports, task lifetime/cancellation, shared snapshots | Concrete Linux/D-Bus/Wayland calls, `std::fs`/process side effects, toolkit types, package installation hidden in diagnosis |
| `nalios-platform` | Independently written Niri client, D-Bus/native adapters, filesystem/process boundaries, discovery/launch mechanics, transport framing | Widgets, product workflow duplication, global root access, copied GPL implementation types |
| `nalios-cli` | Argument parsing, text/JSON output, exit codes, composition root, commands through app or private shell control | Reimplemented pairing/theme policy, GUI requirement for doctor, raw credential logging |
| `nalios-shell` | Rust component views, per-surface interaction state, design-token mapping, surface/focus lifecycle, composition root | Direct widget-to-D-Bus calls, second canonical device store, blocking I/O in view/update, password authentication engine |

**Why add app?** Theme apply, device commands and recovery are application
workflows, not filesystem drivers or widgets. Keeping them in one layer lets
the CLI, shell and tests exercise the same rules. This is a useful fifth crate,
not permission for a crate per feature. All five manifests/comment-only roots
are now scaffolded at the user's request, not implemented. Core + CLI behavior
starts T002; app/platform effects follow T004; shell behavior waits for T007.

Use private modules and `pub(crate)` by default. Public types must have a real
consumer. Keep backend-specific DTOs/errors inside platform and UI messages
inside shell. Do not use `utils.rs`, an ever-growing shared `Manager`, or one
universal event enum as a dumping ground. The concrete
[source map](implementation.md) assigns files to these boundaries.

## 4. Ports, use cases and composition

A port is a small contract for an actual external boundary: read diagnostic
facts, request Niri actions, observe Bluetooth facts, launch an app, or stage
managed theme files. Define ports alongside their app feature, implemented by
platform and by deterministic test fakes. Use typed inputs/results; don't pass
`serde_json::Value`, zbus connections or arbitrary shell strings through them.

Do not invent a trait for every struct or impose one CRUD-like interface on
unrelated services. Pure functions need no mock trait. Prefer concrete types
internally; generics or trait objects are chosen only where a real adapter/fake
substitution is needed. Wire dependencies explicitly in the CLI/shell startup
functions; no runtime DI container or global mutable singleton.

Example Bluetooth command path:

```text
user selects Connect
  → shell emits ConnectDevice(device_id)
  → app checks capability, target lifetime and conflicting operation
  → app records Pending(operation_id)
  → platform requests BlueZ operation through the Bluetooth port
  → service events/results are normalized into owned domain facts
  → app reducer reconciles confirmed/stale/failed state
  → snapshot reaches all views; UI renders state, not guessed success
```

A method reply may only mean request acceptance. Some effects complete after
a later signal; others are known complete from their return contract. Document
this per adapter. A launcher can confirm dispatch, not that the application
will remain running forever. Do not impose false success semantics on either.

## 5. State ownership and feature organization

Organize around features: diagnostics, windows/workspaces, apps, Bluetooth,
network, audio/media, power, notifications, themes and privacy. Each has a
small state model, commands, effects/results, reducer/use case and tests—not
one giant desktop `update()` that knows every protocol.

- Platform owns transport connections, protocol caches and raw incoming facts.
- App owns canonical **application** state: normalized backend snapshots,
  capability/freshness and operations. System services remain the ultimate
  authority for physical state; app never invents success.
- Shell owns local input/search/selection/expanded-card/focus state plus
  read-only app projections. A bar and dashboard share the same app source.
- Persistent config is user intent; runtime snapshots are observations. Do not
  write every backend event to config or pretend saved intent is hardware truth.

Each feature has a single writer (an app task/coordinator), not an
`Arc<Mutex<Everything>>` modified by widgets and workers. Immutable snapshots
may be shared with `Arc`. Carry a backend epoch and state revision so stale
results after restart cannot overwrite current state or act on reused IDs.
Cross-feature consistency is explicit: a lock privacy action coordinates
notifications/clipboard; it does not assume independent streams form an atomic
snapshot of the whole operating system.

## 6. Runtime, queues and task lifetime

Recommend **one Tokio backend runtime per process**, introduced when needed;
core stays synchronous and runtime-free. The UI/Wayland event loop runs on the
thread required by the selected toolkit. Platform native loops, if necessary,
have explicit thread/stop ownership; no nested `block_on` in UI callbacks.

T007 must choose one integration: a toolkit executor using the owned Tokio
runtime, or one dedicated backend runtime with a bounded UI bridge. Do not
combine a default toolkit-created Tokio with a second hidden backend Tokio.
A Wayland/calloop event loop is not another general-purpose async runtime.
Configure zbus for the selected runtime and inspect resolved features rather
than pulling Tokio and a second async I/O runtime in by default.

| Channel/work | Policy |
|---|---|
| Ordinary feature state | Latest complete snapshot may replace older snapshots; revisions let consumers detect missed changes |
| User commands | Bounded queue with explicit Busy/closed outcome; never silently drop or overwrite |
| Operation completion | Correlated response retained until caller delivery/expiry; a dropped view is not an undo |
| Authentication challenge | Purpose-specific, bounded, non-replayed path; no secret-bearing global event/history stream |
| Protocol deltas | Preserve required ordering; on gap/overflow mark stale and resynchronize, not apply an incomplete delta sequence |
| Thumbnails/search | Bounded concurrent work, query/generation IDs, discard obsolete results and cancel what is safely cancellable |

Starting engineering bounds to test: 128 queued commands, 64 KiB shell-control
request and 1 MiB Niri frame. These are proposals, not upstream guarantees;
limits must be documented, adjustable where justified and tested against large
legitimate sessions. Each backend gets its own timeout/backoff/concurrency
limits, not one arbitrary deadline for every operation.

Structured lifetimes: process → feature task → operation/view-bound task.
Retain join handles and cancellation ownership. Closing a popover cancels its
transient work, not the shared network/audio subscription. Shutdown stops new
commands, cancels safe pending work, closes listeners and waits with a bounded
budget. Do not exit midway through a durable commit without recovery state.

Cancellation/timeout after a side effect was dispatched can mean **outcome
unknown**, not “nothing happened.” Reconcile state. Never automatically repeat
pairing, authorization, power, deletion or formatting on reconnect. Coalesce
volume/brightness setter intent only where appropriate; completion semantics
still reflect confirmed backend state. No static-desktop 60-Hz poll loop.

## 7. Rust-native UI selection gate

Leading experiment: **Iced + `iced_exwlshell`**, not a frozen dependency choice.
[Source research](sources.md) found Iced 0.14.0 and an integration declaring the
0.14 family, but no NaliOS build/UI test has been run. Upstream is migrating
from `iced_layershell`; do not base new code on old examples blindly. The Iced
release README calls the toolkit experimental; support gaps are real risks.

The spike must demonstrate:

1. Ordinary settings window and anchored bar/overlays in the chosen runtime;
   do not run incompatible event loops on one main thread. If a separate
   process is necessary, record that trade-off before adding it.
2. Exclusive zone, fullscreen coexistence, overlay open/Escape/focus restoration,
   per-output surfaces, output removal and fractional scaling.
3. Text selection/editing, compose keys, an actual IME, font fallback/emoji and
   required scripts; verify screen-reader/AT-SPI reachability and keyboard use.
4. Event-driven rendering, no unconditional redraw feature, reduced motion,
   bounded GPU/RAM use and measured startup/idle on the actual laptop.
5. UI↔app bounded bridge, actual task cancellation/shutdown, no duplicate Tokio
   executor, and errors without UI stalls.
6. Theme style mapping, asset limits, renderer failure/fallback policy, exact
   crate/features/native library and license inventory.

A panel that draws is not a pass. Missing accessibility/input/normal-window
capabilities block claiming the corresponding experience. Do not silently
lower the requirements or reintroduce GTK: record the blocker and decide on a
maintained Rust alternative or author-approved native-toolkit exception.
Raw Smithay client toolkit is a lower-level option, not a ready widget engine.
No new custom locker even if the UI integration exposes session-lock APIs.

## 8. Independent Niri adapter and protocol contracts

Use Niri as a separate upstream process; do not link GPL `niri-ipc`/`niri-config`
into the proposed differently licensed application. Implement needed
interoperability from public documentation and authored test observations,
not copied Rust implementation/type definitions. Review legal/source duties
for any upstream binaries we redistribute separately.

Discover `$NIRI_SOCKET` in the selected session, validate endpoint use, bound
newline-delimited JSON, normalize supported messages and tolerate compatible
unknown fields/events. Event-stream initial state followed by updates is the
source for covered state; do not race a separate snapshot-then-subscribe.
Transient cross-object ordering is permitted. Reconnect resets epoch and
resynchronizes; no replay of unsafe actions. Version/capability gating must
use the actual selected release, not online “next release” documentation.

Keep protocol wire types separate from app/domain and persistent schema types.
CLI JSON report, CLI↔shell protocol, config, theme and Niri protocol have distinct
versions; changing one Rust struct is not permission to break them all.

## 9. CLI, doctor and one mutation owner

Doctor works without a shell or active Niri and must not initialize mutating
adapters. Passive checks do not start scans, register agents, claim singleton
bus names, unlock a keyring, request polkit, restart services or create config.
Avoid service auto-activation when probing D-Bus; query existing ownership and
use supported no-auto-start calls. Prefer file/environment/bus facts to
executing unknown programs from an untrusted PATH. Any executable version
probe needs an explicit trusted-program policy and bounded output/runtime.

A running shell owns its session's mutable workflows. CLI UI/device mutations
normally use the private control channel and the same app handlers. Headless
config/theme recovery may run the same workflow locally only after exclusive
transaction ownership; refuse if another owner is active. Never have a GUI
and CLI concurrently overwrite the same managed generation or each register
an authentication agent. Read-only diagnosis/discovery is a separate mode.

Control channel: private `$XDG_RUNTIME_DIR/nalios-dev/<session-id>/`, mode 0700,
socket 0600, peer UID and session checks, bounded framing/timeouts, protocol
handshake and request IDs. Handle stale endpoints only after ownership/liveness
checks. No arbitrary command execution, root authorization, passphrase or
credential methods. Same-UID access control is not a sandbox against hostile
programs already running as that user. Ambiguous sessions require selection.

## 10. Files, configuration, themes and recovery

Core parses/validates versioned data; app plans migrations/apply/revert; platform
performs filesystem/native validation/reload effects. Separate user source
config, package defaults and managed generated outputs. XDG path resolution
takes explicit inputs; a production adapter reads environment once, tests pass
fake paths. Runtime directories require valid ownership/modes and do not fall
back to a predictable world-writable `/tmp` socket.

Themes are semantic TOML data. They cannot execute scripts, inject templates,
fetch arbitrary remote assets or choose arbitrary output paths. Convert theme
roles to Rust UI styles; external app-format adapters are separately authored.
UI preview is nonpersistent. Respect supported GTK/Qt/browser settings without
claiming control of every upstream application's styling.

App owns the theme transaction: validate → plan → stage → journal → commit
steps → verify → mark active. Platform owns bounded/path-safe reads, checksums,
file replacement, native validators and reload effects. Per-file atomic rename
is not multi-file atomicity. Test interruptions, disk full, concurrent edits,
symlink/path attacks and journal/marker ordering. Keyboard/headless revert and
a built-in safe theme must work when the UI is broken. See [themes](themes.md).

## 11. Process, privilege and session boundary

Retain supported upstream Niri startup and one full graphical session per UID
initially. User bus/systemd environment can be shared across sessions. Nested
development does not run the production `niri-session` or take over desktop
singletons. Separate user-bus mocks do not isolate system-bus hardware actions.
See [session](session.md) for agents, portals, ownership and lifecycle.

External programs are called with typed, explicit argv, bounded output/time,
known scope and observed exit status; no shell interpolation. Launching an
installed desktop entry is an explicit user action with desktop-entry semantics,
not a privileged command API. Respect terminal/D-Bus activation, field codes,
Hidden/TryExec and locale rules; a parser alone is not a correct launcher.

Never run shell UI as root. Delegate PAM/polkit/secrets to reviewed purpose-
specific flows and secure locking to a proven compositor-backed locker.
Authentication data is excluded from generic Debug/Clone snapshots, process
args/env, logs and support bundles. Do not promise perfect memory erasure.

Shell-only restart leaves applications and the locker intact. Device/portal
service failure degrades its feature, not the entire session. Full compositor
or session failure has different consequences; do not promise it preserves all
applications. Do not use blanket process-name kills, silently modify PAM or
install passwordless administration rules. Later privileged helpers require
narrow caller/target authorization and an independent threat review.

## 12. Coding rules and enforceable architecture tests

- Core is safe Rust with `#![forbid(unsafe_code)]`; use safe Rust throughout by
  default. Any first-party unsafe/FFI needs a small reviewed boundary, safety
  invariants and tests, not an easy lint bypass.
- Use typed errors at library boundaries; expected missing/denied/busy/canceled/
  timeout/stale states are values, not panics. Reserve `unwrap`/`expect` for
  tests or a documented proven invariant. Prefer exhaustive enums over flags
  permitting invalid combinations; avoid blanket lint suppressions.
- No lock guard across `.await`; no blocking work on the UI or async executor.
  Bound blocking jobs and child processes. No fire-and-forget tasks without an
  owner, error path and stop behavior.
- Libraries do not initialize global logging/runtime/environment. Composition
  roots do. Logs use operation IDs and safe codes; no raw payload Debug dumps.
- Commit Cargo.lock, minimize features, review transitive/native dependencies,
  and keep normal dependencies separate from test/tool-only dependencies.
- Test reducers/config/migrations with ordinary Rust tests; fake ports drive
  permission loss, cancellation, timeout, reordered events and disconnects.
  Rust integration harnesses use disposable sockets/files/buses, never physical
  device mutations during ordinary `cargo test`.
- CI checks Cargo dependency direction and the **resolved transitive feature
  graph**, not just direct manifest names: no UI/native rendering graph in
  core/app/headless CLI. Platform features must not pull GTK/GIO in by default.
  Check forbidden environment/files/process calls in core/app through review
  and targeted tooling; Rust crate separation alone cannot forbid std I/O.
- Required jobs: fmt, selected-feature builds, clippy, pure/fixture/CLI tests,
  dependency/advisory/license review and architecture check. UI/native/hardware
  jobs are separate and opt-in where disruptive. No mock result counts as a
  real laptop acceptance pass.

A new crate/daemon needs a named consumer or measurable build/lifecycle/security
reason. A new abstraction needs a real use case and test, not speculative future
plugins. Each change should fit one feature with a testable failure/recovery
story. [First milestone](first-milestone.md) defines the next concrete work;
[delivery](delivery.md) defines when the finished desktop can be released.
