# Build, packaging, installation, and release gates

**Status: proposed delivery contract, not a runnable installer.** Default to
Kali's apt/dpkg and the existing display manager. No ISO, new package manager,
self-updating root UI, or blanket dotfiles replacement is needed.

## 1. Resolve compositor supply before claiming reproducibility

Observed on 2026-10-16: Niri 26.04 runs from `/usr/local/bin/niri`; the session
script and satellite also resolve in `/usr/local/bin`. Their paths were not
identified as owned by dpkg in the read-only check. Cached apt had no Niri or
satellite candidate. The old installation must keep working while we audit it.

Order of preference:

1. Recheck supported Kali package availability at install planning time; prefer
   normal distro packages if suitable, documenting exact tested versions.
2. If unavailable, review a maintained compatible Debian packaging route and
   its trust/update model; do not assume a random third-party repository or
   an Ubuntu PPA is safe for Kali.
3. If needed, build versioned local `.deb` artifacts from **selected upstream
   releases** in a clean builder with recorded commits/archive checksums,
   Cargo.lock/vendor inputs, native dependencies, patches and build features.
   This is a separate upstream supply task, not GPL source copied into our app.

A self-built Niri package must include upstream session/portal/systemd resources
in correct distro locations and explicitly account for dynamically loaded
`libwayland-server0`. Do not use `--all-features`. Satellite needs its own
source/dependency/release record; establish its actual revision, not an assumed
version from its existence. Supply corresponding sources/notices when required
for redistributed binaries.

`/usr/local/bin` usually precedes `/usr/bin`: installing a Debian package may
leave the old binary in use. The migration must inventory executable paths,
service `ExecStart` paths and desktop entries; choose an explicit switch with
backup/rollback. Never silently overwrite/remove local binaries or leave a
new compositor paired with an old session script accidentally.

Exit gate: a clean test Kali environment can reproduce the selected Niri +
satellite session and X11 app behavior from documented artifacts. Existing
local installation alone does not satisfy this gate.

## 2. Build pipeline

### Application workspace

- Review dependencies when added; choose first-party licenses before sharing.
  Private local scaffolding can use `publish = false` with D01 still open;
  do not invent a license on the author's behalf.
- Five manifests exist; core + CLI have their first real increment. Continue
  with config parsing, app/platform probes and UI after its spike. Follow
  [first milestone](first-milestone.md).
- Keep first-party code/tests/migrations/automation in Rust. Use Cargo and an
  optional Rust xtask, not Python/Bash logic. Conventional Debian glue is
  minimal invocation only, under [decision 0002](../decisions/0002-rust-codebase-boundaries.md).
- Pick the distro Rust or rustup route deliberately. Do not install conflicting
  toolchain packages or silently upgrade the developer's toolchain.
- Commit Cargo.lock, record toolchain/native versions, build with locked inputs.
  A chosen supported Rust floor must actually build the selected crate family.
- Run format, clippy, unit/property tests, fixture tests, advisory/license checks
  and a native build against the documented Kali baseline. No real-home writes,
  privileged operations or physical device changes in default tests.
- Keep release feature sets explicit. Generate dependency/SBOM inventories for
  the enabled target, not a misleading list of all optional crate features.
- Separate safe headless CI from user-approved graphical/hardware tests. Never
  mark a mock pairing/lock/screen-share response as a real integration result.

### Release builder

Use a clean VM/container or other documented Kali builder for compile/package
checks; a container alone cannot prove display-manager/PAM/suspend/GPU behavior.
Decide whether Debian builds use packaged Rust dependencies, a reviewed vendor
archive, or another reproducible workflow. `dh-cargo` does not automatically
solve crates.io network/offline/review requirements. Pin inputs for a release,
not the entire rolling distribution indefinitely.

## 3. Proposed first-party Debian artifact split

These are **proposed future package names**, not installed packages or approved
control files. Avoid unnecessary fragmentation if a smaller split is clearer.

