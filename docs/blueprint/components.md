# Component catalog: what the user will write

**Code owner for every first-party component below: the user.** The assistant
helps explain interfaces, review designs/code, and plan tests. Upstream services
remain upstream-maintained. This is a capability inventory, not 25 daemons or
25 crates to create immediately. Milestones/tickets are in the
[roadmap](roadmap.md).

For every implemented component record its status, entry points, dependencies,
config keys, error states, tests, and the supported platform versions. Nothing
in this catalog is implemented merely because it is listed.

The [first-party source map](implementation.md) assigns these capabilities to
proposed Rust modules/files and tests. The revised [architecture](architecture.md)
separates pure core values, shared app workflows/ports and platform
implementations, with thin CLI/shell frontends. All first-party behavior/tests are Rust; create modules,
not a crate/daemon per row. The [package inventory](packages.md),
[source register](sources.md), [session contract](session.md), and
[theme specification](themes.md) define their concrete integration inputs.
The [master plan](master-plan.md) supplies the end-to-end execution order.

## Foundation

### C01 — configuration, state, and error core

- **Why:** all views/commands need one consistent model rather than ad hoc files.
- **Write:** versioned schema, defaults, validation, XDG paths, migration,
  atomic file operations/recovery, common operation/error types, mockable state.
- **Reuse:** reviewed serialization, filesystem, and logging crates.
- **Done when:** invalid config cannot replace the last good one; interrupted
  updates recover; tests use temporary directories and cannot touch real home.

### C02 — independent Niri adapter

- **Why:** workspace/window/output information and actions belong to Niri.
- **Write:** documented IPC client, snapshots/event subscription, reconnect,
  version/capability checks, focus/workspace/window commands.
- **Reuse:** upstream Niri as a separate process; **not its GPL Rust crates**.
- **Done when:** opening/closing/moving apps updates the bar correctly and a
  connection loss resynchronizes without freezing or duplicate actions.

### C03 — CLI, diagnostics, and redacted logs

- **Why:** recovery must work even when the UI does not.
- **Write:** `doctor`, config validation, theme operations, shell controls,
  text/JSON results, actionable dependency/permission/version checks.
- **Reuse:** reviewed CLI/logging libraries, service introspection.
- **Done when:** doctor explains a stopped BlueZ service versus an absent or
  blocked adapter and does not mutate the system or print secrets.

## Original shell UI

### C04 — UI primitives and surface/focus manager

- **Why:** every panel needs consistent rendering, keyboard behavior, and scale.
- **Write:** independent buttons/toggles/lists/cards, design-token mapping,
  loading/error states, keyboard navigation, overlay/surface lifecycle,
  monitor placement, animation/reduced-motion behavior.
- **Reuse:** chosen licensed toolkit and supported Wayland layer-shell APIs.
- **Done when:** overlays open/close without trapping focus; scale/hotplug and
  accessibility checks pass. A crash leaves normal applications running.

### C05 — application catalog and launcher

- **Why:** reliable launching is the first useful desktop feature.
- **Write:** XDG desktop-entry discovery/filtering/index, search/ranking,
  optional favorites, terminal/URL actions, launch result/error display.
- **Reuse:** a reviewed Rust desktop-entry parser plus explicitly tested
  launching/activation semantics, not a naive `Exec` split. GIO is no longer
  assumed under the Rust-native direction. A parser alone is not a launcher;
  review licenses, field codes, terminal and D-Bus activation behavior.
- **Done when:** Firefox ESR, a GUI tool, a terminal-only entry, names/paths
  with spaces, hidden/NoDisplay entries, and failed launches behave correctly;
  repeated shortcuts do not create duplicate launcher instances.

### C06 — bar and workspace controls

- **Why:** make workspaces, focus, time, and laptop status continuously visible.
- **Write:** configurable bar composition, per-output placement, workspace/app
  indicators, click/keyboard actions, clock/status popovers, sensible hiding.
- **Reuse:** C02 events and service snapshots, not shell-output polling loops.
- **Done when:** multiple outputs and fractional scaling work, window changes
  update promptly, and the static bar does not animate/repaint constantly.

### C07 — full dashboard and quick controls

- **Why:** one distinct NaliOS overview for status, media, tasks, and actions.
- **Write:** original layout, full-panel visibility/focus behavior, search/actions,
  configurable cards for workspaces, media, performance, devices, and power.
- **Reuse:** shared platform state and UI components.
- **Done when:** `Super+Shift+Space` always toggles the full dashboard; slow or
  absent backends degrade individual cards rather than breaking the panel.

### C08 — theme model, picker, preview, and application

- **Why:** a coherent identity needs a single configurable visual vocabulary.
- **Write:** semantic theme schema, validator, preview/picker, staged apply,
  rollback/recovery, supported app-output adapters, palette accessibility checks.
- **Reuse:** reviewed image/color libraries; independently authored templates
  or programmatic output, not automatic imports from the prototype.
