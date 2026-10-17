# Shared test resources

**Reserved structure only. No tests or fixtures are implemented.**

`fixtures/` is for future sanitized protocol/config/asset test data. Do not copy
personal window titles, networks, device addresses, secrets or old upstream
implementation fixtures into it.

This is a virtual Cargo workspace: Rust files at the repository root under
`tests/` would **not** automatically become Cargo integration tests. Put real
test targets in the relevant `crates/<name>/tests/` directory (or crate-local
unit tests). Add a dedicated Rust harness only when its scope is justified.
No Python/Bash test framework is selected.

Default tests must be deterministic and avoid real-home/device/authorization
side effects. Hardware/session checks are explicit and separately recorded.
A successful check of empty library targets is not a passing feature test.

See the [first milestone](../docs/blueprint/first-milestone.md) and
[acceptance plan](../docs/blueprint/acceptance.md). `.gitkeep` only preserves the
future fixture directory in Git.
