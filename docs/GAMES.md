# Games and stages

[English](GAMES.md) · [한국어](ko/GAMES.md) · [简体中文](zh-CN/GAMES.md)

## Arcade games

| Game | Stage | Difficulty difference | Primary controls |
| --- | --- | --- | --- |
| Minesweeper | `beginner` | 9×9, 10 mines | Move, `Enter` reveal, `F` flag |
| Minesweeper | `intermediate` | 16×12, 30 mines | Move, `Enter` reveal, `F` flag |
| Minesweeper | `expert` | 24×16, 70 mines | Move, `Enter` reveal, `F` flag |
| Connect Four | `easy` | AI chooses a valid column randomly | Left/right, `Enter` drop |
| Connect Four | `normal` | AI detects immediate wins and blocks | Left/right, `Enter` drop |
| Connect Four | `hard` | AI searches five plies with alpha-beta pruning | Left/right, `Enter` drop |
| Memory Match | `small` | 4×3, 6 pairs | Move, `Enter` reveal/confirm |
| Memory Match | `classic` | 4×4, 8 pairs | Move, `Enter` reveal/confirm |
| Memory Match | `grand` | 6×4, 12 pairs | Move, `Enter` reveal/confirm |
| Maze | `alley` | 15×9 perfect maze | Arrow keys or `WASD` |
| Maze | `labyrinth` | 25×13 perfect maze | Arrow keys or `WASD` |
| Maze | `abyss` | 35×17 perfect maze | Arrow keys or `WASD` |

The first Minesweeper reveal is always safe. Mismatched Memory Match cards remain visible until the player acknowledges them with `Enter`. Every generated maze has a path between its start and exit, and the same seed produces the same layout.

## Design rules

- Stages differ in rules, board size, speed, or AI behavior—not only in name.
- Random games expose seeded constructors for reproducible failures.
- Pressing `Esc` or `Q` before completion records one abandoned result.
- New stages are rendered automatically at the minimum `64×24` size and a wide `120×40` size.
- Arcade games remain separate from the casino wallet.

## Extended stages for existing games

| Game | Stage | Difference |
| --- | --- | --- |
| Snake | `relaxed` | 230 ms movement interval |
| Snake | `classic-1` | 160 ms movement interval |
| Snake | `turbo` | 95 ms movement interval |
| Sudoku | `easy` | Target of 46 clues |
| Sudoku | `classic-1` | Target of 40 clues |
| Sudoku | `hard` | Target of 32 clues |

Sudoku generates only uniquely solvable puzzles. If the target clue count cannot be reached safely, the generator keeps extra clues to preserve a unique solution.
