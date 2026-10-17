# Niri session, service ownership, and recovery plan

**Status: integration specification; nothing here has been enabled.** Follow the
selected upstream release's [session/integration documentation](https://niri-wm.github.io/niri/Integrating-niri.html),
not an improvised list of background processes. See [packages](packages.md) for
candidate programs and [acceptance](acceptance.md) for real laptop scenarios.

## 1. Supported session modes

| Mode | Allowed behavior | Not equivalent to |
|---|---|---|
| Headless unit/CLI tests | Temp XDG paths, mocks, separate test bus where needed | Real hardware, PAM, or a full graphical session |
| Nested Niri developer window | Plain nested compositor, explicit test config/runtime, isolated notification/control services | A production `niri-session`; upstream notes nested hotkey/accessibility limitations |
| Dedicated test user/full session | Upstream supported Niri session with owned NaliOS services | Authorization to alter the primary user's devices, networks, or vaults |
| Personal daily session | Existing display manager → supported Niri session → NaliOS units | GNOME already running to initialize missing agents |

Initial support policy: **one full graphical session per UID at a time**.
The systemd user manager, user D-Bus, keyring, and some settings are shared per
user. A second same-user desktop is not isolated by a different Wayland socket.
Concurrent full desktops remain unsupported until explicitly designed/tested.
TTY recovery is retained; a recovery desktop can be selected after logout.

Do not run `niri-session` inside the active same-user desktop merely to test a
panel: it can import environment/start session targets globally. A separate
test bus isolates its own user-bus services, but **does not isolate system-bus
Bluetooth, networking, power operations or actual hardware**.

## 2. Namespaces and files

| Kind | Proposed development path | Rules |
|---|---|---|
| User config | `$XDG_CONFIG_HOME/nalios-dev/` | Defaults under normal XDG rules when variable unset; versioned, user-editable |
| Original Niri config | `$XDG_CONFIG_HOME/nalios-dev/niri/config.kdl` | Select through supported process-scoped `NIRI_CONFIG`; do not overwrite `~/.config/niri/config.kdl` |
| Persistent state | `$XDG_STATE_HOME/nalios-dev/` | Recovery journals/theme generations; no passwords |
| Cache | `$XDG_CACHE_HOME/nalios-dev/` | Bounded thumbnails/indexes, safe to regenerate |
| Runtime | `$XDG_RUNTIME_DIR/nalios-dev/<session-id>/` | Private directory 0700, control socket 0600, ownership/session checks |
| Installed defaults | Future `/usr/share/nalios/` and documented `/etc` policy | Package-owned templates are separate from generated user data |

Production eventually uses `nalios`, not `nalios-dev`, after migration. The
existing `~/.local/share/nalios` alias and installed `nalios-*` prototype
commands are **not** this namespace and must remain intact during development.
Session ID must be validated, not an arbitrary path supplied to the filesystem.
A CLI with multiple discoverable sessions requires explicit selection rather
than sending actions to whichever socket was listed first.

## 3. Startup sequence and readiness

1. **Display manager / PAM / logind:** authenticate the user, create the session,
   and use the credential store's supported unlock mechanism. Do not add a
   new password collector. Autologin does not prove a password keyring unlock;
   specify a safe prompt/fallback if autologin is later supported.
2. **Upstream Niri session entry:** start `niri-session`; if scoped NaliOS
   configuration needs wrapper behavior, author that behavior in Rust rather
   than an expanding shell script. Preserve upstream service,
   shutdown target and portal resource semantics. Do not replace the login
   manager. Do not create two competing `niri.service` instances.
3. **Preflight:** identify actual compositor/config, Wayland/Niri sockets,
   session owner and expected environment; check for conflicts. Refuse an
   unsafe takeover; do not kill another desktop or silently reset its settings.
4. **Session environment:** rely on supported Niri environment import. Verify
   `WAYLAND_DISPLAY`, `NIRI_SOCKET`, generated `DISPLAY` where supported,
   desktop/session identity and user-bus availability. Never hard-code display
   numbers or globally export `GDK_BACKEND`, `GTK_THEME` or a Qt override.