- **Done when:** invalid data/disk failure cannot leave an unreported broken
  theme; preview does not persist; original dark/light test themes are usable.

### C09 — wallpaper and original asset management

- **Why:** personalize outputs without tying system behavior to image files.
- **Write:** asset index/provenance checks, preview, per-monitor selection,
  aspect/scale handling, bounded decode/cache work, safe apply/revert.
- **Reuse:** an external wallpaper renderer initially if appropriate and its
  license is recorded; custom Rust rendering can be a later choice.
- **Done when:** missing/corrupt/huge images and monitor changes do not crash
  the shell; every bundled asset has recorded provenance.

### C10 — settings, display, input, and profile UI

- **Why:** the laptop must be configurable without editing source code.
- **Write:** settings pages for supported shell options, keyboard shortcuts,
  display profiles, layout/scale, touchpad/mouse preferences, theme/power choices.
- **Reuse:** supported Niri configuration/IPC; validate behavior changes with
  native tools. Clearly label settings that require a reload or login.
- **Done when:** unsafe display changes can revert, per-device config does not
  erase unrelated user rules, and keyboard-only recovery remains possible.

## Everyday laptop integrations

### C11 — Bluetooth controller, pairing agent, and controls

- **Why:** a device list alone cannot handle authentication or reconnection.
- **Write:** BlueZ adapter discovery/power/block status, bounded scans, device
  selection, Agent1 interaction, pair/trust/connect/disconnect/forget flows,
  progress/cancel/error states, service-restart reconciliation.
- **Reuse:** BlueZ D-Bus APIs; kernel drivers/firmware/rfkill. Do not implement
  Bluetooth transport or blindly auto-approve pairing requests.
- **Done when:** the actual MX Master 2S pairs, reconnects after cold boot/sleep,
  wrong/canceled authentication is handled, and no GNOME visit is required.

### C12 — networking, VPN visibility, and secret prompts

- **Why:** Wi-Fi credentials and authorization are part of desktop integration.
- **Write:** adapter/connection state, network list, explicit connection actions,
  saved connection selection, VPN visibility/actions where supported, redacted
  errors and clear handoff to an existing editor for advanced settings.
- **Reuse:** NetworkManager and a working secret-agent/credential-store flow;
  document/delegate advanced enterprise/captive portal behavior initially.
- **Done when:** new and saved Wi-Fi connections work, cancel/wrong secret is
  recoverable, radio loss does not freeze the UI, credentials are not logged.

### C13 — audio, microphone, media, and OSD

- **Why:** headphones, meetings, and media need correct source/sink state.
- **Write:** device/default selection, volume/mute, source control, hotplug,
  media-player state/actions and OSD updates with coalescing.
- **Reuse:** PipeWire/WirePlumber-compatible control interfaces and MPRIS.
- **Done when:** switching headphones/HDMI/mic works, permission/device changes
  appear, multiple media players are handled, and OSD does not steal typing focus.

### C14 — battery, brightness, power profile, and laptop policy

- **Why:** battery life and safe sleep matter more than decorative animations.
- **Write:** battery/AC/remaining estimate display, low-battery notices,
  brightness controls, supported power-profile controls, capability/errors.
- **Reuse:** UPower, logind, supported backlight/power-profile interfaces; use
  delegated helpers if permissions require them. Do not invent battery estimates.
- **Done when:** battery/AC changes update, unsupported features are disabled,
  brightness permissions are handled, and one documented lid/power policy applies.

### C15 — lock, idle, and suspend orchestration

- **Why:** an attractive fullscreen window is not a secure lock screen.
- **Write:** state/commands coordinating a proven locker, idle/inhibit handling,
  lock-before-suspend with verified readiness, wake/resume error/recovery paths.
- **Reuse:** existing compositor-backed session-lock/PAM implementation and
  logind inhibitors; do not write a new password authenticator for the preview.
- **Done when:** suspend/resume never briefly exposes the desktop, locker/UI
  crashes do not unlock it, and presentation/inhibition semantics are tested.

### C16 — session startup, authentication, and credential-store integration

- **Why:** many desktop failures are missing agents/environment, not widgets.
- **Write:** session preflight, owned-process lifecycle, supported session
  environment setup, agent detection, health/errors, explicit fallback paths.
- **Reuse:** upstream Niri session model, login manager, logind/systemd user
  services, one polkit agent, and a credential store with supported unlock.
- **Done when:** a cold NaliOS login works without another desktop running;
  logout/relogin does not duplicate services or kill unrelated session processes.

### C17 — notifications, history, and do-not-disturb

- **Why:** apps need the standard notification API, not only our own toasts.
- **Write:** compatible notification service, actions/close/replacement semantics,
  bounded history, DND policy, per-app controls, privacy-aware presentation.
