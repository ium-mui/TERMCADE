use std::cmp::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::blackjack::{Card, Rank, Suit};
use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HoldemDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
}

impl HoldemDifficulty {
    pub fn label(self) -> &'static str {
        match self {
            Self::Easy => "쉬움",
            Self::Normal => "보통",
            Self::Hard => "어려움",
            Self::Expert => "최상",
        }
    }

    fn random(rng: &mut StdRng) -> Self {
        match rng.gen_range(0..4) {
            0 => Self::Easy,
            1 => Self::Normal,
            2 => Self::Hard,
            _ => Self::Expert,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HoldemStreet {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
}

impl HoldemStreet {
    pub fn label(self) -> &'static str {
        match self {
            Self::PreFlop => "프리플랍",
            Self::Flop => "플랍",
            Self::Turn => "턴",
            Self::River => "리버",
            Self::Showdown => "쇼다운",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HoldemAction {
    CheckCall,
    Raise(u32),
    Fold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HoldemAiAction {
    Check,
    Call,
    Raise,
    Fold,
}

impl HoldemAiAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Check => "체크",
            Self::Call => "콜",
            Self::Raise => "레이즈",
            Self::Fold => "폴드",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HoldemOutcome {
    PlayerWin,
    AiWin,
    Tie,
    PlayerFold,
}

impl HoldemOutcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::PlayerWin => "승리",
            Self::AiWin => "패배",
            Self::Tie => "무승부",
            Self::PlayerFold => "폴드",
        }
    }
}

pub struct HoldemSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    rng: StdRng,
    deck: Vec<Card>,
    player: Vec<Card>,
    opponent: Vec<Card>,
    community: Vec<Card>,
    street: HoldemStreet,
    difficulty: HoldemDifficulty,
    last_ai_action: Option<HoldemAiAction>,
    outcome: Option<HoldemOutcome>,
    payout: u64,
    pot: u32,
    attempts: u32,
    game_over: bool,
}

impl HoldemSession {
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
        let mut rng = StdRng::seed_from_u64(seed);
        let mut deck = standard_deck();
        deck.shuffle(&mut rng);
        let difficulty = HoldemDifficulty::random(&mut rng);
        let mut session = Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant: clock.now(),
            clock,
            rng,
            deck,
            player: Vec::with_capacity(2),
            opponent: Vec::with_capacity(2),
            community: Vec::with_capacity(5),
            street: HoldemStreet::PreFlop,
            difficulty,
            last_ai_action: None,
            outcome: None,
            payout: 0,
            pot: 20,
            attempts: 0,
            game_over: false,
        };
        session.deal_to_player();
        session.deal_to_opponent();
        session.deal_to_player();
        session.deal_to_opponent();
        session
    }

    pub fn player_cards(&self) -> &[Card] {
        &self.player
    }

    pub fn opponent_cards(&self) -> &[Card] {
        &self.opponent
    }

    pub fn community_cards(&self) -> &[Card] {
        &self.community
    }

    pub fn street(&self) -> HoldemStreet {
        self.street
    }

    pub fn difficulty(&self) -> HoldemDifficulty {
        self.difficulty
    }

    pub fn last_ai_action(&self) -> Option<HoldemAiAction> {
        self.last_ai_action
    }

    pub fn outcome(&self) -> Option<HoldemOutcome> {
        self.outcome
    }

    pub fn payout(&self) -> u64 {
        self.payout
    }

    pub fn pot(&self) -> u32 {
        self.pot
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

    pub fn player_hand_name(&self) -> Option<&'static str> {
        (self.community.len() == 5).then(|| evaluate_seven(&self.player, &self.community).label())
    }

    pub fn opponent_hand_name(&self) -> Option<&'static str> {
        (self.community.len() == 5).then(|| evaluate_seven(&self.opponent, &self.community).label())
    }

    pub fn act(&mut self, action: HoldemAction) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.attempts += 1;
        if action == HoldemAction::Fold {
            self.last_ai_action = Some(HoldemAiAction::Check);
            self.outcome = Some(HoldemOutcome::PlayerFold);
            self.game_over = true;
            return Some(self.result(RoundStatus::Completed));
        }

        if let HoldemAction::Raise(amount) = action {
            self.pot = self.pot.saturating_add(amount);
        }
        let ai_action = self.choose_ai_action(action);
        self.last_ai_action = Some(ai_action);
        if ai_action == HoldemAiAction::Fold {
            self.outcome = Some(HoldemOutcome::PlayerWin);
            self.payout = 3;
            self.game_over = true;
            return Some(self.result(RoundStatus::Completed));
        }
        if ai_action == HoldemAiAction::Raise {
            self.pot = self.pot.saturating_add(10);
        }

        match self.street {
            HoldemStreet::PreFlop => {
                self.deal_community(3);
                self.street = HoldemStreet::Flop;
                None
            }
            HoldemStreet::Flop => {
                self.deal_community(1);
                self.street = HoldemStreet::Turn;
                None
            }
            HoldemStreet::Turn => {
                self.deal_community(1);
                self.street = HoldemStreet::River;
                None
            }
            HoldemStreet::River => Some(self.showdown()),
            HoldemStreet::Showdown => None,
        }
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.game_over = true;
        Some(self.result(RoundStatus::Abandoned))
    }

    fn choose_ai_action(&mut self, player_action: HoldemAction) -> HoldemAiAction {
        let strength = self.ai_strength();
        let (fold_threshold, fold_chance, raise_threshold, raise_chance, bluff_chance) =
            match self.difficulty {
                HoldemDifficulty::Easy => (0.18, 0.14, 0.80, 0.24, 0.02),
                HoldemDifficulty::Normal => (0.26, 0.28, 0.70, 0.40, 0.04),
                HoldemDifficulty::Hard => (0.34, 0.46, 0.60, 0.60, 0.07),
                HoldemDifficulty::Expert => (0.42, 0.64, 0.50, 0.78, 0.10),
            };
        let facing_raise = matches!(player_action, HoldemAction::Raise(_));
        if facing_raise {
            if strength < fold_threshold && self.rng.gen_bool(fold_chance) {
                return HoldemAiAction::Fold;
            }
            if strength < fold_threshold + 0.10 && self.rng.gen_bool(fold_chance * 0.45) {
                return HoldemAiAction::Fold;
            }
        }
        let strong_enough_to_raise = strength >= raise_threshold;
        let bluff = !facing_raise && self.rng.gen_bool(bluff_chance);
        if strong_enough_to_raise || bluff {
            let pressure = if strong_enough_to_raise {
                (strength - raise_threshold)
                    .mul_add(0.60, raise_chance)
                    .min(0.95)
            } else {
                raise_chance * 0.5
            };
            if self.rng.gen_bool(pressure) {
                return HoldemAiAction::Raise;
            }
        }
        if facing_raise || self.street == HoldemStreet::PreFlop {
            HoldemAiAction::Call
        } else {
            HoldemAiAction::Check
        }
    }

    fn ai_strength(&self) -> f64 {
        if self.community.len() >= 3 {
            let value = evaluate_seven(&self.opponent, &self.community);
            return (made_hand_strength(value) + draw_strength(&self.opponent, &self.community))
                .min(1.0);
        }
        let first = rank_value(self.opponent[0].rank);
        let second = rank_value(self.opponent[1].rank);
        let high = f64::from(first.max(second)) / 14.0;
        let secondary = f64::from(first.min(second)) / 14.0;
        let pair = if first == second {
            0.24 + high * 0.16
        } else {
            0.0
        };
        let suited = if self.opponent[0].suit == self.opponent[1].suit {
            0.09
        } else {
            0.0
        };
        let connected = if first.abs_diff(second) <= 1 {
            0.10
        } else if first.abs_diff(second) <= 3 {
            0.05
        } else {
            0.0
        };
        let broadway = if first >= 10 && second >= 10 {
            0.08
        } else {
            0.0
        };
        (high * 0.38 + secondary * 0.16 + pair + suited + connected + broadway).min(1.0)
    }

    fn showdown(&mut self) -> RoundResult {
        self.street = HoldemStreet::Showdown;
        let player_value = evaluate_seven(&self.player, &self.community);
        let opponent_value = evaluate_seven(&self.opponent, &self.community);
        self.outcome = Some(match player_value.cmp(&opponent_value) {
            Ordering::Greater => {
                self.payout = 3;
                HoldemOutcome::PlayerWin
            }
            Ordering::Less => HoldemOutcome::AiWin,
            Ordering::Equal => {
                self.payout = 1;
                HoldemOutcome::Tie
            }
        });
        self.game_over = true;
        self.result(RoundStatus::Completed)
    }

    fn deal_to_player(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.player.push(card);
        }
    }

    fn deal_to_opponent(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.opponent.push(card);
        }
    }

    fn deal_community(&mut self, count: usize) {
        for _ in 0..count {
            if let Some(card) = self.deck.pop() {
                self.community.push(card);
            }
        }
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        let won = matches!(self.outcome, Some(HoldemOutcome::PlayerWin));
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct HandValue {
    category: u8,
    tiebreak: [u8; 5],
}

impl HandValue {
    fn label(self) -> &'static str {
        match self.category {
            8 => "스트레이트 플러시",
            7 => "포카드",
            6 => "풀하우스",
            5 => "플러시",
            4 => "스트레이트",
            3 => "트리플",
            2 => "투페어",
            1 => "원페어",
            _ => "하이카드",
        }
    }
}

