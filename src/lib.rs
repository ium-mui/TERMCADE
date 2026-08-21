pub mod app;
pub mod arithmetic;
pub mod blackjack;
pub mod breakout;
pub mod casino;
pub mod cli;
pub mod connect_four;
pub mod domain;
pub mod game_2048;
pub mod game_session;
pub mod history;
pub mod holdem;
pub mod maze;
pub mod memory_match;
pub mod minesweeper;
pub mod roulette;
pub mod round;
pub mod runtime;
pub mod slots;
pub mod snake;
pub mod sudoku;
pub mod tictactoe;
pub mod typing;
pub mod ui;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use app::{App, Screen};
pub use blackjack::BlackjackSession;
pub use breakout::BreakoutSession;
pub use cli::{Cli, CliRoute, resolve_route};
pub use connect_four::{
    ConnectFourCell, ConnectFourDifficulty, ConnectFourOutcome, ConnectFourSession,
};
pub use domain::{
    BreakoutGame, CatalogError, ConnectFourGame, GamblingGame, GameCatalog, GameDefinition, GameId,
    GameKind, GameModule, MazeGame, MemoryMatchGame, MinesweeperGame, SnakeGame, StageDefinition,
    StageId, SudokuGame, TicTacToeGame, TwentyFortyEightGame,
};
pub use game_2048::Game2048Session;
pub use game_session::GameSession;
pub use history::{HistoryStore, JsonHistoryStore, MemoryHistoryStore, PlayRecord};
pub use holdem::{
    HoldemAction, HoldemAiAction, HoldemDifficulty, HoldemOutcome, HoldemSession, HoldemStreet,
};
pub use maze::{MazeDifficulty, MazeSession};
pub use memory_match::{MemoryCard, MemoryCardState, MemoryDifficulty, MemoryMatchSession};
pub use minesweeper::{
    CellState as MinesweeperCellState, MinesweeperDifficulty, MinesweeperOutcome,
    MinesweeperSession,
};
pub use roulette::{RouletteColor, RouletteSession};
pub use round::{RoundResult, RoundSession, RoundStatus};
pub use runtime::run_tui;
pub use slots::{SlotSymbol, SlotsSession};
pub use snake::{Direction, Point, SnakePace, SnakeSession};
pub use sudoku::{SudokuDifficulty, SudokuSession};
pub use tictactoe::{Cell as TicTacToeCell, TicTacToeSession};
pub use typing::{TypingPracticeSession, TypingSubmission};
