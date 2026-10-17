# First implementation milestone: a trustworthy Rust foundation

**Status: structural setup only; no commands or tests implemented.** The
[workspace scaffold](../../crates/README.md) supplies five manifests, local
wiring and documentation-only library roots. F01/T002 are not complete.
Scope: T002–T005 and the minimum T001/T011 decisions to do them safely. No
installer, package changes, active-session replacement or UI is included.

## 1. Outcome and boundary

A clean checkout builds a headless **`nalios-dev`** that can:

1. print help/version without connecting to services;
2. validate a supplied config without creating/changing files;
3. report passive host/session readiness in text and versioned JSON;
4. inspect supported Niri state on an explicitly selected available socket,
   including a bounded testable event subscription/reconnect path.

It must behave sensibly from a TTY/no-Niri environment. Tests cannot touch the
real home directory or perform hardware/power/authentication operations. The
CLI is not installed over the prototype's `nalios` binary. All code/tests are
Rust; follow the five-crate direction in [architecture](architecture.md).

Not included: bar, launcher UI, dashboard, Bluetooth mutations, theme file
application, installer, VM helper, custom locker, networking secret agent or a
permanent NaliOS backend daemon. Mock-only success is not a laptop release.

## 2. Decisions that block work versus decisions that can wait

| Decision | Needed now? | Working recommendation |
|---|---|---|
| Language/ownership | Yes | First-party executable behavior/tests in Rust; no old source import |
| Minimal dependency review | At each addition | Review exact serde/TOML/error/CLI graph; add Tokio/zbus only with probes; core has no runtime |
| Local package publishing | Yes | `publish = false` until deliberate release policy; no automatic public push/install |
| First-party license | Before sharing/granting permissions | Keep D01 open if necessary during private local work; do not invent a license or `license` field on the author's behalf |
| Crate boundary | Scaffolded, behavior still pending | Five manifests exist; implement core + CLI first, app/platform probes next; no workflow duplication |
| GUI toolkit | No, until T007 | Rust-native Iced integration spike later; no GTK headers/dependencies for this milestone |
| Palette/bar location | No | Existing Night/Day/top-bar drafts remain proposals |
| Niri/satellite installation route | Before clean session installation, not pure tests | T011 inventory can proceed read-only; T012 remains a release/integration gate |

Private implementation can progress without resolving every design and release
choice. Lack of a license is not permission to redistribute the project or its
dependencies; comply with dependencies from the moment they are used.

## 3. Small, reviewable change sequence

Each row is one focused change/review, not necessarily one day. No honest
calendar estimate is possible until we measure build and protocol work.

| Change | Files introduced/changed | Demonstrable exit |
|---|---|---|
| F01 / T002 | Manifests/local-only lock/comment roots already scaffolded. When authorized, add a real explicit `nalios-dev` binary target/entry, core behavior and meaningful tests. | `--help` / `--version`, clean build and real unit test; current empty-library check does not complete this gate |
| F02 / T003 | Core `paths.rs`, `config/{mod,schema,validate}.rs`, `error.rs`; Rust test fixtures | Pure validation from supplied strings/bytes and XDG inputs; no CLI file reading until the platform boundary exists |
| F03 / T003–T004 | Implement the existing empty app/platform boundaries: config/diagnostic ports/workflows, file/environment probes, CLI `config check` and reports. | Bounded file read uses platform → pure validation; fake-host report deterministic; real passive probes work with no writes/activation |
| F04 / T004 | App health policy, platform read-only D-Bus/Niri endpoint probes; bounded async and tests | Absent bus/service/socket differs from denied/unresponsive; doctor never starts a scan, agent, service or auth prompt |
| F05 / T005 | Core window/workspace IDs/models; app Niri observation reducer/port; platform Niri framing/wire/client; authored fixtures | Bounded JSON/unknown-field/error tests; partial-frame/EOF/epoch/out-of-order cases covered |
| F06 / T005 | Read-only Niri CLI inspection + event-stream task lifetime/reconnect | Changes in disposable test windows update observed state; disconnect marks stale, reconnect rebuilds; cancel exits cleanly |
| F07 / T002–T005 | Rust CLI integration tests; dependency boundary check; documented invocations/limitations | Reproducible locked headless build, fmt/clippy/tests, no renderer/native GUI graph in core/app/CLI; evidence reviewed |

