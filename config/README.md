# Configuration resources

**Reserved structure only. No usable configuration has been generated.**

- `niri/`: future original Niri KDL, validated against the selected release.
- `portals/`: future session-specific portal routing.
- `delegated/`: future reviewed locker/idle/wallpaper/notification configuration.
- A future `defaults.toml` will follow the authored shell schema; it is not
  created before that schema exists.

These are non-executable data/templates, not installed user settings. Do not
copy prototype config or automatically write to `~/.config`. Managed outputs,
user source settings and package defaults remain separate. Any required
first-party executable session/setup behavior is Rust, not a Bash/Python helper.

See [configuration architecture](../docs/blueprint/architecture.md),
[session integration](../docs/blueprint/session.md), and
[theme application](../docs/blueprint/themes.md). `.gitkeep` files only preserve
the directory layout in Git.