| Artifact | Content | Dependency/lifecycle boundary |
|---|---|---|
| `nalios-cli` | Headless binary, help/man/completions, diagnostic schemas | Works without shell/renderer/GTK/GIO graph; uses shared app workflows |
| `nalios-shell` | Original Rust UI binary and resources | Rust-native toolkit candidate and reviewed native graphics/input boundaries; never a root service |
| `nalios-session` | Original config templates, session entry/glue, owned user-unit/portal integration | Depends on the resolved upstream compositor supply; no login-manager replacement |
| `nalios-themes` | Approved original theme definitions, artwork exports and notices | Can be architecture-independent; include editable-source/provenance distribution plan |
| `nalios-desktop` | Optional metapackage for the approved daily desktop profile | Explicit runtime dependencies/recommendations, not `kali-linux-everything` |
| `nalios-vm` | Later optional VM application/integration | Separate privilege/data review; no desktop prerequisite |

Core/app/platform are libraries linked into the appropriate executable, not
three additional daemons/packages by default. Cargo crate names do not need a
one-to-one Debian package mapping. Use generated
shlib dependencies for linked libraries and explicit dependencies for programs
invoked at runtime. Validate public package names/collisions before publishing.

System packages own binaries/default templates. User configuration/state are
created or changed only by an explicit user-scope setup/apply flow, not by root
postinst guessing which user's home directory to edit. Do not automatically
enable a new session, switch the active display manager, or start per-user
agents for every account during package installation.

## 4. Installation as an explicit plan

Future CLI setup produces a human-readable and machine-readable plan before
any change:

1. Supported OS/architecture and actual executables/versions; current session
   and recovery route; available space and backup destination.
2. Direct packages plus apt's simulated solver result: installs, removals,
   replacements, recommendations, services and estimated space. Unexpected
   removal of a recovery desktop/driver/security component stops the plan.
3. Existing local/prototype binaries, symlinks, units, autostart, Niri config,
   portal config and user dotfiles that might conflict.
4. Exact proposed files, scope, ownership/modes, backups, managed markers and
   validation/recovery steps. No arbitrary recursive dotfile import.
5. Separate explicit authorization for apt/system files and consent for
   user-scope config changes. Never run the shell UI as root.
6. Install/stage first, validate next, enable/select the new session only after
   consent. Keep the current session/recovery desktop untouched.
7. First-login health check and guided unresolved tasks, without storing any
   password in argv, env, logs or install manifests.

Repeated setup must be idempotent for already managed state. Detect unrelated
user edits and ask rather than overwriting. A read-only `doctor`/`plan` does
not request authorization or silently apply a missing dependency.

## 5. Migration from the prototype

- Preserve the [repository boundary](../decisions/0001-repository-boundary.md),
  prototype pending changes, source/notices, compatibility alias and existing
  VM/user data until the migration gate passes.
- Develop with `nalios-dev` commands and XDG paths. Production names may
  currently resolve to prototype behavior: audit paths before invoking them.
- Build an explicit link/file inventory with ownership and hashes. List which
  production names will switch and how to undo each switch.
- Import only deliberately supported **data** formats as untrusted input.
  Do not copy old source/templates/art into the new repository, source shell
  metadata, run arbitrary legacy hooks or automatically migrate credentials.
- Test the new session in an isolated user/VM, then rehearse a reversible switch
  on the laptop with TTY/recovery available.
- Retire legacy links only after cold login, laptop acceptance and rollback
  pass. A source snapshot is not a full machine or VM-data backup.

## 6. Updates, rollback, and uninstall

Keep NaliOS artifact version, schema version, theme generation, dependency/API
support and upstream compositor version distinct. A config migration needs a
backup/recovery journal and tests, not merely a new package version.

User explicitly requests apt updates; there is no hidden login-time upgrade or
root self-updater. Before publishing a new NaliOS release, smoke-test against
current supported Kali package versions and record the result. If upstream APIs
change, detect incompatibility and disable unsafe controls with a useful error.
Do not hold the whole distro back or disable security updates to preserve appearance.

