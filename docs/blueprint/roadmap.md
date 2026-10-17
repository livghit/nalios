# Roadmap, responsibilities, and first tickets

All application implementation is **planned**, not completed. The user writes
the code and owns the merge/release decisions. The assistant supports each
step with explanations, specification/code review, and test planning. Build
one verifiable increment at a time; do not create every component at once.

The [master plan](master-plan.md) groups this roadmap into usable vertical
slices. [Packages](packages.md), [sources](sources.md),
[implementation files](implementation.md), [session behavior](session.md),
[themes](themes.md), and [delivery gates](delivery.md) supply the detailed
contracts. Ticket numbers are identifiers, not an instruction to postpone the
T011 baseline until after T010. The [first-milestone brief](first-milestone.md)
now defines F01–F07 for T002–T005 with T001 dependency decisions and the read-only
T011 audit alongside. All first-party implementation/tests are Rust; the
[revised architecture](architecture.md) adds `nalios-app` and removes the
GTK-first assumption. Private scaffolding need not wait for every release choice.

Repository setup is complete: the new source is `~/nalios`, the prototype is
`~/.local/share/kalios-deprecated`, and the source/history snapshot is recorded
in the [boundary decision](../decisions/0001-repository-boundary.md). This
completes the naming/repository-separation part of T001/M0. The
[architecture scaffold](../decisions/0003-architecture-only-scaffold.md) added
five manifests, local dependency edges and inherited lints. F01–F03 are now
implemented (`nalios-dev --help`/`--version`/`config check`, pure paths and
config validation, app port plus platform file reads) with 49 offline tests;
T002 still needs fmt/clippy evidence, and all backend/UI features remain
unimplemented. License and hardware work remain open.

## A lightweight workflow for each ticket

1. User selects one ticket and records its acceptance criteria.
2. Identify the public API/protocol and review candidate dependency licenses.
3. User writes the smallest implementation and associated tests.
4. Assistant reviews the diff if requested: correctness, boundaries, failure
   handling, ownership/provenance, and test gaps.
5. User runs isolated tests, then the relevant real-session/hardware checks.
6. Record evidence, limitations, and next tasks before marking it done.

A ticket is not complete because a widget draws, a command exited zero, or a
mock test passed. Its specified failure/recovery behavior must be exercised.

## Milestones

### M0 — project contract and recoverable development environment

Components: C01/C25 groundwork; no session replacement.

- Agree dependency policy; keep first-party license selection explicit before
  distribution. Private local scaffolding can proceed without granting a license.
  Record that upstream Niri remains GPL and `niri-ipc` is not a permissive
  application dependency.
- Create a new repository/directory with original source, separate config,
  runtime sockets, binary names, and developer-only session startup.
- Preserve the old tracked changes, untracked docs, runtime links/configs, and
  a list of modified system files. Do not copy the vendor/assets tree into the
  new repository or break links by moving the old directory.
- Record hardware/service baselines, recovery login path, intended lid policy,
  and test accounts/session strategy.
- Audit the observed local Niri/satellite executables and decide a reproducible
  supply route (T011/T012). Cached package metadata is recorded under
  [research](../research/README.md), but source provenance and clean-install
  reproducibility remain unverified. Do not overwrite the current local install.

Exit: separate project and recovery are usable; scope/license decisions are
written; the old installation and data are intact.

### M1 — headless core and diagnostics

Components: C01, C02, C03.

- Versioned config, paths, structured errors and redacted logs.
- Read-only `doctor` that works without a GUI and reports platform capabilities.
- Introduce shared app workflows/ports and platform implementations without
  leaking I/O into core or duplicating logic in the CLI. Fake-host tests first.
- Independently implemented Niri snapshot/event client with fixtures/reconnect.
- CI/build formatting, linting, unit tests, dependency/license review.

Exit: one can inspect a real Niri session and explain failures from the CLI;
tests never write to the real home or invoke privileged operations.

### M2 — developer-preview shell

Components: C04, C05, C06, initial C08.

- Rust-native UI spike and written decision: Iced + `iced_exwlshell` first,
  with layer and normal-window strategy, IME/accessibility and single-runtime
  bridge verified. No assumed GTK fallback; no full dashboard/polished art yet.
