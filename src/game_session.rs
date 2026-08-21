use crate::blackjack::BlackjackSession;
use crate::breakout::BreakoutSession;
use crate::connect_four::ConnectFourSession;
use crate::domain::{GameId, GameKind, StageDefinition, StageId};
use crate::game_2048::Game2048Session;
use crate::holdem::HoldemSession;
use crate::maze::MazeSession;
use crate::memory_match::MemoryMatchSession;
use crate::minesweeper::MinesweeperSession;
use crate::roulette::RouletteSession;
use crate::round::{RoundResult, RoundSession};
use crate::slots::SlotsSession;
use crate::snake::SnakeSession;
use crate::sudoku::SudokuSession;
use crate::tictactoe::TicTacToeSession;
use crate::typing::TypingPracticeSession;

/// The single game session owned by the application.
///
/// Keeping sessions in one enum prevents impossible states such as Blackjack
/// and Roulette being active at the same time. Add new game session types here
/// first; the compiler will then point to every dispatch site that needs work.
pub enum GameSession {
    Arithmetic(RoundSession),
    Snake(SnakeSession),
    TicTacToe(TicTacToeSession),
    TwentyFortyEight(Game2048Session),
    Sudoku(SudokuSession),
    Blackjack(BlackjackSession),
    Roulette(RouletteSession),
    Slots(SlotsSession),
    Holdem(HoldemSession),
    TypingPractice(TypingPracticeSession),
    Breakout(BreakoutSession),
    Minesweeper(MinesweeperSession),
    ConnectFour(ConnectFourSession),
    MemoryMatch(MemoryMatchSession),
    Maze(MazeSession),
}

impl GameSession {
    pub fn start(game_id: GameId, stage_id: StageId, stage: StageDefinition) -> Option<Self> {
        Some(match stage.game_kind {
            GameKind::Arithmetic => Self::Arithmetic(RoundSession::new(game_id, stage_id, stage)),
            GameKind::Snake => Self::Snake(SnakeSession::new(game_id, stage_id)),
            GameKind::TicTacToe => Self::TicTacToe(TicTacToeSession::new(game_id, stage_id)),
            GameKind::TwentyFortyEight => {
                Self::TwentyFortyEight(Game2048Session::new(game_id, stage_id))
            }
            GameKind::Sudoku => Self::Sudoku(SudokuSession::new(game_id, stage_id)),
            GameKind::Blackjack => Self::Blackjack(BlackjackSession::new(game_id, stage_id)),
            GameKind::Roulette => Self::Roulette(RouletteSession::new(game_id, stage_id)),
            GameKind::Slots => Self::Slots(SlotsSession::new(game_id, stage_id)),
            GameKind::Holdem => Self::Holdem(HoldemSession::new(game_id, stage_id)),
            GameKind::TypingPractice => {
                Self::TypingPractice(TypingPracticeSession::new(game_id, stage_id))
            }
            GameKind::Breakout => Self::Breakout(BreakoutSession::new(game_id, stage_id)),
            GameKind::Minesweeper => Self::Minesweeper(MinesweeperSession::new(game_id, stage_id)),
            GameKind::ConnectFour => Self::ConnectFour(ConnectFourSession::new(game_id, stage_id)),
            GameKind::MemoryMatch => Self::MemoryMatch(MemoryMatchSession::new(game_id, stage_id)),
            GameKind::Maze => Self::Maze(MazeSession::new(game_id, stage_id)),
            GameKind::Gambling => return None,
        })
    }

    pub fn restart_casino(kind: GameKind, game_id: GameId, stage_id: StageId) -> Option<Self> {
        Some(match kind {
            GameKind::Blackjack => Self::Blackjack(BlackjackSession::new(game_id, stage_id)),
            GameKind::Roulette => Self::Roulette(RouletteSession::new(game_id, stage_id)),
            GameKind::Slots => Self::Slots(SlotsSession::new(game_id, stage_id)),
            GameKind::Holdem => Self::Holdem(HoldemSession::new(game_id, stage_id)),
            _ => return None,
        })
    }

    pub fn kind(&self) -> GameKind {
        match self {
            Self::Arithmetic(_) => GameKind::Arithmetic,
            Self::Snake(_) => GameKind::Snake,
            Self::TicTacToe(_) => GameKind::TicTacToe,
            Self::TwentyFortyEight(_) => GameKind::TwentyFortyEight,
            Self::Sudoku(_) => GameKind::Sudoku,
            Self::Blackjack(_) => GameKind::Blackjack,
            Self::Roulette(_) => GameKind::Roulette,
            Self::Slots(_) => GameKind::Slots,
            Self::Holdem(_) => GameKind::Holdem,
            Self::TypingPractice(_) => GameKind::TypingPractice,
            Self::Breakout(_) => GameKind::Breakout,
            Self::Minesweeper(_) => GameKind::Minesweeper,
            Self::ConnectFour(_) => GameKind::ConnectFour,
            Self::MemoryMatch(_) => GameKind::MemoryMatch,
            Self::Maze(_) => GameKind::Maze,
        }
    }

