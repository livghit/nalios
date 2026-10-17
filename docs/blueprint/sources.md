# Source, API, and dependency register

**Status: candidate register plus selected dependencies.** Permissive crates are
locked for the configuration engine ([decision 0004](../decisions/0004-core-config-dependencies.md))
and Niri JSON ([decision 0005](../decisions/0005-niri-json-dependency.md)); the
rest remain candidates. Published crate metadata and upstream documentation were inspected on
**2026-10-16**. Version/license metadata is evidence for review, not legal advice,
a security audit, or proof that a dependency graph builds.

After the user's Rust-only clarification, [decision 0002](../decisions/0002-rust-codebase-boundaries.md)
supersedes the GTK-first assumption. Rust-native UI candidates and their risks
are below; native service/graphics bindings require explicit review. No code,
UI build or full transitive graph was tested during this research.

## 1. Four distinct kinds of source

1. **Our source:** independently authored Rust, configuration, UI definitions,
   documentation, tests, theme data, and original editable artwork in this
   repository. The author chooses licenses; none is selected here.
2. **Linked dependencies:** Cargo crates and native libraries used by NaliOS
   binaries. Review the complete resolved graph and native licenses.
3. **External programs/services:** Niri, BlueZ, NetworkManager, portals, etc.
   Communicate through documented interfaces; retain their separate identity,
   licensing, package provenance, and redistribution obligations.
4. **Interoperability references:** public protocols/documentation and test
   observations. These inform independent clients, not permission to paste
   implementation code or remove notices.

The deprecated prototype is a behavior/data inventory only, not a source
provider. No Caelestia/Quickshell tree, old application source, theme templates,
fonts, or artwork is to be imported automatically.

## 2. Candidate Rust dependencies

Versions below are **observed stable registry versions**, not a Cargo.lock or
an approved install set. Add a dependency only with the ticket that needs it.
Licenses are the published direct-crate SPDX expressions; transitive/native
review remains open. Repository links identify source, not a vendoring plan.

