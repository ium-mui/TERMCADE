# Release and deployment process

[English](RELEASING.md) · [한국어](ko/RELEASING.md) · [简体中文](zh-CN/RELEASING.md)

TERMCADE is a CLI application, so deployment means publishing versioned executable archives to GitHub Releases. `main` is the only release source.

## Release model

The `Release` workflow runs whenever `main` changes:

```text
Conventional commits on main
  → Release Please updates one release PR
  → maintainer reviews and merges the release PR
  → vX.Y.Z tag and GitHub Release are created
  → native binaries build for five targets
  → archives and SHA256SUMS are attached to the release
```

Release Please updates `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and `.release-please-manifest.json`. Version bumps follow Semantic Versioning:

| Commit | Version effect | Example |
| --- | --- | --- |
| `fix:` | Patch | `1.2.3 → 1.2.4` |
| `feat:` | Minor | `1.2.3 → 1.3.0` |
| `type!:` or `BREAKING CHANGE:` | Major | `1.2.3 → 2.0.0` |
| `docs:`, `test:`, `chore:` | No bump alone | Included when another releasable change exists |

Do not edit the automated release PR's version or changelog without a documented reason.

## One-time repository setup

1. Enable GitHub Actions and allow actions to create pull requests in **Settings → Actions → General → Workflow permissions**.
2. Create a fine-grained bot token with repository `Contents: write`, `Issues: write`, `Pull requests: write`, and `Workflows: write` permissions.
3. Store it as the Actions secret `RELEASE_PLEASE_TOKEN`.
4. Configure the protected `main` rules in [GIT_WORKFLOW.md](GIT_WORKFLOW.md).
5. Create the labels listed in the Git workflow, plus Release Please labels `autorelease: pending`, `autorelease: tagged`, and `autorelease: snapshot` if automation does not create them automatically.

The workflow falls back to `GITHUB_TOKEN`, but pull requests created by that token do not start other workflows. The dedicated token is therefore required for fully automatic release-PR CI.

## Supported artifacts

| Runner | Rust target | Archive |
| --- | --- | --- |
| Ubuntu x86-64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Ubuntu ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| Windows x86-64 | `x86_64-pc-windows-msvc` | `.zip` |

Every archive contains the `tcade` executable, canonical README, and license. `SHA256SUMS` covers every archive.

## Maintainer release checklist

Before merging the release pull request:

- Confirm every included pull request passed CI and review.
- Read the generated changelog as user-facing release notes.
- Confirm `Cargo.toml` and the manifest contain the same version.
- Check that breaking changes and migrations are explicit.
- Confirm documentation and translations describe the release.
- Merge with the generated Conventional Commit title.

After merge:

- Confirm the `vX.Y.Z` tag points to the release commit on `main`.
- Confirm all five archives and `SHA256SUMS` are attached.
- Download one artifact, verify its checksum, and run `tcade --version` plus a smoke game.
- If any asset build fails, rerun the failed jobs. The upload step safely replaces assets for the same release.
- Announce the release only after all artifacts are present.

## Rollback and hotfixes

GitHub Releases and published tags are immutable historical records. Do not move or delete a public version to conceal a problem.

For a non-security defect, revert or fix it through a normal pull request using `fix:`. Merge the new release PR to publish a patch. For a security defect, coordinate privately under [SECURITY.md](../SECURITY.md), prepare the fix and advisory, then publish a patch release.

If an attached binary is corrupt while source and tag are correct, rerun the asset jobs and verify checksums before announcing completion. If the source itself is wrong, publish a new patch version instead of replacing history.

## Manual recovery

`workflow_dispatch` may rerun Release Please when a GitHub event was missed. A maintainer may also rerun failed jobs from the Actions UI. Creating tags or releases manually is a last resort and must preserve the `vX.Y.Z` format, point to a commit contained in `main`, use the generated changelog, and include the same five artifacts and checksums.
