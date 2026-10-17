# Package and runtime inventory

**Status: candidate integration manifest, not an install script.** The author
approves dependencies and profiles before implementation. No packages were
installed, removed, upgraded, or enabled to produce this plan.

Observed metadata: Kali Rolling **2026.3**, amd64, **2026-10-16**, existing apt
indexes (package-list files dated 2026-10-14 and 2026-10-15). Exact candidate/installed versions
and Debian source-package names are in the
[read-only package snapshot](../research/packages-2026-10-16.tsv).
A candidate means available in that cache, **not tested or license-approved**.
An unavailable entry means no candidate in that cache, not universal absence.

**Rust-only revision:** all first-party executable code/tests stay in Rust.
The UI spike now prioritizes Iced + `iced_exwlshell`, not GTK; previous GTK and
Python/shell-tool rows remain research/alternative context, not a baseline
install set. External GTK portals/apps stay valid separate programs. Native
renderer/input dependencies must be identified from the chosen Rust UI build;
see [sources](sources.md) and [decision 0002](../decisions/0002-rust-codebase-boundaries.md).

## 1. Profiles, not one enormous installation

| Profile | Purpose | Includes / excludes |
|---|---|---|
| `base-session` | A working Kali/Niri laptop | Compositor supply, session integration, network/audio/power/auth/portals, one locker, one notification owner, basic applications. |
| `nalios-desktop` | Original daily desktop | First-party shell/CLI/config/theme packages plus the approved base; replaces delegated UI only at tested ownership boundaries. |
| `developer` | Write/test Rust | Rust toolchain and review tools first; selected UI's measured native build requirements only after T007. No automatic GTK headers or whole compositor build toolchain. |
| `upstream-build` | Only if packaging Niri/satellite ourselves | Exact upstream release build dependencies in a clean builder; never automatically install all headers on users' laptops. |
| `extras` | User-selected workflow features | VPN plugins, clipboard history, annotation, removable-media UI, mirroring, optional Qt configuration. |
| `vm` | Later, separate scope | Libvirt/KVM/QEMU/cryptsetup; never a mandatory desktop dependency. |
| `release-test` | Build and verify Debian artifacts | Packaging/lint/integration tools in a test environment. |

A profile is a desired dependency set, not a new program or a package that
already exists. Debian packages will distinguish `Depends`, `Recommends`, and
`Suggests` after real dependency/size/solver tests. Apt/dpkg resolve the
transitive closure; do not maintain a handwritten list of every shared library.

## 2. Compositor, graphics, and session foundation

| Package/program | Intended role | Selection rule |
|---|---|---|
| `niri`, including `niri-session` and upstream session resources | Compositor, supported session environment/lifecycle, IPC | **Supply blocker:** no apt candidate observed; locally available 26.04 under `/usr/local/bin`, package ownership not established. See [delivery](delivery.md). |
| `xwayland-satellite` | Run legacy X11 apps as normal Niri windows | No apt candidate observed; existing `/usr/local/bin` binary has unverified revision. Establish a reproducible upstream package. |
| `xwayland` | X server used by satellite | Keep as a dependency of the tested X11 path; do not launch a second hard-coded display. |
| `systemd`, `libpam-systemd`, `dbus-user-session` | User services, login session and user bus | Use Kali's existing stack. `dbus-broker` is an alternative bus implementation, not an extra required daemon. |
| `libwayland-server0` | Niri's dynamically loaded Wayland server library | Explicitly account for it in a self-built Niri package; normal shlib scanning may miss it. |
| `libgl1-mesa-dri`, `mesa-vulkan-drivers` | Host GPU acceleration | Retain Kali packages; validate actual EGL/rendering path. Do not replace Mesa to fix a theme. |
| `firmware-amd-graphics`, `firmware-realtek` | Firmware for the observed AMD/Realtek laptop | Host/hardware baseline, not unconditional dependencies for every possible user. Do not remove/replace existing firmware. |
| `xdg-user-dirs`, `xdg-utils`, `desktop-file-utils`, `libglib2.0-bin` | User directories, default handlers, desktop-entry checks, GSettings utilities | Standard integration and diagnosis. Do not reset MIME defaults without opt-in. |

Use Niri's documented automatic satellite integration when the tested versions
support it (upstream documents Niri >=25.08 and satellite >=0.7). Do not add
manual `$DISPLAY=:0` or duplicate startup. Installed satellite's version must
be established from build/package provenance; a `--version` flag was not
supported by the locally observed binary.

