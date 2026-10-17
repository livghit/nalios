# Planning research snapshots

These are **read-only observations**, not package locks, implementation,
installation approval, or proof of working integrations.

## 2026-10-16 package metadata

[packages-2026-10-16.tsv](packages-2026-10-16.tsv) records the selected direct
package candidates/alternatives discussed in the desktop plan. Environment:
Kali Rolling 2026.3, amd64; existing apt package lists dated 2026-10-14 and
2026-10-15.
No `apt update`, install, removal, repository change or service enable occurred.

Method: `apt-cache policy PACKAGE`, `apt-cache show PACKAGE=VERSION` and its
`Source` field; where that field is absent, source name equals package name.
Columns are tab-separated:

- `package`: binary package name queried;
- `candidate`: cached apt candidate, not a version pin or supported minimum;
- `installed`: apt/dpkg package version, **not** presence of an unowned binary;
- `source`: source-package field, including source version where metadata gives
  one; name equals binary package if no separate field was present.

`NO_CANDIDATE` means no cached candidate; `NO_METADATA` means no apt policy
record was returned; `(none)` is apt's explicit not-installed value. These do
not imply universal repository absence. Niri/satellite are available as local
executables despite the lack of package metadata in this snapshot.

Local executable-path checks found `niri`, `niri-session` and
`xwayland-satellite` under `/usr/local/bin`; `dpkg-query -S` did not identify
owners for those paths. `niri --version` reported `26.04 (v26.04)`. Satellite's
`--version` invocation rejected the argument; its exact build/version remains
unverified. No device-operation tests were performed.

The table covers explicit planning candidates, not the complete host package
set, every optional XCB builder dependency, or a release's transitive graph.
Generate the latter from the eventual build/solver output and source inventory.
Do not install every row: many rows are alternatives, diagnostics, optional
profiles or unavailable old names retained to explain a finding.

## Upstream and Cargo research

The [source register](../blueprint/sources.md) records registry versions,
published licenses, repository references and compatibility caveats. Metadata
was read from crates.io's public crate/dependency endpoints on the same date.
At research time no workspace/lockfile existed. The later
[scaffold](../../crates/README.md) has a local-only lockfile; it contains none of
these candidate external crates and is not an audited application graph.

Niri's getting-started, important-software, IPC, screencasting, packaging,
integration, example-systemd, accessibility and include documentation were read
at <https://niri-wm.github.io/niri/>. GTK4 layer-shell's upstream README/license
and binding metadata were also inspected. Online docs are moving references;
release-marked and “next release” options must be checked against the actual
selected compositor release before use. This planning pass has not archived
or redistributed upstream application source.

## Rust-native architecture follow-up (2026-10-16)

After the Rust-only clarification, crates.io metadata was also read for Iced,
`iced_layershell`, its successor `iced_exwlshell`, `exwlshellev`, Smithay client
toolkit, wayland-client and desktop-entry/icon candidates. The
[source register](../blueprint/sources.md) records published versions/licenses,
feature risks and the MPL-2.0 parser gate. No dependency was installed or built.

Read upstream Iced's 0.14.0 README and the Waycrate event-loop project's README/
0.19-to-0.20 migration notes. The former calls Iced experimental; the latter
announces the migration to `iced_exwlshell`. These justify a versioned technical
spike, not production suitability, full accessibility or zero native libraries.

The earlier package snapshot remains historical evidence. It still includes
GTK headers and Python/shell tooling queried before this refinement; their
presence in the TSV does not select them for the Rust-only codebase.

## Rust-native UI toolkit survey (2026-10-16)

Throwaway resolutions in temporary projects (all-platform lockfile counts; the
Linux tree line counts are from `cargo tree --target x86_64-unknown-linux-gnu`):

| Stack | Locked packages | Linux tree lines |
|---|---|---|
| `iced` 0.14 + `iced_exwlshell` 0.20.1 (`wgpu`, `tiny-skia`, `crisp`, `wayland`, `tokio`) | 428 | 861 |
| `gpui` 0.2.2, default features | 709 | 1391 |
| `gpui` 0.2.2, wayland only | 678 | 1359 |

Layer-shell and accessibility evidence:

| Option | License | Layer-shell | Accessibility | Notes |
|---|---|---|---|---|
| Iced + `iced_exwlshell` | MIT | Third-party, published (`0.21.0-rc1` active) | no `accesskit` in `iced_winit`/`iced_wgpu` | `lala-bar` (17.6k downloads) is an Iced layer-shell bar |
| GPUI | Apache-2.0 | Upstream `LayerShellOptions`, unpublished | `accesskit_unix` on Wayland/X11 | `gpui-shell` (152★), `Capsule` (48★) and `kumaUI` (Niri/Sway) build shells with it |
| egui + third-party layer-shell | MIT OR Apache-2.0 | Third-party, immature (`layer-shell-wgpu-egui`, 8★) | `accesskit_winit` | large ecosystem, no first-party shell path |
| libcosmic | MPL-2.0 | Yes (COSMIC panels/applets) | partial | full DE framework, git-only; would make NaliOS a COSMIC derivative |
| Xilem/Masonry + Vello | Apache-2.0 / MIT | no | `accesskit` in Masonry | very early (0.4.0, ~22k downloads) |
| Slint | GPL-3.0-only OR royalty-free/commercial | unclear | none found | license conflicts with the permissive goal |
| Raw `smithay-client-toolkit` + renderer | ours | full control | we would build it | smallest deps, most work; the plan discourages a custom engine |

GPUI findings:

- The published 0.2.2 API does **not** expose `LayerShellOptions`; the
  layer-shell module and `gpui_platform`/`gpui_linux`/`gpui_wgpu` exist on
  `main` and are unpublished. A GPUI shell needs a git-pinned Zed revision;
  `gpui_wgpu` also pins `naga` from a wgpu git revision.
- The existing shell projects track unpinned branches or forks; NaliOS would
  pin an exact revision and vendor it for release. They also use the GPL
  `niri-ipc` crate, which NaliOS deliberately avoids.

This is reconnaissance, not an approved dependency set. It informs the T007
toolkit decision: the spike must justify the surface with real focus, input,
IME, accessibility, scaling and idle-cost results. See
[decision 0006](../decisions/0006-ui-toolkit-spike.md) (proposed, awaiting the
author). No UI dependency has been added to this repository.

Color-pair numbers in [themes](../blueprint/themes.md) are calculations from
the proposed opaque RGB tokens using relative-sRGB luminance, not measured
screenshots or a full accessibility audit.

No MAC addresses, SSIDs, serials, credentials, personal clipboard data or raw
system journals belong in these public snapshots. Keep private test evidence
out of version control.