| Crate | Observed version | Published license | Purpose / source |
|---|---|---|---|
| `serde` | 1.0.229 | MIT OR Apache-2.0 | Data models; [source](https://github.com/serde-rs/serde) |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | Niri/control JSON; [source](https://github.com/serde-rs/json) |
| `toml` | 1.1.8+spec-1.1.0 | MIT OR Apache-2.0 | Candidate config/theme format; [source](https://github.com/toml-rs/toml) |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | Typed errors; [source](https://github.com/dtolnay/thiserror) |
| `clap` | 4.6.7 | MIT OR Apache-2.0 | CLI arguments/help; [source](https://github.com/clap-rs/clap) |
| `tracing` | 0.1.44 | MIT | Structured redacted events; [source](https://github.com/tokio-rs/tracing) |
| `tracing-subscriber` | 0.3.23 | MIT | Logging setup/filtering; [source](https://github.com/tokio-rs/tracing) |
| `tokio` | 1.53.3 | MIT | Candidate worker runtime, bounded channels, socket I/O; [source](https://github.com/tokio-rs/tokio) |
| `zbus` | 5.19.0 | MIT | D-Bus clients/selected services; [source](https://github.com/z-galaxy/zbus/) |
| `iced` | 0.14.0 | MIT | Rust-native widgets/layout/style candidate; [source](https://github.com/iced-rs/iced) |
| `iced_exwlshell` | 0.20.1 | MIT | Leading Wayland shell integration spike, not proven production support; [source](https://github.com/waycrate/exwlshelleventloop) |
| `exwlshellev` | 0.20.1 | MIT | Integration's event-loop dependency, not a reason for an extra direct dependency; [source](https://github.com/waycrate/exwlshelleventloop) |
| `smithay-client-toolkit` | 0.21.1 | MIT | Wayland client foundation/transitive integration dependency; not a complete widget toolkit; [source](https://github.com/smithay/client-toolkit) |
| `wayland-client` | 0.31.15 | MIT | Rust Wayland protocol client when needed; [source](https://github.com/smithay/wayland-rs) |
| `freedesktop-desktop-entry` | 0.8.3 | MPL-2.0 | Rust parser candidate, **license gate and not a full launcher**; [source](https://github.com/pop-os/freedesktop-desktop-entry) |
| `freedesktop-icons` | 0.4.0 | MIT | Optional Rust icon lookup candidate, no decoding/asset-safety guarantee; [release docs](https://docs.rs/freedesktop-icons/0.4.0/freedesktop_icons/) |
| `libpulse-binding` | 2.30.1 | MIT OR Apache-2.0 | Candidate event-driven audio client for `pipewire-pulse`; [source](https://github.com/jnqnfe/pulse-binding-rust) |
| `rustix` | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Candidate Unix/ownership/peer checks if std APIs are insufficient; [source](https://github.com/bytecodealliance/rustix) |
| `image` | 0.25.10 | MIT OR Apache-2.0 | Only if needed for bounded thumbnails/asset validation; [source](https://github.com/image-rs/image) |
| `notify` | 8.2.0 | CC0-1.0 | Optional filesystem-change watcher; [source](https://github.com/notify-rs/notify) |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | Isolated filesystem tests/staging; [source](https://github.com/Stebalien/tempfile) |
| `proptest` | 1.12.0 | MIT OR Apache-2.0 | Parser/state/rollback property tests; [source](https://github.com/proptest-rs/proptest) |
| `cargo-deny` | 0.20.2 | MIT OR Apache-2.0 | Dependency policy tooling; [source](https://github.com/EmbarkStudios/cargo-deny) |
| `cargo-audit` | 0.22.2 | Apache-2.0 OR MIT | Advisory checks; [source](https://github.com/rustsec/rustsec) |
| `cargo-cyclonedx` | 0.5.9 | Apache-2.0 | SBOM tooling; [source](https://github.com/CycloneDX/cyclonedx-rust-cargo) |

Registry metadata can be rechecked at `https://crates.io/api/v1/crates/NAME` and
`https://crates.io/api/v1/crates/NAME/VERSION/dependencies`. Review the actual
release source/license files and Cargo feature graph before acceptance.

### Selected for the core configuration engine

[Decision 0004](../decisions/0004-core-config-dependencies.md) records the
selection, evidence and remaining release review.

| Crate | Requirement | Locked | License | Use |
|---|---|---|---|---|
| `serde` (`derive`) | 1.0.228 | 1.0.229 | MIT OR Apache-2.0 | Config/theme/Niri model derives |
| `toml` | 1.1.2 | 1.1.8+spec-1.1.0 | MIT OR Apache-2.0 | Versioned TOML parsing |
| `thiserror` | 2.0.18 | 2.0.21 | MIT OR Apache-2.0 | Typed error display/source |
| `serde_json` | 1.0.149 | 1.0.154 | MIT OR Apache-2.0 | Niri IPC and control JSON |
| `rustix` | 1.1.5 | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Linux peer credentials for the control socket |
| `freedesktop-desktop-entry` (no default features) | 0.8.3 | 0.8.3 | MPL-2.0 | Desktop-entry/Exec parsing for the app catalog |
| `freedesktop-icons` | 0.4.0 | 0.4.0 | MIT | Icon-theme lookup for launcher results |
| `time` (`local-offset`) | 0.3.55 | 0.3.55 | MIT OR Apache-2.0 | Local time for the clock and dashboard |
| `tempfile` (dev-only) | 3.27.0 | 3.27.0 | MIT OR Apache-2.0 | Disposable test fixtures |

`cargo tree` also resolves `serde_core`, `serde_derive`, `serde_spanned`,
`toml_datetime`, `toml_parser`, `toml_writer`, `winnow`, `proc-macro2`,
`quote`, `syn`, `unicode-ident`, `thiserror-impl`, and for Niri JSON `itoa`,
`memchr` and `zmij`; tempfile's dev-only graph adds `fastrand`, `getrandom`,
`once_cell`, `rustix`, `linux-raw-sys`, `bitflags`, `libc` and `cfg-if`.
Their licenses, notices, advisory status and SBOM entries still require release
review. None is GPL/AGPL.

### Version-family, maturity and feature constraints

- Iced 0.14.0 advertises Rust **1.88** minimum; `iced_exwlshell` 0.20.1 declares
  Iced component dependencies ^0.14 and `exwlshellev` ^0.20.1. Its metadata did
  not declare a Rust minimum. This is family alignment, **not** a resolved build
  or proof the whole graph meets 1.88. Select/test one toolchain and lockfile.
- Upstream's [README](https://github.com/waycrate/exwlshelleventloop) says
  `iced_layershell`/`iced_sessionlock` are deprecated from 0.20 in favor of
  `iced_exwlshell`; the old observed `iced_layershell` stable was 0.19.1.
  [Migration notes](https://github.com/waycrate/exwlshelleventloop/blob/master/docs/MIGRATION-19.1-%3E20.0.md)
  describe changing size/output/event APIs. These are moving docs, not a stable
  NaliOS API promise. Use the selected release's docs/tests when implementing.
- Iced's [0.14.0 README](https://github.com/iced-rs/iced/blob/0.14.0/README.md)
  calls it experimental. Validate layer and normal surfaces, focus, IME,
  accessibility, output scaling and idle cost before choosing it. Low-level
  Wayland support alone does not supply usable widgets or accessibility.
- Iced defaults include renderers, platform backends and a thread-pool option;
  integrations also enable transitive features. Audit the **resolved** graph
  and executor construction. Target one Tokio backend runtime, not a hidden
  second runtime created by the UI executor. Do not enable unconditional
  rendering, debug/time-travel or extra backends reflexively.
- The integration uses Smithay client toolkit, whose default xkbcommon path
  includes native-library tooling. Renderers depend on native graphics/input
  boundaries too. “Rust-native UI” is not “zero native libraries.” Record exact
  native requirements from the actual builder instead of recycling GTK headers.
- `freedesktop-desktop-entry` publishes **MPL-2.0**, not MIT. Its default gettext
  feature pulls `gettext-rs`; review native/transitive obligations and whether
  disabling it preserves required localization. Do not solve a license gate
  by copying its implementation. A parser does not settle Exec field codes,
  D-Bus activation, terminal dispatch or startup notification semantics.
- `freedesktop-icons` lists GTK only as an upstream dev dependency in the
  observed metadata; that is not a normal library dependency automatically
  inherited by consumers. Check the actual app graph. Review lookup, decode,
  missing assets and hostile/unbounded input separately.
- Configure zbus for supported Tokio integration; avoid unintentionally adding
  another async I/O runtime. Prefer std over redundant crates; minimize image
  codecs/features and audit proc macros/build scripts. Native audio bindings
  such as `libpulse-binding` remain separately reviewed candidates, not a
  license to write a new audio stack just for language purity.
- `niri-ipc`, `niri-config`, old shell implementation crates and unreviewed
  GPL/AGPL snippets are **not approved application dependencies**. Niri remains
  a separate GPL program under the independent-client policy.

### Historical GTK candidates, no longer the assumed frontend

Earlier research identified `gtk4` 0.11.5, `glib`/`gio` 0.22.10 and
`gtk4-layer-shell` 0.8.1 (published Rust binding licenses: MIT). Their metadata
aligned at the 0.11/0.22 family, but no build was run. They are retained as
research context, **not baseline dependencies**; adopting GTK/GIO requires an
explicit exception to the Rust-native direction. Native GTK/GLib and related
libraries retain their own licenses. Do not add these through launcher or
platform defaults and claim the headless/application graph is unaffected.

## 3. External source families and authoritative references

Use Kali's binary/source packages by default. The package snapshot records
Debian `Source` fields; if absent, source name equals binary name. A binNMU
suffix such as `+b1` is not a separate upstream source release. A library's
native license is not inferred from its Debian package name.

| Family / package source names | Upstream code and public interface | What we author |
|---|---|---|
| `niri` (supply unresolved) | [Upstream](https://github.com/niri-wm/niri), [IPC](https://niri-wm.github.io/niri/IPC.html), [packaging](https://niri-wm.github.io/niri/Packaging-niri.html), [integration](https://niri-wm.github.io/niri/Integrating-niri.html) | Independent protocol client, original KDL/config and version tests; no linked GPL Niri crates. |
| `xwayland-satellite` (supply unresolved), `xwayland` | [Satellite](https://github.com/Supreeeme/xwayland-satellite), [Niri Xwayland guide](https://niri-wm.github.io/niri/Xwayland.html), [X server](https://gitlab.freedesktop.org/xorg/xserver) | Supply/build metadata and X11 smoke tests, not a replacement server. |
| GTK/GLib native family, earlier UI alternative | [GTK](https://gitlab.gnome.org/GNOME/gtk), [GLib](https://gitlab.gnome.org/GNOME/glib), [layer shell](https://github.com/wmww/gtk4-layer-shell), [binding docs](https://gtk-rs.org/) | External apps/portals can still use these. Not the first-party UI default; explicit exception if reconsidered. |
| `systemd`, `dbus`, `util-linux` | [systemd APIs](https://www.freedesktop.org/software/systemd/man/latest/), [D-Bus specification](https://dbus.freedesktop.org/doc/dbus-specification.html), [util-linux](https://github.com/util-linux/util-linux) | Lifecycle/health checks; logind/UPower policy integration, not a new init or bus. |
| `mesa`, `wayland`, `libinput`, `libxkbcommon`, `firmware-nonfree` | [Mesa](https://gitlab.freedesktop.org/mesa/mesa), [Wayland](https://gitlab.freedesktop.org/wayland/wayland), [libinput](https://gitlab.freedesktop.org/libinput/libinput), [xkbcommon](https://github.com/xkbcommon/libxkbcommon), [linux-firmware](https://gitlab.com/kernel-firmware/linux-firmware) | Capability diagnostics and hardware tests; preserve distro build/patch provenance. |
| `bluez` | [Source](https://github.com/bluez/bluez), [D-Bus APIs](https://bluez.readthedocs.io/en/latest/) | BlueZ adapter/device state and independent Agent1 workflow. |
| `network-manager`, `network-manager-applet`, VPN plugin sources | [NetworkManager](https://gitlab.freedesktop.org/NetworkManager/NetworkManager), [API docs](https://networkmanager.dev/docs/api/latest/), [applet](https://gitlab.gnome.org/GNOME/network-manager-applet) | State/control client; secrets initially delegated, custom agent only with explicit tests. |
| `pipewire`, `wireplumber`, `pulseaudio`, `playerctl` | [PipeWire](https://gitlab.freedesktop.org/pipewire/pipewire), [WirePlumber](https://gitlab.freedesktop.org/pipewire/wireplumber), [PulseAudio](https://gitlab.freedesktop.org/pulseaudio/pulseaudio), [MPRIS](https://specifications.freedesktop.org/mpris/latest/), [playerctl](https://github.com/altdesktop/playerctl) | Audio controls/subscriptions, MPRIS adapter and OSD, not graph/session policy. |
| `upower`, `power-profiles-daemon`, `brightnessctl` | [UPower](https://upower.freedesktop.org/), [profiles](https://gitlab.freedesktop.org/upower/power-profiles-daemon), [brightnessctl](https://github.com/Hummer12007/brightnessctl) | Battery/profile/backlight controls and truthful capability/failure states. |
| `swaylock`, `swayidle` | [Locker](https://github.com/swaywm/swaylock), [idle](https://github.com/swaywm/swayidle), [session-lock protocol](https://wayland.app/protocols/ext-session-lock-v1) | Coordination/configuration and failure tests; not PAM/password verification. |
| `policykit-1`, `mate-polkit`, `gnome-keyring`, `libsecret`, `gcr4` | [polkit](https://github.com/polkit-org/polkit), [MATE agent](https://github.com/mate-desktop/mate-polkit), [keyring](https://gitlab.gnome.org/GNOME/gnome-keyring), [libsecret](https://gitlab.gnome.org/GNOME/libsecret), [GCR](https://gitlab.gnome.org/GNOME/gcr) | Agent detection/delegation, supported credential unlock, narrow future authorization boundaries. |
| `xdg-desktop-portal`, GNOME/GTK backends | [Broker and docs](https://github.com/flatpak/xdg-desktop-portal), [GNOME backend](https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome), [GTK backend](https://github.com/flatpak/xdg-desktop-portal-gtk), [Niri sharing](https://niri-wm.github.io/niri/Screencasting.html) | Backend selection, capability diagnostics, real application tests. |
| `mako`, later our notification implementation | [mako](https://github.com/emersion/mako), [notification protocol](https://specifications.freedesktop.org/notification/latest/) | Independent C17 service and privacy/history UI after delegated baseline. |
| Tray/menu interoperability | [StatusNotifierItem](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/), [DBusMenu project/protocol](https://launchpad.net/libdbusmenu) | Independent bounded watcher/host/menu adapter, not copied upstream rendering code. |
| `swaybg`, `wl-clipboard`, `cliphist`, `swappy`, `wl-mirror` | [wallpaper](https://github.com/swaywm/swaybg), [clipboard](https://github.com/bugaevc/wl-clipboard), [history](https://github.com/sentriz/cliphist), [annotation](https://github.com/jtheoof/swappy), [mirroring](https://github.com/Ferdi265/wl-mirror) | Owned-process orchestration and explicit optional privacy/capture policies. |
| Desktop discovery/launch/defaults | [desktop-entry spec](https://specifications.freedesktop.org/desktop-entry/latest/), [menu spec](https://specifications.freedesktop.org/menu/latest/), [XDG paths](https://specifications.freedesktop.org/basedir/latest/) | Rust catalog/ranking/favorites and reviewed standards-aware launch/activation; no automatic GIO dependency. |
| Kali categories/tool packages | [Kali package sources](https://gitlab.com/kalilinux/packages), [metapackages](https://www.kali.org/docs/general-use/metapackages/) | Original declarative tool metadata; installed-command checks, no auto-run actions. |
| `libvirt`, `qemu`, `cryptsetup`, `edk2`, `swtpm` | [libvirt](https://libvirt.org/), [QEMU](https://www.qemu.org/), [cryptsetup](https://gitlab.com/cryptsetup/cryptsetup), [EDK II](https://github.com/tianocore/edk2), [swtpm](https://github.com/stefanberger/swtpm) | Optional VM state/metadata/authorization, not hypervisor or cryptography. |

Daily applications/fonts/icons are upstream packages, not source vendored into
our shell. Find each package's Homepage and Source through apt metadata and
its license through `/usr/share/doc/PACKAGE/copyright` and the matching source
package. This covers application/font/toolchain families without pretending
their licenses have all been reviewed by this planning pass.

## 4. Reproducible source rules

### Cargo graph

Commit `Cargo.lock` for the application workspace once it exists. Record Rust
version, Cargo version, targets, enabled features, native pkg-config versions,
and source checksums. Use locked builds; release builds must not silently
resolve new dependencies. Review `cargo tree`/metadata, advisory checks,
license reports, build scripts and proc macros. Unknown/GPL/AGPL linked licenses
block automatic acceptance; interpret SPDX alternatives rather than matching
license words naively.

### Debian/native graph

For each direct package, record binary version/architecture, source package
and source version, repository/release identity, and any local patches. Resolve
source via apt metadata; obtaining source may require separately enabling the
matching source indexes with user approval. Do not add a foreign distro source
just because `apt-get source` has no source entry today.

Archive the exact release's dependency closure and source/build recipe. If
redistributing GPL binaries, plan corresponding-source/notice compliance, not
only an upstream homepage link. Distro-packaged installation versus our own
binary redistribution are distinct arrangements.

### Niri/satellite exception

Audit the currently used paths first. If self-packaging is necessary, select
upstream release tags/commit IDs, archive hashes, release Cargo.lock/vendor
archives where provided, feature selection, native build requirements, patches,
and artifact hashes. Build in a clean environment; include upstream session
resources. Never track `main` as a release dependency or build Niri with
`--all-features`. Keep external GPL source separate from NaliOS application
source and obey its redistribution conditions.

The online Niri documentation is moving: it currently includes options marked
“next release.” Such options are **not assumed available in installed 26.04**.
Gate on the actual release docs/observed capabilities. The documented event
stream supplies initial state then updates; use that rather than a racy
independent snapshot-plus-subscribe sequence for its covered state.

## 5. Review record format and status vocabulary

Each dependency record needs: name/kind; purpose/component IDs; source URL;
exact release/hash; dependency features; license/notice evidence; linked versus
external relationship; redistribution decision; authorization/secret behavior;
maintenance/advisory status; tested versions; fallback; reviewer; review date.

Statuses: **candidate → source identified → license reviewed → build tested →
integration tested → release approved**. A crate metadata lookup reaches only
candidate/source-identification status. The first-party license remains an
independent author decision. See [ownership](ownership.md).

Assets have a separate provenance record: original editable source, author,
materials/tools, applicable terms, export recipe, checksum, and attribution.
Neither an apt package nor a pleasing internet image becomes “our own artwork”
through installation.
