# First-party source map and implementation contracts

**Status: early implementation; most feature paths/APIs below remain planned.**
The workspace exists, `nalios-cli` implements help/version/config-check/doctor/
niri-inspect/niri-watch, core has tested pure paths, config validation and Niri
domain values, and app/platform implement config, diagnostics and an
independent Niri snapshot client plus bounded event stream with a shared
reducer, stale-state marking and reconnect backoff. Shell behavior is not
implemented. See the
[scaffold decision](../decisions/0003-architecture-only-scaffold.md) and
[workspace guide](../../crates/README.md). Do not create every planned module
now. C01–C25 are capabilities, not 25 daemons. All first-party executable
code/tests are Rust; `nalios-app` owns shared workflows.
See [decision 0002](../decisions/0002-rust-codebase-boundaries.md), the
[architecture](architecture.md), and the concrete [first milestone](first-milestone.md).

## 1. Workspace and repository shape

The tree mixes existing code and future files: Cargo manifests/lock, package
roots, the first CLI/core code and resource directories exist. Toolchain
pinning, detailed feature modules, config/theme documents, artwork, package
recipes, fixtures and `xtask` remain absent until their tickets start. The
shell still uses a temporary library target; the CLI now has an explicit
development binary.

```text
Cargo.toml                         workspace, shared dependency policy
Cargo.lock                         resolved application graph (commit it)
rust-toolchain.toml                only if the chosen toolchain route uses it
crates/
  nalios-core/                      pure models, schemas, validation, domain values
  nalios-app/                       shared workflows, state reducers, ports, task lifetimes
  nalios-platform/                  implementations of Linux/protocol/filesystem ports
  nalios-cli/                       headless presentation and composition root
  nalios-shell/                     original Rust-native UI and composition root
config/
  defaults.toml                    authored shell defaults
  niri/                            original KDL fragments, no upstream config crate
  portals/                         session-specific backend selection
  delegated/                       authored locker/idle/wallpaper/notification config
assets/
  themes/                          versioned semantic theme documents
  wallpapers/sources/              editable original work, when created
  wallpapers/exports/              generated final assets, when created
  icons/                           original shell-specific icons, if needed
  provenance.toml                  source/license/export metadata for bundled assets
packaging/
  debian/                          chosen source/binary package recipes, later
  systemd/user/                    session-bound user units, later
  sessions/                        declarative entry; any first-party wrapper logic in Rust
  applications/                    first-party settings/diagnostic desktop entries
  completions/                     generated/checked CLI completion packaging
xtask/                             optional Rust development automation, added when needed
tests/fixtures/                    sanitized shared data for Rust test targets
docs/                              specifications, decisions, evidence templates
```

Crate-local unit/integration tests belong beside the crate, with actual Cargo
test targets. A virtual workspace root's `tests/` directory is not automatically
executed; use it for shared fixture data, not silently unrun test programs.
No root helper, VM executable, plugin loader, or persistent backend daemon is
required for the first shell. Split only for a justified lifecycle/security need.

Allowed dependencies (`A → B` means A imports B):

```text
nalios-app      → nalios-core
nalios-platform → nalios-app, nalios-core
nalios-cli      → nalios-app, nalios-platform, nalios-core
nalios-shell    → nalios-app, nalios-platform, nalios-core
```

App defines external-effect ports; platform implements them; the frontends wire
concrete adapters at startup. Widgets and CLI commands use shared app workflows,
not direct service calls. Core is pure; app has no concrete I/O/toolkit/native
handles; platform has no widgets. Select platform features so headless diagnosis
does not pull a renderer/GTK/GIO graph. See [sources](sources.md) for candidates.

## 2. `nalios-core/src/` — pure contracts

| Proposed path | Responsibility / output | Required tests |
|---|---|---|
| `lib.rs`, `error.rs` | Public models; typed redacted errors with stable codes, context and recovery hints | Error classes/serialization do not disclose secrets |
| `paths.rs` | Namespace/path descriptions, XDG precedence inputs; actual OS checks in platform | Explicit temp-home inputs; never discover/write the test runner's real home implicitly |
| `config/{mod,schema,validate,migrate}.rs` | Versioned shell config, defaults, supported-key validation and migrations | Invalid/unknown versions, ranges, cross-field constraints, migration round trips |
| `state/{mod,capability,operation}.rs` | Live/stale/unavailable snapshots; target IDs; pending/cancel/result transitions | Out-of-order updates, cancellation, disconnect, impossible transitions |
| `theme/{mod,schema,color,contrast,validate}.rs` | Semantic theme roles, bounded tokens, contrast pairs and schema validation | Malformed colors/missing roles, opacity/contrast assumptions, limits |
| `theme/plan.rs`, `transaction.rs` | Pure plan/journal value models and invariants; orchestration lives in app | Valid plans and states, schema round trips; execution fault tests in app |
| `control.rs` | Versioned CLI↔shell request/result models; allowlisted actions | Unknown versions/methods, limits, duplicate IDs and safe errors |
| `privacy.rs` | Shared history/retention/redaction policy models | Disabled-by-default optional recording, bounded retention |

