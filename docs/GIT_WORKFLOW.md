# Git workflow

[English](GIT_WORKFLOW.md) · [한국어](ko/GIT_WORKFLOW.md) · [简体中文](zh-CN/GIT_WORKFLOW.md)

These rules keep TERMCADE changes easy to review and release. They cover development work only.

## Issues

Use an issue when a change needs discussion or tracking:

- **Bug report** for reproducible incorrect behavior.
- **Feature request** for a new game or behavior.
- **Documentation issue** for missing, incorrect, or outdated content.

Small, obvious fixes may go directly to a pull request. Include reproduction steps for bugs and a clear expected result for features. Use the short [security guide](../SECURITY.md) only for security-sensitive bugs.

## Branches

Create a short-lived branch from the latest `main`:

```bash
git switch main
git pull --ff-only
git switch -c feat/new-game
```

Use lowercase ASCII kebab-case because branch names are consumed by tooling. Common prefixes are:

| Prefix | Use |
| --- | --- |
| `feat/` | New behavior |
| `fix/` | Bug fix |
| `docs/` | Documentation |
| `refactor/` | Internal restructuring |
| `test/` | Test-only change |
| `build/` | Dependencies or packaging |
| `ci/` | GitHub Actions and automation |
| `chore/` | Other maintenance |

Add an issue number when useful, for example `fix/42-history-write`. Delete the branch after merge.

## Commits

Use Conventional Commits:

```text
<type>[optional scope][!]: <summary>
```

Supported types are `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `build`, `ci`, `chore`, and `revert`. Keep the type and optional scope in ASCII for release tooling, and write a clear, concise summary.

Examples:

```text
feat(maze): add deterministic abyss stage
fix(history): preserve wallet after a failed write
docs: clarify minesweeper controls
```

Use `!` or a `BREAKING CHANGE:` footer for an incompatible change. Never commit secrets, generated release archives, or unrelated formatting.

## Pull requests

- Keep one logical change per pull request.
- Use a Conventional Commit title; squash merge uses it as the final commit.
- Explain the purpose, important implementation choices, and verification performed.
- Add tests for changed behavior and update relevant user-facing documentation.
- Run `./scripts/check.sh` locally.
- Resolve review conversations and update the branch when `main` has moved.

Use precise code identifiers and commands so the change is easy to verify.

## Protected `main`

Changes reach `main` through pull requests. The required checks are `branch-name`, `pr-title`, `quality`, and `msrv`. The branch must be current with `main`, review conversations must be resolved, linear history is required, and force pushes or branch deletion are blocked. Normal pull requests use squash merge.
