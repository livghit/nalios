# Original design, theme, and wallpaper brief

This is a future creative brief, not a requirement to produce artwork before
the shell functions. The user designs/writes the implementation and approves
the identity. No Caelestia layouts/assets or prototype wallpapers are imported
by this plan.

## Desired character

NaliOS should feel like one intentional personal desktop, not a collection of
unrelated utilities. A possible direction is a quiet technical workspace:
strong typography, restrained surface depth, clear device state, compact
controls, and one recognizable accent. These are ideas, not final palette or
branding decisions.

Kali-inspired means drawing from the user's security/technical workflow and
creating original compositions. It does not require copying Kali's dragon,
logos, wallpaper, typography, or upstream artwork. Review relevant copyright
and trademark policies before suggesting an official affiliation.

## Design the system before the gallery

1. List the main journeys: launch, switch workspace, inspect system, connect a
   device, change settings, lock/sleep, recover from failure.
2. Draw independent wireframes for bar, launcher, dashboard, quick controls,
   settings, notifications, and future VM management.
3. Choose spacing, type hierarchy, focus/selection behavior, radii, borders,
   depth, icon treatment, and motion rules.
4. Build one original neutral dark theme and a usable light counterpart.
5. Check keyboard navigation, contrast, scaling, and power cost on the laptop.
6. Add expressive palettes and wallpaper collections only after roles/formats
   are stable.

Do not aim for exact Caelestia source/layout parity. Preserve user-valued
interactions, such as separate launcher and full dashboard shortcuts, while
authoring a NaliOS visual system.

## Theme vocabulary

Define semantic roles rather than hard-coding colors per widget:

| Group | Example roles |
|---|---|
| Surfaces | background, surface, elevated surface, overlay, border |
| Content | primary text, secondary text, disabled text, link |
| Interaction | accent, on-accent, selection, focus, hover, pressed |
| Status | success, warning, error, information, associated readable foregrounds |
| Layout | spacing steps, corner radii, border widths, type sizes, icon sizes |
| Motion | durations, easing choices, reduced-motion/static alternatives |

A versioned theme has ID/name/author/license/mode, semantic roles, optional
wallpaper references, and compatible config version. It is data validated by
Rust, not executable code. Each widget uses roles so that new palettes do not
require new Rust branches.

Optional palette generation from wallpaper is a later feature. It must respect
contrast/role constraints and not silently replace manually chosen colors.
Do not keep the abandoned shell's palette schema merely to preserve a bridge
we intend to remove.

## Interaction rules

- State is communicated by text/icon and progress, not color alone.
- Pending operations distinguish queued, waiting for authorization, executing,
  canceled, failed, and confirmed results where relevant.
- Bluetooth: clearly separate power, scan, discoverability, pair, trust, and
  connection. Show selected device identity during authentication.
- Network: distinguish an available AP, saved profile, active connection,
  limited connectivity, and credential request.
- Keep destructive actions visually separate and identify the target.
- Escape and focus behavior remain consistent across surfaces; a secure lock
  is not dismissed like an ordinary overlay.
- Keyboard navigation and visible focus are required, not later polish.
- Themes include readable disabled/error states. Target WCAG contrast guidance
  (normally 4.5:1 for normal text and 3:1 for large text/appropriate UI elements)
  as a measurable design check, not a claim of full accessibility conformance.
- Transparency/blur have opaque fallbacks. Respect text scaling and reduced
  motion; stop animations/visualizers when not visible.

## Layout decisions to draw and approve

- Bar position: top, side, or bottom; single versus per-output bar.
- Workspace representation and grouping of app/system actions.
- Launcher size, search scope, category navigation, and results/actions.
- Full dashboard sections and which cards are useful rather than decorative.
- Control-center navigation and small-display behavior.
- Settings density, help/error locations, and recovery affordances.
- Notification placement/history, DND/presentation behavior, and lock privacy.
- Multi-monitor output selection/focus and fractional-scale rules.

## Original wallpaper pipeline

Create a small collection with different purposes rather than many near-copies:

- quiet everyday background with legible foreground windows;
- expressive technical/abstract composition;
- low-distraction dark and light variants;
- matching lock-friendly image if useful to the delegated locker's capabilities.

Keep editable sources (`SVG`, layered artwork, or the user's own generator),
export recipes, previews, final files, and provenance separate. Export at
resolutions/aspect ratios relevant to the actual laptop and external displays;
check crop/fill behavior rather than stretching images. Use bounded decoding
and cache generation to avoid memory spikes with large assets.

For each asset record author, original-source path, tools, source materials,
license/attribution, creation date, modifications, and intended use. External
fonts/icons/material and generated assets need their own review. Original
NaliOS authorship must not be assumed for arbitrary internet/generated images.

## Deliverables for the later creative phase

- A short visual identity guide and independent wireframes.
- Semantic token/theme specification with examples authored for NaliOS.
- Reusable component state sheet, including loading, errors, focus, disabled,
  light/dark, high-contrast-friendly, and reduced-motion states.
- Asset provenance manifest and original wallpaper source/export collection.
- Preview screenshots at actual laptop scales and accessibility/performance
  findings; keep personal content out of promotional screenshots.

Theme/artwork work is complete when it is coherent, configurable, attributed,
and usable—not merely when it resembles an existing shell.
