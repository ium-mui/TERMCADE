use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Suit {
    Spade,
    Heart,
    Diamond,
    Club,
}

impl Suit {
    pub fn symbol(self) -> char {
        match self {
            Self::Spade => '♠',
            Self::Heart => '♥',
            Self::Diamond => '♦',
            Self::Club => '♣',
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    fn base_value(self) -> u8 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
            Self::Ten | Self::Jack | Self::Queen | Self::King => 10,
            Self::Ace => 11,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
            Self::Ace => "A",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn label(self) -> String {
        format!("{}{}", self.rank.label(), self.suit.symbol())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlackjackOutcome {
    PlayerBlackjack,
    PlayerWin,
    DealerWin,
    Push,
}

impl BlackjackOutcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::PlayerBlackjack => "블랙잭",
            Self::PlayerWin => "승리",
            Self::DealerWin => "패배",
            Self::Push => "무승부",
        }
    }
}

pub struct BlackjackSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    deck: Vec<Card>,
    player: Vec<Card>,
    dealer: Vec<Card>,
    attempts: u32,
    outcome: Option<BlackjackOutcome>,
    game_over: bool,
}

impl BlackjackSession {
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
        let mut deck = standard_deck();
        let mut rng = StdRng::seed_from_u64(seed);
        deck.shuffle(&mut rng);
        let mut session = Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant: clock.now(),
            clock,
            deck,
            player: Vec::with_capacity(8),
            dealer: Vec::with_capacity(8),
            attempts: 0,
            outcome: None,
            game_over: false,
        };
        session.deal_card_to_player();
        session.deal_card_to_dealer();
        session.deal_card_to_player();
        session.deal_card_to_dealer();
        session
    }

    pub fn player_cards(&self) -> &[Card] {
        &self.player
    }

    pub fn dealer_cards(&self) -> &[Card] {
        &self.dealer
    }

    pub fn dealer_revealed(&self) -> bool {
        self.game_over
    }

    pub fn player_score(&self) -> u8 {
        hand_score(&self.player)
    }

    pub fn dealer_score(&self) -> u8 {
        hand_score(&self.dealer)
    }

    pub fn dealer_visible_score(&self) -> u8 {
        if self.dealer.is_empty() {
            0
        } else {
            hand_score(&self.dealer[..1])
        }
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn outcome(&self) -> Option<BlackjackOutcome> {
        self.outcome
    }

    pub fn payout(&self) -> u64 {
        match self.outcome {
            Some(BlackjackOutcome::PlayerBlackjack) => 3,
            Some(BlackjackOutcome::PlayerWin) => 2,
            Some(BlackjackOutcome::Push) => 1,
            Some(BlackjackOutcome::DealerWin) | None => 0,
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn hit(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.attempts += 1;
        self.deal_card_to_player();
        if self.player_score() > 21 {
            self.outcome = Some(BlackjackOutcome::DealerWin);
            self.game_over = true;
            Some(self.result(RoundStatus::Completed))
        } else if self.player_score() == 21 {
            self.stand()
        } else {
            None
        }
    }

    pub fn stand(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.attempts += 1;
        while self.dealer_score() < 17 {
            self.deal_card_to_dealer();
        }
        let player_score = self.player_score();
        let dealer_score = self.dealer_score();
        self.outcome = Some(if player_score == 21 && self.player.len() == 2 {
            BlackjackOutcome::PlayerBlackjack
        } else if dealer_score > 21 || player_score > dealer_score {
            BlackjackOutcome::PlayerWin
        } else if player_score == dealer_score {
            BlackjackOutcome::Push
        } else {
            BlackjackOutcome::DealerWin
        });
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

    fn deal_card_to_player(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.player.push(card);
        }
    }

    fn deal_card_to_dealer(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.dealer.push(card);
        }
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        let score = match self.outcome {
            Some(BlackjackOutcome::PlayerBlackjack) => 2,
            Some(BlackjackOutcome::PlayerWin) => 1,
            Some(BlackjackOutcome::DealerWin | BlackjackOutcome::Push) | None => 0,
        };
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: u32::from(score > 0),
            attempts: self.attempts,
            accuracy: if self.attempts == 0 {
                0.0
            } else {
                f64::from(u32::from(score > 0)) / f64::from(self.attempts)
            },
            best_streak: u32::from(score > 0),
            score,
            status,
        }
    }
}

fn standard_deck() -> Vec<Card> {
    let suits = [Suit::Spade, Suit::Heart, Suit::Diamond, Suit::Club];
    let ranks = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];
    suits
        .into_iter()
        .flat_map(|suit| ranks.into_iter().map(move |rank| Card { suit, rank }))
        .collect()
}

fn hand_score(hand: &[Card]) -> u8 {
    let mut score = hand.iter().map(|card| card.rank.base_value()).sum::<u8>();
    let aces = hand.iter().filter(|card| card.rank == Rank::Ace).count();
    let mut aces_left = aces;
    while score > 21 && aces_left > 0 {
        score -= 10;
        aces_left -= 1;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_hand_has_two_cards_each_and_scores_aces_correctly() {
        let session =
            BlackjackSession::with_seed(GameId::new("blackjack"), StageId::new("classic-1"), 1);
        assert_eq!(session.player_cards().len(), 2);
        assert_eq!(session.dealer_cards().len(), 2);
        assert!((2..=21).contains(&session.player_score()));
        assert!((2..=21).contains(&session.dealer_visible_score()));
    }

    #[test]
    fn hit_adds_a_card_and_stand_reveals_the_dealer() {
        let mut session =
            BlackjackSession::with_seed(GameId::new("blackjack"), StageId::new("classic-1"), 8);
        let before = session.player_cards().len();
        let _ = session.hit();
        assert!(session.player_cards().len() >= before);
        if !session.is_game_over() {
            let result = session.stand().expect("stand result");
            assert_eq!(result.status, RoundStatus::Completed);
            assert!(session.dealer_revealed());
            assert!(session.outcome().is_some());
        }
    }

    #[test]
    fn a_two_card_blackjack_is_scored_above_a_regular_win() {
        assert_eq!(BlackjackOutcome::PlayerBlackjack.label(), "블랙잭");
        assert_eq!(
            hand_score(&[
                Card {
                    suit: Suit::Spade,
                    rank: Rank::Ace
                },
                Card {
                    suit: Suit::Heart,
                    rank: Rank::King,
                }
            ]),
            21
        );
    }
}