5. **Delegated components:** start/detect the selected polkit agent, network
   secret agent, locker/idle coordinator, notification service, and wallpaper
   renderer exactly once. Some come from XDG autostart/D-Bus activation: inspect
   that before adding user units or `spawn-at-startup` entries.
6. **Portals/audio:** allow the installed systemd/D-Bus activation model to
   start them with the correct environment/backend config; do not launch
   duplicate brokers or audio servers by hand.
7. **NaliOS shell:** initial snapshots/capabilities, then bar and launcher.
   Missing optional backends degrade individual controls, not the session.
   Expose actionable health issues without an endless login-blocking spinner.
8. **Ready:** register private control endpoint and report component health.
   A visually shown bar does not mean secrets, locking or portals passed.

A dedicated NaliOS display-manager entry is proposed for the packaged release,
without replacing the upstream Niri or recovery desktop entries. Its desktop
identity must retain compatible portal selection; if `XDG_CURRENT_DESKTOP`
changes, explicitly provide the corresponding `*-portals.conf` and test
resolution. Do not rename the session and assume Niri's portal file is found.

## 4. One deliberate owner for each desktop responsibility

| Responsibility | Initial owner | Eventual NaliOS role / conflict check |
|---|---|---|
| Composition/input/windows | Upstream Niri | Independent IPC/config client; no compositor fork |
| X11 sockets/satellite lifecycle | Supported Niri/satellite integration | Detect supplied path/revision, no duplicate manual `$DISPLAY` setup |
| Shell surfaces and control socket | `nalios-shell-dev` | One instance per selected session; restart does not close apps |
| System Bluetooth state | BlueZ | Client and later Agent1 registration; deliberate default-agent choice |
| Bluetooth authentication UI | Delegated Blueman initially or original Agent1 later | Modes are explicit; never auto-approve unrelated requests or accidentally compete for defaults |
| Networks and stored profiles | NetworkManager | Client, not a second network daemon |
| Wi-Fi/VPN secret prompts | NM applet initially | Keep delegated until an explicit custom secret-agent/security milestone passes |
| Credential storage / Secret service | GNOME Keyring candidate | Verify actual `org.freedesktop.secrets` owner; not the same API as NM secrets or polkit |
| Authorization dialogs | `mate-polkit` candidate | Detect session registration; do not start multiple agents for the same session |
| Audio graph/policy | PipeWire + WirePlumber | Subscribe/control through reviewed API; do not duplicate policy |
| Notifications | mako initially; original C17 later | `org.freedesktop.Notifications` has one owner on the user bus; never replace another desktop behind its back |
| Tray watcher/host | Existing compatible watcher or our C18 | Detect watcher/host protocol role; avoid duplicate watchers and stale items |
| Secure lock / PAM verification | swaylock | NaliOS requests/co-ordinates, never substitutes an ordinary overlay |
| Idle / sleep readiness | swayidle + logind coordination | One policy, documented lock-ready signal and inhibitor deadlines |
| Lid/power-key behavior | logind policy chosen by user | No second daemon independently suspending on the same lid event |
| Portals and permission dialogs | xdg-desktop-portal + compatible backends | Per-interface routing/configuration and diagnosis |
| Wallpaper | swaybg initially | Manage only NaliOS-owned process/config; fallback solid background |

The notification bus name is a singleton; Bluetooth agents/NM agents/polkit
registrations have their own APIs and scoping, not that same singleton model.
“Only one owner” here means an intentional policy for the user interaction,
not a claim that all agents use one well-known D-Bus name.

## 5. Portal routing that must be tested

