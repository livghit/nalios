# Packaging and session resources

**Reserved structure only. No installer, packages, scripts or services exist.**

- `debian/`: future Debian package metadata/recipes after build policy review.
- `systemd/user/`: future explicitly owned session user units.
- `sessions/`: future display-manager session entry/resources.
- `applications/`: future NaliOS desktop entries.
- `completions/`: future CLI completion resources generated from real commands.

Nothing here installs, enables, replaces or starts a session/service. There is
no `install.sh`, root helper, PAM edit or production-command symlink. First-party
setup/migration logic belongs in Rust; any unavoidable conventional packaging
glue stays minimal and explicitly reviewed.

The Niri/satellite supply route, existing `/usr/local` binaries and prototype
compatibility links remain untouched. See the
[delivery contract](../docs/blueprint/delivery.md) and
[session specification](../docs/blueprint/session.md). `.gitkeep` files are
structural markers, not runnable packaging artifacts.
