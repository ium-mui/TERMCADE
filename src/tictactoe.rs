use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Empty,
    X,
    O,
}

pub struct TicTacToeSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    board: [Cell; 9],
    cursor: usize,
    moves: u32,
    winner: Option<Cell>,
    game_over: bool,
}

impl TicTacToeSession {
    pub fn new(game_id: GameId, stage_id: StageId) -> Self {
        Self::with_clock(game_id, stage_id, Arc::new(SystemClock))
    }

    pub fn with_clock(game_id: GameId, stage_id: StageId, clock: Arc<dyn Clock>) -> Self {
        let started_instant = clock.now();
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant,
            clock,
            board: [Cell::Empty; 9],
            cursor: 0,
            moves: 0,
            winner: None,
            game_over: false,
        }
    }

    pub fn board(&self) -> &[Cell; 9] {
        &self.board
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn winner(&self) -> Option<Cell> {
        self.winner
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        if self.game_over {
            return;
        }
        let x = (self.cursor % 3) as i32;
        let y = (self.cursor / 3) as i32;
        let next_x = (x + dx).clamp(0, 2);
        let next_y = (y + dy).clamp(0, 2);
        self.cursor = (next_y * 3 + next_x) as usize;
    }

    pub fn place(&mut self) -> Option<RoundResult> {
        if self.game_over || self.board[self.cursor] != Cell::Empty {
            return None;
        }

        self.board[self.cursor] = Cell::X;
        self.moves += 1;
        if self.finish_if_over(Cell::X) {
            return Some(self.result(RoundStatus::Completed));
        }

        if let Some(index) = choose_ai_move(&self.board) {
            self.board[index] = Cell::O;
            if self.finish_if_over(Cell::O) || is_draw(&self.board) {
                return Some(self.result(RoundStatus::Completed));
            }
        }
        None
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.game_over = true;
        Some(self.result(RoundStatus::Abandoned))
    }

    fn finish_if_over(&mut self, cell: Cell) -> bool {
        if winner(&self.board) == Some(cell) {
            self.winner = Some(cell);
            self.game_over = true;
            true
        } else if is_draw(&self.board) {
            self.game_over = true;
            true
        } else {
            false
        }
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        let score = u32::from(self.winner == Some(Cell::X));
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: score,
            attempts: self.moves,
            accuracy: f64::from(score),
            best_streak: score,
            score,
            status,
        }
    }
}

const WINNING_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

fn winner(board: &[Cell; 9]) -> Option<Cell> {
    WINNING_LINES.iter().find_map(|line| {
        let [first, second, third] = *line;
        (board[first] != Cell::Empty
            && board[first] == board[second]
            && board[first] == board[third])
            .then_some(board[first])
    })
}

fn is_draw(board: &[Cell; 9]) -> bool {
    board.iter().all(|cell| *cell != Cell::Empty) && winner(board).is_none()
}

fn choose_ai_move(board: &[Cell; 9]) -> Option<usize> {
    find_winning_move(board, Cell::O)
        .or_else(|| find_winning_move(board, Cell::X))
        .or_else(|| (board[4] == Cell::Empty).then_some(4))
        .or_else(|| board.iter().position(|cell| *cell == Cell::Empty))
}

fn find_winning_move(board: &[Cell; 9], cell: Cell) -> Option<usize> {
    board.iter().enumerate().find_map(|(index, current)| {
        if *current != Cell::Empty {
            return None;
        }
        let mut candidate = *board;
        candidate[index] = cell;
        (winner(&candidate) == Some(cell)).then_some(index)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_can_place_and_ai_takes_a_turn() {
        let mut session =
            TicTacToeSession::new(GameId::new("tictactoe"), StageId::new("classic-1"));
        assert_eq!(
            session
                .board()
                .iter()
                .filter(|cell| **cell != Cell::Empty)
                .count(),
            0
        );
        assert!(session.place().is_none());
        assert_eq!(session.board()[0], Cell::X);
        assert_eq!(session.board()[4], Cell::O);
        assert_eq!(session.moves, 1);
    }

    #[test]
    fn cursor_stays_inside_the_board() {
        let mut session =
            TicTacToeSession::new(GameId::new("tictactoe"), StageId::new("classic-1"));
        session.move_cursor(-1, -1);
        assert_eq!(session.cursor(), 0);
        session.move_cursor(1, 1);
        assert_eq!(session.cursor(), 4);
        session.move_cursor(10, 10);
        assert_eq!(session.cursor(), 8);
    }

    #[test]
    fn ai_blocks_an_immediate_player_win() {
        let board = [
            Cell::X,
            Cell::X,
            Cell::Empty,
            Cell::O,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
        ];
        assert_eq!(choose_ai_move(&board), Some(2));
    }
}
