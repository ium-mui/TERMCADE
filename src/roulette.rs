use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouletteColor {
    Red,
    Black,
    Green,
}

pub const WHEEL_NUMBERS: [u8; 38] = [
    0, 28, 9, 26, 30, 11, 7, 20, 32, 17, 5, 22, 34, 15, 3, 24, 36, 13, 1, 37, 27, 10, 25, 29, 12,
    8, 19, 31, 18, 6, 21, 33, 16, 4, 23, 35, 14, 2,
];

impl RouletteColor {
    pub fn for_number(number: u8) -> Self {
        if number == 0 || number == 37 {
            return Self::Green;
        }
        match number {
            1 | 3 | 5 | 7 | 9 | 12 | 14 | 16 | 18 | 19 | 21 | 23 | 25 | 27 | 30 | 32 | 34 | 36 => {
                Self::Red
            }
            _ => Self::Black,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Red => "빨강",
            Self::Black => "검정",
            Self::Green => "초록",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Self::Red => "R",
            Self::Black => "B",
            Self::Green => "G",
        }
    }
}

pub struct RouletteSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    rng: StdRng,
    choice: Option<RouletteColor>,
    result_number: Option<u8>,
    result_color: Option<RouletteColor>,
    payout: u64,
    attempts: u32,
    game_over: bool,
}

impl RouletteSession {
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
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant,
            clock,
            rng: StdRng::seed_from_u64(seed),
            choice: None,
            result_number: None,
            result_color: None,
            payout: 0,
            attempts: 0,
            game_over: false,
        }
    }

    pub fn choose(&mut self, color: RouletteColor) {
        if !self.game_over {
            self.choice = Some(color);
        }
    }

    pub fn choice(&self) -> Option<RouletteColor> {
        self.choice
    }

    pub fn result_color(&self) -> Option<RouletteColor> {
        self.result_color
    }

    pub fn result_number(&self) -> Option<u8> {
        self.result_number
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn payout(&self) -> u64 {
        self.payout
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn spin(&mut self) -> Option<RoundResult> {
        if self.game_over || self.choice.is_none() {
            return None;
        }
        self.attempts = 1;
        let number = self.rng.gen_range(0..38);
        let result_color = RouletteColor::for_number(number);
        self.result_number = Some(number);
        self.result_color = Some(result_color);
        self.payout = match (self.choice.expect("choice exists"), result_color) {
            (RouletteColor::Green, RouletteColor::Green) => 36,
            (RouletteColor::Red, RouletteColor::Red)
            | (RouletteColor::Black, RouletteColor::Black) => 2,
            _ => 0,
        };
        self.game_over = true;
        Some(self.result(RoundStatus::Completed))
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.game_over = true;
        Some(self.result(RoundStatus::Abandoned))
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        let won = self.payout > 0;
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: u32::from(won),
            attempts: self.attempts,
            accuracy: if self.attempts == 0 {
                0.0
            } else {
                f64::from(u32::from(won)) / f64::from(self.attempts)
            },
            best_streak: u32::from(won),
            score: self.payout as u32,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_roulette_spin_requires_a_choice_and_records_the_result() {
        let mut session =
            RouletteSession::with_seed(GameId::new("gambling"), StageId::new("roulette-1"), 3);
        assert!(session.spin().is_none());
        session.choose(RouletteColor::Red);
        let result = session.spin().expect("spin result");
        assert_eq!(result.attempts, 1);
        assert!(session.result_number().is_some());
        assert!(session.result_color().is_some());
        assert!(session.is_game_over());
    }
}