Do not put device-specific sysfs paths, external process spawning, credential
storage, environment discovery, or toolkit color objects into these types.
Persistent formats need schema versions; preserve compatible unknown data
where intentional and reject unsafe
interpretations, rather than silently accepting misspelled critical settings.

## 3. `nalios-app/src/` — shared application behavior

| Proposed module/path | Owns | Boundary tests |
|---|---|---|
| `lib.rs`, `runtime/{engine,supervisor}.rs` | Explicit service construction inputs, feature handles, task lifetime/shutdown policy | No hidden adapters/global runtime; fake ports start/stop deterministically |
| `config/{port,check,migrate}.rs` | Request bounded config reads, invoke pure validation, plan explicit migration | No writes during check; file effects only via port; migration backup policy |
| `diagnostics/{port,report,run}.rs` | Passive probe contract, health policy and report aggregation | Missing/denied/unknown/optional state; no mutating port construction |
| `niri/{port,state,reduce}.rs` | Workspace/window/output app state, freshness/epoch, supported action intent | Initial events, reconnect, stale IDs, transient reference order |
| `apps/{port,catalog,search,launch}.rs` | Catalog policy, ranking/favorites and explicit launch intent/result | Hidden entries, obsolete queries, dispatch not fabricated lifetime success |
| `bluetooth/{port,state,reduce,commands}.rs` | Pair/connect/scan workflows, intended device/auth challenge lifetime | Confirmed state, cancellation, blocked radio, wrong device, reconnect |
| `network/`, `audio/`, `media/`, `power/` | Feature-specific policies/state/ports, not concrete service APIs | Authoritative state, absent capability, denied operation, late result |
| `theme/{ports,plan,apply,recover}.rs` | Single apply owner, journal sequencing, validation/apply/revert/recovery | Fail every effect step, disk-full fake, concurrent edit/apply, partial output recovery |
| `session/`, `privacy/` | Lock-ready/sleep requests and coordinated history/presentation policy | No suspend on failed explicit lock, privacy ordering, bounded cancellation |
| `notifications/`, `tray/`, `displays/`, `capture/`, `clipboard/`, `kali/` | Later workflow/state contracts, introduced only per ticket | Limits, disappearance, rollback and explicit user intent |
| `control.rs` | Allowlisted dispatch to existing workflows, not a second implementation | Duplicate requests, active ownership, unsupported version/method |

These names are proposed; a small feature can start as one file. Avoid a trait
per method or a generic service framework. Ports use owned typed values; app
does not import `nalios-platform`, zbus, GTK, Iced, raw Wayland or native handles.
Tokio may support feature task ownership; core stays runtime-free. Clock/time
and external-effect inputs are controllable in tests. One app feature task
owns its state; frontends consume projections rather than reimplementing it.

## 4. `nalios-platform/src/` — independently authored adapters

Each directory below normally contains `mod.rs`, a small public model/adapter,
private protocol parsing, and tests as needed—not a mandatory four-file pattern.

