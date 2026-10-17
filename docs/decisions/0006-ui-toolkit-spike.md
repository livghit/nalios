# 0006 — Rust-native UI toolkit spike (proposed)

Status: **accepted; GPUI is the production UI stack.** The spike passed the
author's live evaluation ("works") on 2026-10-16, and the shell work now builds
on the pinned GPUI revision. Recorded after dependency/API reconnaissance and
the approved spike.

## Context

T007 requires a Rust-native UI toolkit for the shell. Two candidates were
investigated; both are pre-1.0.

| | Iced + `iced_exwlshell` | GPUI (Zed) |
|---|---|---|
| License | MIT | Apache-2.0 |
| Layer-shell | Third-party `iced_exwlshell` 0.20.1, published; upstream mid-migration | Upstream `LayerShellOptions` (namespace, layer, anchors, exclusive zone, margins, keyboard interactivity) |
| Layer-shell availability | crates.io release | Only on `main`; published `gpui` 0.2.2 lacks it and `gpui_platform` is unpublished → **git dependency required** |
| Accessibility | No `accesskit` in `iced_winit`/`iced_wgpu` | `accesskit` + `accesskit_unix` on the Wayland/X11 backends |
| Locked packages (measured) | 428 (861 Linux tree lines) | 678 wayland-only / 709 default (1359–1391 lines) |
| Supply chain | crates.io releases | Git pin to the Zed monorepo plus a transitive git `naga` pin from `gpui_wgpu` |
| Ecosystem evidence | `lala-bar` (Iced layer-shell bar, 17.6k downloads) | `gpui-shell` (152★), `Capsule` (48★), `kumaUI` (Niri/Sway) |
| Main risk | Experimental; third-party shell integration; no AT-SPI | Pre-1.0 API churn; large git-pinned dependency surface |

Other options checked: egui needs an immature third-party layer-shell backend
(8★); libcosmic is MPL-2.0 but a git-only full DE framework that would make
NaliOS a COSMIC derivative; Xilem/Masonry has no layer-shell and is very
early; Slint's license conflicts with the permissive goal; a custom
sctk/renderer stack is the last resort. Details in the
[research notes](../research/README.md).

The current headless workspace has 40 locked packages. Both options add an
order-of-magnitude larger transitive surface and require full
license/notice/advisory review before distribution. Neither has been built or
run; no UI dependency is in this repository.

## Options

1. **GPUI spike first** (recommended): the upstream layer-shell API and
   `accesskit_unix` match the shell's hard requirements. Accept a git-pinned
   Zed revision and ~680–710 locked packages. The spike decides whether GPUI
   behaves well as a panel/launcher.
2. **Iced spike first**: smaller, crates.io-only surface and a published
   layer-shell integration, but no accessibility support and a third-party
   integration mid-migration.
3. **Defer the UI** and continue headless hardening (D-Bus probes, packaging
   groundwork).

## Implementation (2026-10-16)

- GPUI pinned to Zed revision `2d4c4831943153f63ad0aeddf49c8f5a3d008e75`,
  `wayland` feature only.
- `nalios-shell-spike` builds in ~5m30s on the target laptop; 104 workspace
  tests still pass; `gpui` does not appear in the core/app/platform/CLI graphs.
- The spike uses `nalios-platform::niri` and `nalios-app::niri::NiriObservation`
  on a reader thread — no `niri-ipc`.
- Run instructions and evaluation criteria are in
  [crates/nalios-shell/README.md](../../crates/nalios-shell/README.md).

## Decision

GPUI is selected for production shell work. Iced remains the documented
fallback if a later capability gap appears (it is not maintained as a parallel
stack). The throwaway `nalios-shell-spike` binary was replaced by the
structured `nalios-shell-dev` development shell.

## Consequences if approved

- The spike is a dedicated, throwaway binary, not production shell code.
- Full transitive license/notice/advisory review is required before
  distribution; git dependencies must be pinned and vendorable for release.
- The spike runs in the user's live session and needs their visual/input
  judgement; it is not a headless CI test.
- A failed spike means recording the blocker and choosing another option; it
  does not authorize a custom rendering engine or a silent GTK fallback.