- One original neutral theme, a simple bar, and app launcher.
- Private shell-control IPC, correct focus/keyboard dismissal, per-monitor
  placement and graceful start/stop/restart.
- Actual desktop-entry launching and visible missing-tool/launch errors.

Exit: `Super+Space` launches real applications in a test Niri session; restarting
the shell leaves applications open. Label this a developer preview.

### M3 — real device/service controls, Bluetooth first

Components: C11, C12, C13, initial C14/C16.

- Implement backend behavior and error states before elaborate UI.
- Complete BlueZ discovery plus pairing-agent lifecycle and user-selected
  device authentication. Test the actual mouse.
- NetworkManager connection/secret-agent integration and audio/media actions.
- Read battery/brightness capabilities and wire supported controls.

Exit: cold login and mouse/network reconnection work without a GNOME visit;
service restarts and denied/canceled operations have usable explanations.

### M4 — integrated laptop session

Components: C10, C14, C15, C16, C19.

- Original Niri session configuration and owned user-service lifecycle.
- Proven locking/idle/suspend integration and one lid/power policy.
- Polkit agent/credential store selection with tested authentication/unlock.
- Working portals, actual browser sharing, display profile and input behavior.

Exit: laptop acceptance scenarios pass after cold boot. Do not call this a
daily-driver until locking, credentials, portals, and recovery work.

### M5 — cohesive NaliOS workflow

Components: C07, C17, C18, C20, C21, C22, remaining C10.

- Full original dashboard on `Super+Shift+Space`.
- Standard notifications/actions/DND, tray/menu support, OSDs.
- Screenshot/clipboard workflows, settings, declarative Kali-tool catalog.
- Privacy/limits for histories and support bundles; accessibility/multi-monitor
  tests for every new surface.

Exit: core desktop workflows are coherent and no singleton service conflicts
with a recovery/development session. Known unsupported features are documented.

### M6 — original theme and wallpaper system

Components: complete C08/C09 and [design work](design.md).

- Stabilize semantic tokens/config formats before producing many themes.
- User designs original NaliOS identity, icons/assets where needed, and
  Kali-inspired wallpaper compositions with provenance.
- Implement preview/apply/revert, caches, app-theme adapters, and interrupted
  update recovery. Add reduced-motion/contrast checks.

Exit: original assets and theme sources are accounted for; a failed change is
recoverable. No artwork is imported merely because it existed in the prototype.

### M7 — optional VM and system administration suite

Components: C23, C24, update/setup parts of C25.

- Agree VM data/privilege model and migration format before coding a helper.
- Implement pure state transitions, read-only discovery, then reversible
  operations. Use disposable real vaults/domains for failure testing.
- Import existing vault metadata as untrusted data; never source it as shell
  or reformat an existing vault during import.
- Define explicit apt/setup plan, consent, failure reporting, and rollback of
  managed configs. Root actions are small and deliberate.

Exit: real guest lifecycle/recovery and privilege tests pass. Valuable VM data
is not used as an implementation experiment.

### M8 — personal release, then optional sharing

Components: C25 and all acceptance gates.

- Package first-party binaries/configs separately from system dependencies;
  prefer normal Debian packages/manifests over a new distro image.
- Test clean supported Kali installation, upgrades, logout/relogin, recovery,
  uninstall, and preservation of other sessions/user files.
- Record tested versions; avoid pinning the entire rolling distribution or
  blocking security updates to hide compatibility problems.
- Review authorship/licenses, native/transitive dependencies, notices, SBOM,
  asset provenance, and any redistributed upstream binaries.
- Retire the old runtime links only after a deliberate migration and rollback
  rehearsal. Keep historical source/notices where required.

Exit: personal daily use is proven; public release has documented support
boundaries and legal/packaging checks, not just attractive screenshots.

## First implementation tickets

T002–T010 describe the first code increments with dependency review as each
is added. T011's passive audit can proceed alongside private headless work;
licenses/source supply T012 must be resolved before distribution/clean-session
claims. Follow [F01–F07](first-milestone.md) for the initial implementation.
Paths below describe future code, not implemented programs.

