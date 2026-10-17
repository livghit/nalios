# nalios-shell

**Status: GPUI accepted; development shell builds.** `nalios-shell-dev` is the
real development shell, promoted from the evaluated spike. It is not installed
over any production command.

## What it does today

- Layer-shell **bar** (top, 36 logical px, exclusive zone, keyboard
  interactivity `None`): live Niri workspace numbers, connection dot, launcher
  button and clock.
- Layer-shell **launcher** overlay (overlay layer, exclusive keyboard): type
  to search installed applications, Up/Down to select, Enter to launch, Escape
  to close. Results show theme icons and the selected app's description;
  favorites from the config come first.
- Live Niri state through our own `nalios-platform::niri` client and
  `nalios-app::niri::NiriObservation`. No `niri-ipc`.
- Layer-shell **dashboard** (`Super+Shift+Space`, or the control command): a
  full overview with workspaces and their windows, the focused window, outputs
  and session health. Escape closes it.
- Private control socket so the CLI can toggle the launcher and dashboard
  (decision 0007).

Run it:

```text
cargo run -p nalios-shell --bin nalios-shell-dev
```

Press Escape to close the launcher (it captures the keyboard while open), then
Ctrl+C in the terminal to stop the shell. Ctrl+C may not reach the terminal
while the launcher holds the keyboard.

Toggle from another terminal:

```text
cargo run -p nalios-cli --bin nalios-dev -- shell toggle-launcher
cargo run -p nalios-cli --bin nalios-dev -- shell toggle-dashboard
```

### Configuration

Create `~/.config/nalios-dev/config.toml` (or `$XDG_CONFIG_HOME/nalios-dev/`):

```toml
schema_version = 1

[launcher]
max_results = 8
favorites = ["firefox-esr.desktop", "org.gnome.Terminal.desktop"]
```

The shell validates it with the same core rules as `nalios-dev config check`;
invalid or missing files fall back to defaults.

### Keyboard shortcut (optional)

Add to `~/.config/niri/config.kdl`, adjusting the path:

```kdl
binds {
    Mod+Space { spawn "/home/gunitsolutions/nalios/target/debug/nalios-dev" "shell" "toggle-launcher"; }
    Mod+Shift+Space { spawn "/home/gunitsolutions/nalios/target/debug/nalios-dev" "shell" "toggle-dashboard"; }
}
```

The shell must be running for the binding to do anything.

## Structure

| File | Responsibility |
|---|---|
| `src/main.rs` | Entry point, key bindings, window wiring |
| `src/shell.rs` | Root view (bar), worker polling, overlay toggling |
| `src/launcher.rs` | Launcher overlay view, icons, favorites and layer-shell options |
| `src/dashboard.rs` | Full dashboard overlay view |
| `src/actions.rs` | Shared shell actions (`Dismiss`) |
| `src/worker.rs` | Niri reader thread, bounded channel, stop flag |
| `src/control.rs` | Control socket worker and action forwarding |
| `src/theme.rs` | Draft semantic tokens (Night palette) |

The library root is still a documentation-only placeholder.

## Dependencies

GPUI is pinned to one Zed revision
(`2d4c4831943153f63ad0aeddf49c8f5a3d008e75`) with the `wayland` feature.
`gpui`/`gpui_platform` are used by `nalios-shell` alone; core/app/platform/CLI
stay free of the renderer graph. See
[decision 0006](../../docs/decisions/0006-ui-toolkit-spike.md).

## Known limitations

- One bar surface; per-output bars and hotplug are not handled yet.
- Dashboard is read-only: no workspace focus/switch actions from the UI yet.
- Launcher text input is key-event based: no IME/composition, cursor movement
  or selection yet.
- No recent applications; favorites are config-only (no UI toggle yet).
- Control channel supports `ping`, `open-launcher` and `toggle-launcher` only.
- Theme tokens are draft values, not the C08 theme system.
