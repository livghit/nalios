# Roadmap, responsibilities, and first tickets

All application implementation is **planned**, not completed. The user writes
the code and owns the merge/release decisions. The assistant supports each
step with explanations, specification/code review, and test planning. Build
one verifiable increment at a time; do not create every component at once.

Repository setup is complete: the new source is `~/nalios`, the prototype is
`~/.local/share/kalios-deprecated`, and the source/history snapshot is recorded
in the [boundary decision](../decisions/0001-repository-boundary.md). This
completes only the naming/repository-separation part of T001/M0. License,
dependency decisions, hardware-baseline work, and Rust implementation remain
open; no Cargo workspace exists yet.

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

- Agree first-party license and dependency policy; record that upstream Niri
  remains GPL and `niri-ipc` is not a permissive application dependency.
- Create a new repository/directory with original source, separate config,
  runtime sockets, binary names, and developer-only session startup.
- Preserve the old tracked changes, untracked docs, runtime links/configs, and
  a list of modified system files. Do not copy the vendor/assets tree into the
  new repository or break links by moving the old directory.
- Record hardware/service baselines, recovery login path, intended lid policy,
  and test accounts/session strategy.

Exit: separate project and recovery are usable; scope/license decisions are
written; the old installation and data are intact.

### M1 — headless core and diagnostics

Components: C01, C02, C03.

- Versioned config, paths, structured errors and redacted logs.
- Read-only `doctor` that works without a GUI and reports platform capabilities.
- Independently implemented Niri snapshot/event client with fixtures/reconnect.
- CI/build formatting, linting, unit tests, dependency/license review.

Exit: one can inspect a real Niri session and explain failures from the CLI;
tests never write to the real home or invoke privileged operations.

### M2 — developer-preview shell

Components: C04, C05, C06, initial C08.

- UI-toolkit spike and written decision; no full dashboard or polished art yet.
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

These are a suggested serial order; the user can revise it. File paths are
suggestions to create in the **new** repository, not files generated here.

| ID | What the user creates | Dependencies | Evidence needed |
|---|---|---|---|
| T001 | Project charter, chosen license/policy, original repository, decision log, dependency/asset registers. | M0 decisions | No copied vendor source/assets; ownership and external dependencies are explicit. |
| T002 | Minimal Cargo workspace, `nalios-core` and `nalios-cli`, build/test workflow. Add platform/shell crates only as work begins. | T001 | Clean build/test/fmt/clippy; no GPL application dependency accepted accidentally. |
| T003 | Core XDG paths, config schema/defaults/validation and structured error type. | T002 | Temp-directory tests; malformed/unknown-version config cannot clobber real settings. |
| T004 | Read-only doctor probes: session, Niri socket, D-Bus/service state, BlueZ, required tools, runtime permissions. | T003 | Works from a recovery session; off/absent/blocked/unauthorized are distinguishable; output is redacted. |
| T005 | Original Niri IPC snapshot client, then event synchronization/reconnect. | T003 | Fixture tests plus opening/moving/closing real test windows; no `niri-ipc` linkage. |
| T006 | Application catalog and standards-aware launching behavior. | T003 | Actual GUI, terminal, URL, spaces/field-code and missing-executable scenarios. |
| T007 | UI spike: ordinary window plus Niri layer surface; document toolkit/runtime choice. | T001/T005 | Focus, Escape, input, scaling, monitor removal, accessibility, performance observations. |
| T008 | Launcher view and private shell-control channel. | T006/T007 | `Super+Space` opens one launcher, launches selected app, restores focus; invalid/unauthorized IPC rejected. |
| T009 | Minimal bar/workspaces and one original semantic theme with live validated reload. | T005/T007 | No event polling loop; works at chosen scale and with launcher; invalid theme leaves last good state. |
| T010 | BlueZ backend model, bounded discovery, Agent1 flow, and simple device-control view. | T004/T007 | Real MX Master 2S pairing/reconnect plus canceled/failed auth and daemon restart tests. |

Do not begin T010 by drawing an empty device list and declaring Bluetooth
complete. Do not start the VM helper, custom locker, theme gallery, and tray
protocol simultaneously with T002.

## Suggested paper files in the new project

- `docs/product.md`, `docs/architecture.md`, and `docs/decisions/`.
- `docs/components.md` with per-component status and known limitations.
- `docs/acceptance.md` plus private/redacted hardware test records.
- `docs/security.md` with threat boundaries and recovery procedures.
- `docs/provenance.md`, dependency review register, asset manifest.
- `docs/install.md`, `docs/troubleshooting.md`, and release notes when applicable.

Keep this blueprint as a reference. Write the user's own implementation
contracts/decisions alongside the new code as actual choices become clear.
