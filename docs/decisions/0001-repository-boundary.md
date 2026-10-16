# 0001 — Repository identity and prototype boundary

Status: **accepted naming and repository separation**, per the user's request.
Application license, dependency selection, and implementation remain open.

## Context

The user wants to write an original Rust desktop layer for Kali + Niri called
`nalios`. The existing Caelestia-based experiment is to be retained as
`kalios-deprecated`, not used as the new source base.

The old checkout contains Git history, uncommitted changes, untracked planning
documents, third-party source/assets, and ignored build caches. Installed user
and system command/config links refer to its old path. Updating root-owned
links requires authentication that is not available noninteractively during
this rename.

## Decision

- New development repository: `/home/gunitsolutions/nalios` (`~/nalios`).
- Deprecated source: `~/.local/share/kalios-deprecated`.
- Keep `~/.local/share/nalios` as a compatibility symlink to the deprecated
  source, so existing installed command/config links continue to resolve.
- Initialize independent Git history on `main`; no remote is configured.
- Copy only the planning blueprint; do not copy legacy implementation,
  templates, vendor source, images/fonts, build products, or Git history.
- Do not create Rust implementation or select a license on the user's behalf.
- Keep the prototype's pending changes uncommitted and intact. Mark its README
  and deprecation notice rather than rewriting all historical names/interfaces.

## Recovery snapshot

Before renaming, save a Git-history bundle, source working-tree archive
(including untracked docs but excluding build/Python caches), staged/unstaged
patches, Git status, and installed-link inventory under:

`~/.local/state/kalios-deprecated/migrations/rename-20261016T112501Z-DTbiOL/`

The existing `.git` and ignored build trees also remain in the renamed
prototype directory. This source snapshot is not a full machine or VM backup.

## Consequences

- The new codebase is isolated from the old GPL shell implementation/history.
  It still depends on upstream software subject to license review.
- `cd ~/nalios` opens the new project. `cd ~/.local/share/nalios` opens the old
  prototype through the compatibility alias; these must not be confused.
- Installed `nalios` / `nalios-*` commands still execute legacy behavior. The
  new repository is not installed and has no runnable shell yet.
- Prototype config/cache/state and VM vault paths are unchanged; no existing
  pairing, guest disk, credential, package, or login session is deleted.
- The compatibility alias may be removed only after installed links/defaults
  are explicitly migrated or the prototype is deliberately uninstalled.
- Public repository creation/push and final source license selection require
  separate decisions; none is implied by local Git initialization.

## Implementation handoff

T001's naming/repository separation is complete. License/dependency policy,
authorship/contribution paperwork, and hardware/recovery baselines remain to
be completed. T002 starts application workspace implementation by the user.