## 3. Laptop services and authentication

| Packages | Intended role | Owner / restriction |
|---|---|---|
| `bluez`, `rfkill` | Bluetooth service and radio diagnostics | BlueZ owns device state. NaliOS does not implement drivers/transport. |
| `blueman` | Temporary/recovery Bluetooth UI and agent | Manual or delegated mode only. Do not also make a custom NaliOS agent the competing default unknowingly. |
| `network-manager` | Wi-Fi/Ethernet/network profile backend | Existing system service; no second connection manager. |
| `network-manager-applet` | Initial NetworkManager secret-agent/delegated UI | Current package name in the observed cache. Avoid duplicate autostart from Niri config and XDG autostart. |
| `nm-connection-editor` | Advanced network profile editor | Separate binary package; source package is `network-manager-applet`. Keep a handoff even after custom basic controls. |
| `pipewire`, `pipewire-audio`, `pipewire-pulse`, `wireplumber`, `libspa-0.2-bluetooth` | Audio graph/session policy, Pulse-compatible controls, Bluetooth audio | `pipewire-audio` pulls in the normal audio pieces. Do not start a competing PulseAudio server. |
| `pulseaudio-utils`, `pavucontrol` | Audio diagnostics and advanced/recovery mixer | Clients can talk to `pipewire-pulse`; they do not imply a PulseAudio server requirement. |
| `playerctl` | MPRIS diagnostic/manual fallback | Optional once first-party MPRIS adapter works. Do not poll human-readable output for the normal bar. |
| `upower` | Battery and AC state | Listen to supported properties/events; use its estimates, not a guessed Rust formula. |
| `power-profiles-daemon` | Supported power profiles | Capability-dependent. Do not run alongside a competing profile manager such as TLP without a deliberate policy. |
| `brightnessctl` | Backlight/key-backlight helper | Verify udev/logind permissions; never add blanket world-writable sysfs rules or silently grant broad groups. |
| `swaylock`, `swayidle` | Proven secure lock and idle/sleep coordination | Initial recommendation. One lock/idle policy; test readiness and logind delay behavior. |
| `polkitd`, `mate-polkit` | System authorization service and a standalone session agent | Candidate agent is installed/available in the observed baseline. Use one selected agent per session. Native/UI style is a separate concern. |
| `gnome-keyring`, `libpam-gnome-keyring`, `libsecret-1-0`, `gcr4` | Credential storage, supported login unlock/prompt integration | Test actual display-manager/PAM path. An installed library does not prove the keyring is unlocked. Do not rewrite PAM from an unreviewed installer. |
| `libsecret-tools` | Manual integration diagnostics | Optional; never collect secrets in a doctor report or public fixture. |

`lxqt-policykit` is another available agent, but its current package depends on
`lxqt-session`; do not pick it just because it has a short name. Old instructions
naming `polkit-gnome` / `policykit-1-gnome` do not match an available candidate
here. `plasma-polkit-agent` also had no candidate under that name in this cache.

## 4. Portals and application permissions

| Packages | Intended role | Integration requirement |
|---|---|---|
| `xdg-desktop-portal` | User-facing portal broker | Activated in the correct supported Niri session, not a second manually started broker. |
| `xdg-desktop-portal-gnome` | Niri's documented screen/window sharing backend | Test with PipeWire and the selected Niri build's GNOME screencast integration. Installing it does not require logging into GNOME. |
| `xdg-desktop-portal-gtk` | Fallback interfaces and proposed file chooser | Route FileChooser explicitly if GTK chooser is selected. |
| `gsettings-desktop-schemas` | Required settings schema integration | Some values are shared per user; preview must not silently persist them. |
| `nautilus` | Dependency of the observed GNOME portal package | This apt version **depends** on Nautilus even if we route FileChooser to GTK and keep Thunar as default. Do not promise it can simply be omitted. |
| `gnome-keyring` | Secret portal/service path | Verify the installed backend's supported Secret interface and unlock; not equivalent to a NetworkManager secret agent. |

`xdg-desktop-portal-wlr` is available, but **not the chosen default** simply
because Niri uses Wayland. Select backends by documented interfaces and tested
versions. The observed GNOME portal recommends GNOME components; inspect apt's
actual solver plan and whether each recommendation is needed before accepting
a large dependency set. Do not globally disable all recommendations without
reviewing missing functionality.

Never set `GDK_BACKEND` globally: upstream Niri warns that this breaks the
screencast portal. Portal/session ownership is detailed in [session](session.md).

