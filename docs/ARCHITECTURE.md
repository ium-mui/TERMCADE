# TERMCADE architecture

[English](ARCHITECTURE.md) · [한국어](ko/ARCHITECTURE.md) · [简体中文](zh-CN/ARCHITECTURE.md)

This document defines the boundaries that keep TERMCADE maintainable as new games and features are added. Game rules, application flow, terminal I/O, rendering, and persistence must remain separate.

## Dependency direction

```text
main
 └─ runtime ── terminal / input / cleanup
      ├─ App ── navigation / input dispatch / result orchestration
      │   ├─ GameCatalog
      │   ├─ GameSession ── one active game session
      │   ├─ CasinoState ── wager and card-hand phases
      │   └─ HistoryStore
      └─ Renderer ── read-only App projection

game modules ── rules, seeded randomness, RoundResult
```

Dependencies point downward only. Individual game modules must not know about `crossterm`, `ui`, or the real file store. The renderer reads `App` state but never mutates it.

## Module responsibilities

| Module | Responsibility |
| --- | --- |
| `runtime.rs` | Raw mode and alternate-screen lifetime, event loop, and terminal restoration after errors |
| `app.rs` | Screen transitions, key routing, game start and finish, result and wallet coordination |
| `game_session.rs` | The single active game session and its common lifecycle |
| `casino.rs` | Wager input, total wager, and pre-reveal card phases |
| `domain.rs` | Game and stage identifiers, definitions, registration, and validation |
| `history.rs` | History and wallet interfaces plus JSON and in-memory implementations |
| `round.rs` | Common round results and the mental-math round engine |
| `ui.rs` | Fixed-size buffer rendering and terminal diff output |
| Game modules | Rules, state, seeded randomness, and result calculation for one game |

## Required invariants

1. `App` has at most one active `GameSession`. Do not add separate game-specific `Option` fields.
2. Card games move through `Covered → Betting → Playing` in order. Do not represent these phases as combinable booleans.
3. In-memory wallet state changes only after `HistoryStore` successfully persists the transaction.
4. The UI does not mutate state. `App` handles input, and the game session handles rules.
5. Random games provide a seeded construction path so tests are reproducible.
6. `GameCatalog::try_from_modules` or `try_register` rejects registration errors before play begins.
7. `TerminalSession` is the only owner of terminal-mode lifetime.

## Main execution flow

```text
KeyEvent
  → App::handle_key
  → screen/game-kind dispatch
  → game session mutation
  → optional RoundResult
  → HistoryStore persistence
  → Renderer::draw (read-only)
```

For time-based games, `App::on_tick` forwards ticks to `GameSession::tick`. Event-loop timing and terminal APIs do not belong inside game modules.

## Reliability boundaries

- History JSON is written to a temporary file, synchronized, and then moved over the destination.
- A damaged history file produces an error and is never silently overwritten.
- `TerminalSession` attempts restoration after normal exit, early errors, and unwinding.
- `Renderer::draw_at` allows layout checks at multiple sizes without a real TTY.
- The catalog rejects empty IDs, duplicate game or stage IDs, and mismatched game kinds.

## Deciding where code belongs

Promote a state transition to a shared type when the same transition is repeated in two or more game modules. Keep rules used by only one game inside that module. If `App` starts calculating game rules or `ui.rs` starts interpreting input, the boundary is being crossed.