| ID | What the user creates | Dependencies | Evidence needed |
|---|---|---|---|
| T001 | Project charter, chosen license/policy, original repository, decision log, dependency/asset registers. | M0 decisions | No copied vendor source/assets; ownership and external dependencies are explicit. |
| T002 | Workspace/local wiring is scaffolded only. Implement core + real CLI entry/tests first; app/platform effects next, shell after spike. | T001 language/boundary + initial dependency review; private `publish = false` | Clean build/test/fmt/clippy and dependency direction; no unreviewed linked dependency. |
| T003 | Pure core XDG-input policy, config schema/defaults/validation and errors; no environment/file reads. | T002 | Explicit fake paths/temp fixtures; malformed/unknown-version config never clobbers real settings. |
| T004 | App diagnostic workflow/ports + platform passive probes: session/socket/bus/BlueZ/tools/runtime permissions. | T003 | Fake-host then real passive checks; no service auto-start, scans, agents or file writes; off/absent/blocked/denied/unknown distinct. |
| T005 | Original Niri IPC snapshot client, then event synchronization/reconnect. | T003 | Fixture tests plus opening/moving/closing real test windows; no `niri-ipc` linkage. |
| T006 | Application catalog and standards-aware launching behavior. | T003 | Actual GUI, terminal, URL, spaces/field-code and missing-executable scenarios. |
| T007 | Rust-native Iced + `iced_exwlshell` spike; ordinary/layer-window strategy and one Tokio/Wayland bridge; record exact features and decision. | Dependency review; T005 for live-state integration | Focus/Escape, IME/text, real accessibility, scale/hotplug, idle rendering and task stop; no claimed pass from drawing alone. |
| T008 | Launcher view and private shell-control channel. | T006/T007 | `Super+Space` opens one launcher, launches selected app, restores focus; invalid/unauthorized IPC rejected. |
| T009 | Minimal bar/workspaces and one original semantic theme with live validated reload. | T005/T007 | No event polling loop; works at chosen scale and with launcher; invalid theme leaves last good state. |
| T010 | BlueZ backend model, bounded discovery, Agent1 flow, and simple device-control view. | T004/T007 | Real MX Master 2S pairing/reconnect plus canceled/failed auth and daemon restart tests. |

Do not begin T010 by drawing an empty device list and declaring Bluetooth
complete. Do not start the VM helper, custom locker, theme gallery, and tray
protocol simultaneously with T002.

## Integration, polish, and delivery tickets

All rows are **planned**. File/module roots refer to the
[source map](implementation.md); create only what the current ticket needs.
D01–D09 are open decision gates in the [master plan](master-plan.md).