Rollback restores known-good **managed config/artifacts** where available;
it cannot promise to reverse arbitrary apt upgrades, kernel changes, credential
changes or unrelated user edits. A whole-system snapshot/backup is a separate
mechanism. Preserve previous compatible NaliOS packages/config backups and
record which rollback paths were actually rehearsed.

Uninstall removes package-owned files and deliberately enabled NaliOS services/
entries. User data remains unless a separate, previewed purge is explicitly
approved. Never delete home directories, VM vaults, saved networks, keyrings,
unrelated packages, another desktop, firmware, or generic `/usr/local` content.
Restore previous managed integration only when it is still the value we wrote;
do not overwrite a newer user change with an old backup.

## 7. Test gates and evidence

Every test record includes repository revision, OS/kernel/Niri/native package
versions, environment/mode, steps, expected/actual result, sanitized evidence,
pass/fail and limitations. Keep raw personal host evidence under ignored
`private/` or an external private store; publish only reviewed redacted records.

| Gate | Required tests / evidence | Failure blocks |
|---|---|---|
| G0 — foundation | Source/license decisions, package origin inventory, backups and TTY recovery | Any installation/session replacement |
| G1 — headless correctness | Temp-home config/parser/property tests, bounded IPC fixtures, doctor non-mutation/redaction | Preview binaries |
| G2 — UI preview | Toolkit focus/Escape/scaling/hotplug spike, real desktop-entry launch, shell restart with apps surviving | Developer-preview claim |
| G3 — independent laptop | A01–A09: cold login, actual mouse/network/audio, agents/keyring, secure sleep/lock, displays, portal sharing | Personal daily-driver claim |
| G4 — complete experience | A09–A10 + original full dashboard, theme pair/state sheets, keyboard/contrast/privacy checks, measured responsiveness | “Finished NaliOS desktop” claim |
| G5 — distribution | A12 on clean supported Kali: install, repeat setup, upgrade/failure, rollback, uninstall/data preservation; license/notices/SBOM | Shareable release |
| G6 — VM add-on | A11 with disposable real guests, fault injection, narrow-helper authorization review | VM feature release only |

A02 launching must include spaces/field codes/terminal/Hidden/failed executable.
A03/A04 must include cancel/wrong auth and service restart. A06 must exercise
lock readiness and safe fault paths, not only successful sleep. A08 must use a
real browser stream. A10 must kill the apply process between commit steps.
A12 must include the `/usr/local` shadowing case and apt failure. See the full
[acceptance plan](acceptance.md) instead of treating this table as a substitute.

## 8. Performance and polish, measured before promises

Establish baseline and repeat measurements for idle desktop, launcher search,
dashboard open, theme reload, backend reconnect and multi-output changes:
CPU, RSS, wakeups, launch/input latency, frame behavior and battery conditions.
Use repeatable brightness/power/workload and distinguish nested versus native
session results. Track metrics/regressions with exact builds before choosing
budgets; no unmeasured “lightweight,” battery-life or zero-frame-drop claims.

Pause hidden animations/visualizers, bound queues/caches/histories, coalesce
state, and avoid timer polling where APIs provide events. Measure at actual
laptop scale and reduced-motion settings, not only a powerful build machine.

## 9. Finished means all of this

- Original, coherent shell and approved dark/light identity; provenance is known.
- Kali/Niri cold login works independently with real agents and secure locking.
- Launch, devices, sound, battery, displays and screen sharing pass actual tests.
- Theme/config operations fail safely and headless recovery is documented.
- No duplicate owners, root UI, secret logs or silent package/system mutations.
- Clean install/update/uninstall and prototype migration are rehearsed.
- Supported/unsupported hardware and third-party styling limitations are honest.
- License/dependency/artwork review is complete for what is actually distributed.

A screenshot is evidence of appearance only, not evidence of any other gate.
