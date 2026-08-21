# Development guide

[English](DEVELOPMENT.md) · [한국어](ko/DEVELOPMENT.md) · [简体中文](zh-CN/DEVELOPMENT.md)

## Prerequisites

- Rust 1.85 or newer
- Git
- A terminal that supports ANSI escape sequences

Clone the repository and validate the workspace:

```bash
git clone https://github.com/ium-mui/TERMCADE.git
cd TERMCADE
./scripts/check.sh
```

The script performs the same documentation parity, formatting, Clippy, and test checks as CI. Use narrower commands while iterating:

```bash
cargo test casino
cargo test --test app_flows
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
```

## Adding a game

1. Create a pure session type in `src/<game>.rs`. It must not depend on the UI or terminal API.
2. Add its kind to `GameKind` and define identifiers and stages through `GameModule::definition`.
3. Register the module in `GameCatalog::default`; never bypass catalog validation.
4. Add the variant, construction path, accessors, `tick`, and `finish_abandoned` routing to `GameSession`.
5. Connect keys in `App::handle_playing`, leaving rule calculation in the session.
6. Add the play view and control hints in `ui.rs`, using only public read-only accessors.
7. Extend the default-stage table and core user-flow coverage in `tests/app_flows.rs`.
8. Update the English game reference and both translations, then run `./scripts/check.sh`.

## Game-session contract

- State is valid immediately after construction.
- The same seed and inputs produce the same state transitions.
- A finished session never emits a second result or reward.
- `finish_abandoned` returns at most one result for an unfinished session.
- Time comes from an injected clock or explicit tick, not an arbitrary read inside game logic.
- Module tests define the meaning of score, attempts, and accuracy.

## Test strategy

`tests/support/AppHarness` sends keys to a real `App` without opening a terminal. Use it for menu navigation, wagers, replay, and other user flows. Testing only internal methods can miss regressions in key mappings and screen transitions.

For rendering, pass fixed dimensions and a `Vec<u8>` to `Renderer::draw_at`. Test just below the supported minimum, at `64×24`, and on a wide terminal. Layout calculations use saturating arithmetic and must not panic on a small screen.

Keep individual game-rule tests in the corresponding module. Use seeded constructors whenever a random result must be asserted.

## Persistence changes

`history.json` is user data. Prefer `#[serde(default)]` for added fields so old files remain readable. For an incompatible change, increment `HISTORY_SCHEMA_VERSION` and implement an explicit migration first. Never silently reset or overwrite a damaged file.

## Documentation changes

English is canonical. Update the matching files under `docs/ko` and `docs/zh-CN` in the same pull request whenever practical. Follow [TRANSLATIONS.md](TRANSLATIONS.md); CI verifies that every canonical document has both current locale files.

## Definition of done

- Core behavior has a regression test.
- Rendering does not panic on small terminals.
- Persistence failure never produces a false success message or in-memory wallet change.
- New routing passes the `GameSession` and catalog validation paths.
- `./scripts/check.sh` succeeds.
- User-facing behavior, architecture, and translations are updated where affected.
- The branch, commit, and pull request follow [GIT_WORKFLOW.md](GIT_WORKFLOW.md).
