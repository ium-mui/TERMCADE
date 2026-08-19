use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotSymbol {
    Cherry,
    Lemon,
    Bell,
    Seven,
}

impl SlotSymbol {
    pub fn label(self) -> &'static str {
        match self {
            Self::Cherry => "🍒",
            Self::Lemon => "🍋",
            Self::Bell => "🔔",
            Self::Seven => "7",
        }
    }
}

pub struct SlotsSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    rng: StdRng,
    symbols: [SlotSymbol; 3],
    payout: u64,
    attempts: u32,
    game_over: bool,
}

impl SlotsSession {
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
            symbols: [SlotSymbol::Cherry; 3],
            payout: 0,
            attempts: 0,
            game_over: false,
        }
    }

    pub fn symbols(&self) -> [SlotSymbol; 3] {
        self.symbols
    }

    pub fn payout(&self) -> u64 {
        self.payout
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn pull(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        let symbols = [
            random_symbol(&mut self.rng),
            random_symbol(&mut self.rng),
            random_symbol(&mut self.rng),
        ];
        self.symbols = symbols;
        self.attempts = 1;
        self.payout = if symbols.iter().all(|symbol| *symbol == SlotSymbol::Seven) {
            20
        } else if symbols.iter().all(|symbol| *symbol == symbols[0]) {
            8
        } else if symbols[0] == symbols[1] || symbols[0] == symbols[2] || symbols[1] == symbols[2] {
            2
        } else {
            0
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

fn random_symbol(rng: &mut StdRng) -> SlotSymbol {
    match rng.gen_range(0..4) {
        0 => SlotSymbol::Cherry,
        1 => SlotSymbol::Lemon,
        2 => SlotSymbol::Bell,
        _ => SlotSymbol::Seven,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulling_slots_finishes_one_round_and_uses_a_payout_table() {
        let mut session =
            SlotsSession::with_seed(GameId::new("gambling"), StageId::new("slots-1"), 4);
        let result = session.pull().expect("pull result");
        assert_eq!(result.attempts, 1);
        assert!(session.is_game_over());
        assert!(session.payout() <= 20);
    }
}
