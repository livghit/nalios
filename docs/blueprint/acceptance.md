# Acceptance, security, and laptop test plan

Tests are a product requirement, not something added after the UI looks right.
The user runs real hardware tests and approves any operation that can disrupt
network, sessions, storage, or guests. Do destructive/privileged experiments
only with disposable data and an accessible recovery path.

The [delivery contract](delivery.md#7-test-gates-and-evidence) maps these A01–A12
scenarios to release gates; the [session specification](session.md) and
[theme specification](themes.md) define the exact ownership, secure sleep and
apply/recovery behavior under test. The [research snapshot](../research/README.md)
is read-only metadata evidence, not a record of passing integration tests.

## Test layers

| Layer | What it proves | What it does not prove |
|---|---|---|
| Pure Rust unit/property tests | Parsing, validation, state transitions, bounds, migration/rollback logic. | Real hardware or correct OS service setup. |
| Protocol fixtures/mocks | Timeouts, denied auth, reconnect, event ordering, version compatibility paths. | Actual BlueZ pairing, Wi-Fi secrets, or suspend safety. |
| Isolated service/session tests | D-Bus ownership, IPC permissions, focus, rendering, process lifecycles. | That the real laptop has no driver/firmware/power issue. |
| Real hardware/integration tests | The laptop workflows described below. | Universal hardware support or a formal security proof. |
| Packaging/migration tests | Clean installation, upgrade, rollback, uninstall, data preservation. | Compliance on every distribution/release. |

Record revision, package versions, session, steps, expected/actual result,
redacted logs, pass/fail, and known limitations. Never claim a test passed
without running the relevant layer.

## Baseline before a rewrite touches the installed session

- [ ] Recovery login and TTY path are documented and usable.
- [ ] Current symlink/config/system-file inventory and uncommitted/untracked
  files have a recoverable backup.
- [ ] Record panel modes/scales, outputs/dock, keyboard layout, touchpad/mouse,
  battery/backlight, sound devices, and relevant kernel/firmware versions.
- [ ] Record service and portal ownership; know which agents supply polkit,
  NetworkManager secrets, and credential-store unlock.
- [ ] Establish idle CPU/RAM/wakeup, battery, launch responsiveness baselines
  under repeatable workload/brightness/power conditions. Set budgets from data.
- [ ] No MAC/SSID/serial/credentials/personal clipboard content enters fixtures
  or public bug reports.

## Critical laptop scenarios

### A01 — cold login and independent session

Boot into NaliOS/Niri without first starting GNOME. Bar, launcher, correct
network state, audio, Bluetooth, credential flow, and portals become usable.
Missing optional services give actionable errors, not a blank or blocked shell.
Logout/relogin and a shell-only restart create no duplicate daemons and leave
other sessions/application data untouched.

### A02 — application launching and keyboard focus

Launch Firefox ESR, terminal, editor, file manager, one GUI Kali tool, and a
terminal-only desktop entry. Exercise spaces, desktop-entry field codes,
Hidden/NoDisplay rules, missing binaries, repeated shortcut presses, and
cancellation. Escape and focus return are predictable. The launcher must not
parse an app request as arbitrary privileged shell code.

### A03 — Bluetooth mouse/headset

With the user's permission and intended device selected: bounded scan,
pair/authenticate, trust, connect, disconnect, forget/re-pair, and cancel.
Test the actual MX Master 2S after reboot and suspend/resume. Separately test
powered-off radio, rfkill, no adapter, BlueZ restart, wrong authentication,
and another agent being present. No indefinite scanning or unsolicited
approval of unrelated nearby devices. UI state comes from BlueZ confirmation.

### A04 — networking and credentials

Connect to saved and new Wi-Fi, submit a wrong secret then correct it, cancel,
disable/enable the radio, lose/recover signal, and inspect a supported VPN.
Record advanced enterprise/captive portal behavior as supported, delegated, or
unsupported. Credential-store unlock and polkit authorization work from a
cold Niri login. No secret appears in process args/env, logs, or support output.

### A05 — audio/media

Exercise speaker/headphone/HDMI/default-device changes, microphone mute,
volume limits, hotplug, Bluetooth audio if used, multiple MPRIS players, and
loss/restart of a backend. Media/OSD updates must not steal typing focus.

### A06 — battery, lid, idle, lock, suspend, and resume

Check AC/battery transitions, supported brightness controls, low-battery
policy, lock shortcut, idle lock, lock-before-suspend readiness, lid close/open,
manual suspend, docked behavior, media/presentation inhibition, and resume.
Test shell/locker failure paths in a safe test environment: an unlocked desktop
must not become visible merely because the shell crashed or resume raced the
locker. Do not change hibernation policy without checking host support/storage.

### A07 — displays, input, and accessibility

Internal display, external monitor connect/remove, scale changes, orientation,
invalid profile rollback, and output removal while an overlay is open. Keyboard
navigation, visible focus, screen-reader/accessibility exposure as supported,
contrast, text scaling, reduced motion, touchpad actions, and actual mouse input.
Unsupported gestures or controls are labeled, not silently accepted.

### A08 — portals and screen sharing

After cold Niri login: actual browser screen/window sharing, portal chooser,
file/open-URI actions, cancel/permission denied, and backend restart. Verify
that incompatible portal backends are not racing and that PipeWire sessions
end when sharing is stopped. A simulated successful D-Bus return is not enough.

### A09 — notifications, tray, and singleton ownership

Notify/replace/expire/close/action/DND behavior, bounded history, disappearing
tray items, menu actions, and locked/presentation privacy. Development must
not steal the active GNOME session's notification/agent service. Display
conflicts and instructions rather than terminating another desktop's process.

### A10 — clipboard, screenshots, and themes

History disable/pause/clear/expiry, oversized/image payloads, storage limits,
lock privacy, and unsupported protocol behavior. Screenshot area/output/window,
copy versus save, scaling, and cancellation. Theme preview/apply/revert,
malformed palette, missing asset, bad path, disk full, concurrent applies,
process kill during commit, and recovery after restart. No false claim of
perfect password detection or atomicity across many unrelated app files.

### A11 — VM lifecycle and authorization (later)

Disposable LUKS vault: create, unlock, register, start/view/stop, snapshots,
restore, unregister, unmount, and close mapping. Test denied auth, wrong
passphrase, insufficient space, guest shutdown timeout, busy mount, crashed
helper, canceled create, and interrupted cleanup. Do not reformat on import.

Test narrow-helper input validation, caller authorization, path/symlink races,
file ownership, forced-stop confirmations, and refusal to delete an active
vault. Verify what libvirt stores outside the encrypted vault: metadata,
snapshot state, dumps, logs, and guest memory may need separate handling.
Encryption at rest is not protection from the host while the vault is unlocked.

### A12 — packaging, updates, and recovery

Clean supported Kali install; dry-run before changes; repeated apply;
configuration migration; binary upgrade; apt failure; logout/relogin; reboot;
uninstall; rollback. Keep another login session/TTY available. No blanket
package removal, passwordless admin rule, driver/firmware replacement, deletion
of VM data, or loss of unrelated user configuration. Track the supported
Niri/system-service/toolkit versions without freezing Kali security updates.

## Rust architecture gates

In addition to functional scenarios, enforce the [architecture](architecture.md):

- [ ] First-party executable code/tests/automation are Rust; config/assets are
  non-executable data, not concealed shell/Python/QML behavior.
- [ ] Cargo graph follows core ← app ← platform with CLI/shell composition
  roots; app does not import platform, CLI does not import shell.
- [ ] Core and app tests need no display, D-Bus, root, network or real home;
  no renderer/GTK/GIO graph leaks into headless CLI through enabled features.
- [ ] CLI and GUI use the same workflow/state rules, with one mutation owner
  for each session/transaction; no direct widget-to-system calls.
- [ ] Runtime/executor ownership, bounded queues, task cancellation, unknown
  outcomes and stale-epoch results have tests.
- [ ] T007 proves Rust-native layer surfaces and normal-window strategy, real
  input/IME/accessibility, scaling/hotplug and idle rendering; no automatic
  fallback to a different toolkit or silent accessibility claim.

The [first milestone](first-milestone.md) narrows these gates to concrete initial
changes. Private scaffolding is not a distribution, hardware or UI-test result.

## Security release gates

- [ ] Shell/UI never runs as root.
- [ ] No arbitrary shell-command endpoint in local IPC or privileged helper.
- [ ] Runtime IPC has ownership/permission/peer checks and size/time limits.
- [ ] Credentials are handled only through reviewed, purpose-specific paths.
- [ ] Lock/suspend behavior uses a real session-lock mechanism and passes tests.
- [ ] Destructive actions identify the target and require appropriate consent.
- [ ] Logs/history/cache/support bundles have bounds and privacy rules.
- [ ] All external assets/dependencies have known license/provenance.
- [ ] Mock tests and actual hardware results are reported separately.

## Performance/quality record

Measure static desktop, launcher search, dashboard open, service reconnection,
and display changes. Note CPU/RSS/wakeups, frame behavior, responsiveness,
backend traffic, and battery conditions. Pause invisible animation/visualizers;
coalesce events; avoid permanent polling and unbounded queues/caches. Compare
with the baseline before choosing performance budgets or making public claims.

A developer preview can ship with documented missing features. Personal
**daily-driver** status requires critical laptop/security tests, and a
**shareable** release additionally requires clean-machine packaging and
licensing/provenance review.
