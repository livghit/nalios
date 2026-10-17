# NaliOS delivery plan: from Kali + Niri to a coherent desktop

**Status: proposed implementation plan, not installed software.** The author
writes the application; this plan specifies the work. Package observations were
made on **2026-10-16**, on Kali Rolling 2026.3 / amd64. Nothing in this plan
selects a first-party license, approves every dependency, or changes the host.

## 1. What we are building

A complete, original **desktop layer for Kali Linux using upstream Niri**:
our own Rust shell, interaction design, configuration, themes, dashboard, and
integration, sitting on maintained system services. This is more than a color
preset, but not a new distribution, compositor, kernel, or package manager.

The finished experience must deliver both halves of the promise:

- **Works:** independent cold login, real credentials/pairing, launching, audio,
  secure locking/sleep, displays, screen sharing, diagnosis, and recovery.
- **Feels beautiful:** original composition, consistent typography and spacing,
  responsive keyboard interaction, readable dark/light themes, restrained
  motion, and coordinated supported applications—not just a pretty bar.

All first-party executable code, UI, tests and automation stay in **Rust**.
The revised [architecture](architecture.md) adds a shared `nalios-app` layer and
prioritizes a Rust-native UI spike instead of GTK. Native OS/graphics boundaries
remain explicit; see [decision 0002](../decisions/0002-rust-codebase-boundaries.md).

Preserve Kali's package/security-update model and existing recovery session.
Do not replace the installed prototype until a tested migration is approved.

## 2. The detailed planning set

| Document | Question it answers |
|---|---|
| [Rust architecture](architecture.md) | Which layers may depend on each other, who owns state/effects, and what rules do tests enforce? |
| [First milestone](first-milestone.md) | Exactly which small changes create the first useful headless program? |
| [Package plan](packages.md) | Which runtime/build/optional packages, why, and which combinations conflict? |
| [Source and dependency register](sources.md) | Where does each code/dependency family come from, and how do we review/pin it? |
| [First-party source map](implementation.md) | Which Rust files/modules do we write, with what contracts and tests? |
| [Session integration](session.md) | Who owns startup, authentication, portals, locking, agents, and recovery? |
| [Theme and experience specification](themes.md) | What does our desktop look/feel like, and how do themes safely reach applications? |
| [Delivery and release plan](delivery.md) | How do we build, package, install, upgrade, undo, and prove the result? |

These extend—not replace—the [product](product.md), [architecture](architecture.md),
[C01–C25 catalog](components.md), [roadmap](roadmap.md), [acceptance tests](acceptance.md),
[creative brief](design.md), and [ownership policy](ownership.md).

## 3. Evidence and the biggest early blocker

Read-only inspection found:

- `niri --version` reports **26.04**. `niri`, `niri-session`, and
  `xwayland-satellite` resolve under `/usr/local/bin`; `dpkg-query -S` did not
  identify a package owner at those paths. Their full provenance is not yet
  established. Do not silently overwrite them.
- Neither `niri` nor `xwayland-satellite` has a candidate in the cached Kali
  indexes examined. This is **not proof of absence from every repository**.
  We did not refresh apt or add repositories.
- Native GTK4/layer-shell package availability was established, but GTK is no
  longer the assumed frontend after the Rust-only clarification. Iced plus
  `iced_exwlshell` is the new experiment; its version-family metadata is not a
  successful build or UI test. See [sources](sources.md).
- The current package name is `network-manager-applet`, not the old
  `network-manager-gnome`. `mate-polkit` is an available standalone agent
  candidate; do not assume old `policykit-1-gnome` instructions still apply.
- Upstream Niri documents GNOME + GTK portal backends, PipeWire, and the
  supported Niri session startup path. Its default config is not a complete
  NaliOS integration.

**Therefore the first integration task is reproducible Niri/satellite sourcing,
not downloading a theme gallery.** Existing locally working programs are useful
for development, not sufficient evidence for clean-machine installation.

## 4. Decisions: established direction versus recommendations

