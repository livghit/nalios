# Architecture and contracts

All names below are proposed interfaces/work packages, not implemented APIs.
The user writes the implementation. Keep decisions small enough to test and
replace without rewriting the entire desktop.

## System boundary

```text
Kali packages: kernel, drivers, firmware, logind, systemd, D-Bus
  |
  +-- upstream Niri process ---- documented IPC ----+
  +-- BlueZ / NetworkManager --- system D-Bus ------+
  +-- audio services / MPRIS --- supported APIs ----+-- Rust platform adapters
  +-- UPower / portals / agents -------------------+          |
                                                            v
                                               state / commands / errors
                                                            |
                                     +----------------------+-----------+
                                     |                                  |
                              nalios-shell                         nalios CLI
                         original panels/windows              diagnostics/actions
```

Libvirt/cryptsetup and restricted privileged VM operations are a later boundary,
not code embedded in the desktop renderer. Applications such as terminal,
browser, editor, file manager, and Kali tools remain separate programs.

## Initial workspace responsibilities

| Crate | Owns | Must not own |
|---|---|---|
| `nalios-core` | Versioned config/theme models, validation, common errors, operation IDs, pure state transitions. | GTK widgets, system bus calls, root operations, hardware-specific paths. |
| `nalios-platform` | Independently written service adapters, Niri protocol client, app discovery, platform capability checks, event reconnection. | Visual layout, copied GPL implementation/type crates, assumed success states. |
| `nalios-cli` | Headless commands, diagnostics, machine-readable output, explicit setup/upgrade plans, shell-control requests. | Hidden automatic system mutation, requiring a running GUI for basic diagnostics. |
| `nalios-shell` | UI components, bar/overlays/dashboard, view models, user-visible prompts, session-specific service ownership. | A root renderer, blocking device/service operations on the UI thread. |

Modules in the [catalog](components.md) initially live inside these crates;
they are not separate services by default. Split out a settings application,
VM application, or backend only when needed for independent lifecycle/security.

## Runtime and UI choice

Prototype GTK4 through Rust bindings plus gtk4-layer-shell for Niri surfaces.
Review licenses before dependency selection. Test ordinary-window rendering
separately from layer-shell rendering. Verify animation, focus, transparency,
fractional scale, accessibility, and multi-output behavior before committing.

Use one documented asynchronous integration strategy. GTK widgets live on the
GTK main thread. Background adapters send bounded state/events to it; they do
not manipulate widgets from a worker thread. Cancel tasks when their owning
session/view ends and coalesce high-frequency updates. Rendering and hardware
polling are not permanent 60-Hz loops.

GTK is an upstream native toolkit, not all-Rust infrastructure. A stricter
all-permissive or all-Rust dependency goal requires another decision, not a
silent change to this architecture.

## Platform adapter contract

For each subsystem implement four concepts, using NaliOS-authored types:

1. **Capabilities:** what this host/session/protocol version can do.
2. **Snapshot:** current state plus whether it is live, stale, or incomplete.
3. **Commands:** explicit requests with target identity and operation ID.
4. **Events/results:** authoritative state updates and actionable failures.

Distinguish at least unavailable service, absent hardware, disabled/blocked
hardware, unsupported operation, unauthorized, busy, canceled, timed out,
disconnected/stale, and confirmed success. Keep user-facing explanations
separate from redacted technical details.

A setter request is not proof of success. Disable duplicate conflicting
operations while pending; reconcile after reconnect. Use bounded timeouts and
backoff. Do not automatically retry pairing, authentication, deleting data,
formatting, or power operations just because a response was lost.

## Niri adapter: independent protocol client

Do not import the GPL `niri-ipc`/`niri-config` crates into an application with a
different licensing policy. Implement the needed interoperability from the
public protocol documentation and recorded supported behavior.

Initial scope:

- Discover the socket from the current session, not a hard-coded UID/path.
- Read output, workspace, window, and focused-window snapshots.
- Subscribe to the event stream; combine initial snapshot/events without
  silently losing changes. Reconnect/resnapshot after loss.
- Request only supported actions for focus/workspace/window behavior.
- Tolerate new fields/events, reject malformed/oversized messages, and report
  incompatible versions. Record the tested Niri version range.

Fixtures should represent protocol exchanges from test sessions or deliberately
written test cases; do not paste upstream Rust implementations/tests.

## CLI and shell-control contract