fn evaluate_seven(hole: &[Card], community: &[Card]) -> HandValue {
    let cards = [
        hole[0],
        hole[1],
        community[0],
        community[1],
        community[2],
        community.get(3).copied().unwrap_or(community[2]),
        community.get(4).copied().unwrap_or(community[2]),
    ];
    let count = 5 + community.len().saturating_sub(3);
    let mut best = None;
    for a in 0..count {
        for b in (a + 1)..count {
            for c in (b + 1)..count {
                for d in (c + 1)..count {
                    for e in (d + 1)..count {
                        let value =
                            evaluate_five([cards[a], cards[b], cards[c], cards[d], cards[e]]);
                        if best.is_none_or(|current| value > current) {
                            best = Some(value);
                        }
                    }
                }
            }
        }
    }
    best.expect("a hold'em hand always has five available cards")
}

fn evaluate_five(cards: [Card; 5]) -> HandValue {
    let mut counts = [0u8; 15];
    for card in cards {
        counts[usize::from(rank_value(card.rank))] += 1;
    }
    let flush = cards.iter().all(|card| card.suit == cards[0].suit);
    let mut distinct: Vec<u8> = (2u8..=14)
        .rev()
        .filter(|rank| counts[usize::from(*rank)] > 0)
        .collect();
    let straight_high = if distinct == [14, 5, 4, 3, 2] {
        Some(5)
    } else if distinct.len() == 5 && distinct.windows(2).all(|window| window[0] == window[1] + 1) {
        Some(distinct[0])
    } else {
        None
    };
    if flush && let Some(high) = straight_high {
        return HandValue {
            category: 8,
            tiebreak: [high, 0, 0, 0, 0],
        };
    }
    if let Some(quad) = (2u8..=14)
        .rev()
        .find(|rank| counts[usize::from(*rank)] == 4)
    {
        let kicker = (2u8..=14)
            .rev()
            .find(|rank| counts[usize::from(*rank)] == 1)
            .unwrap_or(0);
        return HandValue {
            category: 7,
            tiebreak: [quad, kicker, 0, 0, 0],
        };
    }
    if let Some(triple) = (2u8..=14)
        .rev()
        .find(|rank| counts[usize::from(*rank)] == 3)
        && let Some(pair) = (2u8..=14)
            .rev()
            .find(|rank| *rank != triple && counts[usize::from(*rank)] >= 2)
    {
        return HandValue {
            category: 6,
            tiebreak: [triple, pair, 0, 0, 0],
        };
    }
    if flush {
        distinct.resize(5, 0);
        return HandValue {
            category: 5,
            tiebreak: [
                distinct[0],
                distinct[1],
                distinct[2],
                distinct[3],
                distinct[4],
            ],
        };
    }
    if let Some(high) = straight_high {
        return HandValue {
            category: 4,
            tiebreak: [high, 0, 0, 0, 0],
        };
    }
    if let Some(triple) = (2u8..=14)
        .rev()
        .find(|rank| counts[usize::from(*rank)] == 3)
    {
        let kickers: Vec<u8> = (2u8..=14)
            .rev()
            .filter(|rank| *rank != triple && counts[usize::from(*rank)] == 1)
            .collect();
        return HandValue {
            category: 3,
            tiebreak: [triple, kickers[0], kickers[1], 0, 0],
        };
    }
    let pairs: Vec<u8> = (2u8..=14)
        .rev()
        .filter(|rank| counts[usize::from(*rank)] == 2)
        .collect();
    if pairs.len() >= 2 {
        let kicker = (2u8..=14)
            .rev()
            .find(|rank| counts[usize::from(*rank)] == 1)
            .unwrap_or(0);
        return HandValue {
            category: 2,
            tiebreak: [pairs[0], pairs[1], kicker, 0, 0],
        };
    }
    if let Some(pair) = pairs.first().copied() {
        let kickers: Vec<u8> = (2u8..=14)
            .rev()
            .filter(|rank| *rank != pair && counts[usize::from(*rank)] == 1)
            .collect();
        return HandValue {
            category: 1,
            tiebreak: [pair, kickers[0], kickers[1], kickers[2], 0],
        };
    }
    distinct.resize(5, 0);
    HandValue {
        category: 0,
        tiebreak: [
            distinct[0],
            distinct[1],
            distinct[2],
            distinct[3],
            distinct[4],
        ],
    }
}