**Status (2026-10-16):** F01–F06 are implemented or partially implemented.
`nalios-dev` has an explicit binary target, `--help`/`--version`,
`config check --file <path>`, passive `doctor`, read-only `niri inspect` and
streaming `niri watch`; core provides tested pure path/config validation and
Niri domain values; app owns config, diagnostic and Niri reducer workflows;
platform implements bounded file reads, passive host probes, an independent
Niri IPC client and a bounded event stream with stale-state marking and
reconnect backoff, verified against the live Niri 26.04 session. Doctor also
emits its versioned JSON envelope. 104 tests pass offline.

Remaining in this scope: F04's D-Bus/service-ownership probes (needs a
reviewed D-Bus client) and explicit task-lifetime supervision for the shell's
Niri worker. `cargo fmt`/`cargo clippy` are not yet part of the evidence
because this machine lacks `rustfmt`/`rust-clippy`.

T007 status: GPUI was approved after a live spike; the development shell
(`nalios-shell-dev`) now provides the layer-shell bar and launcher on live Niri
state ([decision 0006](../decisions/0006-ui-toolkit-spike.md)).

T008 status: the private control channel is implemented — `nalios-dev shell
ping|open-launcher|toggle-launcher` talks to the running shell over a
peer-checked, bounded socket
([decision 0007](../decisions/0007-shell-control-channel.md)). The launcher now
searches the discovered catalog (`apps list`/`apps search`) and launches with
argv, with `xdg-terminal-exec` for terminal apps
([decision 0008](../decisions/0008-desktop-entry-dependency.md)). Results now
show theme icons ([decision 0009](../decisions/0009-icon-lookup-dependency.md))
and config favorites rank first. A read-only full dashboard
(`Super+Shift+Space`) shows workspaces with their windows, focus, outputs and
session health. IME text input, a favorites UI toggle and
`StartupNotify`/D-Bus activation remain.

Do not add all planned feature directories as empty scaffolding. Make the first
fake/real diagnostic port small; extract a common primitive only after it has
clear consumers. `xtask` is introduced only when recurring checks justify it.

Dependency staging: F01 uses std plus reviewed CLI parsing only if needed;
F02 adds reviewed serialization/TOML/error crates for pure models; F03 adds no
GUI dependencies; F04 introduces the selected Tokio/zbus feature combination;
F05 adds JSON protocol parsing. Test-only tempfile/property-test crates stay
in dev dependencies. Do not add the whole candidate register at workspace setup.

## 4. Proposed CLI contract

Future development invocations (not existing programs):

```text
nalios-dev --help
nalios-dev --version
nalios-dev config check --file <path>
nalios-dev doctor
nalios-dev doctor --json
nalios-dev niri inspect --socket <path>
nalios-dev niri watch --socket <path>
```

Within a real session a Niri subcommand may use its validated `$NIRI_SOCKET`
when `--socket` is absent. Never choose a random other user's/session's socket.
Doctor remains passive and does not dump raw window titles, SSIDs or credentials.
A detailed Niri inspection is a separate explicit command; identifiers/title
redaction defaults must be documented before adding its output to reports.

Proposed exit codes:

- `0`: operation completed, or doctor has no blocking failure for its scope;
- `1`: invalid config / blocking readiness check failed;
- `2`: invalid CLI invocation;
- `3`: internal execution failure that prevented a trustworthy result;
- `130`: user interrupt during a long observation, if following Unix convention.

Doctor marks each check `pass`, `warn`, `fail`, `unavailable` or `skipped` with
an explanation, severity/requiredness and a safe remediation hint. Normal TTY
absence of Niri is reported, not treated as a panic or necessarily a broken
host. Report scope distinguishes baseline host readiness from active-session
readiness. Fix hints are text, not executed commands.

