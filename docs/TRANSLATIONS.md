# Documentation and translation policy

[English](TRANSLATIONS.md) · [한국어](ko/TRANSLATIONS.md) · [简体中文](zh-CN/TRANSLATIONS.md)

TERMCADE documentation is English-first and currently supports Korean (`ko`) and Simplified Chinese (`zh-CN`). The layout is designed to accept more locales without moving canonical files.

## Source-of-truth layout

```text
README.md                  English project README
README.ko.md               Korean project README
README.zh-CN.md            Simplified Chinese project README
docs/<NAME>.md             English canonical document
docs/ko/<NAME>.md          Korean translation
docs/zh-CN/<NAME>.md       Simplified Chinese translation
<COMMUNITY>.md             English canonical community policy
docs/<locale>/<COMMUNITY>.md  Informational policy translation
```

English controls whenever a translation and its canonical document disagree. This rule is a fallback for ambiguity, not permission to leave translations outdated.

## Updating documentation

1. Edit the English source first and make its behavior, command names, and links final.
2. Update the Korean and Simplified Chinese files in the same pull request.
3. Preserve headings, code, identifiers, command output, tables, warnings, and link destinations.
4. Translate explanations naturally; do not translate Rust names, CLI subcommands, key names, paths, or version strings.
5. Add or update language navigation at the top of every localized page.
6. Run `./scripts/check.sh`. `scripts/check-docs.sh` rejects missing locale files.

If a contributor cannot update a translation, the pull request must open and link an `area: i18n` follow-up issue. Maintainers decide whether the user impact permits merging with that temporary gap.

## Adding a language

Use a valid BCP 47 language tag, such as `ja` or `pt-BR`.

1. Open an issue defining the locale code, written language name, initial translator, and maintenance plan.
2. Add `README.<locale>.md`.
3. Add `docs/<locale>/` with every file listed by `scripts/check-docs.sh`.
4. Add the locale to the navigation line in every README and document.
5. Extend `scripts/check-docs.sh` so CI requires the new locale.
6. Translate the current canonical content; do not begin from another translation.
7. Request review from a fluent speaker and a maintainer familiar with the technical content.

## Translation style

- Preserve meaning and level of certainty; do not add promises absent from English.
- Prefer established software terminology for the target community.
- Keep examples executable and punctuation inside code unchanged.
- Use accessible language and avoid gendered assumptions.
- Mark an intentionally untranslated product name consistently as `TERMCADE`.
- Record a terminology decision in the translation pull request when it may affect future pages.

Machine translation may be used as a draft, but a human must review technical accuracy, fluency, commands, links, and safety language before merge.
