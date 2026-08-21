# Language support

[English](TRANSLATIONS.md) · [한국어](ko/TRANSLATIONS.md) · [简体中文](zh-CN/TRANSLATIONS.md)

TERMCADE aims to make its project documentation usable in multiple languages. English, Korean (`ko`), and Simplified Chinese (`zh-CN`) are currently supported, and the structure can grow to include more languages.

## Current layout

```text
README.md                  English edition
README.ko.md               Korean edition
README.zh-CN.md            Simplified Chinese edition
docs/<NAME>.md             English edition
docs/ko/<NAME>.md          Korean edition
docs/zh-CN/<NAME>.md       Simplified Chinese edition
```

The root placement of the English files follows common GitHub repository conventions. It does not make English the policy authority. All supported editions should describe the same commands and behavior. If they disagree, check the code, tests, and current release rather than assuming one language wins.

## Updating documentation

- Start a correction or improvement in any supported language.
- Update the related editions in the same pull request when possible.
- Keep commands, Rust identifiers, paths, key names, and version strings unchanged.
- Translate explanations naturally instead of copying sentence structure.
- If you cannot review another language accurately, mark it for language review in the pull request. A separate tracking issue is not required for every small gap.
- Run `./scripts/check.sh`; CI confirms that each supported language has the expected document set.

## Adding a language

Use a BCP 47 language tag such as `ja` or `pt-BR`.

1. Add `README.<locale>.md` and `docs/<locale>/` with the same project documents.
2. Add the language to the navigation line on each edition.
3. Extend `scripts/check-docs.sh` to check the new document set.
4. Open a pull request describing which pages were reviewed and which still need help.

Machine translation can help produce a draft, but commands and technical meaning must be checked before merge.