| ID | Components / work | Depends on | Evidence needed |
|---|---|---|---|
| T011 | Baseline/recovery/source audit: actual binary paths/package ownership, service/agent/portal owners, config/link backup and test-user strategy. | Safe read-only access; T001 decisions can proceed alongside | Sanitized inventory, private recovery instructions, no system mutation; local compositor provenance gaps explicitly identified. Existing research is a starting point, not completion. |
| T012 | Reproducible Niri/satellite supply: selected sources, build/packaging recipe, session resources and `/usr/local` shadowing plan. | T011, dependency/redistribution review | Clean test Kali session + X11 app from documented artifacts; exact revisions/checksums and rollback; current local session untouched. |
| T013 | C12 network state/controls with delegated secret flow and advanced editor handoff (`platform/network`, shell controls). | T004/T007, D05 | A04 new/saved Wi-Fi, cancel/wrong secret, daemon restart; no credential logging. |
| T014 | C13 event-driven audio/default device/mic, MPRIS and OSD (`platform/audio`, media, shell controls/OSD). | T007, reviewed audio interface | A05 speakers/headphones/HDMI/microphone and player selection; OSD never steals focus. |
| T015 | C14 battery/backlight/power-profile capabilities and basic controls (`platform/power`). | T004/T007, permission review | A06 AC changes, supported controls, denied permissions, accurate unsupported state; no second lid policy. |
| T016 | C16 independent session startup/agent/keyring integration (platform session + user units/session config). | T011/T012, D05, T013–T015 | A01 cold login without GNOME first, correct secret/polkit prompts, logout/relogin, no duplicate owners or global env damage. |
| T017 | C15 lock/idle/inhibitor/suspend state machine and user-approved lid policy. | T016, D06 | A06 confirmed lock readiness, explicit suspend refusal on lock failure, lid/deadline/resume/fault tests in safe environment. |
| T018 | C19 portal routing/diagnostics, real browser sharing and chooser. | T012/T016 | A08 cold-login share/stop/cancel/file chooser, correct PipeWire/backend lifecycle; no global `GDK_BACKEND`. |
| T019 | C10 display/input profiles and keyboard-safe invalid-display rollback. | T005/T007/T016 | A07 actual scales/outputs/input, hot-unplug mid-overlay, invalid-profile revert; user rules preserved. |
| T020 | C07 full original dashboard using shared backend state and approved wireframes. | T009/T010/T013–T015, D04 | `Super+Shift+Space` opens full panel, keyboard navigation, useful cards, individual backend failure handling, laptop layout. |
| T021 | C17 original notification protocol/history/DND, deliberate migration from delegated mako. | T007/T016/T017, D08 | A09 actions/replacement/expiry/privacy/bounds on isolated bus then real apps; never steals another desktop's bus name. |
| T022 | C18 tray watcher/host/menu model and renderer. | T007/T016, protocol/dependency review | A09 real tray menu/activation, disappearing owners, malicious/oversized menu/icon bounds, documented legacy limits. |
| T023 | C20/C21 screenshot/annotation handoff and optional clipboard history/privacy picker. | T007/T017/T018, privacy policy | A10 target/scale/cancel, opt-in retention/pause/clear/expiry, payload bounds and lock/presentation privacy. |
| T024 | C10/C22 settings completion and original declarative Kali tool catalog. | T006/T013–T020 | A02/A07 keyboard settings, installed/missing GUI/terminal tools, no auto-execution or unnecessary tools metapackage. |
| T025 | C08 approved original visual identity and semantic dark/light theme pair, complete state sheet. | T009/T020/T024, D04 and asset-license decisions | Token/schema examples, all interaction states, actual-scale keyboard/contrast/text-scale/reduced-motion review. |
| T026 | C08 app-owned durable theme workflow, core data and platform file/validator/reload adapters for supported targets. | T003/T025, chosen target versions | A10 crash-at-every-step/disk-full/concurrency/user-edit tests; shell-only preview is nonpersistent; headless recovery. |
| T027 | C09 original wallpaper sources/exports, asset manifest, bounded previews and per-output apply. | T019/T025, asset provenance/license review | Missing/corrupt/huge images, crop/fill/output removal, fallback, reproducible exports and complete notices. |
| T028 | C25 versioned first-party Debian packages and explicit install/change-plan workflow. | T012/T016, runtime dependency graph | Clean builder, exact artifact/source inventory, correct dependencies/session entry, no postinst home edits or forced session switch. |
| T029 | C25 end-to-end install/upgrade/failure/rollback/uninstall and prototype migration rehearsal. | T017–T019/T026–T028 | A12 clean supported Kali VM and approved laptop rehearsal; repeated setup, apt failure, `/usr/local` shadowing, user/VM data preserved. |
| T030 | Desktop release review: provenance/notices/SBOM, support boundaries, performance/polish evidence and release notes. | T020–T029, all relevant A01–A10/A12 gates | Reproducible approved artifacts, measured laptop report, sanitized screenshots, known limitations, no unverified daily-driver/shareable claims. |

C23/C24 VM/helper implementation is deliberately **not** folded into these
desktop tickets. Define its own threat/data/authorization design and disposable
real-guest tests before issuing implementation tickets. A working desktop does
not need a custom privileged helper.

## Maintain the plan beside actual implementation

Keep these blueprint contracts in `docs/blueprint/` rather than duplicating
product/architecture files elsewhere. Add accepted choices to `docs/decisions/`
and real component status/test evidence beside the code as it is written.
Future additional records include:

- `docs/security.md`: threat boundaries, credential handling and recovery;
- Rust architecture/dependency checks and toolkit-spike evidence, including
  any native-library exception and measured support gaps;
- dependency review entries and `assets/provenance.toml` as dependencies/assets
  are actually accepted; the current register is only a candidate inventory;
- private/redacted hardware test records clearly distinguishing mocks from
  real session/hardware evidence;
- `docs/install.md`, `docs/troubleshooting.md` and release notes when the
  installer/packages exist.

Do not tick a ticket complete based only on this documentation, an empty module,
a screenshot, an apt candidate or a mock service return.