## 5. Shell support, fonts, and daily applications

| Packages | Role | Default / lifecycle |
|---|---|---|
| `libgtk-4-1`, `libgtk4-layer-shell0` | Earlier GTK frontend alternative, not baseline NaliOS renderer | Rust-native direction supersedes it. GTK may still exist for external apps/portals; any first-party toolkit exception needs approval. |
| `adwaita-icon-theme` | Fallback application/status icon coverage | NaliOS can have original interface assets without redrawing every third-party application's icon. Record licenses. |
| `fonts-inter` | Proposed interface typography | Available/installed; user may select another family. Use distro package rather than copying font files. |
| `fonts-noto-core`, `fonts-noto-color-emoji`, `fonts-noto-mono` | Language/emoji/monospace fallbacks | Review required language coverage and package size. These do not supply every script by themselves. |
| `fonts-jetbrains-mono` | Optional terminal typography | Not mandatory if existing mono fallback is sufficient. |
| `alacritty` | Proposed terminal, already present | Preserve user config; first theme adapter candidate. Not a screen-reader compatibility claim. |
| `firefox-esr` | Proposed browser, already present | Set default only with consent; no forced browser UI/CSS injection. |
| `thunar`, `gvfs`, `gvfs-backends`, `udisks2` | Proposed file manager, supported remote/removable file handling | Keep external applications and polkit authorization. `gvfs-fuse` is optional compatibility, not required by the shell renderer. |
| `xdg-terminal-exec` | Standards-oriented terminal dispatch | Verify desktop-entry behavior and user terminal choice. Never split `Exec` into arbitrary shell code. |
| `fuzzel` | Independent fallback launcher | Keep for recovery/development, not an extra automatically opened launcher. |
| `swaybg` | Initial external wallpaper renderer | One owned invocation per configuration; per-output behavior/stop/reload is managed deliberately. |
| `mako-notifier` | Initial delegated notification service | Replace only after original C17 passes the protocol/ownership tests. `dunst` is an alternative, not an additional owner. |
| `libnotify-bin` | Notification integration tests | Useful `notify-send` client, not a notification service. |
| `orca`, optional `gnome-console` | Screen-reader test and accessible terminal candidate | Test in a real supported session; nested mode is not equivalent. Current Console candidate is a prerelease, so do not silently choose it as a stable default. |

Do not install Alacritty, Foot, and Kitty as mandatory terminals. Foot/Kitty and
Nautilus-as-primary are alternatives only. `wlogout` is unnecessary if our own
power confirmation UI covers the flow. The normal user's graphical editor is
configurable, not a requirement to build another editor.

## 6. Optional desktop/workflow packages

| Packages | Use | Policy |
|---|---|---|
| `wl-clipboard` | Copy/paste helper and optional clipboard watcher | Verify compositor capability; do not automatically record clipboard content. |
| `cliphist` | Possible delegated persistent history | **Opt-in, not baseline.** Bound retention; disclose disk persistence and clear-on-lock limits. Do not claim perfect secret detection or secure erasure. |
| `swappy` | Optional screenshot annotation handoff | Niri capture is primary; annotations are a separate program. |
| `grim`, `slurp` | Optional capture/region helpers | Only if supported protocols and the intended workflow are verified. Niri's built-in screenshot workflow is already a valid initial choice. |
| `wl-mirror` | Possible presentation mirroring | Test separately; do not advertise native mirrored-output profiles just because extended displays work. |
| `network-manager-openvpn`, `network-manager-openvpn-gnome` | User-chosen OpenVPN support and editor/auth UI | Optional; the `-gnome` suffix does not imply a GNOME desktop session is required. Test the actual plugin prompt. |
| `network-manager-openconnect`, `network-manager-openconnect-gnome` | User-chosen OpenConnect support and editor/auth UI | Same explicit plugin scope; no blanket VPN compatibility promise. |
| `udiskie` | Optional removable-media UI/agent | Do not auto-mount everything merely because this package is present. |
| `xarchiver` | Optional archive manager | File-manager handoff, not first-party code. |
| `qt6ct` | Possible opt-in Qt styling | Scope environment/settings carefully; no global cross-session theme hack. |
| `kali-menu` | Existing distro tool-menu metadata | Preserve; review format/license before importing any metadata. Our categorization is independently authored. |
| `kali-tools-top10`, `kali-linux-default` | Existing/user-selected Kali tool collections | **Not desktop dependencies.** Never install a giant tools metapackage just to make launcher categories nonempty. |

