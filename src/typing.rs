use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

pub const TYPING_SENTENCES: &[&str] = &[
    "오늘도 천천히 정확하게 입력합니다.",
    "작은 도전이 큰 기록을 만듭니다.",
    "즐겁게 연습하면 실력이 자랍니다.",
    "정확한 타자가 가장 빠른 타자입니다.",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypingSubmission {
    Empty,
    Correct,
    Incorrect,
}

pub struct TypingPracticeSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    rng: StdRng,
    sentence: &'static str,
    input: String,
    attempts: u32,
    correct_answers: u32,
    last_correct: Option<bool>,
    game_over: bool,
}

impl TypingPracticeSession {
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
        let sentence = next_sentence(&mut rng);
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant: clock.now(),
            clock,
            rng,
            sentence,
            input: String::new(),
            attempts: 0,
            correct_answers: 0,
            last_correct: None,
            game_over: false,
        }
    }

    pub fn sentence(&self) -> &str {
        self.sentence
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn correct_answers(&self) -> u32 {
        self.correct_answers
    }

    pub fn earned_won(&self) -> u64 {
        u64::from(self.correct_answers)
    }

    pub fn last_correct(&self) -> Option<bool> {
        self.last_correct
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn push_char(&mut self, character: char) {
        if !self.game_over && !character.is_control() {
            self.input.push(character);
        }
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    pub fn submit(&mut self) -> TypingSubmission {
        if self.game_over || self.input.is_empty() {
            return TypingSubmission::Empty;
        }
        self.attempts += 1;
        let correct = self.input == self.sentence;
        self.last_correct = Some(correct);
        self.input.clear();
        if correct {
            self.correct_answers += 1;
            self.sentence = next_sentence(&mut self.rng);
            TypingSubmission::Correct
        } else {
            self.sentence = next_sentence(&mut self.rng);
            TypingSubmission::Incorrect
        }
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.game_over = true;
        Some(self.result(RoundStatus::Abandoned))
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
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
            best_streak: self.correct_answers,
            score: self.correct_answers,
            status,
        }
    }
}

fn next_sentence(rng: &mut StdRng) -> &'static str {
    TYPING_SENTENCES[rng.gen_range(0..TYPING_SENTENCES.len())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_exact_sentence_earns_one_won() {
        let mut session = TypingPracticeSession::with_seed(
            GameId::new("gambling"),
            StageId::new("typing-mine"),
            5,
        );
        for character in "틀린 문장".chars() {
            session.push_char(character);
        }
        assert_eq!(session.submit(), TypingSubmission::Incorrect);
        assert_eq!(session.earned_won(), 0);

        let sentence = session.sentence().to_string();
        for character in sentence.chars() {
            session.push_char(character);
        }
        assert_eq!(session.submit(), TypingSubmission::Correct);
        assert_eq!(session.earned_won(), 1);
    }
}