fn made_hand_strength(value: HandValue) -> f64 {
    let category_strength = match value.category {
        0 => 0.16,
        1 => 0.31,
        2 => 0.48,
        3 => 0.63,
        4 => 0.74,
        5 => 0.80,
        6 => 0.88,
        7 => 0.96,
        _ => 1.0,
    };
    category_strength + f64::from(value.tiebreak[0]) / 14.0 * 0.05
}

fn draw_strength(hole: &[Card], community: &[Card]) -> f64 {
    let mut cards = Vec::with_capacity(hole.len() + community.len());
    cards.extend_from_slice(hole);
    cards.extend_from_slice(community);

    let suits = [Suit::Spade, Suit::Heart, Suit::Diamond, Suit::Club];
    let max_suit_count = suits
        .iter()
        .map(|suit| cards.iter().filter(|card| card.suit == *suit).count())
        .max()
        .unwrap_or(0);
    let flush_draw = match max_suit_count {
        4 => 0.13,
        3 => 0.03,
        _ => 0.0,
    };

    let mut ranks: Vec<u8> = cards.iter().map(|card| rank_value(card.rank)).collect();
    ranks.sort_unstable();
    ranks.dedup();
    let best_straight_window = (5u8..=14)
        .map(|high| {
            if high == 5 {
                [14, 2, 3, 4, 5]
            } else {
                [high - 4, high - 3, high - 2, high - 1, high]
            }
        })
        .map(|window| window.iter().filter(|rank| ranks.contains(rank)).count())
        .max()
        .unwrap_or(0);
    let straight_draw = match best_straight_window {
        4 => 0.12,
        3 => 0.03,
        _ => 0.0,
    };

    flush_draw + straight_draw
}

