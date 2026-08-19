pub mod app;
pub mod arithmetic;
pub mod blackjack;
pub mod breakout;
pub mod cli;
pub mod domain;
pub mod game_2048;
pub mod history;
pub mod roulette;
pub mod round;
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
pub use domain::{
    BreakoutGame, GamblingGame, GameCatalog, GameDefinition, GameId, GameKind, GameModule,
    SnakeGame, StageDefinition, StageId, SudokuGame, TicTacToeGame, TwentyFortyEightGame,
};
pub use game_2048::Game2048Session;
pub use history::{HistoryStore, JsonHistoryStore, MemoryHistoryStore, PlayRecord};
pub use roulette::{RouletteColor, RouletteSession};
pub use round::{RoundResult, RoundSession, RoundStatus};
pub use slots::{SlotSymbol, SlotsSession};
pub use snake::{Direction, Point, SnakeSession};
pub use sudoku::SudokuSession;
pub use tictactoe::{Cell as TicTacToeCell, TicTacToeSession};
pub use typing::{TypingPracticeSession, TypingSubmission};
