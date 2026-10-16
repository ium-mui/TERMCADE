# SOURCE KNOWLEDGE BASE

## OVERVIEW
Flat Rust implementation: application orchestration, terminal rendering, persistence, and pure game sessions.

Scope score: 14 (26 code files, Rust crate boundary, dense symbols/exports, and >20 measured App references).
This file covers implementation contracts; repository commands and navigation live in the parent.

## WHERE TO LOOK
| Change | Location | Integration point |
|--------|----------|-------------------|
| Game registration | `domain.rs` | `GameKind`, `GameModule`, validated `GameCatalog` |
| Active-game lifecycle | `game_session.rs` | Variant, construction, accessors, tick, abandonment |
| Navigation and key handling | `app.rs` | Screen-specific and game-specific dispatch |
| Wagers and card phases | `casino.rs` | Wager input and explicit phase transitions |
| Shared results and math rounds | `round.rs` | `RoundResult`, `RoundSession`, timeout |
| Game mechanics | Named game module | Seeded sessions, rules, score and result |
| Terminal output | `ui.rs` | `Renderer`, cell buffer, diff output |
| Terminal lifetime | `runtime.rs` | `TerminalSession`, event loop, restoration |
| Saved records and wallet | `history.rs` | `HistoryStore`, JSON and memory stores |
| Public library surface | `lib.rs` | Module declarations and convenience re-exports |

## CONVENTIONS
- `App` owns one `GameSession`; extend its enum instead of adding per-game `Option` fields.
- Add a game through catalog definition/validation, session dispatch, App keys, and UI views.
- Extend the default-stage integration table in `tests/app_flows.rs` when adding stages.
- Keep scoring and move legality in each game module, not in App input handlers.
- Session construction must produce valid state; seeded constructors reproduce transitions.
- Finished or abandoned sessions emit a result/reward at most once.
- Route explicit ticks through `App::on_tick` and `GameSession::tick`; keep event-loop timing in runtime.
- Render only through read-only App/session accessors.
- Card phases proceed Covered -> Betting -> Playing using explicit state.
- Persist wallet transactions successfully before changing the in-memory balance.
- New compatible history fields use serde defaults; incompatible schemas need a versioned migration.
- Test game rules beside their implementation; use `AppHarness` for cross-module user flows.
- Test rendering with `Renderer::draw_at`, a byte buffer, and tiny, 64x24, and wide sizes.

## ANTI-PATTERNS
- Game modules importing terminal APIs, UI, or the concrete file store.
- UI code mutating game state or interpreting key input.
- Multiple simultaneous game-session fields or combinable card-phase booleans.
- Bypassing catalog validation or rewarding a finished session twice.
- Silently replacing corrupt history, resetting user data, or reporting failed persistence as success.
- Terminal-mode ownership outside `TerminalSession`.