- **Reuse:** freedesktop notification protocol; separate test bus for conflicts.
- **Done when:** actions/expiry/replacement work, the correct bus owner is used,
  no GNOME service is stolen, and sensitive content is hidden while locked.

### C18 — application tray and menus

- **Why:** some VPN, sync, and Kali/utility apps depend on a status indicator.
- **Write:** StatusNotifier host/watcher behavior where applicable, icon/menu
  rendering, tooltips, activation, item removal, bounded untrusted menu handling.
- **Reuse:** StatusNotifierItem and DBusMenu protocols, reviewed libraries.
- **Done when:** real tray items/menu actions work, disappearing apps clean up,
  and unsupported legacy tray behavior is documented rather than promised.

### C19 — desktop portals, permissions, and screen sharing

- **Why:** browsers, Electron apps, and sandboxed apps need these to function.
- **Write:** session portal selection/configuration, capability diagnostics,
  onboarding instructions, integration tests and failure/handoff UX.
- **Reuse:** compatible upstream portal backends, PipeWire and Niri support;
  do not write a portal backend just to claim ownership of everything.
- **Done when:** actual browser screen/window sharing, file chooser/open-URI,
  and relevant permission dialogs work after a cold NaliOS login.

### C20 — screenshot workflow

- **Why:** capture and share selected content with clear privacy semantics.
- **Write:** area/window/output actions, optional save/copy/annotation handoff,
  error/status UI, configurable destination, redaction-aware history policy.
- **Reuse:** Niri screenshot or supported portal interfaces; not Sway-specific
  tree commands assumed to work on Niri.
- **Done when:** correct target/scale is captured, clipboard/disk behavior is
  explicit, and cancellation or denied access leaves no unwanted saved image.

### C21 — clipboard history and picker

- **Why:** improve workflow without turning the shell into a secret recorder.
- **Write:** optional bounded history, text/image handling, picker, clear/pause,
  expiry/storage policy, app-specific hints where supported, lock privacy.
- **Reuse:** supported Wayland clipboard/data-control facilities or a reviewed
  external history backend at first. Detect unsupported compositor capabilities.
- **Done when:** copy/paste works, history can be disabled/cleared, items expire,
  oversized payloads are bounded. Do not promise perfect password detection.

## Kali and administration tools

### C22 — Kali application/tool catalog

- **Why:** offer Kali-oriented navigation without hard-coding shell pipelines.
- **Write:** original declarative categories/metadata, installed-command checks,
  terminal/GUI launch definitions, documentation links, missing-package hints.
- **Reuse:** installed Kali programs and reviewed desktop-entry launch behavior.
- **Done when:** category search is useful, missing tools are marked, no command
  runs just because the menu opened, and privilege requests are explicit.

### C23 — encrypted VM library, CLI, and later management window

- **Why:** retain the useful VM workflow with recoverable state transitions.
- **Write:** defined vault/domain metadata, import validation, operation state
  machine, start/stop/unlock/lock/snapshot/restore/delete UX, recovery reporting.
- **Reuse:** libvirt/KVM/QEMU and cryptsetup. Review libraries' licenses before
  linking. Do not reimplement encryption or assume isolated bridge means air gap.
- **Done when:** disposable real guests survive full lifecycle/failure tests;
  existing vault import never reformats data or executes metadata as shell.

### C24 — narrowly privileged VM/setup operations

- **Why:** filesystem/crypto/network changes may need authorization, UI does not.
- **Write:** an explicit allowed-action contract and audited validation,
  caller/session-bound authorization, safe secret transport, cleanup/recovery.
- **Reuse:** polkit and established system utilities/APIs; security review before
  making a persistent helper. A helper is not required just to start the UI.
- **Done when:** unauthorized/path-traversal/arbitrary-command requests fail,
  secrets are absent from logs/argv/env, and interrupted operations are recoverable.

### C25 — installer, updates, onboarding, and release packaging

- **Why:** a cohesive personal desktop must be installable and reversible.
- **Write:** dependency/version preflight, explicit change/dry-run plan,
  user/system scope separation, package manifests, onboarding, migration,
  uninstall/rollback, documentation and redacted support-bundle generation.
- **Reuse:** apt/dpkg/systemd and the existing login manager. No custom package
  manager, broad distro pins, automatic full upgrades, or self-updating root UI.
- **Done when:** a clean supported Kali machine can install/upgrade/uninstall,
  existing sessions/user files remain, and a rollback does not need the GUI.

## Intentional external components

Kernel/drivers/firmware, Niri, BlueZ, NetworkManager, PipeWire/WirePlumber,
UPower/logind, portals, a locker, polkit/credential agents, libvirt/QEMU,
cryptsetup, terminal/editor/browser/file manager, and package management are
not new NaliOS implementations. Their selection, licenses, configuration,
version support, and tests are still NaliOS integration work.

Future custom Rust replacements for delegated agents/renderers require their
own justification and security/compatibility tests. They are not prerequisites
for an original NaliOS application codebase.
