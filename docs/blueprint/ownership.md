# Authorship, licensing, and provenance

This is an engineering policy for discussion, not a legal opinion. Review
actual licenses and obtain qualified advice before making distribution or
licensing promises where the distinction matters.

## The ownership goal

The author writes and maintains the original NaliOS application code,
configuration, documentation, and artwork. We do not copy, translate, fork,
vendor, or relabel Caelestia/Quickshell implementation code as a new project.

Ownership and licensing are separate: authors keep copyrights subject to any
agreements; a chosen license grants recipients permissions. A permissive
license permits others to reuse the code; it does not mean exclusive control
over every downstream use. Other contributors retain their rights unless an
actual agreement says otherwise.

## Important verified boundary: Niri is GPL

The existing local Niri 26.04 source declares `GPL-3.0-or-later` in its workspace
package metadata. Its `niri-ipc` crate inherits that license too. Therefore:

- Do not assume a small API/types crate is permissively licensed.
- Do not link `niri-ipc`, `niri-config`, or another GPL implementation crate
  into an application intended to have a different licensing policy without
  reviewing the consequences.
- Use upstream Niri as a separate program and write an independent client for
  its documented IPC protocol. Record supported messages and versions; test
  interoperability rather than copying Rust enum/type implementations.
- Protocol interoperability is not a blanket legal exemption. Check the
  actual implementation and distribution arrangement, especially if it grows
  beyond ordinary independent IPC.
- If redistributing Niri binaries or modifications, meet Niri's GPL
  obligations separately. Using the distro's package is different from
  publishing our own bundled binary.

Kali/Linux and several normal desktop services include GPL software. A
requirement of **no GPL software anywhere** conflicts with the chosen Kali +
Niri foundation. The achievable application goal here is independent NaliOS
code with reviewed dependencies and no inherited Caelestia implementation.

## New-repository policy

1. Create a separate repository, not a copied vendor tree with a new language.
2. Write behavior requirements from the user's workflows and public
   interoperability specifications; write the implementation independently.
3. Do not port old QML/Python/Bash algorithms line by line, paste GPL snippets,
   or ask a code generator to translate the old port.
4. Treat the old prototype as a behavior/data inventory, not a source library.
   Do not automatically import its themes, SVGs, wallpaper images, fonts, icon
   sets, templates, tests, or metadata.
5. Record external references and authorship as work is added. Our prior access
   to third-party source means we should not advertise a formal legal
   "clean-room" process that we did not perform.
6. If external code is genuinely needed, identify its license/author and make
   a deliberate compatibility decision instead of removing notices.

The planning documents may describe protocol requirements and user-visible
behavior; they are not authorization to copy third-party implementations.

## Dependency review policy

Prefer actively maintained, narrowly scoped dependencies. Rust-only code is
not automatically original, permissively licensed, secure, or maintained.

For each Cargo crate **and native library**, record:

- name, exact version/source, purpose, maintainer/project URL;
- license expression and notices, including transitive dependencies;
- whether linked into our binary, loaded dynamically, or invoked as a separate
  program/service;
- redistribution obligations and compatibility with the chosen app license;
- update/security review status and replacement/fallback options.

Permissive dependencies are the default preference, not an approved list.
GTK/native libraries may use LGPL licenses: they can often be used without
licensing the application as GPL, but have their own conditions, including
ones affected by linking/redistribution. GTK Rust bindings do not erase the
native GTK library's license. A strictly all-permissive linked dependency
policy would change the UI-toolkit choice and must be decided explicitly.

Do not auto-approve GPL/AGPL linked application dependencies or unknown
licenses. A deny rule is a review gate, not a substitute for interpreting
license expressions correctly. Adopt license/security tooling such as
`cargo-deny`, `cargo-audit`, and a software bill of materials once dependencies
exist; tools supplement manual review.

## Choose the first-party license explicitly

Possible direction: MIT OR Apache-2.0 for original application code, after
compatibility review. The author must select it; these documents do not add a
LICENSE file or grant a license on the author's behalf.

Keep code, documentation, and artwork license decisions explicit. Original
wallpapers may use a different license. Before public release add the chosen
license text(s), copyright/author attribution, third-party notices, and a
contribution policy. A Developer Certificate of Origin can document contributor
permission without claiming that it transfers ownership.

## Asset and brand policy

- Create new NaliOS compositions, symbols, wallpapers, and palette definitions.
- Being Kali-inspired does not grant permission to copy a Kali dragon, logo,
  upstream wallpaper, or other protected artwork. Review applicable copyright
  and trademark policies before using Kali branding or suggesting affiliation.
- Record fonts, icons, stock material, and generation tools separately.
  Bundling an external font/icon set requires its license even when our UI is
  original.
- Asset manifest fields: ID, path, author, source URL or original-source path,
  creation date, license, modifications, required attribution, and export
  recipe. Include generation/model terms if relevant; do not promise exclusive
  rights to generated outputs without checking them.
- Keep editable source files and reproducible exports. Do not assume every
  palette/image in the prototype belongs to us.

## Proposed repository paperwork

The user should create these when setting up the new repository:

- chosen `LICENSE` file(s), `AUTHORS`, and `CONTRIBUTING.md`;
- `THIRD_PARTY_NOTICES.md` and a dependency review register;
- asset provenance manifest and artwork-license files;
- decision records for IPC implementation, UI toolkit, linked dependencies,
  external agents, and redistribution strategy;
- versioned release inventory/SBOM before sharing binaries.

These records explain what the project owns, what it depends on, and which
obligations remain. Do not describe the entire desktop as exclusively owned
NaliOS software.
