# Security policy

[English](SECURITY.md) · [한국어](docs/ko/SECURITY.md) · [简体中文](docs/zh-CN/SECURITY.md)

## Supported versions

TERMCADE provides security fixes for the latest released minor line. Development snapshots and older versions may be used to reproduce a report, but fixes are released only from current `main`.

| Version | Supported |
| --- | --- |
| Latest release | Yes |
| Older releases | No |
| Unreleased `main` | Best effort |

## Reporting a vulnerability

Do not open a public issue. Use GitHub's [private vulnerability reporting](https://github.com/ium-mui/TERMCADE/security/advisories/new) and include:

- Affected version, commit, operating system, and terminal.
- Vulnerability class and realistic impact.
- Minimal reproduction steps or proof of concept.
- Preconditions and whether untrusted input is required.
- Suggested mitigation, if known.
- Whether the report or exploit has been disclosed elsewhere.

Remove unrelated personal data and never test against systems or users you do not own or have permission to assess.

## Response process

Maintainers aim to acknowledge a complete report within 7 days and provide an initial assessment within 14 days. These are targets, not guarantees. The team will validate impact, coordinate a fix and advisory privately, and agree on a disclosure date with the reporter when possible.

If accepted, the fix follows the normal review and CI standards in a private fork or restricted branch. A security release receives a new SemVer patch or larger version; existing tags and releases are never rewritten. Credit is offered unless the reporter prefers anonymity.

If declined, maintainers explain why the behavior is not considered a vulnerability or why it is outside the project threat model. Good-faith research that follows this policy will not be treated as hostile.

## Security boundaries

TERMCADE is a local terminal game. Important security-sensitive surfaces include history-file parsing and replacement, terminal state restoration, command-line and keyboard input, archive integrity, release automation, and third-party dependencies. Reports about cheating in the local in-game wallet without a broader integrity or code-execution impact are normally treated as bugs rather than vulnerabilities.
