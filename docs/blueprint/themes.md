# Original NaliOS experience and theme specification

**Status: proposed visual/format contract for author approval.** Theme names,
layout, colors and token values here are drafts, not final branding or shipped
assets. The [creative/provenance brief](design.md) remains authoritative about
independent design and original wallpaper sources.

## 1. A clear visual direction

A quiet technical desktop: deep neutral surfaces, readable typography,
restrained teal identity, precise focus cues, useful status information, and
small intentional transitions. Kali-oriented workflows belong in the launcher
and tools, not in constant “hacker” animations, fake telemetry or copied logos.

Proposed release collection:

- **Nali Night:** everyday neutral dark theme with one teal accent.
- **Nali Day:** a genuinely designed light counterpart, not inverted dark CSS.
- **Nali Signal:** optional later expressive accent variant once the role model
  and dark/light pair are stable; not a separate widget implementation.
- At least one original quiet wallpaper in matching dark/light exports; a more
  expressive technical composition later. No bundled asset without provenance.

Approve the first pair before producing a gallery. Use the same components and
state semantics for every theme. No Caelestia layout/source/palette schema import.

## 2. Proposed surfaces and navigation

```text
Top bar:   [Nali / launcher] [workspaces] [focused app, bounded] ... [status] [time]
Launcher:  one focused search field -> keyboard results -> explicit app/actions
Dashboard: full overview, not a narrow strip:
           [workspaces / recent useful actions] [media]
           [network + Bluetooth] [audio + brightness + battery]
           [settings] [lock] [power confirmation] [diagnostics]
Settings:  ordinary resizable window with navigation and searchable supported options
```

- **Bar:** proposed 36 logical-pixel top height, per-output workspace state,
  sane text truncation and clickable status targets. No permanent CPU waveform.
  Narrow screens collapse low-priority content rather than shrinking text.
- **Launcher:** proposed maximum width 640 logical pixels with responsive
  margins. Search includes installed apps first; optional Kali categories and
  actions are labeled, not mixed with arbitrary shell execution.
- **Dashboard:** proposed comfortable maximum width around 1040 logical pixels;
  adapt to the internal display and text scaling. A useful full overview opens
  at `Super+Shift+Space`; empty backend cards explain unavailability.
- **Settings:** general, appearance, keyboard/input, displays, network/device
  handoff, sound/power, privacy, integrations and diagnostics. Hide unsupported
  setters or explain why disabled. Never fake a successful setting.
- **OSD / notifications:** placed away from the main typing area; OSD never
  steals focus. Decide per-output versus focused-output placement explicitly.

`Super+Space` remains launcher; Escape dismisses normal overlays and restores
previous valid focus. All critical bar/dashboard actions also have a keyboard
path; upstream Niri has accessibility limitations for focusing layer panels.
Test dedicated shortcuts/normal-window alternatives rather than assuming a
screen reader can reach a passive bar.

Layer-shell exclusive zones, overlay keyboard interactivity, outside-click
behavior, fullscreen coexistence, hotplug placement and focus restoration are
part of the toolkit spike. Do not force all these surfaces into one giant
exclusive layer window.

## 3. Design tokens and component states

| Token group | Proposed baseline | Validation/use |
|---|---|---|
| Spacing | 4, 8, 12, 16, 24, 32 logical pixels | Shared layout rhythm; text scaling may expand controls |
| Type | Body 14, small labels 12, section 18, title 24 logical units | Interface font Inter candidate with Noto/system fallbacks; not tiny all-caps text everywhere |
| Monospace | Existing Noto Mono or optional JetBrains Mono | Technical values only where appropriate; not all prose |
| Radii | Small 6, control/card 10, panel 14 logical pixels | Consistent rather than a different radius on every widget |
| Border/focus | 1 px border, visible 2 px focus treatment | Crisp at fractional scale; do not rely on color-only focus |
| Targets | Normal controls at least 36 px, primary touch-friendly targets around 44 px | Expand for scale/accessibility; compact bar icons still have usable hit boxes |
| Motion | Feedback 100–140 ms, overlay 160–220 ms | Reduced-motion uses immediate/fade-only alternatives; no hidden continuous animation |
| Depth | Three main surface levels, restrained optional shadow | Opaque/no-effects fallback; no compositor blur assumption |