Established: Kali + upstream Niri, original Rust-only first-party code,
author-written implementation, prototype separation, no new locker/crypto
stack. The five-package structural
[scaffold](../decisions/0003-architecture-only-scaffold.md) and a first
CLI/paths increment exist. Runtime/toolkit/dependency choices below still
require review; no feature is completed by scaffolding or a help command.

| ID | Decision | Recommendation | Must be resolved before |
|---|---|---|---|
| D01 | Code/docs/artwork licenses | Review MIT OR Apache-2.0 for original code; decide docs/assets separately. No license selected; private scaffold can proceed with publishing disabled. | Before distributing/licensing first-party work; dependency review at each addition |
| D02 | UI and async runtime | Spike Iced + `iced_exwlshell`; one Tokio backend runtime with a verified bounded UI bridge/executor arrangement. No assumed GTK fallback. | Production shell work after T007 |
| D03 | Niri/satellite supply | Prefer supported Kali packages if available; otherwise reviewed, versioned local Debian packages from upstream releases. | Reproducible session install |
| D04 | Layout and initial visual identity | Compact top bar, separate launcher and full dashboard, original Night/Day theme pair. | UI composition beyond primitives |
| D05 | Login/credential/agent strategy | Existing display manager/PAM; GNOME Keyring, `mate-polkit`, NM applet secrets initially. Test real unlock. | Cold-login milestone |
| D06 | Lid/idle/power policy | One logind lid owner, swayidle + swaylock coordination, no hibernation promise yet. | Suspend acceptance |
| D07 | Default applications | Existing Alacritty/Firefox ESR/Thunar first; accessible terminal alternative and user overrides. | Launcher/session defaults |
| D08 | External service replacement | Delegate first, replace notification/Bluetooth UI deliberately; never run competing owners accidentally. | Each custom protocol service |
| D09 | First release scope | Daily-driver desktop first; encrypted VM suite optional and later. | Sprint selection |

Each decision record needs context, alternatives, exact versions/evidence,
choice, consequences, and a revisit trigger. Do not mark a proposal accepted
just because it appears in a file tree or dependency table.

## 5. Build in usable vertical slices

There is no fixed calendar estimate yet. A session/agent/toolkit spike may
change effort substantially. Work on **one slice at a time**, with one clear
exit test, rather than implementing 25 empty modules at once.

| Slice | Existing milestone/tickets | Deliverable and exit gate |
|---|---|---|
| S0 — recoverable foundation | M0, T001; T011–T012 | Dependency policy, open license decisions tracked, package/source baseline, backups/recovery and compositor supply plan. Read-only audit can run alongside private S1; installs wait for approval/recovery. |
| S1 — useful headless tool | M1, T002–T005 | Config validation + read-only doctor + Niri state/events. Build/test/lint from a clean checkout; no GUI or root needed for doctor. |
| S2 — first coherent preview | M2, T006–T009 | Toolkit spike, app launcher, minimal original themed bar, shell control IPC. Real apps launch; shell restart leaves apps open. |
| S3 — everyday controls | M3, T010, T013–T015 | Real mouse pairing/reconnect, Wi-Fi secret flow, audio/mic/media, battery and brightness. Backend state is authoritative. |
| S4 — independent laptop session | M4, T016–T019 | Owned startup, secure lock/suspend, agents/keyring, portals, input and display rollback. Pass cold boot without entering GNOME first. |
| S5 — complete NaliOS interaction | M5, T020–T024 | Full dashboard, notifications/DND, tray, settings, capture/clipboard privacy, Kali catalog. Keyboard and multi-output checks included. |
| S6 — distinctive visual finish | M6, T025–T027 | Original dark/light identity, wallpapers with sources, theme preview/apply/revert, supported app adapters, reduced-motion and contrast checks. |
| S7 — shippable desktop | M8, T028–T030 | Debian packages, clean-machine install/upgrade/uninstall, rollback rehearsal, legal/source inventory, measured quality report. |
| S8 — optional VM edition | M7, separately scoped | Disposable-guest lifecycle and narrow privilege boundary. Not a blocker for the desktop release. |