Read upstream [important software](https://niri-wm.github.io/niri/Important-Software.html)
and [screencasting](https://niri-wm.github.io/niri/Screencasting.html).

| Interface/workflow | Candidate route | Required evidence |
|---|---|---|
| ScreenCast (screen/window) | GNOME backend with Niri's supported integration + PipeWire | Actual Firefox ESR share chooser/stream/stop after cold login; another relevant browser/Electron app if claimed |
| FileChooser | Explicit GTK backend route | Open/save/cancel in real portal-using app; Nautilus remains a dependency of the observed GNOME backend package |
| Settings / preferred dark style | Supported GNOME/settings provider route | Native/sandboxed apps see expected mode; shared per-user settings side effects disclosed |
| Secret | Installed supported keyring portal path | Backend availability and supported credential unlock; separate from Wi-Fi secret-agent test |
| OpenURI and remaining basics | Compatible upstream fallback selection | Real handlers/permission dialogs; no indiscriminate “all portal interfaces supported” claim |
| Screenshot | Niri built-in initially; portal route if explicitly supported | Correct area/output/window/scale and cancellation behavior |

Keep routing scoped to the selected desktop identity, using backend names from
the installed `.portal` metadata. Do not assume `xdg-desktop-portal-wlr` is the
right choice from Niri's compositor ancestry. Starting a second broker or
restarting another session's broker is not a fix for a bad configuration.

Do not claim RemoteDesktop/remote input just because ScreenCast works. Likewise,
working extended displays do not prove native mirrored-output support.

## 6. Lock, idle, lid and suspend state machine

Proposed states: `Unlocked → LockRequested → LockReady → SleepRequested →
Sleeping → ResumedLocked → UnlockedByLocker`. A request failure is visible and
does not transition to an invented locked/sleeping success.

- Use the selected locker's documented ready/daemonization behavior with the
  compositor's real session-lock protocol. Process existence is not readiness.
- Explicit NaliOS suspend first requests lock and waits for confirmed ready;
  if locking fails, **do not request suspend** and explain the failure.
- For lid/external sleep events, use a supported logind delay-inhibitor /
  swayidle before-sleep path. Document the delay deadline and test what happens
  if the locker fails or the deadline expires. Do not promise an unlimited
  inhibitor or that our UI can cancel every system-initiated sleep request.
- Resume leaves the compositor locked; only the proven locker/PAM flow unlocks.
  Theme/shell crashes/restarts must not reveal the desktop.
- Idle display power-off is not a lock. Lock-before-display-off timing and
  wake behavior are tested separately.
- Presentation/media inhibitors do not disable an explicit user lock. Define
  whether they inhibit idle dim, idle sleep, or a specific logind action.
- Lid while docked, AC versus battery, low-battery policy and power-key action
  are user decisions. Hibernation is excluded until host/storage support and
  resume security are checked. Do not enable it from a theme/settings preset.

Swaylock's theme support is intentionally narrower than the shell's. A safe,
readable lock screen beats exact visual parity. Native locker fault tests
belong in a disposable/test session with a TTY/recovery path available.

## 7. Unit lifecycle and restart boundaries

Proposed user artifacts: NaliOS shell, wallpaper and idle units plus an owned
session target/wrapper only if the upstream lifecycle requires it. Delegated
agents with existing units/autostart should be integrated rather than copied.
Unit names/edges are fixed after the startup spike, not generated blindly now.

Require explicit session conditions and stop binding (`PartOf`/`Requisite` or
other tested upstream patterns). An `After=` ordering alone does not arrange
startup or shutdown. Avoid circular dependencies, `default.target` startup for
a graphical-only program, and unbounded restart loops. Capture the correct
session environment without assuming systemd's shared user environment is safe
for simultaneous full desktops.

Shell recovery can restart **only the shell**, never Niri/applications, the
locker, NetworkManager, or the user bus. Backend restarts invalidate adapter
snapshots and trigger bounded reconnect. A stale control socket is removed only
when ownership, session identity and dead instance are verified.

On logout, stop NaliOS-owned session components and release bus/socket names.
Do not use `pkill`, stop every user's compositor, or delete unrelated runtime
directories. Full session logout may end its applications under upstream Niri
semantics; this is different from the shell-only restart guarantee.

## 8. Recovery path and exit gate

Document the actual TTY/recovery login before testing installation or sleep.
From a headless path the future CLI should validate config, diagnose missing
services, restore a known-good managed generation, and restart only the owned
shell. Keep an independent terminal/fallback-launch shortcut and an upstream
Niri/recovery desktop entry available. These are future commands, not installed
by this documentation.

Session acceptance requires A01–A09 plus A12 where installation applies:
cold boot, logout/relogin, real keyring/secret prompts, intended mouse after
resume, correct portals, no duplicate owners, lock-ready failure tests,
external output removal, and TTY recovery without the dashboard. Record exact
packages/Niri revision and sanitized results, not “works after opening GNOME.”
