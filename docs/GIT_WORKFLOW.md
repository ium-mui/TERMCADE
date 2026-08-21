# Issue, branch, commit, and pull request workflow

[English](GIT_WORKFLOW.md) · [한국어](ko/GIT_WORKFLOW.md) · [简体中文](zh-CN/GIT_WORKFLOW.md)

This is the normative collaboration workflow for TERMCADE. Small documentation fixes may skip prior issue discussion, but every code change still goes through a pull request.

## 1. Issues

Search open and closed issues before creating one. Use the provided form:

- **Bug report** for reproducible incorrect behavior.
- **Feature request** for a new game or behavior change.
- **Documentation issue** for missing, incorrect, or untranslated content.
- **Support request** for a focused usage or development question.

A triage-ready issue states the user problem, reproduction or acceptance criteria, affected version, and relevant environment. Maintainers may add these labels:

| Label group | Values | Meaning |
| --- | --- | --- |
| Type | `bug`, `enhancement`, `documentation`, `support`, `dependencies` | Nature of the work |
| Status | `needs-triage`, `needs-info`, `accepted`, `blocked` | Current decision state |
| Priority | `priority: critical`, `priority: high`, `priority: normal`, `priority: low` | Scheduling signal, not a promise |
| Difficulty | `good first issue`, `help wanted` | Contributor readiness |
| Area | `area: game`, `area: ui`, `area: persistence`, `area: ci`, `area: i18n` | Main ownership area |

Security reports never belong in public issues; follow [SECURITY.md](../SECURITY.md).

## 2. Branches

`main` is always releasable and accepts changes only through pull requests. Create short-lived branches from the latest `main`:

```bash
git switch main
git pull --ff-only
git switch -c feat/123-memory-timer
```

Use lowercase kebab-case and one of these prefixes:

| Prefix | Use |
| --- | --- |
| `feat/` | New user-facing behavior |
| `fix/` | Bug fix |
| `docs/` | Documentation or translation only |
| `refactor/` | Internal change without behavior change |
| `test/` | Test-only work |
| `perf/` | Performance improvement |
| `build/` | Build or dependency changes |
| `ci/` | Automation changes |
| `chore/` | Maintenance that fits no narrower type |
| `release/` | Reserved for automated release pull requests |

Include the issue number when one exists: `fix/42-history-corruption`. Delete the branch after merge. Long-lived development and personal branches are not part of the official workflow.

## 3. Commits

Commit subjects follow Conventional Commits:

```text
<type>[optional scope][!]: <imperative summary>

[optional body]

[optional footers]
```

Allowed types are `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `build`, `ci`, `chore`, and `revert`. Prefer a stable scope such as `ui`, `history`, `casino`, `snake`, `docs`, or `release` when it adds useful context.

Examples:

```text
feat(maze): add deterministic abyss stage
fix(history): preserve wallet after a failed write
docs(ko): clarify the release checklist
refactor(session)!: replace per-game options with one session enum
```

Rules:

- Write the summary in English, imperative mood, with no final period.
- Keep the subject focused; explain motivation and tradeoffs in the body.
- Reference issues with `Refs: #123` or close them from the pull request with `Closes #123`.
- Mark incompatible changes with `!` and a `BREAKING CHANGE:` footer.
- Never include secrets, generated binaries, unrelated formatting, or private data.

Release Please maps `fix` to a patch release, `feat` to a minor release, and breaking changes to a major release. Other types appear in history but do not normally trigger a version bump.

## 4. Pull requests

Open a draft early for feedback, then mark it ready when all checklist items are true. The pull request title must itself be a valid Conventional Commit because squash merge uses it as the final commit subject.

Each pull request must:

1. Address one logical change and link its issue when applicable.
2. Explain behavior and motivation, not only list edited files.
3. Include tests proportional to risk and exact verification commands.
4. Update English documentation and supported translations for user-visible changes.
5. Pass `branch-name`, `pr-title`, `quality`, and `msrv` CI checks.
6. Resolve every review conversation. When another maintainer is available, receive at least one approval; during the solo-maintainer phase, record a self-review in the pull request and rely on all required checks.
7. Be current with `main` and have no unresolved conflicts.

Maintainers use **squash merge** for normal contributions. Use **rebase merge** only when preserving a deliberately curated series is valuable. Merge commits are disabled. The person merging must ensure the resulting subject is a valid Conventional Commit.

## 5. Protected `main` settings

Repository administrators configure a branch ruleset for `main` with:

- Pull requests required. Use zero required approvals while the project has only one maintainer, then raise the minimum to one as soon as an independent reviewer is available.
- Stale approvals dismissed when new commits are pushed once approvals are enabled.
- Code-owner review required once a valid CODEOWNERS team is established.
- Required checks: `branch-name`, `pr-title`, `quality`, and `msrv` from the `CI` workflow.
- Conversation resolution required.
- Force pushes and branch deletion blocked.
- Linear history required.
- Administrators included; bypass restricted to emergency maintainers.
- Direct pushes blocked.

The solo-maintainer exception never permits bypassing CI. An emergency bypass must be documented in a follow-up issue, reviewed after the incident, and never used merely to avoid waiting for CI.