fn rank_value(rank: Rank) -> u8 {
    match rank {
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
        Rank::Ace => 14,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_hand_has_two_hole_cards_and_random_difficulty() {
        let session =
            HoldemSession::with_seed(GameId::new("gambling"), StageId::new("holdem-1"), 7);
        assert_eq!(session.player_cards().len(), 2);
        assert_eq!(session.opponent_cards().len(), 2);
        assert!(session.community_cards().is_empty());
        assert!(matches!(
            session.difficulty(),
            HoldemDifficulty::Easy
                | HoldemDifficulty::Normal
                | HoldemDifficulty::Hard
                | HoldemDifficulty::Expert
        ));
    }

    #[test]
    fn four_non_fold_actions_reach_showdown_or_ai_fold() {
        let mut session =
            HoldemSession::with_seed(GameId::new("gambling"), StageId::new("holdem-1"), 11);
        for _ in 0..4 {
            if session.is_game_over() {
                break;
            }
            let _ = session.act(HoldemAction::CheckCall);
        }
        assert!(session.is_game_over());
        assert!(session.outcome().is_some());
    }

    #[test]
    fn raise_amount_is_added_to_the_pot() {
        let mut session =
            HoldemSession::with_seed(GameId::new("gambling"), StageId::new("holdem-1"), 17);
        let initial_pot = session.pot();
        let _ = session.act(HoldemAction::Raise(25));
        assert!(session.pot() >= initial_pot + 25);
    }

    #[test]
    fn royal_flush_beats_every_lower_hand() {
        let cards = [
            Card {
                suit: Suit::Spade,
                rank: Rank::Ace,
            },
            Card {
                suit: Suit::Spade,
                rank: Rank::King,
            },
            Card {
                suit: Suit::Spade,
                rank: Rank::Queen,
            },
            Card {
                suit: Suit::Spade,
                rank: Rank::Jack,
            },
            Card {
                suit: Suit::Spade,
                rank: Rank::Ten,
            },
        ];
        assert_eq!(evaluate_five(cards).label(), "스트레이트 플러시");
    }
}
