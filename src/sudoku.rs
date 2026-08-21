use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

pub const SUDOKU_SIZE: usize = 9;
const CELL_COUNT: usize = SUDOKU_SIZE * SUDOKU_SIZE;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SudokuDifficulty {
    Easy,
    Classic,
    Hard,
}

impl SudokuDifficulty {
    fn from_stage_id(stage_id: &StageId) -> Self {
        match stage_id.as_str() {
            "easy" => Self::Easy,
            "hard" => Self::Hard,
            _ => Self::Classic,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Classic => "CLASSIC",
            Self::Hard => "HARD",
        }
    }

    pub fn target_clues(self) -> usize {
        match self {
            Self::Easy => 46,
            Self::Classic => 40,
            Self::Hard => 32,
        }
    }
}

pub struct SudokuSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    difficulty: SudokuDifficulty,
    solution: [u8; CELL_COUNT],
    puzzle: [u8; CELL_COUNT],
    board: [u8; CELL_COUNT],
    givens: [bool; CELL_COUNT],
    cursor: usize,
    attempts: u32,
    correct_answers: u32,
    current_streak: u32,
    best_streak: u32,
    mistakes: u32,
    game_over: bool,
}

impl SudokuSession {
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
        let difficulty = SudokuDifficulty::from_stage_id(&stage_id);
        let mut rng = StdRng::seed_from_u64(seed);
        let solution = generate_solution(&mut rng);
        let puzzle = generate_puzzle(solution, &mut rng, difficulty.target_clues());
        let givens = puzzle.map(|value| value != 0);
        let started_instant = clock.now();
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant,
            clock,
            difficulty,
            solution,
            puzzle,
            board: puzzle,
            givens,
            cursor: 0,
            attempts: 0,
            correct_answers: 0,
            current_streak: 0,
            best_streak: 0,
            mistakes: 0,
            game_over: false,
        }
    }

    pub fn board(&self) -> &[u8; CELL_COUNT] {
        &self.board
    }

    pub fn difficulty(&self) -> SudokuDifficulty {
        self.difficulty
    }

    pub fn puzzle(&self) -> &[u8; CELL_COUNT] {
        &self.puzzle
    }

    pub fn solution(&self) -> &[u8; CELL_COUNT] {
        &self.solution
    }

    pub fn givens(&self) -> &[bool; CELL_COUNT] {
        &self.givens
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn correct_answers(&self) -> u32 {
        self.correct_answers
    }

    pub fn current_streak(&self) -> u32 {
        self.current_streak
    }

    pub fn best_streak(&self) -> u32 {
        self.best_streak
    }

    pub fn mistakes(&self) -> u32 {
        self.mistakes
    }

    pub fn filled_count(&self) -> usize {
        self.board.iter().filter(|value| **value != 0).count()
    }

    pub fn is_complete(&self) -> bool {
        self.game_over && self.board == self.solution
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn is_given(&self, index: usize) -> bool {
        self.givens.get(index).copied().unwrap_or(false)
    }

    pub fn is_wrong(&self, index: usize) -> bool {
        index < CELL_COUNT && self.board[index] != 0 && self.board[index] != self.solution[index]
    }

    pub fn is_conflict(&self, index: usize) -> bool {
        if index >= CELL_COUNT || self.board[index] == 0 {
            return false;
        }
        let row = index / SUDOKU_SIZE;
        let column = index % SUDOKU_SIZE;
        for other in 0..SUDOKU_SIZE {
            let row_index = row * SUDOKU_SIZE + other;
            let column_index = other * SUDOKU_SIZE + column;
            if row_index != index && self.board[row_index] == self.board[index]
                || column_index != index && self.board[column_index] == self.board[index]
            {
                return true;
            }
        }
        let box_row = row / 3 * 3;
        let box_column = column / 3 * 3;
        for y in box_row..box_row + 3 {
            for x in box_column..box_column + 3 {
                let other = y * SUDOKU_SIZE + x;
                if other != index && self.board[other] == self.board[index] {
                    return true;
                }
            }
        }
        false
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
        let x = (self.cursor % SUDOKU_SIZE) as i32;
        let y = (self.cursor / SUDOKU_SIZE) as i32;
        let next_x = (x + dx).clamp(0, (SUDOKU_SIZE - 1) as i32);
        let next_y = (y + dy).clamp(0, (SUDOKU_SIZE - 1) as i32);
        self.cursor = next_y as usize * SUDOKU_SIZE + next_x as usize;
    }

    pub fn select_cell(&mut self, index: usize) {
        if index < CELL_COUNT {
            self.cursor = index;
        }
    }

    pub fn place_digit(&mut self, digit: u8) -> Option<RoundResult> {
        if self.game_over || !(1..=9).contains(&digit) || self.is_given(self.cursor) {
            return None;
        }
        self.board[self.cursor] = digit;
        self.attempts += 1;
        if digit == self.solution[self.cursor] {
            self.correct_answers += 1;
            self.current_streak += 1;
            self.best_streak = self.best_streak.max(self.current_streak);
        } else {
            self.mistakes += 1;
            self.current_streak = 0;
        }
        if self.board == self.solution {
            self.game_over = true;
            Some(self.result(RoundStatus::Completed))
        } else {
            None
        }
    }

    pub fn clear_selected(&mut self) -> bool {
        if self.game_over || self.is_given(self.cursor) || self.board[self.cursor] == 0 {
            return false;
        }
        self.board[self.cursor] = 0;
        true
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.game_over = true;
        Some(self.result(RoundStatus::Abandoned))
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        let score = self
            .correct_answers
            .saturating_mul(10)
            .saturating_sub(self.mistakes);
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: self.correct_answers,
            attempts: self.attempts,
            accuracy: if self.attempts == 0 {
                0.0
            } else {
                f64::from(self.correct_answers) / f64::from(self.attempts)
            },
            best_streak: self.best_streak,
            score,
            status,
        }
    }
}

