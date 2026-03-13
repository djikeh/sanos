# Releasing `sanos`

This repository contains several workspace crates, but the public crates.io release
target is currently the library crate `sanos`.

The helper crates `sanos-schema`, `sanos-io`, and `sanos-cli` are marked
`publish = false` to avoid accidental publication before their public API is
stabilized.

## Preconditions

- Review and update [CHANGELOG.md](CHANGELOG.md).
- Ensure the git worktree is clean for release-related files.
- Ensure crates.io credentials are configured (`cargo login` or
  `$env:CARGO_REGISTRY_TOKEN`).

## Validation

From the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File tools\release_sanos.ps1
```

This runs:

- `cargo test -p sanos`
- `cargo test -p sanos --no-default-features`
- `cargo doc -p sanos --no-deps` with `RUSTDOCFLAGS="-D warnings"`
- `cargo package -p sanos`

## Publish

After validation succeeds and the release commit is ready:

```powershell
powershell -ExecutionPolicy Bypass -File tools\release_sanos.ps1 -Publish
```

## Suggested release flow

1. Update `crates/sanos/Cargo.toml` version if needed.
2. Update `CHANGELOG.md`.
3. Commit release changes.
4. Run `tools/release_sanos.ps1`.
5. Tag the release, for example `git tag v0.2.0`.
6. Push the release commit so README image links are already live on GitHub.
7. Run `tools/release_sanos.ps1 -Publish`.
8. Push the version tag, then create the GitHub release from the changelog entry.