    pub fn tick(&mut self) -> Option<RoundResult> {
        match self {
            Self::Arithmetic(session) => session.check_timeout(),
            Self::Snake(session) => session.tick(),
            Self::Breakout(session) => session.tick(),
            _ => None,
        }
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        match self {
            Self::Arithmetic(session) => session.finish_abandoned(),
            Self::Snake(session) => session.finish_abandoned(),
            Self::TicTacToe(session) => session.finish_abandoned(),
            Self::TwentyFortyEight(session) => session.finish_abandoned(),
            Self::Sudoku(session) => session.finish_abandoned(),
            Self::Blackjack(session) => session.finish_abandoned(),
            Self::Roulette(session) => session.finish_abandoned(),
            Self::Slots(session) => session.finish_abandoned(),
            Self::Holdem(session) => session.finish_abandoned(),
            Self::TypingPractice(session) => session.finish_abandoned(),
            Self::Breakout(session) => session.finish_abandoned(),
            Self::Minesweeper(session) => session.finish_abandoned(),
            Self::ConnectFour(session) => session.finish_abandoned(),
            Self::MemoryMatch(session) => session.finish_abandoned(),
            Self::Maze(session) => session.finish_abandoned(),
        }
    }

    pub fn payout_multiplier(&self) -> u64 {
        match self {
            Self::Blackjack(session) => session.payout(),
            Self::Roulette(session) => session.payout(),
            Self::Slots(session) => session.payout(),
            Self::Holdem(session) => session.payout(),
            _ => 0,
        }
    }

    pub fn as_arithmetic(&self) -> Option<&RoundSession> {
        match self {
            Self::Arithmetic(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_arithmetic_mut(&mut self) -> Option<&mut RoundSession> {
        match self {
            Self::Arithmetic(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_snake(&self) -> Option<&SnakeSession> {
        match self {
            Self::Snake(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_snake_mut(&mut self) -> Option<&mut SnakeSession> {
        match self {
            Self::Snake(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_tictactoe(&self) -> Option<&TicTacToeSession> {
        match self {
            Self::TicTacToe(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_tictactoe_mut(&mut self) -> Option<&mut TicTacToeSession> {
        match self {
            Self::TicTacToe(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_2048(&self) -> Option<&Game2048Session> {
        match self {
            Self::TwentyFortyEight(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_2048_mut(&mut self) -> Option<&mut Game2048Session> {
        match self {
            Self::TwentyFortyEight(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_sudoku(&self) -> Option<&SudokuSession> {
        match self {
            Self::Sudoku(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_sudoku_mut(&mut self) -> Option<&mut SudokuSession> {
        match self {
            Self::Sudoku(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_blackjack(&self) -> Option<&BlackjackSession> {
        match self {
            Self::Blackjack(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_blackjack_mut(&mut self) -> Option<&mut BlackjackSession> {
        match self {
            Self::Blackjack(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_roulette(&self) -> Option<&RouletteSession> {
        match self {
            Self::Roulette(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_roulette_mut(&mut self) -> Option<&mut RouletteSession> {
        match self {
            Self::Roulette(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_slots(&self) -> Option<&SlotsSession> {
        match self {
            Self::Slots(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_slots_mut(&mut self) -> Option<&mut SlotsSession> {
        match self {
            Self::Slots(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_holdem(&self) -> Option<&HoldemSession> {
        match self {
            Self::Holdem(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_holdem_mut(&mut self) -> Option<&mut HoldemSession> {
        match self {
            Self::Holdem(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_typing(&self) -> Option<&TypingPracticeSession> {
        match self {
            Self::TypingPractice(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_typing_mut(&mut self) -> Option<&mut TypingPracticeSession> {
        match self {
            Self::TypingPractice(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_breakout(&self) -> Option<&BreakoutSession> {
        match self {
            Self::Breakout(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_breakout_mut(&mut self) -> Option<&mut BreakoutSession> {
        match self {
            Self::Breakout(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_minesweeper(&self) -> Option<&MinesweeperSession> {
        match self {
            Self::Minesweeper(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_minesweeper_mut(&mut self) -> Option<&mut MinesweeperSession> {
        match self {
            Self::Minesweeper(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_connect_four(&self) -> Option<&ConnectFourSession> {
        match self {
            Self::ConnectFour(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_connect_four_mut(&mut self) -> Option<&mut ConnectFourSession> {
        match self {
            Self::ConnectFour(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_memory_match(&self) -> Option<&MemoryMatchSession> {
        match self {
            Self::MemoryMatch(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_memory_match_mut(&mut self) -> Option<&mut MemoryMatchSession> {
        match self {
            Self::MemoryMatch(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_maze(&self) -> Option<&MazeSession> {
        match self {
            Self::Maze(session) => Some(session),
            _ => None,
        }
    }

    pub fn as_maze_mut(&mut self) -> Option<&mut MazeSession> {
        match self {
            Self::Maze(session) => Some(session),
            _ => None,
        }
    }
}
