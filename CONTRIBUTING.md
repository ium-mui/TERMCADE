# Contributing to TERMCADE

[English](CONTRIBUTING.md) · [한국어](docs/ko/CONTRIBUTING.md) · [简体中文](docs/zh-CN/CONTRIBUTING.md)

Contributions to the game, tests, and documentation are welcome.

## Before starting

- Search existing issues and pull requests.
- Open an issue before adding a game, changing saved-data compatibility, or making a large architectural change.
- Small fixes and documentation corrections may go directly to a pull request.

## Development workflow

Create a focused branch from the latest `main`:

```bash
git clone https://github.com/<your-account>/TERMCADE.git
cd TERMCADE
git remote add upstream https://github.com/ium-mui/TERMCADE.git
git fetch upstream
git switch -c fix/42-short-description upstream/main
```

Rust 1.85 or newer is required. Run the same checks as CI before submitting:

```bash
./scripts/check.sh
```

See the [development guide](docs/DEVELOPMENT.md) for architecture, tests, persistence, and the new-game checklist.

## Change requirements

- Keep game rules independent from terminal rendering.
- Add tests for new behavior and bug fixes.
- Keep seeded game behavior reproducible.
- Preserve compatibility with existing history files, or add an explicit migration.
- Do not add `unsafe` code, secrets, generated binaries, or unrelated changes.
- Update user-facing documentation when behavior changes.

## Branches, commits, and pull requests

Follow [GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md). The short version is:

- Use a prefixed branch such as `feat/new-game`, `fix/history-write`, or `docs/update-controls`.
- Use a Conventional Commit type such as `feat:`, `fix:`, or `docs:` with a clear summary.
- Make the pull request title a Conventional Commit because it becomes the squash commit.
- Explain what changed, why it changed, and how it was verified.
- Keep one logical change per pull request.

## Merge requirements

Required CI checks must pass and review conversations must be resolved. Pull requests are normally squash merged. Submitted work is licensed under the repository's [MIT License](LICENSE); no separate contributor license agreement is required.