| Module/path | Inputs / authoritative output | Failure and integration requirements |
|---|---|---|
| `xdg.rs`, `files.rs` | Validated path policy → safe managed reads/writes/staging | Ownership/symlink/path checks, bounds, disk-full and interrupted writes; no recursive home rewrites |
| `niri/{client,wire,state}.rs` | `$NIRI_SOCKET`, documented JSON → windows/workspaces/outputs, action results | Independent types, message bounds, initial event-stream state, unknown fields, reconnect/resnapshot; no linked `niri-ipc` |
| `bus.rs`, `health.rs` | Session/system D-Bus endpoints and executable/package probes → capabilities | Missing bus versus missing owner versus denied service; name-owner changes invalidate stale state |
| `control/{client,server}.rs` | Private session socket → allowlisted shell-control operations | Peer UID, socket/directory ownership, size/time limits, protocol version, per-session identity |
| `apps/{discover,desktop,launch}.rs` | XDG entries + explicit launch request → normalized facts + dispatch receipt | Reviewed Rust desktop-entry semantics, locale/Hidden/TryExec/OnlyShowIn, field codes, terminal and D-Bus activation; parser alone is insufficient; no shell splitting/GIO-by-default |
| `bluetooth/{state,client,agent}.rs` | BlueZ objects/properties/Agent1 calls → device state + explicit auth challenge | Bounded scanning, intended device identity, cancel/timeouts, radio block/absence, daemon loss; no auto-accept |
| `network/{state,client,secrets}.rs` | NetworkManager objects → devices/profiles/active connections | Delegate secrets initially; wrong/canceled credentials, service restart, VPN editor handoff; no logged SSIDs/secrets by default |
| `audio/{state,client}.rs`, `media.rs` | Chosen Pulse-compatible subscription + MPRIS → sink/source/player state | Worker/mainloop integration, defaults/hotplug, microphone privacy, multi-player selection, no pactl-output polling |
| `power/{battery,profiles,brightness}.rs` | UPower/profile APIs and bounded helper calls → real capabilities/state | AC changes, unsupported profiles, permission checks, command confirmation, helper timeout |
| `session/{environment,ownership,lock,inhibit}.rs` | Explicit session identity + supported agents/logind/locker → health and lock/sleep coordination | No global environment clobbering, duplicate starts, blanket kills, or fake lock window |
| `portals.rs` | Installed/active broker/backend config → diagnosis/test handoff | Session routing, file chooser, GNOME screencast/PipeWire path; no custom portal backend |
| `notifications/{service,wire,store}.rs` | Standard notification calls/storage effects → normalized app inputs | Protocol mechanics only; app owns policy/history bounds; original service replaces delegated owner only after tests |
| `tray/{watcher,item,menu}.rs` | StatusNotifier/DBusMenu objects → bounded item/menu models | Existing watcher cooperation, remote icon/menu limits, owner disappearance, no legacy XEmbed promise |
| `wallpaper.rs`, `theme/{store,outputs}.rs` | App-selected transaction steps → bounded filesystem/validator/reload effects | App owns commit/recovery order; adapter owns safe paths, atomic file primitive, native validation and effective-state observations |
| `displays.rs`, `input.rs` | Supported Niri config/IPC + user profile → validated settings change | Connector identity not serial dumps; timed display revert; user rule preservation |
| `capture.rs`, `clipboard.rs` | Explicit capture/copy/history action → result and policy state | Cancellation, payload bounds, opt-in history, no background password detection claim |
| `kali.rs` | Original declarative tool catalog + installed entry lookup → categories/actions | Metadata is data, not sourced shell; opening a category never executes tools |
| `vm/{wire,client}.rs` | Later: app-defined VM requests → libvirt/tool adapter effects | Lifecycle policy belongs in app/separately reviewed VM tool; no root helper or valuable writable vaults in desktop MVP |

Public adapter seam: **capabilities + snapshot + commands + events/results**.
Use operation IDs and stable target identities. A D-Bus method returning does
not automatically mean the requested physical connection completed. Service
confirmation updates the displayed final state. Pending operations are visible;
reconnect must not automatically repeat destructive/authentication actions.

## 5. `nalios-cli/src/` — headless access and recovery

| Path | Proposed commands / behavior |
|---|---|
| `main.rs`, `args.rs`, `output.rs` | Parsing, stable exit classes, text/JSON output; no raw error/debug dump containing secrets |
| `commands/doctor.rs` | Read-only probes of versions, path ownership, session, services, agents and capabilities |
| `commands/config.rs` | Validate/explain config and intentional migration plan |
| `commands/niri.rs` | Read-only state inspection; explicit supported focus/workspace actions only if needed |
| `commands/shell.rs` | Open/toggle launcher/dashboard/settings; status/reload/restart requests scoped to a known session |
| `commands/theme.rs` | List/check/preview/apply/revert/recover managed themes |
| `commands/session.rs` | Session health/preflight; lock/suspend requests using the secure orchestration contract |
| `commands/install.rs` | Later: explicit package/config dry-run and apply workflow, not implicit mutation in doctor |
| `commands/support.rs` | Later: previewed, bounded, redacted support bundle, opt-in collection only |

Development binaries must not shadow existing installed prototype commands.
Proposed development names: `nalios-dev` and `nalios-shell-dev`; production
`nalios` naming is activated only during deliberate migration. Commands in
these documents describe future interfaces, not usable programs today.
Diagnostic-only adapter construction must not register agents, acquire desktop
singleton names, start scans, request authentication or mutate service state.

## 6. `nalios-shell/src/` — original Rust presentation

