use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

pub const BOARD_WIDTH: i32 = 18;
pub const BOARD_HEIGHT: i32 = 12;
pub const BRICK_ROWS: usize = 4;
pub const BRICK_COLUMNS: usize = 6;
pub const PADDLE_WIDTH: i32 = 4;
pub const PADDLE_Y: i32 = BOARD_HEIGHT - 1;
pub const INITIAL_LIVES: u32 = 3;
pub const DEFAULT_MOVE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct BreakoutSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    bricks: [[bool; BRICK_COLUMNS]; BRICK_ROWS],
    ball: Point,
    velocity_x: i32,
    velocity_y: i32,
    paddle_x: i32,
    score: u32,
    bricks_destroyed: u32,
    lives: u32,
    last_move: Instant,
    move_interval: Duration,
    game_over: bool,
}

impl BreakoutSession {
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
        let mut rng = StdRng::seed_from_u64(seed);
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant,
            clock,
            bricks: [[true; BRICK_COLUMNS]; BRICK_ROWS],
            ball: Point {
                x: BOARD_WIDTH / 2,
                y: PADDLE_Y - 1,
            },
            velocity_x: if rng.gen_bool(0.5) { 1 } else { -1 },
            velocity_y: -1,
            paddle_x: (BOARD_WIDTH - PADDLE_WIDTH) / 2,
            score: 0,
            bricks_destroyed: 0,
            lives: INITIAL_LIVES,
            last_move: started_instant,
            move_interval: DEFAULT_MOVE_INTERVAL,
            game_over: false,
        }
    }

    pub fn ball(&self) -> Point {
        self.ball
    }

    pub fn paddle_x(&self) -> i32 {
        self.paddle_x
    }

    pub fn paddle_y(&self) -> i32 {
        PADDLE_Y
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn lives(&self) -> u32 {
        self.lives
    }

    pub fn bricks_remaining(&self) -> u32 {
        self.bricks.iter().flatten().filter(|brick| **brick).count() as u32
    }

    pub fn brick_at(&self, x: i32, y: i32) -> bool {
        if !(0..BOARD_WIDTH).contains(&x) || !(0..BRICK_ROWS as i32).contains(&y) {
            return false;
        }
        self.bricks[y as usize][(x as usize * BRICK_COLUMNS) / BOARD_WIDTH as usize]
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn move_paddle(&mut self, delta: i32) {
        if self.game_over {
            return;
        }
        self.paddle_x = (self.paddle_x + delta).clamp(0, BOARD_WIDTH - PADDLE_WIDTH);
    }

    pub fn tick(&mut self) -> Option<RoundResult> {
        if self.game_over
            || self.clock.now().saturating_duration_since(self.last_move) < self.move_interval
        {
            return None;
        }

        let now = self.clock.now();
        self.last_move = now;
        let mut next = Point {
            x: self.ball.x + self.velocity_x,
            y: self.ball.y + self.velocity_y,
        };

        if next.x < 0 || next.x >= BOARD_WIDTH {
            self.velocity_x = -self.velocity_x;
            next.x = self.ball.x + self.velocity_x;
        }
        if next.y < 0 {
            self.velocity_y = -self.velocity_y;
            next.y = self.ball.y + self.velocity_y;
        }

        if self.velocity_y > 0
            && next.y == PADDLE_Y
            && (self.paddle_x..self.paddle_x + PADDLE_WIDTH).contains(&next.x)
        {
            self.velocity_y = -1;
            next.y = PADDLE_Y - 1;
        }

        if (0..BRICK_ROWS as i32).contains(&next.y) && (0..BOARD_WIDTH).contains(&next.x) {
            let row = next.y as usize;
            let column = (next.x as usize * BRICK_COLUMNS) / BOARD_WIDTH as usize;
            if self.bricks[row][column] {
                self.bricks[row][column] = false;
                self.score += 10;
                self.bricks_destroyed += 1;
                self.velocity_y = -self.velocity_y;
                next.y = self.ball.y + self.velocity_y;
                if self.bricks_remaining() == 0 {
                    self.game_over = true;
                    return Some(self.result(RoundStatus::Completed));
                }
            }
        }

        if next.y >= BOARD_HEIGHT {
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.game_over = true;
                return Some(self.result(RoundStatus::Completed));
            }
            self.reset_ball();
            return None;
        }

        self.ball = next;
        None
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.game_over = true;
        Some(self.result(RoundStatus::Abandoned))
    }

    fn reset_ball(&mut self) {
        self.ball = Point {
            x: BOARD_WIDTH / 2,
            y: PADDLE_Y - 1,
        };
        self.velocity_y = -1;
        self.last_move = self.clock.now();
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: self.bricks_destroyed,
            attempts: self.bricks_destroyed,
            accuracy: if self.bricks_destroyed == 0 { 0.0 } else { 1.0 },
            best_streak: self.bricks_destroyed,
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

    fn session(clock: Arc<FakeClock>) -> BreakoutSession {
        BreakoutSession::with_seed_and_clock(
            GameId::new("breakout"),
            StageId::new("classic-1"),
            21,
            clock,
        )
    }

    #[test]
    fn new_game_has_a_full_wall_and_three_lives() {
        let session = session(Arc::new(FakeClock::new()));
        assert_eq!(
            session.bricks_remaining(),
            (BRICK_ROWS * BRICK_COLUMNS) as u32
        );
        assert_eq!(session.lives(), INITIAL_LIVES);
        assert_eq!(session.paddle_y(), PADDLE_Y);
    }

    #[test]
    fn paddle_stays_inside_the_board() {
        let mut session = session(Arc::new(FakeClock::new()));
        session.move_paddle(-100);
        assert_eq!(session.paddle_x(), 0);
        session.move_paddle(100);
        assert_eq!(session.paddle_x(), BOARD_WIDTH - PADDLE_WIDTH);
    }

    #[test]
    fn the_ball_breaks_a_brick_after_the_move_interval() {
        let clock = Arc::new(FakeClock::new());
        let mut session = session(clock.clone());
        session.bricks = [[false; BRICK_COLUMNS]; BRICK_ROWS];
        session.bricks[3][0] = true;
        session.ball = Point { x: 1, y: 4 };
        session.velocity_x = 1;
        session.velocity_y = -1;
        clock.advance(DEFAULT_MOVE_INTERVAL);

        let result = session.tick().expect("last brick result");
        assert_eq!(session.score(), 10);
        assert_eq!(session.bricks_remaining(), 0);
        assert!(session.is_game_over());
        assert_eq!(result.status, RoundStatus::Completed);
    }
}