A component state sheet must cover: default, hover, focused, selected, pressed,
disabled, loading/pending, awaiting authorization, empty, stale, disconnected,
error and confirmed success. Draft wireframes must show these states, not only
the ideal screenshot. Bluetooth power/scan/discoverable/pair/trust/connect have
separate labels and state transitions.

Use accessible names, logical focus order and non-color state cues. Contrast
and screen-reader exposure are tested, not inferred from any toolkit choice.
T007 must verify the proposed Rust-native toolkit's actual accessibility/IME
support; a successful rendered panel alone does not pass.

## 4. Draft opaque color roles

These values are independently proposed starting tokens, **not final art**.
Use semantic roles, not palette names in widget branches.

| Role | Nali Night | Nali Day |
|---|---|---|
| Background | `#11161C` | `#F3F6FA` |
| Surface | `#19222C` | `#FFFFFF` |
| Elevated surface | `#243140` | `#E6EDF5` |
| Primary text | `#E8EEF5` | `#17212C` |
| Secondary text | `#AAB7C6` | `#48596D` |
| Accent | `#6CD9C8` | `#00695C` |
| On accent | `#0C201E` | `#FFFFFF` |
| Focus | `#A8F1E6` | `#005B50` |
| Structural border | `#64788D` | `#6C7E91` |
| Error foreground on surface | `#FFB4AB` | `#A02F28` |
| Warning foreground on surface | `#F4D18A` | `#735000` |
| Success foreground on surface | `#8FDFAC` | `#1E663B` |

A local calculation using the WCAG relative-sRGB luminance formula gave these
**token-pair** ratios on 2026-10-16; this is not a rendered-UI test or a complete
accessibility audit:

| Pair | Night | Day |
|---|---|---|
| Primary text / surface | 13.76:1 | 16.27:1 |
| Secondary text / surface | 7.88:1 | 7.18:1 |
| On-accent / accent | 9.99:1 | 6.61:1 |
| Focus / surface | 12.54:1 | 8.04:1 |
| Structural border / surface | 3.53:1 | 4.17:1 |
| Secondary text / elevated surface | 6.48:1 | 6.08:1 |

Before approval define the remaining selection/hover/pressed/disabled/overlay
roles and any status-filled surfaces with matching readable foregrounds. Check
**every actual adjacent pair**, including focus against selected/hover colors,
not just this table. Normal text target is generally 4.5:1; large text and
appropriate UI elements 3:1. Transparency must be evaluated after compositing
or replaced by a guaranteed opaque fallback; wallpapers cannot determine text
legibility. Never reduce state visibility to get a softer screenshot.

## 5. Theme document contract

Proposed format: schema-versioned TOML. Final field spelling is settled in C08,
not a compatibility promise made by this document.

Required content:

- identity: schema version, bounded unique ID/name, author, license/provenance,
  dark/light mode, optional description, compatible config range;
- semantic roles for surfaces/content/interaction/status, each with defined use
  and readable paired foregrounds;
- bounded layout/type/motion tokens and reduced-motion behavior;
- optional asset references resolved inside approved theme/asset roots;
- declared built-in output adapters and whether an app needs reopen/reload.

Separate a user's selection (theme ID, wallpaper override, text scale) from the
bundled theme definition. A theme **cannot** contain commands, executable
plugins, environment assignments, arbitrary output paths, remote downloads,
Rust/JS/Lua code, or a user-supplied unescaped template. Adapter code is authored
and allowlisted in the application. No absolute/traversal paths or symlink
escape from an imported theme pack. Do not support remote theme installation
in the first release.

Unknown schema versions fail clearly. Incomplete token sets cannot replace the
last good theme. Release validation rejects missing/unresolved licenses or
provenance; this document does not choose the author's asset license. Malformed,
recursive/active SVG or excessive image dimensions/decoded byte sizes need
bounded handling even if their file extensions look familiar.