| Path | Owns |
|---|---|
| `main.rs`, `bootstrap.rs` | Composition root: wire app ports/adapters, select one runtime/bridge, instance/control registration, orderly shutdown |
| `bridge.rs`, `view_model.rs` | Toolkit messages ↔ app commands/snapshots; UI projections and view-local work cancellation, no duplicated backend state owner |
| `surfaces/{manager,output,focus}.rs` | Layer/toplevel creation, output identity/hotplug, keyboard focus/dismissal, recovery |
| `ui/{button,toggle,list,card,dialog,search}.rs` | Accessible reusable primitives and all interaction states |
| `style/{tokens,components}.rs` | Validated semantic tokens → Rust widget styles; no CSS/GTK assumption or executable theme |
| `bar/` | Per-output workspaces, app/status indicators and clock; read existing shared state |
| `launcher/` | Query/results/actions/favorites and predictable launch/focus behavior |
| `dashboard/` | Full `Super+Shift+Space` overview with useful device/media/workspace/power cards |
| `controls/` | Bluetooth/network/audio/power popovers and pending/error/auth handoff views |
| `settings/` | General/appearance/keyboard/input/displays/privacy/integrations/diagnostics pages |
| `notifications/`, `tray/`, `osd/` | Render adapter state; do not duplicate protocol owners or steal typing focus |
| `themes/`, `wallpapers/` | Picker/preview/status/revert UI using the same headless apply engine |
| `capture/`, `clipboard/`, `kali/` | Explicit workflow UIs with privacy and launch policies |

The first Rust-native spike evaluates Iced + `iced_exwlshell`: ordinary settings
window, bar and overlays with proven focus/input/scaling/accessibility behavior.
Verify they can coexist under the chosen loop rather than assuming a layer-shell
adapter also solves normal toplevel windows. Do not request exclusive input for
every surface. No secure lock renderer lives in this shell. Failure leads to
a reviewed toolkit decision, not an automatic GTK fallback or custom UI engine.

## 7. State, concurrency, and IPC contracts to settle before coding

- One Tokio backend runtime per process, integrated with the toolkit-owned
  UI/Wayland loop. T007 must prove executor reuse or one explicit backend bridge;
  no hidden second Tokio, blocking device scan or nested event-loop spin.
- State updates may be coalesced by target/revision. Authentication challenges,
  completion/error results, and destructive operations must not be dropped as
  “old UI frames.” On overflow, resnapshot safe state or report backpressure.
- Provisional limits to test: 64 KiB shell-control request, 1 MiB Niri frame,
  128 queued UI commands, at most one conflicting operation per target.
  These are starting bounds, not upstream protocol guarantees; large legitimate
  sessions may need negotiated/adjusted limits with bounded memory.
- Ordinary IPC request timeouts should be short and cancellable; device/auth
  operations have separate bounded, visible deadlines. A CLI timeout is not
  permission to retry pairing, power, deletion, or formatting automatically.
- Control methods are named actions, not arbitrary argv or shell strings.
  Same-UID checks are a local boundary, **not a sandbox against malicious
  programs already running as that user**. Never route credentials here.
- D-Bus object paths/window IDs are identities within a backend lifetime.
  Revalidate after daemon/compositor restart; do not apply a stale selection to
  a newly reused target ID.

## 8. Coverage of all catalog components

| Components | Main first-party source | External boundary / gate |
|---|---|---|
| C01, C03 | Core config/value types + app diagnostics/workflows + platform files/probes + CLI | Temp-home tests; doctor read-only |
| C02 | Platform Niri protocol + app reducer + shell projection | Upstream IPC fixture/live-window tests |
| C04–C07 | Shell primitives/surfaces/views + app apps/windows workflows | Toolkit spike; desktop-entry tests; focus/multi-output |
| C08–C09 | Core theme + app transaction + platform file/renderer effects + shell pickers + assets | Native validators, journal fault injection, asset provenance |
| C10 | Platform displays/input + shell settings + original KDL | Unsafe display revert and user-rule preservation |
| C11–C14 | Platform Bluetooth/network/audio/media/power + shell controls | Real pairing/secrets/audio/backlight tests |
| C15–C16 | Platform session + packaging session/user units | Proven locker, PAM/keyring/agents, cold login |
| C17–C18 | Platform notifications/tray + shell renderers | Singleton/protocol/privacy limits and real applications |
| C19 | Platform portals + portal config + diagnostic UI | Actual browser sharing and chooser after cold boot |
| C20–C22 | Platform capture/clipboard/Kali + shell views | Explicit capture/retention/launch behavior |
| C23–C24 | Later VM module/application/helper specification | Separate privilege review and disposable real guests |
| C25 | CLI install/support + packaging + docs/tests | Clean install/upgrade/uninstall/rollback and licensing |

In every remaining capability, app owns workflow policy and platform owns
concrete effects; the abbreviated table does not move behavior back into UI or
drivers. For every path added, the ticket records source/API references,
dependency review, configuration, privacy, failure handling, tests and actual completion
status. An empty file tree is not an implementation milestone.
