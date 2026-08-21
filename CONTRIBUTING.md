# Contributing to TERMCADE

[English](CONTRIBUTING.md) · [한국어](docs/ko/CONTRIBUTING.md) · [简体中文](docs/zh-CN/CONTRIBUTING.md)

Thank you for helping improve TERMCADE. Contributions of code, tests, documentation, translations, bug reports, and design feedback are welcome.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md). Do not use public issues for vulnerabilities; follow the private process in [SECURITY.md](SECURITY.md).

## Before starting

1. Search existing issues and pull requests.
2. Open a feature request before implementing a new game, dependency, persistence format, public interface, or large architectural change.
3. Wait for an `accepted` decision before investing in substantial work. Maintainer discussion defines scope but does not guarantee a merge.
4. Small bug fixes, tests, and documentation corrections may go directly to a pull request when the intent is clear.

## Development workflow

Fork the repository, create a branch from current `main`, and keep it focused:

```bash
git clone https://github.com/<your-account>/TERMCADE.git
cd TERMCADE
git remote add upstream https://github.com/ium-mui/TERMCADE.git
git fetch upstream
git switch -c fix/42-short-description upstream/main
```

Rust 1.85 or newer is required. Run the full local gate before submitting:

```bash
./scripts/check.sh
```

See the [development guide](docs/DEVELOPMENT.md) for architecture, tests, persistence safety, and the new-game checklist.

## Change requirements

- Preserve the boundaries in [ARCHITECTURE.md](docs/ARCHITECTURE.md).
- Add regression tests for fixes and behavior tests for features.
- Keep random behavior reproducible through seeded construction paths.
- Treat the history file as durable user data and maintain backward compatibility.
- Keep `unsafe` code out of the project.
- Update English documentation and the Korean and Simplified Chinese translations for user-visible changes.
- Avoid unrelated formatting, dependency upgrades, generated artifacts, and drive-by refactors.

## Commits and pull requests

Branches, commits, and pull requests follow [GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md). In short:

- Use a prefixed kebab-case branch such as `feat/123-new-game`.
- Write English Conventional Commit subjects such as `fix(ui): avoid clipping narrow boards`.
- Make the pull request title a Conventional Commit because it becomes the squash commit.
- Explain the problem, approach, risks, and exact verification performed.
- Link the issue with `Closes #123` when the pull request fully resolves it.
- Respond to review comments and resolve conversations only after the concern is addressed.

Maintainers may request a smaller scope, additional tests, documentation, or design discussion. A contribution may be declined when it conflicts with project direction, carries disproportionate maintenance cost, lacks a safe migration, or cannot be supported reliably.

## Documentation and translations

English is canonical. Localized documents are maintained under `docs/ko` and `docs/zh-CN`, with localized project READMEs at the repository root. Follow [TRANSLATIONS.md](docs/TRANSLATIONS.md) when editing or adding a locale.

A translation pull request should identify the source revision it reviewed, preserve code and links exactly, and receive review for both language quality and technical accuracy.

## Review and merge

All required CI jobs must pass. At least one maintainer approval and resolution of all review conversations are required. Maintainers normally squash merge and may edit the final title so Release Please can calculate the correct version.

Contributors retain copyright in their work and license submitted contributions under the repository's [MIT License](LICENSE). No separate contributor license agreement is required.
