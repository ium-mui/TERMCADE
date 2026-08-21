use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

pub const CONNECT_FOUR_WIDTH: usize = 7;
pub const CONNECT_FOUR_HEIGHT: usize = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectFourCell {
    Empty,
    Player,
    Cpu,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectFourDifficulty {
    Easy,
    Normal,
    Hard,
}

impl ConnectFourDifficulty {
    pub fn from_stage_id(stage_id: &StageId) -> Self {
        match stage_id.as_str() {
            "normal" => Self::Normal,
            "hard" => Self::Hard,
            _ => Self::Easy,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Normal => "NORMAL",
            Self::Hard => "HARD",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectFourOutcome {
    PlayerWin,
    CpuWin,
    Draw,
}

pub struct ConnectFourSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    rng: StdRng,
    board: [ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT],
    cursor_column: usize,
    difficulty: ConnectFourDifficulty,
    player_moves: u32,
    outcome: Option<ConnectFourOutcome>,
    game_over: bool,
}

impl ConnectFourSession {
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
        let difficulty = ConnectFourDifficulty::from_stage_id(&stage_id);
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant: clock.now(),
            clock,
            rng: StdRng::seed_from_u64(seed),
            board: [ConnectFourCell::Empty; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT],
            cursor_column: CONNECT_FOUR_WIDTH / 2,
            difficulty,
            player_moves: 0,
            outcome: None,
            game_over: false,
        }
    }

    pub fn board(&self) -> &[ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT] {
        &self.board
    }

    pub fn cursor_column(&self) -> usize {
        self.cursor_column
    }

    pub fn difficulty(&self) -> ConnectFourDifficulty {
        self.difficulty
    }

    pub fn outcome(&self) -> Option<ConnectFourOutcome> {
        self.outcome
    }

    pub fn player_moves(&self) -> u32 {
        self.player_moves
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn move_cursor(&mut self, delta: i32) {
        if self.game_over {
            return;
        }
        self.cursor_column =
            (self.cursor_column as i32 + delta).clamp(0, CONNECT_FOUR_WIDTH as i32 - 1) as usize;
    }

    pub fn drop_disc(&mut self) -> Option<RoundResult> {
        if self.game_over
            || drop_into(&mut self.board, self.cursor_column, ConnectFourCell::Player).is_none()
        {
            return None;
        }
        self.player_moves += 1;
        if winner(&self.board) == Some(ConnectFourCell::Player) {
            return Some(self.finish(ConnectFourOutcome::PlayerWin));
        }
        if board_full(&self.board) {
            return Some(self.finish(ConnectFourOutcome::Draw));
        }

        let cpu_column = self.choose_cpu_column();
        drop_into(&mut self.board, cpu_column, ConnectFourCell::Cpu)
            .expect("AI must choose a playable column");
        if winner(&self.board) == Some(ConnectFourCell::Cpu) {
            return Some(self.finish(ConnectFourOutcome::CpuWin));
        }
        if board_full(&self.board) {
            return Some(self.finish(ConnectFourOutcome::Draw));
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

    fn choose_cpu_column(&mut self) -> usize {
        let playable = playable_columns(&self.board);
        match self.difficulty {
            ConnectFourDifficulty::Easy => playable[self.rng.gen_range(0..playable.len())],
            ConnectFourDifficulty::Normal => immediate_move(&self.board, ConnectFourCell::Cpu)
                .or_else(|| immediate_move(&self.board, ConnectFourCell::Player))
                .unwrap_or_else(|| center_biased_column(&playable, &mut self.rng)),
            ConnectFourDifficulty::Hard => best_minimax_column(&self.board, 5, &mut self.rng),
        }
    }

    fn finish(&mut self, outcome: ConnectFourOutcome) -> RoundResult {
        self.outcome = Some(outcome);
        self.game_over = true;
        self.result(RoundStatus::Completed)
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        let score = match self.outcome {
            Some(ConnectFourOutcome::PlayerWin) => 1000u32.saturating_sub(self.player_moves * 20),
            Some(ConnectFourOutcome::Draw) => 250,
            _ => 0,
        };
        let correct = u32::from(self.outcome == Some(ConnectFourOutcome::PlayerWin));
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: correct,
            attempts: self.player_moves,
            accuracy: f64::from(correct),
            best_streak: correct,
            score,
            status,
        }
    }
}

fn index(column: usize, row: usize) -> usize {
    row * CONNECT_FOUR_WIDTH + column
}

fn drop_into(
    board: &mut [ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT],
    column: usize,
    disc: ConnectFourCell,
) -> Option<usize> {
    if column >= CONNECT_FOUR_WIDTH {
        return None;
    }
    for row in (0..CONNECT_FOUR_HEIGHT).rev() {
        let position = index(column, row);
        if board[position] == ConnectFourCell::Empty {
            board[position] = disc;
            return Some(position);
        }
    }
    None
}

fn playable_columns(
    board: &[ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT],
) -> Vec<usize> {
    (0..CONNECT_FOUR_WIDTH)
        .filter(|column| board[index(*column, 0)] == ConnectFourCell::Empty)
        .collect()
}

fn board_full(board: &[ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT]) -> bool {
    playable_columns(board).is_empty()
}

fn winner(
    board: &[ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT],
) -> Option<ConnectFourCell> {
    for row in 0..CONNECT_FOUR_HEIGHT {
        for column in 0..CONNECT_FOUR_WIDTH {
            let disc = board[index(column, row)];
            if disc == ConnectFourCell::Empty {
                continue;
            }
            for (dx, dy) in [(1i32, 0i32), (0, 1), (1, 1), (1, -1)] {
                if (1..4).all(|step| {
                    let x = column as i32 + dx * step;
                    let y = row as i32 + dy * step;
                    x >= 0
                        && x < CONNECT_FOUR_WIDTH as i32
                        && y >= 0
                        && y < CONNECT_FOUR_HEIGHT as i32
                        && board[index(x as usize, y as usize)] == disc
                }) {
                    return Some(disc);
                }
            }
        }
    }
    None
}

fn immediate_move(
    board: &[ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT],
    disc: ConnectFourCell,
) -> Option<usize> {
    playable_columns(board).into_iter().find(|column| {
        let mut candidate = *board;
        drop_into(&mut candidate, *column, disc);
        winner(&candidate) == Some(disc)
    })
}

fn center_biased_column(playable: &[usize], rng: &mut StdRng) -> usize {
    let mut weighted = Vec::new();
    for column in playable {
        let weight = 4usize.saturating_sub(column.abs_diff(CONNECT_FOUR_WIDTH / 2));
        weighted.extend(std::iter::repeat_n(*column, weight.max(1)));
    }
    weighted[rng.gen_range(0..weighted.len())]
}

fn best_minimax_column(
    board: &[ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT],
    depth: u8,
    rng: &mut StdRng,
) -> usize {
    if let Some(column) = immediate_move(board, ConnectFourCell::Cpu) {
        return column;
    }
    if let Some(column) = immediate_move(board, ConnectFourCell::Player) {
        return column;
    }
    let mut best_score = i32::MIN;
    let mut best = Vec::new();
    for column in playable_columns(board) {
        let mut candidate = *board;
        drop_into(&mut candidate, column, ConnectFourCell::Cpu);
        let score = minimax(
            &candidate,
            depth.saturating_sub(1),
            false,
            i32::MIN,
            i32::MAX,
        );
        match score.cmp(&best_score) {
            std::cmp::Ordering::Greater => {
                best_score = score;
                best.clear();
                best.push(column);
            }
            std::cmp::Ordering::Equal => best.push(column),
            std::cmp::Ordering::Less => {}
        }
    }
    best[rng.gen_range(0..best.len())]
}

fn minimax(
    board: &[ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT],
    depth: u8,
    maximizing: bool,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    if let Some(disc) = winner(board) {
        return if disc == ConnectFourCell::Cpu {
            100_000 + i32::from(depth)
        } else {
            -100_000 - i32::from(depth)
        };
    }
    if depth == 0 || board_full(board) {
        return evaluate_board(board);
    }
    if maximizing {
        let mut value = i32::MIN;
        for column in playable_columns(board) {
            let mut candidate = *board;
            drop_into(&mut candidate, column, ConnectFourCell::Cpu);
            value = value.max(minimax(&candidate, depth - 1, false, alpha, beta));
            alpha = alpha.max(value);
            if alpha >= beta {
                break;
            }
        }
        value
    } else {
        let mut value = i32::MAX;
        for column in playable_columns(board) {
            let mut candidate = *board;
            drop_into(&mut candidate, column, ConnectFourCell::Player);
            value = value.min(minimax(&candidate, depth - 1, true, alpha, beta));
            beta = beta.min(value);
            if alpha >= beta {
                break;
            }
        }
        value
    }
}

fn evaluate_board(board: &[ConnectFourCell; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT]) -> i32 {
    let mut score = 0;
    for row in 0..CONNECT_FOUR_HEIGHT {
        score += match board[index(CONNECT_FOUR_WIDTH / 2, row)] {
            ConnectFourCell::Cpu => 6,
            ConnectFourCell::Player => -6,
            ConnectFourCell::Empty => 0,
        };
    }
    for row in 0..CONNECT_FOUR_HEIGHT {
        for column in 0..CONNECT_FOUR_WIDTH {
            for (dx, dy) in [(1i32, 0i32), (0, 1), (1, 1), (1, -1)] {
                let mut cpu = 0;
                let mut player = 0;
                let mut valid = true;
                for step in 0..4 {
                    let x = column as i32 + dx * step;
                    let y = row as i32 + dy * step;
                    if x < 0
                        || x >= CONNECT_FOUR_WIDTH as i32
                        || y < 0
                        || y >= CONNECT_FOUR_HEIGHT as i32
                    {
                        valid = false;
                        break;
                    }
                    match board[index(x as usize, y as usize)] {
                        ConnectFourCell::Cpu => cpu += 1,
                        ConnectFourCell::Player => player += 1,
                        ConnectFourCell::Empty => {}
                    }
                }
                if valid {
                    score += window_score(cpu, player);
                }
            }
        }
    }
    score
}

fn window_score(cpu: i32, player: i32) -> i32 {
    match (cpu, player) {
        (3, 0) => 50,
        (2, 0) => 10,
        (1, 0) => 1,
        (0, 3) => -60,
        (0, 2) => -12,
        (0, 1) => -1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discs_stack_from_the_bottom() {
        let mut board = [ConnectFourCell::Empty; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT];
        assert_eq!(drop_into(&mut board, 2, ConnectFourCell::Player), Some(37));
        assert_eq!(drop_into(&mut board, 2, ConnectFourCell::Cpu), Some(30));
    }

    #[test]
    fn winner_detects_vertical_horizontal_and_diagonal_lines() {
        let mut vertical = [ConnectFourCell::Empty; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT];
        for _ in 0..4 {
            drop_into(&mut vertical, 1, ConnectFourCell::Cpu);
        }
        assert_eq!(winner(&vertical), Some(ConnectFourCell::Cpu));

        let mut horizontal = [ConnectFourCell::Empty; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT];
        for column in 0..4 {
            drop_into(&mut horizontal, column, ConnectFourCell::Player);
        }
        assert_eq!(winner(&horizontal), Some(ConnectFourCell::Player));

        let mut diagonal = [ConnectFourCell::Empty; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT];
        for (column, row) in [(0, 5), (1, 4), (2, 3), (3, 2)] {
            diagonal[index(column, row)] = ConnectFourCell::Cpu;
        }
        assert_eq!(winner(&diagonal), Some(ConnectFourCell::Cpu));
    }

    #[test]
    fn hard_ai_takes_an_immediate_win() {
        let mut board = [ConnectFourCell::Empty; CONNECT_FOUR_WIDTH * CONNECT_FOUR_HEIGHT];
        for _ in 0..3 {
            drop_into(&mut board, 4, ConnectFourCell::Cpu);
        }
        let mut rng = StdRng::seed_from_u64(1);
        assert_eq!(best_minimax_column(&board, 5, &mut rng), 4);
    }

    #[test]
    fn stage_selects_real_ai_difficulty() {
        let session =
            ConnectFourSession::with_seed(GameId::new("connect-four"), StageId::new("hard"), 4);
        assert_eq!(session.difficulty(), ConnectFourDifficulty::Hard);
    }
}
