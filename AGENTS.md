# PROJECT KNOWLEDGE BASE

**Generated:** 2026-10-16
**Commit:** 48eda2e
**Branch:** main

## OVERVIEW
TERMCADE is a terminal arcade collection built as one Rust 2024 package, requiring Rust 1.85+.
The package is `cli-game`, its library is `cli_game`, and its executable is `tcade`.
Rendering uses a custom crossterm cell buffer rather than a separate TUI framework.

## STRUCTURE
```text
cli-game/
|-- src/                 # Flat implementation; see src/AGENTS.md
|-- tests/               # Real-App integration flows and shared harness
|-- scripts/             # Canonical quality and documentation checks
|-- docs/                # Architecture, development, game and release references
|   |-- ko/              # Korean documentation
|   `-- zh-CN/           # Simplified Chinese documentation
|-- .github/workflows/   # Quality/MSRV checks and release automation
`-- Cargo.toml           # Package, binary identity, MSRV and unsafe-code policy
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Understand ownership | `docs/ARCHITECTURE.md` | Dependency direction and required invariants |
| Add a game | `docs/DEVELOPMENT.md`, `src/AGENTS.md` | Full integration checklist |
| Change CLI routes | `src/cli.rs`, `src/main.rs` | Resolve catalog-backed game/stage paths |
| Test user flows | `tests/app_flows.rs`, `tests/support/mod.rs` | Keys drive a real App without a TTY |
| Change saved data | `src/history.rs` | Versioned history and wallet persistence |
| Update controls/game lists | `docs/GAMES.md`, localized counterparts | Match current user-visible behavior |
| Validate changes | `scripts/check.sh` | Same quality sequence as CI |
| Change release process | `docs/RELEASING.md`, `.github/workflows/release.yml` | Automated target builds |
| Prepare contributions | `CONTRIBUTING.md`, `docs/GIT_WORKFLOW.md` | Branch and PR-title conventions |

## CODE MAP
LSP outlines and references confirm the core types; App has more than 20 references.
Other reference counts were not measured.

| Symbol | Type | Location | Refs | Role |
|--------|------|----------|------|------|
| `App` | struct | `src/app.rs` | >20 | Screens, input dispatch, result orchestration |
| `GameCatalog` | struct | `src/domain.rs` | unmeasured | Validated game and stage registry |
| `GameSession` | enum | `src/game_session.rs` | unmeasured | Single active-game lifecycle |
| `run_tui` | function | `src/runtime.rs` | unmeasured | Interactive loop and terminal cleanup |
| `HistoryStore` | trait | `src/history.rs` | unmeasured | Records and wallet abstraction |
| `Renderer` | struct | `src/ui.rs` | unmeasured | Read-only buffered terminal projection |

## CONVENTIONS
- Source modules live directly in `src/`; this is not a multi-crate workspace.
- Documentation is maintained in English, Korean, and Simplified Chinese.
- Root READMEs use `README.ko.md` and `README.zh-CN.md`; detailed translations live in `docs/`.
- Rust files use four-space indentation; `.editorconfig` otherwise defaults to two.
- CI checks branch prefixes and Conventional Commit PR titles.

## ANTI-PATTERNS (THIS PROJECT)
- Unsafe Rust is forbidden by Cargo lint configuration.
- Do not commit secrets, generated binaries, or unrelated changes.
- Do not move released version tags.

## UNIQUE STYLES
- Public modules and convenience re-exports are collected in `src/lib.rs`.
- Game-rule tests live in module test blocks; user flows live in `tests/app_flows.rs`.
- Application messages include Korean text; preserve existing text conventions when editing.

## COMMANDS
```sh
cargo run --                         # Start the arcade
cargo run -- snake classic-1         # Open a catalog-backed stage
./scripts/check.sh                   # Docs, fmt, Clippy -D warnings, all tests
./scripts/check-docs.sh              # Required localized document inventory
cargo test --test app_flows          # Real-App integration coverage
cargo test casino                   # Focused module tests
cargo check --locked --all-targets --all-features # CI also checks on Rust 1.85.0
```

## NOTES
- The docs checker verifies required files exist, not translation parity or link correctness.
- History defaults to the platform data directory, with `.cli-game/history.json` as fallback.
- `src/app.rs` and `src/ui.rs` are orchestration/rendering hotspots; inspect targeted symbols.
- Follow source-level guidance for session, wallet, rendering, and persistence invariants.