fn generate_solution(rng: &mut StdRng) -> [u8; CELL_COUNT] {
    let mut digits = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    digits.shuffle(rng);
    let rows = shuffled_units(rng);
    let columns = shuffled_units(rng);
    let mut solution = [0; CELL_COUNT];
    for (row_index, row) in rows.into_iter().enumerate() {
        for (column_index, column) in columns.iter().copied().enumerate() {
            let pattern_index = (row * 3 + row / 3 + column) % SUDOKU_SIZE;
            solution[row_index * SUDOKU_SIZE + column_index] = digits[pattern_index];
        }
    }
    solution
}

fn shuffled_units(rng: &mut StdRng) -> [usize; SUDOKU_SIZE] {
    let mut bands = [0, 1, 2];
    bands.shuffle(rng);
    let mut result = [0; SUDOKU_SIZE];
    for (band_index, band) in bands.into_iter().enumerate() {
        let mut within = [0, 1, 2];
        within.shuffle(rng);
        for (offset, value) in within.into_iter().enumerate() {
            result[band_index * 3 + offset] = band * 3 + value;
        }
    }
    result
}

fn generate_puzzle(
    solution: [u8; CELL_COUNT],
    rng: &mut StdRng,
    target_clues: usize,
) -> [u8; CELL_COUNT] {
    let mut puzzle = solution;
    let mut cells: Vec<_> = (0..CELL_COUNT).collect();
    cells.shuffle(rng);
    let mut clues = CELL_COUNT;
    for index in cells {
        if clues <= target_clues {
            break;
        }
        let saved = puzzle[index];
        puzzle[index] = 0;
        let mut candidate = puzzle;
        if count_solutions(&mut candidate, 2) == 1 {
            clues -= 1;
        } else {
            puzzle[index] = saved;
        }
    }
    puzzle
}

fn count_solutions(board: &mut [u8; CELL_COUNT], limit: u32) -> u32 {
    let Some(index) = best_empty_cell(board) else {
        return 1;
    };
    let mut count = 0;
    for digit in 1..=9 {
        if is_valid_digit(board, index, digit) {
            board[index] = digit;
            count += count_solutions(board, limit);
            board[index] = 0;
            if count >= limit {
                return count;
            }
        }
    }
    count
}

