# Release process

[English](RELEASING.md) · [한국어](ko/RELEASING.md) · [简体中文](zh-CN/RELEASING.md)

TERMCADE is distributed as executable archives on GitHub Releases. Releases are built from `main` by `.github/workflows/release.yml`.

## Automated flow

```text
Conventional Commits on main
  → Release Please prepares a version PR
  → the version PR is reviewed and merged
  → GitHub creates a vX.Y.Z release
  → CI builds five platform archives and SHA256SUMS
```

Release Please updates `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and `.release-please-manifest.json`.

| Commit | Version change |
| --- | --- |
| `fix:` | Patch |
| `feat:` | Minor |
| `type!:` or `BREAKING CHANGE:` | Major |
| `docs:`, `test:`, `chore:` | No version change by itself |

## Repository setup

The `Release` workflow needs `contents: write` to create a tag, release, and assets. For automatic release-PR creation, add a dedicated Actions secret named `RELEASE_PLEASE_TOKEN` with write access to contents, issues, and pull requests. Without that secret, the workflow can publish a correctly structured release PR after it is merged, but it does not open the next release PR automatically.

## Release files

| Platform | Rust target | Archive |
| --- | --- | --- |
| Linux x86-64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| Windows x86-64 | `x86_64-pc-windows-msvc` | `.zip` |

Each archive contains the executable, project README, and `LICENSE`. `SHA256SUMS` contains checksums for all archives.

## Release check

Before merging a version PR:

- Confirm CI passes.
- Check the generated version and changelog.
- Confirm user-facing documentation reflects the release where needed.

After merge:

- Confirm the `vX.Y.Z` release exists.
- Confirm all five archives and `SHA256SUMS` are attached.
- Download one archive, verify its checksum, and run `tcade --version`.

If a build or upload fails, rerun the failed workflow jobs after fixing the workflow. Do not move an existing version tag; publish a patch release when released source code must change.