JSON envelope has its own schema version, report scope, safe build identity,
checks with stable codes and a summary. Human output may improve without
breaking JSON. Data goes to stdout; diagnostics to stderr. Stable check codes,
not translated prose, drive exit policy. Bound inputs and serialized output.
Do not include a whole process environment, raw D-Bus property dump or logs.

## 5. Diagnostic checks to implement first

| Check | Input/effect boundary | Safe behavior |
|---|---|---|
| OS/architecture | Platform reads bounded OS metadata | Unknown/unsupported is a report, not an installer |
| XDG paths/runtime directory | Explicit environment snapshot + metadata | Validate absolute paths/ownership/modes; no automatic directory creation or `/tmp` socket fallback |
| Program availability/provenance | PATH/path metadata and optional known package metadata | Explain local versus packaged paths; do not execute arbitrary PATH matches to obtain a version |
| Wayland/Niri endpoint readiness | Session environment and bounded socket checks | Report missing/wrong-type/denied/unresponsive distinctly; no session switch |
| D-Bus availability/known owners | Existing session/system bus, no-auto-start queries | Do not activate optional services or acquire names/agents |
| BlueZ status | Read only from an already running service if allowed | Separate service absent, no adapter, powered off, blocked/unknown; no scan or power setter |
| Dependencies of later features | Package/program availability where safely observable | Missing optional tools are not auto-installed or sufficient reason to fail all host checks |

A metadata permission check does not prove future connection permission.
A socket path existing does not prove a live compositor. Unavailable protocol
facts must remain unknown instead of guessed. Version probes for trusted
executables, if added, have explicit opt-in/policy, deadlines and capped output.
T011's manually observed version is evidence, not a hard-coded doctor result.

## 6. Tests required before UI work

- Config: empty/missing/malformed values, unknown version, invalid ranges,
  unknown critical keys, oversized input and valid round trip.
- Paths: unset/empty/relative XDG values, malicious runtime paths, missing home,
  owned versus unowned directory, long/non-UTF-8 paths where supported; no global
  environment mutation in parallel Rust tests.
- Doctor: fake complete host, normal TTY, missing bus, service stopped, adapter
  missing/powered off/blocked/unknown, denied call and timeout. Tests assert
  that mutating ports were never constructed or called.
- Reports: schema version, stable codes/exit policy, deterministic fake output,
  bounded strings and redaction of planted fake secrets/identifying values.
- Niri framing: split/coalesced reads, malformed/over-limit frame, unknown field/
  event, EOF, backend epoch reset, cancellation and stale old-epoch update.
- Niri state: initial stream state plus updates, transient cross-object ordering,
  reconnect state replacement and no replay of actions. Snapshot command success
  alone does not prove event synchronization.
- Isolation: temp filesystem, disposable Unix listener, separate mocked bus if
  needed; default tests do not inspect or mutate the author's active session.
- Architecture: core/app compile without UI/Linux transport dependencies;
  CLI→shell dependency forbidden; no renderer/native GUI transitive graph in
  headless builds; no first-party scripting-language test harness.

Real read-only Niri verification is explicit and recorded separately from the
unit suite. Native hardware pairing/lock/suspend tests are not part of this
milestone and cannot be implied by a passing doctor.

## 7. Finish gate and next milestone

All F01–F07 exits pass with exact revision/toolchain/dependency evidence; no
system settings changed; no root request; invalid input fails safely; doctor
works without a shell; independent Niri client recovers from a test disconnect;
README accurately labels implemented versus missing features.

Then T007 tests the Rust-native UI stack in a small disposable example; T006
settles standards-aware app discovery/launch. T008/T009 integrate only a
minimal original themed launcher/bar with the existing app contracts. The
[larger roadmap](roadmap.md) remains the delivery order, not permission to
start notifications, VM code, a custom locker and a theme gallery at once.
