use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

pub const BOARD_WIDTH: i32 = 24;
pub const BOARD_HEIGHT: i32 = 12;
pub const INITIAL_SNAKE_LENGTH: usize = 3;
pub const DEFAULT_MOVE_INTERVAL: Duration = Duration::from_millis(160);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnakePace {
    Relaxed,
    Classic,
    Turbo,
}

impl SnakePace {
    fn from_stage_id(stage_id: &StageId) -> Self {
        match stage_id.as_str() {
            "relaxed" => Self::Relaxed,
            "turbo" => Self::Turbo,
            _ => Self::Classic,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Relaxed => "RELAXED",
            Self::Classic => "CLASSIC",
            Self::Turbo => "TURBO",
        }
    }

    pub fn move_interval(self) -> Duration {
        match self {
            Self::Relaxed => Duration::from_millis(230),
            Self::Classic => DEFAULT_MOVE_INTERVAL,
            Self::Turbo => Duration::from_millis(95),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn is_opposite(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Up, Self::Down)
                | (Self::Down, Self::Up)
                | (Self::Left, Self::Right)
                | (Self::Right, Self::Left)
        )
    }

    fn offset(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }
}

pub struct SnakeSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    rng: StdRng,
    snake: Vec<Point>,
    food: Point,
    direction: Direction,
    pending_direction: Direction,
    pace: SnakePace,
    started: bool,
    last_move: Instant,
    move_interval: Duration,
    score: u32,
    game_over: bool,
}

impl SnakeSession {
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
        let pace = SnakePace::from_stage_id(&stage_id);
        let center = Point {
            x: BOARD_WIDTH / 2,
            y: BOARD_HEIGHT / 2,
        };
        let snake = vec![
            center,
            Point {
                x: center.x - 1,
                y: center.y,
            },
            Point {
                x: center.x - 2,
                y: center.y,
            },
        ];
        let started_instant = clock.now();
        let mut session = Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant,
            clock,
            rng: StdRng::seed_from_u64(seed),
            snake,
            food: center,
            direction: Direction::Right,
            pending_direction: Direction::Right,
            pace,
            started: false,
            last_move: started_instant,
            move_interval: pace.move_interval(),
            score: 0,
            game_over: false,
        };
        session.food = session.random_food();
        session
    }

    pub fn snake(&self) -> &[Point] {
        &self.snake
    }

    pub fn food(&self) -> Point {
        self.food
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn pace(&self) -> SnakePace {
        self.pace
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn set_direction(&mut self, direction: Direction) {
        if !direction.is_opposite(self.direction) {
            self.pending_direction = direction;
            if !self.started {
                self.direction = direction;
            }
            self.started = true;
        }
    }

    pub fn tick(&mut self) -> Option<RoundResult> {
        if self.game_over
            || !self.started
            || self.clock.now().saturating_duration_since(self.last_move) < self.move_interval
        {
            return None;
        }

        self.last_move = self.clock.now();
        self.direction = self.pending_direction;
        let (dx, dy) = self.direction.offset();
        let head = self.snake[0];
        let next = Point {
            x: head.x + dx,
            y: head.y + dy,
        };
        let hits_wall = next.x < 0 || next.x >= BOARD_WIDTH || next.y < 0 || next.y >= BOARD_HEIGHT;
        let hits_body = self.snake.contains(&next)
            && !(next == *self.snake.last().expect("snake has a tail") && next != self.food);
        if hits_wall || hits_body {
            self.game_over = true;
            return Some(self.result(RoundStatus::Completed));
        }

        self.snake.insert(0, next);
        if next == self.food {
            self.score += 1;
            self.food = self.random_food();
        } else {
            self.snake.pop();
        }
        None
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.game_over {
            None
        } else {
            self.game_over = true;
            Some(self.result(RoundStatus::Abandoned))
        }
    }

    fn random_food(&mut self) -> Point {
        let available: Vec<_> = (0..BOARD_HEIGHT)
            .flat_map(|y| (0..BOARD_WIDTH).map(move |x| Point { x, y }))
            .filter(|point| !self.snake.contains(point))
            .collect();
        if available.is_empty() {
            return self.snake[0];
        }
        available[self.rng.gen_range(0..available.len())]
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: self.score,
            attempts: self.score,
            accuracy: if self.score == 0 { 0.0 } else { 1.0 },
            best_streak: self.score,
            score: self.score,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    struct FakeClock(Mutex<Instant>);

    impl FakeClock {
        fn new() -> Self {
            Self(Mutex::new(Instant::now()))
        }

        fn advance(&self, duration: Duration) {
            *self.0.lock().expect("clock lock") += duration;
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> Instant {
            *self.0.lock().expect("clock lock")
        }
    }

    #[test]
    fn snake_starts_with_three_segments_and_food_outside_the_snake() {
        let session = SnakeSession::with_seed(GameId::new("snake"), StageId::new("classic-1"), 10);
        assert_eq!(session.snake().len(), INITIAL_SNAKE_LENGTH);
        assert!(!session.snake().contains(&session.food()));
    }

    #[test]
    fn stages_change_the_actual_move_interval() {
        let relaxed = SnakeSession::with_seed(GameId::new("snake"), StageId::new("relaxed"), 1);
        let classic = SnakeSession::with_seed(GameId::new("snake"), StageId::new("classic-1"), 1);
        let turbo = SnakeSession::with_seed(GameId::new("snake"), StageId::new("turbo"), 1);
        assert!(relaxed.move_interval > classic.move_interval);
        assert!(classic.move_interval > turbo.move_interval);
        assert_eq!(turbo.pace(), SnakePace::Turbo);
    }

    #[test]
    fn the_snake_waits_for_input_and_ignores_an_opposite_direction() {
        let clock = Arc::new(FakeClock::new());
        let mut session = SnakeSession::with_seed_and_clock(
            GameId::new("snake"),
            StageId::new("classic-1"),
            11,
            clock.clone(),
        );
        let original_head = session.snake()[0];
        session.set_direction(Direction::Left);
        clock.advance(DEFAULT_MOVE_INTERVAL);
        assert!(session.tick().is_none());
        assert_eq!(session.direction(), Direction::Right);
        assert_eq!(session.snake()[0], original_head);

        session.set_direction(Direction::Up);
        clock.advance(DEFAULT_MOVE_INTERVAL);
        assert!(session.tick().is_none());
        assert_eq!(session.direction(), Direction::Up);
        assert_eq!(session.snake()[0].y, original_head.y - 1);
    }

    #[test]
    fn hitting_the_wall_finishes_the_round_with_a_score() {
        let clock = Arc::new(FakeClock::new());
        let mut session = SnakeSession::with_seed_and_clock(
            GameId::new("snake"),
            StageId::new("classic-1"),
            12,
            clock.clone(),
        );
        session.set_direction(Direction::Right);
        for _ in 0..BOARD_WIDTH {
            clock.advance(DEFAULT_MOVE_INTERVAL);
            if session.tick().is_some() {
                break;
            }
        }
        assert!(session.is_game_over());
        let result = session.finish_abandoned();
        assert!(result.is_none());
    }
}