## 6. Exactly what we can promise to theme

| Target | Integration | Promise / limitation |
|---|---|---|
| NaliOS shell | Semantic tokens → Rust-native widget styles | Full control of authored components, all states and light/dark pair; no GTK/CSS assumption |
| Niri | Managed KDL colors/focus/border/gaps where chosen | Use supported release grammar and native validation; don't overwrite user window/input rules |
| Wallpaper | Approved original image → owned swaybg configuration | Per-output crop/fill and fallback; no stretched images or background animation requirement |
| Alacritty | Managed colors include/import compatible with selected release | First proposed terminal adapter; preserve unrelated user config |
| swaylock | Supported foreground/background/image/color options | Secure readable approximation, not a custom NaliOS password renderer |
| mako, while delegated | Authored managed configuration | Compatible subset of tokens; remove/retire adapter when original notification owner takes over |
| GTK3/GTK4 third-party apps | Supported settings, font/icon choices, preferred color scheme | Best-effort external integration; Rust shell styles do not restyle every app |
| libadwaita applications | Supported light/dark/accent behavior where available | No forced global stylesheet or claim of arbitrary palette parity |
| Qt apps | Optional reviewed style/settings adapter, e.g. Qt6ct if appropriate | Opt-in and version-scoped; no global override breaking another desktop |
| Firefox/Chromium/Electron | Supported dark preference, user choice | No mandatory userChrome/theme injection or forced private browser settings |
| Kali tools | Their supported native styling, if any | Many have independent/custom toolkits; label unthemed applications honestly |

The [package plan](packages.md) selects **one** initial terminal/notification/
wallpaper path. Additional adapters are separate tickets, not a reason to
install every themable application. Global GSettings can affect another
session for the same user; clearly disclose/preserve previous values.

## 7. Safe preview/apply/revert pipeline

1. Resolve the requested theme and assets; validate schema, licenses required
   for bundling, contrast constraints, supported targets and path/size limits.
2. Build an immutable staged generation under NaliOS state with a unique ID,
   input hashes, adapter versions, planned output files and prior values.
3. Generate only declared managed outputs. Use native syntax validators where
   available (including Niri); distinguish unsupported from validated syntax.
4. Preview **inside the shell/test surface only** unless a separately designed
   reversible external preview is explicitly requested. Browsing previews does
   not write GSettings, reload the terminal, or change the lock screen.
5. For apply, take a transaction lock; record a recoverable journal, old hashes/
   backups and intended generation. Detect user modifications since planning;
   refuse or ask, never silently overwrite them.
6. Commit with per-file atomic replace and durability appropriate to recovery.
   Multi-file/app changes are **not globally atomic**. Track each completed
   step, including any shared setting and owned-process reload.
7. Confirm active shell/Niri/delegated outputs where possible. Report partial
   application or restart-required targets accurately, not universal success.
   Update the active-generation marker consistently with the committed state.
8. On interruption/failure, recover/rollback according to the journal. Retain
   keyboard/headless revert and a built-in safe theme with no wallpaper needed.

Test disk full, malformed data, bad permissions, symlink changes, user edits,
concurrent applies, lost service, monitor unplug and process death at each
step. Recovery must work without the theme picker. Cleanup old generations
only after a safe retention/backup policy, never while rollback needs them.

## 8. Wallpaper/icon/font production and acceptance

Keep editable sources, export recipe/tool versions, final hashes and attribution.
Create dark/light compositions at the observed laptop and useful external-display
aspect ratios; test crop/fill, portrait monitors and lock readability. Package
fonts/icons via the distro where suitable, retaining their identity/license.
Original shell symbols must still have accessible labels and recognizable
status meaning. Kali inspiration does not grant Kali trademark/logo permission.

The visual finish is accepted only with an approved identity sheet, component
states, theme schema examples, original asset manifest, dark/light screenshots
at actual supported scales, keyboard/screen-reader findings, measured animation
cost, and successful preview/apply/revert/failure tests. Screenshots must use
sanitized content. No promise that every third-party Kali tool can be forced
to match the shell exactly.