`swww` / renamed `awww`, `swaylock-effects`, `bluetuith`, and `wlr-protocols`
had no candidate under those package names in this cache. They are not required
for the plan. Do not resolve that by mixing Ubuntu PPAs, Arch instructions, or
unreviewed curl-to-shell installers into Kali.

## 7. Build and test dependencies

| Packages/tools | When needed | Notes |
|---|---|---|
| `build-essential`, `pkg-config`, `git`, `ca-certificates`; `curl` for explicit downloads | Basic development | No `curl ... | sh` bootstrap. Verify downloaded source/archive identity. |
| `rustc`, `cargo`, `rustfmt`, `rust-clippy` | Distro-toolchain development route | Current rustc/cargo are 1.95.0; fmt/clippy are separate packages here. Pick one toolchain strategy. |
| `rustup` | Alternative toolchain route | Cached Debian rustup package conflicts with distro Rust packages. Do not blindly install both or switch a working toolchain. |
| `libgtk-4-dev`, `libgtk4-layer-shell-dev` | Historical UI alternative, not needed for the headless milestone or assumed Rust-native UI | Do not install by default. Resolve the actual Iced/Wayland renderer/input requirements in T007. |
| `libpulse-dev` | Candidate audio adapter using `libpulse-binding` | Talks to `pipewire-pulse`. Only if this interface is approved. |
| `libsecret-1-dev` | Only if a linked libsecret adapter is selected | Not necessary merely to use an external keyring/agent. |
| `cargo-deny`, `cargo-audit`, `cargo-cyclonedx` | License/advisory/SBOM review | Cargo tool candidates, not asserted Kali binary packages; pin/review their own dependencies. |
| `tempfile`, `proptest` | Rust test dependencies | Cargo crates, not apt packages. See [source register](sources.md). |
| `shellcheck`, `shfmt` | Only if unavoidable packaging glue needs review | Not a first-party desktop implementation language. Behavior and test automation live in Rust/Cargo/optional xtask. |
| `debhelper`, `devscripts`, `lintian`, `autopkgtest` | Debian release builder | `dh-cargo` only if that build workflow is chosen; not an automatic solution to crates.io/network policy. |
| `python3-pytest` | Historical candidate, not selected for our tests | First-party tests/harnesses are Rust. External upstream test suites may have their own tooling. |
| `clang`, `mold`, `libssl-dev` | Conditional tools | Clang may be needed for upstream builds; mold is optional; OpenSSL headers only for an actual selected dependency, not by habit. |

Rust-native UI libraries arrive through Cargo, not assumed apt packages named
`iced`. Native libraries may still include input/graphics/font components; the
T007 builder records the resolved closure and feature choices before updating
the runtime/development profiles. No claim that Iced removes all native deps.

For a self-packaged Niri build, upstream currently documents a dependency family
including `libudev-dev`, `libgbm-dev`, `libxkbcommon-dev`, `libegl-dev` (check the
release's exact EGL pkg-config requirements), `libwayland-dev`, `libinput-dev`,
`libdbus-1-dev`, `libsystemd-dev`, `libseat-dev`, `libpipewire-0.3-dev`,
`libpango1.0-dev`, and `libdisplay-info-dev`, with GCC/Clang and Cargo. Check the
**selected release**, not just current main, and build without `--all-features`.
Satellite has its own release-specific XCB/X11/xkbcommon development closure;
resolve that in its builder, not by guessing it from Niri's list.

## 8. Optional VM profile, explicitly later

`libvirt-daemon-system`, `libvirt-clients`, `qemu-system-x86`, `qemu-utils`,
`virt-viewer`, and `cryptsetup` provide the backend family. `virt-manager` can
remain the advanced external UI; `ovmf` and `swtpm` depend on chosen guest needs.
No network bridge, libvirt group grant, LUKS format, root service, or vault
migration is a side effect of installing the normal desktop.

## 9. What the implementation manifest must record

For every **direct** package: profile, reason, required/optional/alternative,
source package, upstream reference, tested version/range, installed executable
path, service ownership, config files, authorization, fallback, acceptance IDs,
and review status. Generate the exact **transitive** installed/package-build
inventory for each release instead of promising that this document lists every
library in Kali. See [sources](sources.md) and [delivery](delivery.md).

Recheck cached availability before each install plan. Inspect apt's simulated
solution for removals, replacements, new desktop components, disk size, and
recommendations. A package table is not permission to apply that solution.