fn best_empty_cell(board: &[u8; CELL_COUNT]) -> Option<usize> {
    let mut best = None;
    let mut best_count = 10;
    for index in 0..CELL_COUNT {
        if board[index] != 0 {
            continue;
        }
        let count = (1..=9)
            .filter(|digit| is_valid_digit(board, index, *digit))
            .count();
        if count < best_count {
            best = Some(index);
            best_count = count;
            if count <= 1 {
                break;
            }
        }
    }
    best
}

fn is_valid_digit(board: &[u8; CELL_COUNT], index: usize, digit: u8) -> bool {
    let row = index / SUDOKU_SIZE;
    let column = index % SUDOKU_SIZE;
    if (0..SUDOKU_SIZE).any(|x| board[row * SUDOKU_SIZE + x] == digit) {
        return false;
    }
    if (0..SUDOKU_SIZE).any(|y| board[y * SUDOKU_SIZE + column] == digit) {
        return false;
    }
    let box_row = row / 3 * 3;
    let box_column = column / 3 * 3;
    !(box_row..box_row + 3)
        .any(|y| (box_column..box_column + 3).any(|x| board[y * SUDOKU_SIZE + x] == digit))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> (GameId, StageId) {
        (GameId::new("sudoku"), StageId::new("classic-1"))
    }

    #[test]
    fn generated_puzzle_has_a_unique_solution_and_valid_clues() {
        for stage in ["easy", "classic-1", "hard"] {
            let session = SudokuSession::with_seed(GameId::new("sudoku"), StageId::new(stage), 42);
            let clues = session.puzzle().iter().filter(|value| **value != 0).count();
            assert!(clues >= session.difficulty().target_clues(), "{stage}");
            assert!(clues < CELL_COUNT);
            assert!(
                session
                    .puzzle()
                    .iter()
                    .enumerate()
                    .all(|(index, value)| *value == 0 || *value == session.solution()[index])
            );
            let mut candidate = *session.puzzle();
            assert_eq!(count_solutions(&mut candidate, 2), 1, "{stage}");
        }
    }

    #[test]
    fn digit_entry_tracks_wrong_answers_and_clear_does_not_add_an_attempt() {
        let (game_id, stage_id) = ids();
        let mut session = SudokuSession::with_seed(game_id, stage_id, 7);
        let index = session
            .givens()
            .iter()
            .position(|given| !given)
            .expect("editable cell");
        session.select_cell(index);
        let solution = session.solution()[index];
        let wrong = if solution == 1 { 2 } else { 1 };
        assert!(session.place_digit(wrong).is_none());
        assert_eq!(session.mistakes(), 1);
        assert!(session.is_wrong(index));
        assert_eq!(session.attempts(), 1);
        assert!(session.clear_selected());
        assert_eq!(session.attempts(), 1);
        assert_eq!(session.board()[index], 0);
    }

    #[test]
    fn filling_every_editable_cell_finishes_the_game() {
        let (game_id, stage_id) = ids();
        let mut session = SudokuSession::with_seed(game_id, stage_id, 99);
        let solution = *session.solution();
        let givens = *session.givens();
        let mut result = None;
        for (index, given) in givens.into_iter().enumerate() {
            if !given {
                session.select_cell(index);
                result = session.place_digit(solution[index]);
            }
        }
        let result = result.expect("completed result");
        assert_eq!(result.status, RoundStatus::Completed);
        assert!(session.is_complete());
        assert_eq!(session.board(), &solution);
    }

    #[test]
    fn stages_change_the_number_of_given_cells() {
        let easy = SudokuSession::with_seed(GameId::new("sudoku"), StageId::new("easy"), 123);
        let classic =
            SudokuSession::with_seed(GameId::new("sudoku"), StageId::new("classic-1"), 123);
        let hard = SudokuSession::with_seed(GameId::new("sudoku"), StageId::new("hard"), 123);
        let clues =
            |session: &SudokuSession| session.puzzle().iter().filter(|value| **value != 0).count();
        assert!(clues(&easy) >= clues(&classic));
        assert!(clues(&classic) >= clues(&hard));
        assert_eq!(hard.difficulty(), SudokuDifficulty::Hard);
    }
}
