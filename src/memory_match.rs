use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryDifficulty {
    Small,
    Classic,
    Grand,
}

impl MemoryDifficulty {
    pub fn from_stage_id(stage_id: &StageId) -> Self {
        match stage_id.as_str() {
            "classic" => Self::Classic,
            "grand" => Self::Grand,
            _ => Self::Small,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "SMALL",
            Self::Classic => "CLASSIC",
            Self::Grand => "GRAND",
        }
    }

    fn dimensions(self) -> (usize, usize) {
        match self {
            Self::Small => (4, 3),
            Self::Classic => (4, 4),
            Self::Grand => (6, 4),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryCardState {
    Hidden,
    Revealed,
    Matched,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryCard {
    pub symbol: u8,
    pub state: MemoryCardState,
}

pub struct MemoryMatchSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    difficulty: MemoryDifficulty,
    width: usize,
    height: usize,
    cards: Vec<MemoryCard>,
    cursor: usize,
    first_pick: Option<usize>,
    mismatch: Option<[usize; 2]>,
    matched_pairs: u32,
    moves: u32,
    game_over: bool,
}

impl MemoryMatchSession {
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
        let difficulty = MemoryDifficulty::from_stage_id(&stage_id);
        let (width, height) = difficulty.dimensions();
        let pair_count = width * height / 2;
        let mut symbols: Vec<u8> = (0..pair_count as u8)
            .flat_map(|symbol| [symbol, symbol])
            .collect();
        symbols.shuffle(&mut StdRng::seed_from_u64(seed));
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant: clock.now(),
            clock,
            difficulty,
            width,
            height,
            cards: symbols
                .into_iter()
                .map(|symbol| MemoryCard {
                    symbol,
                    state: MemoryCardState::Hidden,
                })
                .collect(),
            cursor: 0,
            first_pick: None,
            mismatch: None,
            matched_pairs: 0,
            moves: 0,
            game_over: false,
        }
    }

    pub fn difficulty(&self) -> MemoryDifficulty {
        self.difficulty
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cards(&self) -> &[MemoryCard] {
        &self.cards
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn matched_pairs(&self) -> u32 {
        self.matched_pairs
    }

    pub fn pair_count(&self) -> usize {
        self.cards.len() / 2
    }

    pub fn moves(&self) -> u32 {
        self.moves
    }

    pub fn awaiting_continue(&self) -> bool {
        self.mismatch.is_some()
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
        if self.game_over || self.mismatch.is_some() {
            return;
        }
        let x = (self.cursor % self.width) as i32;
        let y = (self.cursor / self.width) as i32;
        let next_x = (x + dx).clamp(0, self.width as i32 - 1);
        let next_y = (y + dy).clamp(0, self.height as i32 - 1);
        self.cursor = next_y as usize * self.width + next_x as usize;
    }

    pub fn select(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        if let Some([first, second]) = self.mismatch.take() {
            self.cards[first].state = MemoryCardState::Hidden;
            self.cards[second].state = MemoryCardState::Hidden;
            return None;
        }
        if self.cards[self.cursor].state != MemoryCardState::Hidden {
            return None;
        }

        self.cards[self.cursor].state = MemoryCardState::Revealed;
        let Some(first) = self.first_pick.take() else {
            self.first_pick = Some(self.cursor);
            return None;
        };
        self.moves += 1;
        if self.cards[first].symbol == self.cards[self.cursor].symbol {
            self.cards[first].state = MemoryCardState::Matched;
            self.cards[self.cursor].state = MemoryCardState::Matched;
            self.matched_pairs += 1;
            if self.matched_pairs as usize == self.pair_count() {
                self.game_over = true;
                return Some(self.result(RoundStatus::Completed));
            }
        } else {
            self.mismatch = Some([first, self.cursor]);
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

    fn result(&self, status: RoundStatus) -> RoundResult {
        let score = self
            .matched_pairs
            .saturating_mul(150)
            .saturating_sub(self.moves.saturating_sub(self.matched_pairs) * 10);
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: self.matched_pairs,
            attempts: self.moves,
            accuracy: if self.moves == 0 {
                0.0
            } else {
                f64::from(self.matched_pairs) / f64::from(self.moves)
            },
            best_streak: self.matched_pairs,
            score,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_symbol_appears_exactly_twice() {
        let session =
            MemoryMatchSession::with_seed(GameId::new("memory"), StageId::new("grand"), 2);
        for symbol in 0..session.pair_count() as u8 {
            assert_eq!(
                session
                    .cards()
                    .iter()
                    .filter(|card| card.symbol == symbol)
                    .count(),
                2
            );
        }
    }

    #[test]
    fn mismatch_waits_for_explicit_continue_before_hiding() {
        let mut session =
            MemoryMatchSession::with_seed(GameId::new("memory"), StageId::new("small"), 8);
        let first = 0;
        let second = (1..session.cards.len())
            .find(|index| session.cards[*index].symbol != session.cards[first].symbol)
            .expect("different card");
        session.cursor = first;
        session.select();
        session.cursor = second;
        session.select();
        assert!(session.awaiting_continue());
        assert_eq!(session.cards[first].state, MemoryCardState::Revealed);
        session.select();
        assert!(!session.awaiting_continue());
        assert_eq!(session.cards[first].state, MemoryCardState::Hidden);
    }

    #[test]
    fn matching_pair_stays_visible() {
        let mut session =
            MemoryMatchSession::with_seed(GameId::new("memory"), StageId::new("small"), 5);
        let first = 0;
        let second = (1..session.cards.len())
            .find(|index| session.cards[*index].symbol == session.cards[first].symbol)
            .expect("pair");
        session.cursor = first;
        session.select();
        session.cursor = second;
        session.select();
        assert_eq!(session.matched_pairs(), 1);
        assert_eq!(session.cards[first].state, MemoryCardState::Matched);
    }

    #[test]
    fn matching_every_pair_finishes_exactly_on_the_last_pair() {
        let mut session =
            MemoryMatchSession::with_seed(GameId::new("memory"), StageId::new("small"), 42);
        let pairs: Vec<[usize; 2]> = (0..session.pair_count() as u8)
            .map(|symbol| {
                let positions: Vec<_> = session
                    .cards
                    .iter()
                    .enumerate()
                    .filter_map(|(index, card)| (card.symbol == symbol).then_some(index))
                    .collect();
                [positions[0], positions[1]]
            })
            .collect();
        for (pair_index, [first, second]) in pairs.into_iter().enumerate() {
            session.cursor = first;
            assert!(session.select().is_none());
            session.cursor = second;
            let result = session.select();
            assert_eq!(result.is_some(), pair_index + 1 == session.pair_count());
        }
        assert!(session.is_game_over());
    }
}
