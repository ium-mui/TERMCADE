<div align="center">

<img src="assets/termcade-logo.png" alt="TERMCADE" width="900">

### A pocket arcade for your terminal

A collection of terminal games built in Rust with a lightweight `crossterm` renderer.
Play a quick round, keep your history, and grow a shared in-game wallet.

[English](README.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

Project documentation is maintained in all three languages as parallel editions, with room for more languages.

[![CI](https://github.com/ium-mui/TERMCADE/actions/workflows/ci.yml/badge.svg)](https://github.com/ium-mui/TERMCADE/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/ium-mui/TERMCADE)](https://github.com/ium-mui/TERMCADE/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[Install](#install) · [Games](#games) · [Controls](#controls) · [Documentation](#documentation) · [Contributing](#contributing)

</div>

## Install

Download the archive for your platform from the [latest GitHub Release](https://github.com/ium-mui/TERMCADE/releases/latest), extract it, and place `tcade` (or `tcade.exe`) somewhere on your `PATH`. Every release includes SHA-256 checksums.

Supported release targets are:

- Linux x86-64 and ARM64
- macOS Intel and Apple Silicon
- Windows x86-64

If Rust 1.85 or newer is installed, you can build directly from GitHub:

```bash
cargo install --git https://github.com/ium-mui/TERMCADE.git --locked
tcade
```

To work from a local clone:

```bash
git clone https://github.com/ium-mui/TERMCADE.git
cd TERMCADE
cargo install --path . --locked
tcade
```

Run without installing during development:

```bash
cargo run --
cargo run -- math
cargo run -- snake classic-1
```

## Games

| Game | Example command | Objective |
| --- | --- | --- |
| Mental math | `math` | Solve addition, subtraction, multiplication, and division quickly |
| Snake | `snake classic-1` | Survive without hitting a wall or your own body |
| Tic-tac-toe | `tictactoe classic-1` | Beat the computer on a 3×3 board |
| 2048 | `2048 classic-1` | Combine tiles to create 2048 |
| Sudoku | `sudoku classic-1` | Complete a uniquely solvable number puzzle |
| Minesweeper | `minesweeper beginner` | Reveal every safe cell while avoiding mines |
| Connect Four | `connect-four easy` | Connect four discs before the difficulty-aware AI |
| Memory Match | `memory small` | Remember card positions and find every pair |
| Maze | `maze alley` | Find the exit from a newly generated maze |
| Casino | `gambling` | Play blackjack, roulette, slots, AI hold'em, and typing mine |
| Breakout | `breakout classic-1` | Keep the ball in play and clear the bricks |

Launch `tcade` to choose a cabinet and stage interactively, or provide them directly:

```bash
tcade gambling blackjack-1
tcade gambling holdem-1
tcade minesweeper expert
tcade connect-four hard
tcade memory grand
tcade maze abyss
tcade snake turbo
tcade sudoku hard
```

## Controls

- Arrow keys, or `W` `A` `S` `D` where supported: move
- `Enter`: select, submit, reveal, or perform the primary action
- `Space`: game-specific secondary action
- `Backspace`: erase a Sudoku digit
- `Esc` or `Q`: go back or leave a game
- `Ctrl+C`: exit immediately

Minesweeper uses `F` to place a flag. Connect Four uses left and right to choose a column. Memory Match uses `Enter` to reveal cards and acknowledge a mismatch. Casino tables display their exact betting and action keys in the game view.

## Data and terminal requirements

Results, history, and wallet balance are stored as JSON in the operating system's application data directory. TERMCADE never silently overwrites a damaged history file. A terminal size of at least `64×24` is recommended so detailed boards are not clipped.

## Documentation

- [Documentation index](docs/README.md)
- [Game and stage reference](docs/GAMES.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Development guide](docs/DEVELOPMENT.md)
- [Issue, branch, commit, and pull request workflow](docs/GIT_WORKFLOW.md)
- [Release and deployment process](docs/RELEASING.md)
- [Language support](docs/TRANSLATIONS.md)

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), discuss substantial changes in an issue, and run `./scripts/check.sh` before opening a pull request. Documentation and pull request discussions may use English, Korean, or Simplified Chinese.

For the small set of security-sensitive bugs, see [SECURITY.md](SECURITY.md).

## License

TERMCADE is available under the [MIT License](LICENSE).