Visual quality starts in S2: spacing, focus, typography, theme roles, and
loading/error states are foundational. S6 adds identity and expressive art;
it is not the first time we consider design.

## 6. Milestone feature contract

| Capability | Preview | Personal daily-driver | Shareable desktop release |
|---|---|---|---|
| Bar, launcher, keyboard navigation | Required | Required | Required |
| Recovery/doctor and safe config reload | Required | Required | Required |
| Device controls | May be delegated | Real laptop scenarios pass | Documented supported hardware/API scope |
| Lock, cold login, credentials, portals | Existing safe setup retained; no daily-driver claim | Required, tested | Clean installation also tested |
| Full dashboard, settings, tray/notifications | May be missing explicitly | Required for the intended finished desktop; delegate where documented | Ownership and compatibility documented |
| Original dark/light themes + wallpaper | One neutral original theme | Theme pair and safe revert | Licensed editable sources and exports |
| Other applications' styling | Explicit support list | Coordinated supported defaults | Limitations documented |
| Clipboard history | Off unless explicitly enabled | Optional, privacy rules tested | Same; not a compulsory recorder |
| Encrypted VM tools | Out of scope | Optional | Optional separate package/profile |
| ISO / replacement package manager | Out of scope | Out of scope | Out of scope for this plan |

A functional daily-driver candidate can be tested before the complete visual
finish. Do not call the whole desktop finished until its interaction and design
acceptance gates also pass.

## 7. Risk register and stop conditions

| Risk | Prevention / stop condition |
|---|---|
| Unowned `/usr/local` compositor binaries mask package versions | Record actual executable paths and upstream revisions; stage replacement packages in a test VM first. Stop if rollback would break the current session. |
| Global user bus/environment conflicts | One full graphical session per UID initially; nested tests do not run `niri-session` or acquire production singleton services. |
| Attractive but insecure lock | Delegate to proven session-lock/PAM; require lock readiness before requested suspend and test failure paths. No custom password UI. |
| Pairing/Wi-Fi UI without working agents | Write/test the authentication flow, or explicitly delegate it; a device list is not completion. |
| A rolling update breaks the shell | Record tested versions, run upgrade smoke tests, disable unsupported controls, retain recovery. Do not freeze Kali security updates. |
| Theme apply damages user configuration | Only declared managed files; stage/validate, journal, detect concurrent edits, rollback. No recursive home-directory theme rewrite. |
| GPL/prototype source or art inadvertently imported | Independently authored application; source/license register and asset review before adding files. |
| Scope balloons into VMs/ISO/custom services | Desktop release excludes these until daily workflows and installation pass. |
| Hidden performance cost | Event-driven adapters; bounded histories/work; measure laptop idle/search/animation before setting budgets. |
| Unverifiable release claims | Distinguish planned, built, mock-tested, session-tested, hardware-tested, and packaged status. |

## 8. Working record for every implementation ticket

Use the existing [ticket workflow](roadmap.md#a-lightweight-workflow-for-each-ticket).
Each ticket should additionally identify:

- component IDs and concrete source paths from [implementation](implementation.md);
- direct packages/crates and upstream API references from [packages](packages.md)
  and [sources](sources.md), with review status;
- happy path, cancellation, permission/service failure, restart, and recovery;
- data collected, persistence/retention, logs/redaction, and authorization;
- relevant A01–A12 acceptance scenarios and actual evidence;
- user-visible limitations and any decision it blocks.

**Immediate next step:** use the [first-milestone change sequence](first-milestone.md)
for T002–T005: core/CLI → pure config → app/platform probes → read-only Niri
state. T001 dependency reviews and T011 read-only baseline proceed alongside;
license choices remain explicit before distribution. No need to settle the
whole gallery/toolkit before pure tests. Do not start with an installer or mass
package-install command.
