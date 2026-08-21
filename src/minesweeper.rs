use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinesweeperDifficulty {
    Beginner,
    Intermediate,
    Expert,
}

impl MinesweeperDifficulty {
    pub fn from_stage_id(stage_id: &StageId) -> Self {
        match stage_id.as_str() {
            "intermediate" => Self::Intermediate,
            "expert" => Self::Expert,
            _ => Self::Beginner,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Beginner => "BEGINNER",
            Self::Intermediate => "INTERMEDIATE",
            Self::Expert => "EXPERT",
        }
    }

    fn board(self) -> (usize, usize, usize) {
        match self {
            Self::Beginner => (9, 9, 10),
            Self::Intermediate => (16, 12, 30),
            Self::Expert => (24, 16, 70),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellState {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinesweeperOutcome {
    Cleared,
    Exploded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CellView {
    pub state: CellState,
    pub adjacent_mines: u8,
    pub is_mine: bool,
}

pub struct MinesweeperSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    difficulty: MinesweeperDifficulty,
    width: usize,
    height: usize,
    mine_count: usize,
    mines: Vec<bool>,
    states: Vec<CellState>,
    cursor: usize,
    first_reveal: bool,
    revealed_safe: usize,
    safe_actions: u32,
    attempts: u32,
    outcome: Option<MinesweeperOutcome>,
    game_over: bool,
}

impl MinesweeperSession {
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
        let difficulty = MinesweeperDifficulty::from_stage_id(&stage_id);
        let (width, height, mine_count) = difficulty.board();
        let mut positions: Vec<usize> = (0..width * height).collect();
        positions.shuffle(&mut StdRng::seed_from_u64(seed));
        let mut mines = vec![false; width * height];
        for index in positions.into_iter().take(mine_count) {
            mines[index] = true;
        }
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant: clock.now(),
            clock,
            difficulty,
            width,
            height,
            mine_count,
            mines,
            states: vec![CellState::Hidden; width * height],
            cursor: 0,
            first_reveal: true,
            revealed_safe: 0,
            safe_actions: 0,
            attempts: 0,
            outcome: None,
            game_over: false,
        }
    }

    pub fn difficulty(&self) -> MinesweeperDifficulty {
        self.difficulty
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn mine_count(&self) -> usize {
        self.mine_count
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn flags(&self) -> usize {
        self.states
            .iter()
            .filter(|state| **state == CellState::Flagged)
            .count()
    }

    pub fn revealed_safe(&self) -> usize {
        self.revealed_safe
    }

    pub fn outcome(&self) -> Option<MinesweeperOutcome> {
        self.outcome
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn cell(&self, x: usize, y: usize) -> Option<CellView> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        Some(CellView {
            state: self.states[index],
            adjacent_mines: self.adjacent_mines(index),
            is_mine: self.mines[index],
        })
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        if self.game_over {
            return;
        }
        let x = (self.cursor % self.width) as i32;
        let y = (self.cursor / self.width) as i32;
        let next_x = (x + dx).clamp(0, self.width as i32 - 1);
        let next_y = (y + dy).clamp(0, self.height as i32 - 1);
        self.cursor = next_y as usize * self.width + next_x as usize;
    }

    pub fn toggle_flag(&mut self) {
        if self.game_over {
            return;
        }
        self.states[self.cursor] = match self.states[self.cursor] {
            CellState::Hidden => CellState::Flagged,
            CellState::Flagged => CellState::Hidden,
            CellState::Revealed => CellState::Revealed,
        };
    }

    pub fn reveal(&mut self) -> Option<RoundResult> {
        if self.game_over || self.states[self.cursor] != CellState::Hidden {
            return None;
        }
        self.attempts += 1;
        if self.first_reveal {
            self.first_reveal = false;
            self.relocate_first_mine();
        }
        if self.mines[self.cursor] {
            self.states[self.cursor] = CellState::Revealed;
            self.outcome = Some(MinesweeperOutcome::Exploded);
            self.game_over = true;
            return Some(self.result(RoundStatus::Completed));
        }

        self.safe_actions += 1;
        self.reveal_region(self.cursor);
        if self.revealed_safe == self.width * self.height - self.mine_count {
            self.outcome = Some(MinesweeperOutcome::Cleared);
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

    fn relocate_first_mine(&mut self) {
        if !self.mines[self.cursor] {
            return;
        }
        if let Some(destination) = self
            .mines
            .iter()
            .enumerate()
            .find_map(|(index, mine)| (!*mine && index != self.cursor).then_some(index))
        {
            self.mines[self.cursor] = false;
            self.mines[destination] = true;
        }
    }

    fn reveal_region(&mut self, start: usize) {
        let mut queue = VecDeque::from([start]);
        while let Some(index) = queue.pop_front() {
            if self.states[index] != CellState::Hidden || self.mines[index] {
                continue;
            }
            self.states[index] = CellState::Revealed;
            self.revealed_safe += 1;
            if self.adjacent_mines(index) == 0 {
                for neighbor in self.neighbors(index) {
                    if self.states[neighbor] == CellState::Hidden && !self.mines[neighbor] {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    fn adjacent_mines(&self, index: usize) -> u8 {
        self.neighbors(index)
            .into_iter()
            .filter(|neighbor| self.mines[*neighbor])
            .count() as u8
    }

    fn neighbors(&self, index: usize) -> Vec<usize> {
        let x = (index % self.width) as i32;
        let y = (index / self.width) as i32;
        let mut neighbors = Vec::with_capacity(8);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    neighbors.push(ny as usize * self.width + nx as usize);
                }
            }
        }
        neighbors
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        let cleared = self.outcome == Some(MinesweeperOutcome::Cleared);
        let score = if cleared {
            (self.mine_count as u32)
                .saturating_mul(100)
                .saturating_sub(self.attempts.saturating_mul(2))
        } else {
            self.revealed_safe.min(u32::MAX as usize) as u32
        };
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: self.safe_actions,
            attempts: self.attempts,
            accuracy: if self.attempts == 0 {
                0.0
            } else {
                f64::from(self.safe_actions) / f64::from(self.attempts)
            },
            best_streak: self.revealed_safe.min(u32::MAX as usize) as u32,
            score,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_stage_has_the_expected_board_and_mine_count() {
        for (stage, expected) in [
            ("beginner", (9, 9, 10)),
            ("intermediate", (16, 12, 30)),
            ("expert", (24, 16, 70)),
        ] {
            let session =
                MinesweeperSession::with_seed(GameId::new("minesweeper"), StageId::new(stage), 7);
            assert_eq!(
                (session.width(), session.height(), session.mine_count()),
                expected
            );
        }
    }

    #[test]
    fn first_reveal_is_always_safe_even_when_cursor_started_on_a_mine() {
        let mut session =
            MinesweeperSession::with_seed(GameId::new("minesweeper"), StageId::new("beginner"), 3);
        let mine = session.mines.iter().position(|mine| *mine).expect("mine");
        session.cursor = mine;
        assert!(session.reveal().is_none());
        assert_ne!(session.outcome(), Some(MinesweeperOutcome::Exploded));
        assert_eq!(session.states[mine], CellState::Revealed);
    }

    #[test]
    fn flagged_cell_cannot_be_revealed() {
        let mut session =
            MinesweeperSession::with_seed(GameId::new("minesweeper"), StageId::new("beginner"), 1);
        session.toggle_flag();
        assert!(session.reveal().is_none());
        assert_eq!(session.states[0], CellState::Flagged);
        assert_eq!(session.attempts, 0);
    }

    #[test]
    fn revealing_all_safe_cells_completes_the_round() {
        let mut session =
            MinesweeperSession::with_seed(GameId::new("minesweeper"), StageId::new("beginner"), 21);
        while !session.is_game_over() {
            let next = (0..session.mines.len()).find(|index| {
                !session.mines[*index] && session.states[*index] == CellState::Hidden
            });
            let Some(next) = next else {
                break;
            };
            session.cursor = next;
            session.reveal();
        }
        assert_eq!(session.outcome(), Some(MinesweeperOutcome::Cleared));
        assert_eq!(
            session.revealed_safe(),
            session.width() * session.height() - session.mine_count()
        );
    }
}
