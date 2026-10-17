# Product brief

## Identity

NaliOS is a personal desktop environment/integration layer for **Kali Linux +
Niri**. It should feel cohesive and recognizably different without pretending
to be an independently developed operating system, kernel, or distribution
image.

Working promise: **a keyboard-friendly, visually coherent Kali laptop desktop
whose controls reflect real system state and whose failures are diagnosable.**

## User and target hardware

The primary user is the author. Public compatibility comes after the author's
laptop works reliably. The observed development laptop has:

- x86-64 AMD Ryzen 7 3700U / Radeon Vega graphics;
- about 14 GiB usable RAM;
- Realtek RTL8822CE Wi-Fi and a Bluetooth controller;
- an internal display, battery, and amdgpu backlight interface;
- an MX Master 2S mouse used for the Bluetooth acceptance scenario;
- AMD-V capability for the optional KVM tools.

These are observed facts, not a claim that all hardware behavior is supported.
Do not publish serial numbers, MAC addresses, SSIDs, machine identifiers, or
personal logs in fixtures/documentation. Record actual display modes/scales
and dock hardware during the hardware-baseline task.

## Priority order

1. Login, application launching, locking, and recovery.
2. Everyday laptop operation: networking, Bluetooth, sound, brightness,
   battery, lid, suspend/resume, external displays, and screen sharing.
3. Consistent shell: bar, launcher, dashboard, notifications, settings, themes.
4. Kali workflow and optional encrypted VMs.
5. Original artwork, animation, onboarding, and sharing-ready packaging.

Visual polish must not hide failed operations or delay the work needed to
make sleep, pairing, or credential prompts reliable.

## Main user journeys

### Start work

Choose the NaliOS session, unlock the existing user credential store through
its supported login mechanism, get a working network and usable desktop, and
launch terminal/browser/editor using keyboard or launcher. A missing optional
service must not prevent the session from starting.

### Connect a mouse or headset

Open Bluetooth controls, see adapter/power/block state, scan for a bounded
period, select the intended device, complete any authentication, and see the
actual result. Cancel safely. Reconnect after reboot or sleep without starting
a GNOME session first.

### Travel with the laptop

Change brightness/volume, inspect battery and network status, connect to a new
Wi-Fi network with a real secret prompt, lock, close the lid, and resume to a
locked session. Power behavior is explicit; do not fight logind with a second
independent lid/power policy.

### Connect a monitor and present

Apply a display profile, keep the launcher and panels reachable, respect
fractional scaling, and share the chosen screen/window through a working
portal/PipeWire path. Sensitive notifications/history are not exposed by
default on a lock screen or presentation mode.

### Change the desktop's character

Pick a valid theme and wallpaper, preview the result, apply it consistently,
and revert if necessary. Keep themes as data, not one Rust implementation per
palette. Third-party applications may require reopening and cannot all be
forced to use identical styling.

### Use Kali tools and VMs

Find installed security tools by category, launch the correct GUI/terminal,
and see missing dependencies or authorization needs clearly. No autonomous
network scans, interface mode changes, or attack activity occurs at login.
Open encrypted VMs deliberately and lock them when finished without losing
track of a running domain or mounted filesystem.

## Required first-party capabilities

The [component catalog](components.md) supplies detailed work packages.

- Session bootstrap, diagnostics, configuration, CLI, logs, and safe reload.
- Niri workspace/window integration and supported keyboard bindings.
- Application catalog/launcher, original bar, dashboard, and settings.
- Themes, wallpaper selection, notifications/DND, tray support, and OSDs.
- Bluetooth pairing, network/secrets, audio/media, power/battery/brightness.
- Correct locker/idle orchestration, a policy agent, credential storage, and
  desktop portals, even where those are delegated to existing programs.
- Clipboard/screenshot workflows with privacy controls.
- Display/input settings and a declarative Kali-tool catalog.
- Optional VM tools, explicit updates, installation/migration/rollback.

## Interaction commitments

- `Super+Space`: application launcher.
- `Super+Shift+Space`: full NaliOS dashboard, not a narrow status strip.
- A visible path to settings, lock, power actions, diagnostics, and recovery.
- Escape closes ordinary overlays and restores focus predictably.
- Full keyboard navigation and visible focus states; mouse-only controls are
  not sufficient.
- UI labels distinguish scan, discoverable, pair, trust, and connect; those
  operations are not interchangeable.

Other bindings remain configurable. A later design decision can preserve or
replace the prototype's additional shortcuts.

## Quality requirements

- All first-party executable behavior and tests are Rust. Keep a toolkit-free
  core and shared application workflows separate from platform adapters/UI;
  see the [architecture](architecture.md). External services/native libraries
  remain reviewed dependencies, not rewritten just to satisfy a language label.
- One owner per desktop service and clear lifecycle boundaries.
- Service state is authoritative; buttons do not report success before the
  backend confirms it.
- Non-blocking UI, bounded background work, capped logs/history, no constant
  animation/polling loop on a static desktop.
- Session/component restarts do not close user applications.
- Treat permission denied, no adapter, unsupported protocol, missing service,
  stale state, timeout, and canceled authentication as different conditions.
- No root shell UI; no secrets in logs, process arguments, exported environment,
  screenshots, or support bundles.
- Preserve user files and provide deliberate migration/rollback.
- Build dependency/provenance review into the workflow, not just the release.

Performance budgets should be set after measuring the target laptop with a
minimal session. Record CPU, RAM, wakeups, battery behavior, launch latency,
and missed frames in repeatable conditions. Do not advertise unmeasured
performance or security claims.

## Non-goals for the first version

- A new compositor, kernel, package manager, display manager, Bluetooth stack,
  hypervisor, or cryptographic format.
- A faithful Caelestia source port or a GNOME Shell replacement running inside
  GNOME.
- Zero GPL software anywhere in the installed Kali/Niri system.
- A plugin ecosystem executing arbitrary third-party code in the shell.
- A new PAM authenticator/lock implementation before session-lock expertise
  and dedicated security testing exist.
- A bootable ISO, broad hardware support, or a custom apt repository on day one.
- Rewriting terminal, browser, editor, file manager, and every Kali application.

## Releases mean different things

- **Developer preview:** small shell can run in a test Niri session. Not a
  daily-driver claim.
- **Personal daily-driver:** laptop acceptance tests pass, including cold boot,
  pair/reconnect, network credentials, lock/suspend, portals, and recovery.
- **Shareable release:** licensing/provenance, clean-machine installation,
  upgrade/uninstall behavior, support boundaries, and packaging are documented.
