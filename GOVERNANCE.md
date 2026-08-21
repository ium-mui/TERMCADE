# Project governance

[English](GOVERNANCE.md) · [한국어](docs/ko/GOVERNANCE.md) · [简体中文](docs/zh-CN/GOVERNANCE.md)

TERMCADE uses a lightweight maintainer-led governance model. The objective is transparent decisions, safe releases, and a low barrier to contribution.

## Roles

### Contributors

Anyone who participates through issues, code, documentation, translation, review, testing, or community help is a contributor. Contributors may propose changes and take part in public design discussions.

### Reviewers

Reviewers are trusted contributors with demonstrated knowledge in one or more areas. They may triage issues and provide formal reviews, but do not merge or publish unless they are also maintainers.

### Maintainers

Maintainers have repository write access and are responsible for scope decisions, reviews, merges, security response, releases, automation, and Code of Conduct enforcement. They must use least privilege, enable strong account security, disclose conflicts of interest, and follow the same protected-branch rules as contributors.

## Becoming or leaving a role

A maintainer may nominate a contributor as reviewer or maintainer based on sustained constructive participation, sound technical judgment, reliable reviews, and conduct aligned with project values. Existing maintainers discuss the nomination privately where personal information is involved and record an accepted role change publicly.

A person may step down at any time. Maintainer access may be removed after prolonged inactivity, loss of account security, repeated policy violations, or inability to perform the role safely. Whenever privacy and safety permit, the project records role changes and the operational reason.

## Decisions

Routine changes are decided through pull request review. Significant changes—new architectural boundaries, incompatible storage formats, removal of a supported platform or locale, licensing changes, and governance changes—require a public issue or proposal before implementation.

The project seeks rough consensus: address substantive objections and prefer solutions the responsible maintainers and active contributors can support. If consensus is not reached in a reasonable period, maintainers make the decision and document the rationale, alternatives, and dissenting concerns. A maintainer with a material conflict of interest recuses themselves.

## Releases and authority

Only maintainers may merge release pull requests, manage repository secrets, publish advisories, or bypass rules during an emergency. Releases follow [RELEASING.md](docs/RELEASING.md), and bypasses require a follow-up audit. No single maintainer should approve and publish a high-risk security or governance change when another qualified maintainer is available.

## Amendments

Governance changes use a pull request labeled `governance`, with at least one public review period and maintainer approval. The pull request explains the motivation and migration impact. Editorial fixes may use the normal documentation workflow.