Diagnostics and configuration validation work without a shell. UI actions such
as `open launcher`, `toggle dashboard`, and `open settings` can use a private,
versioned local control channel to the shell.

Candidate transport: a Unix socket below `$XDG_RUNTIME_DIR/nalios-dev/`, with a
private directory, socket mode 0600, peer-UID checks, message-size limits,
request IDs, bounded timeouts, and a protocol-version handshake. Never expose
an arbitrary command execution method. Check stale socket ownership before
cleanup. Do not use a predictable world-writable `/tmp` endpoint.

Agent authentication exchanges use their actual service APIs. Do not send VM
passphrases through command-line arguments or a general-purpose CLI socket.
Choose a dedicated prompt/secret-transfer boundary when designing the VM tool.

## Configuration and state

Use XDG paths and distinct development/production namespaces:

| Kind | Development purpose |
|---|---|
| Config | User choices, validated theme/layout/settings documents. |
| State | Selected profile, recovery journal, non-secret persistent operation state. |
| Cache | Regenerable thumbnails/indexes; bounded and safe to delete. |
| Runtime | Private sockets, process/session identity, ephemeral locks. |
| Credentials | Existing supported secret store/agents, not config/state/cache. |

Give every persistent schema a version. Provide parse/validate/default/migrate
operations and preserve unknown compatible fields where appropriate. Display
invalid config errors without destroying the last good configuration. Do not
execute configuration as shell, Lua, JavaScript, or arbitrary Rust plugins in
the first release.

Niri behavior is an independently authored config referencing explicit shell
commands. Validate it with Niri before activation. A missing external tool
produces a health error rather than silently breaking a shortcut.

## Theme application contract

A theme contains semantic roles and metadata, not widget implementation code.
The pipeline validates all required roles, stages generated outputs, asks
native programs to validate where supported, records backups/recovery state,
and applies the change before reporting completion.

Per-file atomic replacement is not a fully atomic multi-file transaction.
Design a journal/versioned generation plus rollback/recovery for interruption
between files, including process crash and disk-full cases. Keep the current
marker consistent with the generation actually installed. Preserve user files
and distinguish managed outputs from source config.

Preview must not persist changes. Revert must be reachable by keyboard/CLI.
External application theming is best-effort with explicit support lists; GTK,
Qt, browsers, and custom-drawn apps do not all obey the same styling rules.

## Session, lifecycle, and service ownership

Use the supported Niri session model with small NaliOS user units/processes.
Session startup must set/check the correct Wayland/display/session environment
and portal configuration without blindly overwriting another active session's
environment. systemd's user manager and the user D-Bus may be shared by multiple
sessions; plan for that explicitly.

- Start only owned components and discover delegated agents.
- Do not use blanket `pkill quickshell`, `pkill dunst`, or process-name cleanup
  against unrelated sessions.
- A shell crash must not terminate applications or unlock the desktop.
- Only one real owner for `org.freedesktop.Notifications` and other singleton
  bus services. During development use an isolated bus/mocked service when
  necessary; never replace GNOME's active services behind the user's back.
- Bluetooth pairing/NetworkManager secret/polkit prompts have different APIs
  and lifecycles. Do not assume a settings widget substitutes for an agent.
- A delegated polkit agent, secret store, or portal backend may come from a
  different desktop project without requiring a GNOME session first.
- Provide recovery commands that do not depend on functioning overlays.

The installed desktop must cold-boot into working Niri independently of GNOME.
Keeping another login session available is a recovery strategy, not a runtime
dependency.

## Security-sensitive boundaries

- Use a proven compositor-backed session locker first; a fullscreen overlay
  that captures keys is not a secure lock.
- Delegate authentication to the proper PAM/polkit/secret APIs and reviewed
  existing agents. Never collect passwords into general application logging.
- Any privileged helper exposes narrow validated actions, binds authorization
  to the caller/session and target, and accepts no arbitrary paths/commands.
- Denied/canceled authorization is a normal outcome, not a reason to run the
  whole application as root or install broad passwordless rules.
- App-launching and Kali catalogs are user actions, not privileged helpers.
- Do not parse legacy VM metadata by sourcing it as shell code. Import it as
  untrusted data with a defined schema and strict validation.

## Dependency decisions to record

Before each new backend, document the public interface, minimum tested version,
license/linkage implications, credential/authorization behavior, disconnect
handling, tests, and fallback. Audio, screen sharing, tray menus, credential
agents, and encrypted VMs need explicit spikes; they are not solved by merely
listing a crate name.
