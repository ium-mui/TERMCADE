use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};
use crate::snake::Direction;

pub const GRID_SIZE: usize = 4;

pub struct Game2048Session {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    rng: StdRng,
    board: [[u32; GRID_SIZE]; GRID_SIZE],
    score: u32,
    moves: u32,
    game_over: bool,
}

impl Game2048Session {
    pub fn new(game_id: GameId, stage_id: StageId) -> Self {
        Self::with_seed_and_clock(game_id, stage_id, rand::random(), Arc::new(SystemClock))
    }

    pub fn with_seed(game_id: GameId, stage_id: StageId, seed: u64) -> Self {
        Self::with_seed_and_clock(game_id, stage_id, seed, Arc::new(SystemClock))
    }

    pub fn with_seed_and_clock(
        game_id: GameId,
        stage_id: StageId,
        seed: u64,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let started_instant = clock.now();
        let mut session = Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant,
            clock,
            rng: StdRng::seed_from_u64(seed),
            board: [[0; GRID_SIZE]; GRID_SIZE],
            score: 0,
            moves: 0,
            game_over: false,
        };
        session.add_random_tile();
        session.add_random_tile();
        session
    }

    pub fn board(&self) -> &[[u32; GRID_SIZE]; GRID_SIZE] {
        &self.board
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn moves(&self) -> u32 {
        self.moves
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn move_direction(&mut self, direction: Direction) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }

        let mut changed = false;
        match direction {
            Direction::Left => {
                for row in &mut self.board {
                    let (next, gained) = process_line(*row);
                    changed |= next != *row;
                    *row = next;
                    self.score += gained;
                }
            }
            Direction::Right => {
                for row in &mut self.board {
                    let original = *row;
                    let reversed = reverse(original);
                    let (mut next, gained) = process_line(reversed);
                    next = reverse(next);
                    changed |= next != original;
                    *row = next;
                    self.score += gained;
                }
            }
            Direction::Up => {
                for x in 0..GRID_SIZE {
                    let original = column(&self.board, x);
                    let (next, gained) = process_line(original);
                    changed |= next != original;
                    set_column(&mut self.board, x, next);
                    self.score += gained;
                }
            }
            Direction::Down => {
                for x in 0..GRID_SIZE {
                    let original = column(&self.board, x);
                    let (mut next, gained) = process_line(reverse(original));
                    next = reverse(next);
                    changed |= next != original;
                    set_column(&mut self.board, x, next);
                    self.score += gained;
                }
            }
        }

        if !changed {
            if self.has_moves() {
                return None;
            }
            self.game_over = true;
            return Some(self.result(RoundStatus::Completed));
        }

        self.moves += 1;
        if self.has_won() {
            self.game_over = true;
            return Some(self.result(RoundStatus::Completed));
        }
        self.add_random_tile();
        if !self.has_moves() {
            self.game_over = true;
            return Some(self.result(RoundStatus::Completed));
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

    fn has_won(&self) -> bool {
        self.board.iter().flatten().any(|tile| *tile >= 2048)
    }

    fn has_moves(&self) -> bool {
        self.board.iter().flatten().any(|tile| *tile == 0)
            || (0..GRID_SIZE).any(|y| {
                (0..GRID_SIZE).any(|x| {
                    (x + 1 < GRID_SIZE && self.board[y][x] == self.board[y][x + 1])
                        || (y + 1 < GRID_SIZE && self.board[y][x] == self.board[y + 1][x])
                })
            })
    }

    fn add_random_tile(&mut self) {
        let empty: Vec<_> = self
            .board
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter_map(move |(x, tile)| (*tile == 0).then_some((y, x)))
            })
            .collect();
        let Some(&(y, x)) = empty.get(self.rng.gen_range(0..empty.len())) else {
            return;
        };
        self.board[y][x] = if self.rng.gen_bool(0.9) { 2 } else { 4 };
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: self.score,
            attempts: self.moves,
            accuracy: if self.moves == 0 { 0.0 } else { 1.0 },
            best_streak: self.score,
            score: self.score,
            status,
        }
    }
}

fn process_line(line: [u32; GRID_SIZE]) -> ([u32; GRID_SIZE], u32) {
    let values: Vec<_> = line.into_iter().filter(|tile| *tile != 0).collect();
    let mut merged = Vec::with_capacity(GRID_SIZE);
    let mut gained = 0;
    let mut index = 0;
    while index < values.len() {
        if index + 1 < values.len() && values[index] == values[index + 1] {
            let value = values[index] * 2;
            merged.push(value);
            gained += value;
            index += 2;
        } else {
            merged.push(values[index]);
            index += 1;
        }
    }
    let mut result = [0; GRID_SIZE];
    for (index, value) in merged.into_iter().enumerate() {
        result[index] = value;
    }
    (result, gained)
}

fn reverse(line: [u32; GRID_SIZE]) -> [u32; GRID_SIZE] {
    [line[3], line[2], line[1], line[0]]
}

fn column(board: &[[u32; GRID_SIZE]; GRID_SIZE], x: usize) -> [u32; GRID_SIZE] {
    [board[0][x], board[1][x], board[2][x], board[3][x]]
}

fn set_column(board: &mut [[u32; GRID_SIZE]; GRID_SIZE], x: usize, values: [u32; GRID_SIZE]) {
    for (y, value) in values.into_iter().enumerate() {
        board[y][x] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_starts_with_two_tiles() {
        let session = Game2048Session::with_seed(GameId::new("2048"), StageId::new("classic-1"), 7);
        assert_eq!(
            session
                .board()
                .iter()
                .flatten()
                .filter(|tile| **tile != 0)
                .count(),
            2
        );
    }

    #[test]
    fn equal_tiles_merge_once_and_increase_score() {
        let mut session =
            Game2048Session::with_seed(GameId::new("2048"), StageId::new("classic-1"), 7);
        session.board = [[2, 2, 4, 0], [0; 4], [0; 4], [0; 4]];
        assert!(session.move_direction(Direction::Left).is_none());
        assert_eq!(&session.board[0][..3], &[4, 4, 0]);
        assert_eq!(session.score(), 4);
        assert_eq!(session.moves(), 1);
    }

    #[test]
    fn full_board_without_a_move_ends_the_game() {
        let mut session =
            Game2048Session::with_seed(GameId::new("2048"), StageId::new("classic-1"), 7);
        session.board = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        let result = session
            .move_direction(Direction::Left)
            .expect("game over result");
        assert_eq!(result.status, RoundStatus::Completed);
        assert!(session.is_game_over());
    }
}
