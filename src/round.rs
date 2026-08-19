use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::arithmetic::ArithmeticProblem;
use crate::domain::{GameId, StageDefinition, StageId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundStatus {
    Completed,
    TimedOut,
    Abandoned,
}

impl RoundStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Completed => "완료",
            Self::TimedOut => "시간 초과",
            Self::Abandoned => "중단",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundResult {
    pub game_id: GameId,
    pub stage_id: StageId,
    pub app_version: String,
    pub started_at: DateTime<Utc>,
    pub actual_play_time_ms: u64,
    pub correct_answers: u32,
    pub attempts: u32,
    pub accuracy: f64,
    pub best_streak: u32,
    pub score: u32,
    pub status: RoundStatus,
}

impl RoundResult {
    pub fn accuracy_percent(&self) -> u8 {
        (self.accuracy * 100.0).round() as u8
    }
}

pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

#[derive(Clone, Debug)]
pub enum SubmissionOutcome {
    Empty,
    Correct,
    Incorrect,
    TimedOut(RoundResult),
    AlreadyFinished,
}

pub struct RoundSession {
    game_id: GameId,
    stage_id: StageId,
    stage: StageDefinition,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    rng: StdRng,
    current_problem: ArithmeticProblem,
    answer_input: String,
    correct_answers: u32,
    attempts: u32,
    current_streak: u32,
    best_streak: u32,
    last_answer_correct: Option<bool>,
    status: Option<RoundStatus>,
}

impl RoundSession {
    pub fn new(game_id: GameId, stage_id: StageId, stage: StageDefinition) -> Self {
        Self::with_seed_and_clock(
            game_id,
            stage_id,
            stage,
            rand::random(),
            Arc::new(SystemClock),
        )
    }

    pub fn with_seed(
        game_id: GameId,
        stage_id: StageId,
        stage: StageDefinition,
        seed: u64,
    ) -> Self {
        Self::with_seed_and_clock(game_id, stage_id, stage, seed, Arc::new(SystemClock))
    }

    pub fn with_seed_and_clock(
        game_id: GameId,
        stage_id: StageId,
        stage: StageDefinition,
        seed: u64,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let current_problem = stage.generator.generate(&mut rng);
        Self {
            game_id,
            stage_id,
            stage,
            started_at: Utc::now(),
            started_instant: clock.now(),
            clock,
            rng,
            current_problem,
            answer_input: String::new(),
            correct_answers: 0,
            attempts: 0,
            current_streak: 0,
            best_streak: 0,
            last_answer_correct: None,
            status: None,
        }
    }

    pub fn game_id(&self) -> &GameId {
        &self.game_id
    }

    pub fn stage_id(&self) -> &StageId {
        &self.stage_id
    }

    pub fn stage(&self) -> &StageDefinition {
        &self.stage
    }

    pub fn current_problem(&self) -> &ArithmeticProblem {
        &self.current_problem
    }

    pub fn answer_input(&self) -> &str {
        &self.answer_input
    }

    pub fn set_answer_input(&mut self, input: String) {
        self.answer_input = input;
    }

    pub fn push_digit(&mut self, digit: char) {
        if digit.is_ascii_digit() {
            self.answer_input.push(digit);
        }
    }

    pub fn backspace(&mut self) {
        self.answer_input.pop();
    }

    pub fn correct_answers(&self) -> u32 {
        self.correct_answers
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn current_streak(&self) -> u32 {
        self.current_streak
    }

    pub fn best_streak(&self) -> u32 {
        self.best_streak
    }

    pub fn last_answer_correct(&self) -> Option<bool> {
        self.last_answer_correct
    }

    pub fn is_finished(&self) -> bool {
        self.status.is_some()
    }

    pub fn status(&self) -> Option<RoundStatus> {
        self.status
    }

    pub fn time_limit(&self) -> Duration {
        self.stage.time_limit
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
            .min(self.stage.time_limit)
    }

    pub fn remaining(&self) -> Duration {
        self.stage.time_limit.saturating_sub(self.elapsed())
    }

    pub fn remaining_seconds(&self) -> u64 {
        self.remaining().as_secs_f64().ceil() as u64
    }

    pub fn check_timeout(&mut self) -> Option<RoundResult> {
        if self.status.is_none() && self.remaining().is_zero() {
            Some(self.finish(RoundStatus::TimedOut))
        } else {
            None
        }
    }

    pub fn submit_answer(&mut self) -> SubmissionOutcome {
        if self.status.is_some() {
            return SubmissionOutcome::AlreadyFinished;
        }
        if self.remaining().is_zero() {
            return SubmissionOutcome::TimedOut(self.finish(RoundStatus::TimedOut));
        }
        if self.answer_input.trim().is_empty() {
            return SubmissionOutcome::Empty;
        }

        let is_correct = self
            .current_problem
            .is_correct(&self.answer_input, self.stage.answer_format);
        self.answer_input.clear();
        self.attempts += 1;
        self.last_answer_correct = Some(is_correct);
        if is_correct {
            self.correct_answers += 1;
            self.current_streak += 1;
            self.best_streak = self.best_streak.max(self.current_streak);
        } else {
            self.current_streak = 0;
        }
        self.current_problem = self.stage.generator.generate(&mut self.rng);

        if is_correct {
            SubmissionOutcome::Correct
        } else {
            SubmissionOutcome::Incorrect
        }
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.status.is_none() {
            Some(self.finish(RoundStatus::Abandoned))
        } else {
            None
        }
    }

    pub fn finish_completed(&mut self) -> Option<RoundResult> {
        if self.status.is_none() {
            Some(self.finish(RoundStatus::Completed))
        } else {
            None
        }
    }

    fn finish(&mut self, status: RoundStatus) -> RoundResult {
        self.status = Some(status);
        let attempts = self.attempts;
        let accuracy = if attempts == 0 {
            0.0
        } else {
            f64::from(self.correct_answers) / f64::from(attempts)
        };
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: self.correct_answers,
            attempts,
            accuracy,
            best_streak: self.best_streak,
            score: self.correct_answers,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::domain::GameCatalog;

    struct FakeClock(Mutex<Instant>);

    impl FakeClock {
        fn new() -> Self {
            Self(Mutex::new(Instant::now()))
        }

        fn advance(&self, duration: Duration) {
            let mut now = self.0.lock().expect("clock lock");
            *now += duration;
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> Instant {
            *self.0.lock().expect("clock lock")
        }
    }

    fn session(clock: Arc<FakeClock>) -> RoundSession {
        let catalog = GameCatalog::default();
        let game = catalog.find_game_by_str("math").expect("math");
        let stage = game.stages.first().expect("addition stage").clone();
        RoundSession::with_seed_and_clock(
            GameId::new("math"),
            StageId::new("addition-1"),
            stage,
            1,
            clock,
        )
    }

    #[test]
    fn empty_submission_does_not_count_as_an_attempt() {
        let clock = Arc::new(FakeClock::new());
        let mut session = session(clock);
        assert!(matches!(session.submit_answer(), SubmissionOutcome::Empty));
        assert_eq!(session.attempts(), 0);
    }

    #[test]
    fn answers_update_score_accuracy_and_streak() {
        let clock = Arc::new(FakeClock::new());
        let mut session = session(clock);
        let answer = session.current_problem().answer.to_string();
        session.set_answer_input(answer);
        assert!(matches!(
            session.submit_answer(),
            SubmissionOutcome::Correct
        ));
        session.set_answer_input("-1".to_string());
        assert!(matches!(
            session.submit_answer(),
            SubmissionOutcome::Incorrect
        ));
        assert_eq!(session.correct_answers(), 1);
        assert_eq!(session.attempts(), 2);
        assert_eq!(session.best_streak(), 1);
        assert_eq!(session.current_streak(), 0);
        let result = session.finish_abandoned().expect("result");
        assert_eq!(result.score, 1);
        assert_eq!(result.accuracy, 0.5);
    }

    #[test]
    fn fixed_clock_ends_the_round_after_thirty_seconds() {
        let clock = Arc::new(FakeClock::new());
        let mut session = session(Arc::clone(&clock));
        clock.advance(Duration::from_secs(30));
        let result = session.check_timeout().expect("timeout result");
        assert_eq!(result.status, RoundStatus::TimedOut);
        assert!(session.is_finished());
    }
}
