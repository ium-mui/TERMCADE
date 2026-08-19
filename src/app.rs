use std::io;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use crate::blackjack::BlackjackSession;
use crate::breakout::BreakoutSession;
use crate::cli::CliRoute;
use crate::domain::{GameCatalog, GameId, GameKind, StageId};
use crate::game_2048::Game2048Session;
use crate::history::{HistoryStore, PlayRecord};
use crate::roulette::{RouletteColor, RouletteSession};
use crate::round::{RoundResult, RoundSession, SubmissionOutcome};
use crate::slots::SlotsSession;
use crate::snake::{Direction, SnakeSession};
use crate::sudoku::SudokuSession;
use crate::tictactoe::TicTacToeSession;
use crate::typing::{TypingPracticeSession, TypingSubmission};
use crate::ui;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    GameSelect,
    StageSelect,
    Ready,
    Playing,
    Result,
}

pub struct App {
    pub catalog: GameCatalog,
    pub screen: Screen,
    pub selected_game: usize,
    pub selected_stage: usize,
    pub session: Option<RoundSession>,
    pub snake_session: Option<SnakeSession>,
    pub tictactoe_session: Option<TicTacToeSession>,
    pub game2048_session: Option<Game2048Session>,
    pub sudoku_session: Option<SudokuSession>,
    pub blackjack_session: Option<BlackjackSession>,
    pub roulette_session: Option<RouletteSession>,
    pub slots_session: Option<SlotsSession>,
    pub typing_session: Option<TypingPracticeSession>,
    pub breakout_session: Option<BreakoutSession>,
    pub wallet_won: u64,
    pub gambling_feedback: Option<String>,
    gambling_feedback_until: Option<Instant>,
    roulette_spin_until: Option<Instant>,
    roulette_spin_frame: usize,
    pub result: Option<RoundResult>,
    pub status_message: Option<String>,
    pub should_quit: bool,
    history: Box<dyn HistoryStore>,
    history_records: Vec<PlayRecord>,
    history_error: Option<String>,
}

impl App {
    pub fn new(catalog: GameCatalog, route: CliRoute, history: Box<dyn HistoryStore>) -> Self {
        let (screen, game_id, stage_id) = match route {
            CliRoute::GameSelect => (Screen::GameSelect, None, None),
            CliRoute::StageSelect { game_id } => (Screen::StageSelect, Some(game_id), None),
            CliRoute::Ready { game_id, stage_id } => (Screen::Ready, Some(game_id), Some(stage_id)),
        };
        let selected_game = game_id
            .as_ref()
            .and_then(|id| catalog.games().iter().position(|game| &game.id == id))
            .unwrap_or(0);
        let selected_stage = stage_id
            .as_ref()
            .and_then(|id| {
                catalog
                    .games()
                    .get(selected_game)
                    .and_then(|game| game.stages.iter().position(|stage| &stage.id == id))
            })
            .unwrap_or(0);
        let (history_records, mut history_error, mut status_message) = match history.load_all() {
            Ok(records) => (records, None, None),
            Err(error) => {
                let message = format!("기존 기록을 읽지 못했습니다: {error}");
                (Vec::new(), Some(message.clone()), Some(message))
            }
        };
        let wallet_won = match history.wallet_balance() {
            Ok(balance) => balance,
            Err(error) => {
                let message = format!("지갑을 읽지 못했습니다: {error}");
                if history_error.is_none() {
                    history_error = Some(message.clone());
                    status_message = Some(message);
                }
                0
            }
        };
        Self {
            catalog,
            screen,
            selected_game,
            selected_stage,
            session: None,
            snake_session: None,
            tictactoe_session: None,
            game2048_session: None,
            sudoku_session: None,
            blackjack_session: None,
            roulette_session: None,
            slots_session: None,
            typing_session: None,
            breakout_session: None,
            wallet_won,
            gambling_feedback: None,
            gambling_feedback_until: None,
            roulette_spin_until: None,
            roulette_spin_frame: 0,
            result: None,
            status_message,
            should_quit: false,
            history,
            history_records,
            history_error,
        }
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn selected_game_definition(&self) -> Option<&crate::domain::GameDefinition> {
        self.catalog.games().get(self.selected_game)
    }

    pub fn selected_stage_definition(&self) -> Option<&crate::domain::StageDefinition> {
        self.selected_game_definition()
            .and_then(|game| game.stages.get(self.selected_stage))
    }

    pub fn history(&self) -> &dyn HistoryStore {
        self.history.as_ref()
    }

    pub fn best_record(&self, game_id: &GameId, stage_id: &StageId) -> Option<&PlayRecord> {
        self.history_records
            .iter()
            .filter(|record| &record.game_id == game_id && &record.stage_id == stage_id)
            .max_by(|left, right| {
                left.score
                    .cmp(&right.score)
                    .then_with(|| left.accuracy.total_cmp(&right.accuracy))
                    .then_with(|| left.best_streak.cmp(&right.best_streak))
                    .then_with(|| left.started_at.cmp(&right.started_at))
            })
    }

    pub fn recent_records(&self, limit: usize) -> Vec<&PlayRecord> {
        let mut records: Vec<_> = self.history_records.iter().collect();
        records.sort_by_key(|record| std::cmp::Reverse(record.started_at));
        records.truncate(limit);
        records
    }

    pub fn history_error(&self) -> Option<&str> {
        self.history_error.as_deref()
    }

    pub fn wallet_won(&self) -> u64 {
        self.wallet_won
    }

    pub fn roulette_is_spinning(&self) -> bool {
        self.roulette_spin_until.is_some()
    }

    pub fn roulette_spin_frame(&self) -> usize {
        self.roulette_spin_frame
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        match self.screen {
            Screen::GameSelect => self.handle_game_select(key),
            Screen::StageSelect => self.handle_stage_select(key),
            Screen::Ready => self.handle_ready(key),
            Screen::Playing => self.handle_playing(key),
            Screen::Result => self.handle_result(key),
        }
    }

    pub fn on_tick(&mut self) {
        if self.roulette_spin_until.is_some() {
            self.advance_roulette_spin();
            if self.roulette_spin_until.is_some() {
                return;
            }
        }
        if self.gambling_feedback.is_some() {
            self.advance_gambling_round();
            if self.gambling_feedback.is_some() {
                return;
            }
        }
        let snake_result = self.snake_session.as_mut().and_then(SnakeSession::tick);
        if let Some(result) = snake_result {
            self.finish_with_result(result);
            return;
        }
        let breakout_result = self
            .breakout_session
            .as_mut()
            .and_then(BreakoutSession::tick);
        if let Some(result) = breakout_result {
            self.finish_with_result(result);
            return;
        }
        let result = self.session.as_mut().and_then(RoundSession::check_timeout);
        if let Some(result) = result {
            self.finish_with_result(result);
        }
    }

    fn handle_game_select(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.move_game(-1),
            KeyCode::Down => self.move_game(1),
            KeyCode::Enter => {
                if !self.catalog.games().is_empty() {
                    self.selected_stage = 0;
                    self.screen = Screen::StageSelect;
                    self.status_message = None;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_stage_select(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.move_stage(-1),
            KeyCode::Down => self.move_stage(1),
            KeyCode::Enter => {
                if self.selected_stage_definition().is_some() {
                    self.screen = Screen::Ready;
                    self.status_message = None;
                }
            }
            KeyCode::Esc => {
                self.screen = Screen::GameSelect;
                self.status_message = None;
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_ready(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.start_round(),
            KeyCode::Esc => {
                self.screen = Screen::StageSelect;
                self.status_message = None;
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_playing(&mut self, key: KeyEvent) {
        if self.snake_session.is_some() {
            self.handle_snake_key(key);
            return;
        }
        if self.tictactoe_session.is_some() {
            self.handle_tictactoe_key(key);
            return;
        }
        if self.game2048_session.is_some() {
            self.handle_2048_key(key);
            return;
        }
        if self.sudoku_session.is_some() {
            self.handle_sudoku_key(key);
            return;
        }
        if self.blackjack_session.is_some() {
            self.handle_blackjack_key(key);
            return;
        }
        if self.roulette_session.is_some() {
            self.handle_roulette_key(key);
            return;
        }
        if self.slots_session.is_some() {
            self.handle_slots_key(key);
            return;
        }
        if self.typing_session.is_some() {
            self.handle_typing_key(key);
            return;
        }
        if self.breakout_session.is_some() {
            self.handle_breakout_key(key);
            return;
        }
        let Some(session) = self.session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        match key.code {
            KeyCode::Char(character) if character.is_ascii_digit() => {
                session.push_digit(character);
                self.status_message = None;
            }
            KeyCode::Backspace => session.backspace(),
            KeyCode::Enter => match session.submit_answer() {
                SubmissionOutcome::Empty => {
                    self.status_message = Some("답을 입력한 뒤 Enter를 눌러 주세요.".to_string());
                }
                SubmissionOutcome::Correct => {
                    self.status_message = Some("정답입니다!".to_string());
                }
                SubmissionOutcome::Incorrect => {
                    self.status_message = Some("오답입니다. 다음 문제로 넘어갑니다.".to_string());
                }
                SubmissionOutcome::TimedOut(result) => self.finish_with_result(result),
                SubmissionOutcome::AlreadyFinished => {}
            },
            KeyCode::Esc => {
                let result = session.finish_abandoned();
                if let Some(result) = result {
                    self.finish_with_result(result);
                }
            }
            _ => {}
        }
    }

    fn handle_snake_key(&mut self, key: KeyEvent) {
        let Some(session) = self.snake_session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        let direction = match key.code {
            KeyCode::Up | KeyCode::Char('w') => Some(Direction::Up),
            KeyCode::Down | KeyCode::Char('s') => Some(Direction::Down),
            KeyCode::Left | KeyCode::Char('a') => Some(Direction::Left),
            KeyCode::Right | KeyCode::Char('d') => Some(Direction::Right),
            KeyCode::Esc | KeyCode::Char('q') => {
                let result = session.finish_abandoned();
                if let Some(result) = result {
                    self.finish_with_result(result);
                }
                return;
            }
            _ => None,
        };
        if let Some(direction) = direction {
            session.set_direction(direction);
        }
    }

    fn handle_tictactoe_key(&mut self, key: KeyEvent) {
        let Some(session) = self.tictactoe_session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        let result = match key.code {
            KeyCode::Up => {
                session.move_cursor(0, -1);
                None
            }
            KeyCode::Down => {
                session.move_cursor(0, 1);
                None
            }
            KeyCode::Left => {
                session.move_cursor(-1, 0);
                None
            }
            KeyCode::Right => {
                session.move_cursor(1, 0);
                None
            }
            KeyCode::Enter => session.place(),
            KeyCode::Esc | KeyCode::Char('q') => session.finish_abandoned(),
            _ => None,
        };
        if let Some(result) = result {
            self.finish_with_result(result);
        }
    }

    fn handle_2048_key(&mut self, key: KeyEvent) {
        let Some(session) = self.game2048_session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        let direction = match key.code {
            KeyCode::Up | KeyCode::Char('w') => Some(Direction::Up),
            KeyCode::Down | KeyCode::Char('s') => Some(Direction::Down),
            KeyCode::Left | KeyCode::Char('a') => Some(Direction::Left),
            KeyCode::Right | KeyCode::Char('d') => Some(Direction::Right),
            KeyCode::Esc | KeyCode::Char('q') => {
                let result = session.finish_abandoned();
                if let Some(result) = result {
                    self.finish_with_result(result);
                }
                return;
            }
            _ => None,
        };
        if let Some(direction) = direction
            && let Some(result) = session.move_direction(direction)
        {
            self.finish_with_result(result);
        }
    }

    fn handle_sudoku_key(&mut self, key: KeyEvent) {
        let Some(session) = self.sudoku_session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        let result = match key.code {
            KeyCode::Up => {
                session.move_cursor(0, -1);
                None
            }
            KeyCode::Down => {
                session.move_cursor(0, 1);
                None
            }
            KeyCode::Left => {
                session.move_cursor(-1, 0);
                None
            }
            KeyCode::Right => {
                session.move_cursor(1, 0);
                None
            }
            KeyCode::Char(character) if ('1'..='9').contains(&character) => {
                let digit = character.to_digit(10).expect("ASCII digit") as u8;
                session.place_digit(digit)
            }
            KeyCode::Backspace | KeyCode::Delete => {
                session.clear_selected();
                None
            }
            KeyCode::Esc | KeyCode::Char('q') => session.finish_abandoned(),
            _ => None,
        };
        if let Some(result) = result {
            self.finish_with_result(result);
        } else if matches!(key.code, KeyCode::Char('1'..='9')) {
            if session.is_wrong(session.cursor()) {
                self.status_message =
                    Some("틀린 숫자입니다. 다른 숫자를 시도해 보세요.".to_string());
            } else {
                self.status_message = None;
            }
        } else if matches!(
            key.code,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
        ) {
            self.status_message = None;
        }
    }

    fn handle_blackjack_key(&mut self, key: KeyEvent) {
        if self.gambling_feedback.is_some() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                self.exit_gambling();
            }
            return;
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.exit_gambling();
            return;
        }
        let Some(session) = self.blackjack_session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        let result = match key.code {
            KeyCode::Enter | KeyCode::Char('h') => session.hit(),
            KeyCode::Char('s') | KeyCode::Char(' ') => session.stand(),
            _ => None,
        };
        if let Some(result) = result {
            self.finish_gambling_round(result);
        }
    }

    fn handle_roulette_key(&mut self, key: KeyEvent) {
        if self.gambling_feedback.is_some() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                self.exit_gambling();
            }
            return;
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.exit_gambling();
            return;
        }
        if self.roulette_is_spinning() {
            return;
        }
        if self.roulette_session.is_none() {
            self.screen = Screen::Ready;
            return;
        }
        match key.code {
            KeyCode::Char('1') => {
                self.roulette_session
                    .as_mut()
                    .expect("roulette session")
                    .choose(RouletteColor::Red);
            }
            KeyCode::Char('2') => {
                self.roulette_session
                    .as_mut()
                    .expect("roulette session")
                    .choose(RouletteColor::Black);
            }
            KeyCode::Char('3') => {
                self.roulette_session
                    .as_mut()
                    .expect("roulette session")
                    .choose(RouletteColor::Green);
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.start_roulette_spin(),
            _ => {}
        }
    }

    fn handle_slots_key(&mut self, key: KeyEvent) {
        if self.gambling_feedback.is_some() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                self.exit_gambling();
            }
            return;
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.exit_gambling();
            return;
        }
        let Some(session) = self.slots_session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        let result = match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => session.pull(),
            _ => None,
        };
        if let Some(result) = result {
            self.finish_gambling_round(result);
        }
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            let result = self
                .typing_session
                .as_mut()
                .and_then(TypingPracticeSession::finish_abandoned);
            if let Some(result) = result {
                self.finish_with_result(result);
            }
            return;
        }
        let Some(session) = self.typing_session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        let submission = match key.code {
            KeyCode::Char(character) => {
                session.push_char(character);
                None
            }
            KeyCode::Backspace => {
                session.backspace();
                None
            }
            KeyCode::Enter => Some(session.submit()),
            _ => None,
        };
        if let Some(submission) = submission {
            match submission {
                TypingSubmission::Empty => {
                    self.status_message = Some("문장을 입력한 뒤 Enter를 눌러 주세요.".to_string());
                }
                TypingSubmission::Correct => {
                    if self.credit_won(1) {
                        self.status_message = Some("정확합니다! 1원을 채굴했습니다.".to_string());
                    }
                }
                TypingSubmission::Incorrect => {
                    self.status_message =
                        Some("틀렸습니다. 이번 문장은 보상이 없습니다.".to_string());
                }
            }
        }
    }

    fn handle_breakout_key(&mut self, key: KeyEvent) {
        let Some(session) = self.breakout_session.as_mut() else {
            self.screen = Screen::Ready;
            return;
        };
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            let result = session.finish_abandoned();
            if let Some(result) = result {
                self.finish_with_result(result);
            }
            return;
        }
        let delta = match key.code {
            KeyCode::Left | KeyCode::Char('a') => -2,
            KeyCode::Right | KeyCode::Char('d') => 2,
            _ => 0,
        };
        if delta != 0 {
            session.move_paddle(delta);
        }
    }

    fn handle_result(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.screen = Screen::StageSelect;
                self.result = None;
                self.status_message = None;
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn move_game(&mut self, delta: isize) {
        let length = self.catalog.games().len();
        if length > 0 {
            self.selected_game = move_index(self.selected_game, delta, length);
            self.selected_stage = self
                .selected_game_definition()
                .map(|game| game.stages.len().saturating_sub(1).min(self.selected_stage))
                .unwrap_or(0);
        }
    }

    fn move_stage(&mut self, delta: isize) {
        if let Some(game) = self.selected_game_definition() {
            let length = game.stages.len();
            if length > 0 {
                self.selected_stage = move_index(self.selected_stage, delta, length);
            }
        }
    }

    fn start_round(&mut self) {
        let (game_id, stage_id, game_kind, stage) = {
            let Some(game) = self.selected_game_definition() else {
                return;
            };
            let Some(stage) = game.stages.get(self.selected_stage) else {
                return;
            };
            (
                game.id.clone(),
                stage.id.clone(),
                stage.game_kind,
                stage.clone(),
            )
        };
        if matches!(
            game_kind,
            GameKind::Blackjack | GameKind::Roulette | GameKind::Slots
        ) && !self.spend_won(1)
        {
            self.screen = Screen::Ready;
            return;
        }
        self.session = None;
        self.snake_session = None;
        self.tictactoe_session = None;
        self.game2048_session = None;
        self.sudoku_session = None;
        self.blackjack_session = None;
        self.roulette_session = None;
        self.slots_session = None;
        self.typing_session = None;
        self.breakout_session = None;
        match game_kind {
            GameKind::Arithmetic => {
                self.session = Some(RoundSession::new(game_id, stage_id, stage));
            }
            GameKind::Snake => {
                self.snake_session = Some(SnakeSession::new(game_id, stage_id));
            }
            GameKind::TicTacToe => {
                self.tictactoe_session = Some(TicTacToeSession::new(game_id, stage_id));
            }
            GameKind::TwentyFortyEight => {
                self.game2048_session = Some(Game2048Session::new(game_id, stage_id));
            }
            GameKind::Sudoku => {
                self.sudoku_session = Some(SudokuSession::new(game_id, stage_id));
            }
            GameKind::Blackjack => {
                self.blackjack_session = Some(BlackjackSession::new(game_id, stage_id));
            }
            GameKind::Roulette => {
                self.roulette_session = Some(RouletteSession::new(game_id, stage_id));
            }
            GameKind::Slots => {
                self.slots_session = Some(SlotsSession::new(game_id, stage_id));
            }
            GameKind::TypingPractice => {
                self.typing_session = Some(TypingPracticeSession::new(game_id, stage_id));
            }
            GameKind::Gambling => {}
            GameKind::Breakout => {
                self.breakout_session = Some(BreakoutSession::new(game_id, stage_id));
            }
        }
        self.result = None;
        self.status_message = None;
        self.gambling_feedback = None;
        self.gambling_feedback_until = None;
        self.roulette_spin_until = None;
        self.roulette_spin_frame = 0;
        self.screen = Screen::Playing;
    }

    fn finish_with_result(&mut self, result: RoundResult) {
        let payout = self
            .blackjack_session
            .as_ref()
            .map(BlackjackSession::payout)
            .or_else(|| self.roulette_session.as_ref().map(RouletteSession::payout))
            .or_else(|| self.slots_session.as_ref().map(SlotsSession::payout))
            .unwrap_or(0);
        if let Err(error) = self.history.add(result.clone()) {
            self.status_message = Some(format!("기록을 저장하지 못했습니다: {error}"));
        } else {
            self.status_message = None;
            self.history_records.push(result.clone());
            if payout > 0 {
                if let Err(error) = self.history.add_won(payout) {
                    self.status_message = Some(format!("당첨금 저장에 실패했습니다: {error}"));
                } else {
                    self.wallet_won = self.wallet_won.saturating_add(payout);
                    self.status_message = Some(format!("당첨금 {payout}원을 받았습니다."));
                }
            }
        }
        self.result = Some(result);
        self.session = None;
        self.snake_session = None;
        self.tictactoe_session = None;
        self.game2048_session = None;
        self.sudoku_session = None;
        self.blackjack_session = None;
        self.roulette_session = None;
        self.slots_session = None;
        self.typing_session = None;
        self.breakout_session = None;
        self.gambling_feedback = None;
        self.gambling_feedback_until = None;
        self.roulette_spin_until = None;
        self.roulette_spin_frame = 0;
        self.screen = Screen::Result;
    }

    fn finish_gambling_round(&mut self, result: RoundResult) {
        let payout = self.gambling_payout();
        let detail = self.gambling_round_detail();
        let mut storage_message = None;
        if let Err(error) = self.history.add(result.clone()) {
            storage_message = Some(format!("기록 저장 실패: {error}"));
        } else {
            self.history_records.push(result);
            if payout > 0 {
                if let Err(error) = self.history.add_won(payout) {
                    storage_message = Some(format!("당첨금 저장 실패: {error}"));
                } else {
                    self.wallet_won = self.wallet_won.saturating_add(payout);
                }
            }
        }
        self.gambling_feedback = Some(match storage_message {
            Some(message) => message,
            None => format!("{detail} · 배당금 +{payout}원 · 다음 판 준비 중"),
        });
        self.gambling_feedback_until = Some(Instant::now() + Duration::from_millis(1400));
        self.status_message = None;
        self.result = None;
    }

    fn start_roulette_spin(&mut self) {
        let can_spin = self
            .roulette_session
            .as_ref()
            .is_some_and(|session| session.choice().is_some());
        if can_spin {
            self.roulette_spin_frame = 0;
            self.roulette_spin_until = Some(Instant::now() + Duration::from_millis(1800));
        }
    }

    fn advance_roulette_spin(&mut self) {
        let Some(until) = self.roulette_spin_until else {
            return;
        };
        if Instant::now() < until {
            self.roulette_spin_frame = self.roulette_spin_frame.wrapping_add(1);
            return;
        }
        self.roulette_spin_until = None;
        self.roulette_spin_frame = 0;
        let result = self
            .roulette_session
            .as_mut()
            .and_then(RouletteSession::spin);
        if let Some(result) = result {
            self.finish_gambling_round(result);
        }
    }

    fn gambling_payout(&self) -> u64 {
        self.blackjack_session
            .as_ref()
            .map(BlackjackSession::payout)
            .or_else(|| self.roulette_session.as_ref().map(RouletteSession::payout))
            .or_else(|| self.slots_session.as_ref().map(SlotsSession::payout))
            .unwrap_or(0)
    }

    fn gambling_round_detail(&self) -> String {
        if let Some(session) = self.blackjack_session.as_ref() {
            return session
                .outcome()
                .map(|outcome| format!("블랙잭 결과: {}", outcome.label()))
                .unwrap_or_else(|| "블랙잭 결과".to_string());
        }
        if let Some(session) = self.roulette_session.as_ref() {
            return match (session.result_number(), session.result_color()) {
                (Some(number), Some(color)) => {
                    let number = if number == 37 {
                        "00".to_string()
                    } else {
                        number.to_string()
                    };
                    format!("룰렛 결과: {number} {}", color.label())
                }
                _ => "룰렛 결과".to_string(),
            };
        }
        if let Some(session) = self.slots_session.as_ref() {
            let symbols = session.symbols();
            return format!(
                "슬롯 결과: {} {} {}",
                symbols[0].label(),
                symbols[1].label(),
                symbols[2].label()
            );
        }
        "도박 결과".to_string()
    }

    fn advance_gambling_round(&mut self) {
        let Some(until) = self.gambling_feedback_until else {
            return;
        };
        if Instant::now() < until {
            return;
        }
        self.gambling_feedback = None;
        self.gambling_feedback_until = None;
        let Some((game_id, stage_id, game_kind)) =
            self.selected_game_definition().and_then(|game| {
                game.stages
                    .get(self.selected_stage)
                    .map(|stage| (game.id.clone(), stage.id.clone(), stage.game_kind))
            })
        else {
            return;
        };
        if !matches!(
            game_kind,
            GameKind::Blackjack | GameKind::Roulette | GameKind::Slots
        ) {
            return;
        }
        if !self.spend_won(1) {
            self.gambling_feedback = Some(format!(
                "잔액 부족 · 다음 판을 시작할 수 없습니다 (보유 {}원) · Esc 나가기",
                self.wallet_won
            ));
            return;
        }
        self.clear_sessions();
        match game_kind {
            GameKind::Blackjack => {
                self.blackjack_session = Some(BlackjackSession::new(game_id, stage_id));
            }
            GameKind::Roulette => {
                self.roulette_session = Some(RouletteSession::new(game_id, stage_id));
            }
            GameKind::Slots => {
                self.slots_session = Some(SlotsSession::new(game_id, stage_id));
            }
            _ => {}
        }
    }

    fn exit_gambling(&mut self) {
        let result = if let Some(session) = self.blackjack_session.as_mut() {
            session.finish_abandoned()
        } else if let Some(session) = self.roulette_session.as_mut() {
            session.finish_abandoned()
        } else if let Some(session) = self.slots_session.as_mut() {
            session.finish_abandoned()
        } else {
            None
        };
        if let Some(result) = result {
            if let Err(error) = self.history.add(result.clone()) {
                self.status_message = Some(format!("기록을 저장하지 못했습니다: {error}"));
            } else {
                self.history_records.push(result);
            }
        }
        self.clear_sessions();
        self.gambling_feedback = None;
        self.gambling_feedback_until = None;
        self.result = None;
        self.screen = Screen::StageSelect;
    }

    fn clear_sessions(&mut self) {
        self.session = None;
        self.snake_session = None;
        self.tictactoe_session = None;
        self.game2048_session = None;
        self.sudoku_session = None;
        self.blackjack_session = None;
        self.roulette_session = None;
        self.slots_session = None;
        self.typing_session = None;
        self.breakout_session = None;
        self.roulette_spin_until = None;
        self.roulette_spin_frame = 0;
    }

    fn spend_won(&mut self, amount: u64) -> bool {
        match self.history.spend_won(amount) {
            Ok(true) => {
                self.wallet_won = self.wallet_won.saturating_sub(amount);
                true
            }
            Ok(false) => {
                self.status_message = Some(format!(
                    "잔액이 부족합니다. 타자 채굴로 원을 모아 주세요. (보유 {}원)",
                    self.wallet_won
                ));
                false
            }
            Err(error) => {
                self.status_message = Some(format!("지갑을 변경하지 못했습니다: {error}"));
                false
            }
        }
    }

    fn credit_won(&mut self, amount: u64) -> bool {
        match self.history.add_won(amount) {
            Ok(()) => {
                self.wallet_won = self.wallet_won.saturating_add(amount);
                true
            }
            Err(error) => {
                self.status_message = Some(format!("채굴한 원을 저장하지 못했습니다: {error}"));
                false
            }
        }
    }
}

fn move_index(current: usize, delta: isize, length: usize) -> usize {
    if length == 0 {
        return 0;
    }
    if delta < 0 {
        if current == 0 {
            length - 1
        } else {
            current - 1
        }
    } else if current + 1 >= length {
        0
    } else {
        current + 1
    }
}

pub fn run_tui(
    catalog: GameCatalog,
    route: CliRoute,
    history: Box<dyn HistoryStore>,
) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let mut renderer = ui::Renderer::new();
    let result = run_event_loop(
        &mut stdout,
        &mut renderer,
        App::new(catalog, route, history),
    );

    disable_raw_mode()?;
    execute!(stdout, ResetColor, Show, LeaveAlternateScreen)?;
    result
}

fn run_event_loop(
    output: &mut io::Stdout,
    renderer: &mut ui::Renderer,
    mut app: App,
) -> io::Result<()> {
    while !app.should_quit {
        renderer.draw(output, &app)?;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key);
        }
        app.on_tick();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEvent, KeyModifiers};

    use super::*;
    use crate::history::{HistoryStore, MemoryHistoryStore};

    fn make_app() -> App {
        App::new(
            GameCatalog::default(),
            CliRoute::GameSelect,
            Box::new(MemoryHistoryStore::default()),
        )
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn menu_keys_move_through_game_and_stage_selection() {
        let mut app = make_app();
        assert_eq!(app.screen(), Screen::GameSelect);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::StageSelect);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.selected_stage, 1);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::Ready);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::Playing);
    }

    #[test]
    fn escape_abandons_a_round_and_ctrl_c_quits() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);
        assert_eq!(
            app.result.as_ref().expect("result").status,
            crate::round::RoundStatus::Abandoned
        );

        let mut app = make_app();
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit);
    }

    #[test]
    fn snake_route_starts_a_snake_session_and_saves_a_result_on_escape() {
        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("snake"),
                stage_id: StageId::new("classic-1"),
            },
            Box::new(MemoryHistoryStore::default()),
        );
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::Playing);
        assert!(app.snake_session.is_some());
        app.handle_key(key(KeyCode::Up));
        assert_eq!(
            app.snake_session
                .as_ref()
                .expect("snake session")
                .direction(),
            Direction::Up
        );
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);
        assert_eq!(
            app.result.as_ref().expect("snake result").game_id,
            GameId::new("snake")
        );
    }

    #[test]
    fn board_game_routes_start_their_own_sessions() {
        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("tictactoe"),
                stage_id: StageId::new("classic-1"),
            },
            Box::new(MemoryHistoryStore::default()),
        );
        app.handle_key(key(KeyCode::Enter));
        assert!(app.tictactoe_session.is_some());
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);

        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("2048"),
                stage_id: StageId::new("classic-1"),
            },
            Box::new(MemoryHistoryStore::default()),
        );
        app.handle_key(key(KeyCode::Enter));
        assert!(app.game2048_session.is_some());
        app.handle_key(key(KeyCode::Left));
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);

        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("sudoku"),
                stage_id: StageId::new("classic-1"),
            },
            Box::new(MemoryHistoryStore::default()),
        );
        app.handle_key(key(KeyCode::Enter));
        assert!(app.sudoku_session.is_some());
        app.handle_key(key(KeyCode::Right));
        app.handle_key(key(KeyCode::Char('1')));
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);

        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("gambling"),
                stage_id: StageId::new("blackjack-1"),
            },
            Box::new({
                let store = MemoryHistoryStore::default();
                store.add_won(1).expect("starting wallet");
                store
            }),
        );
        app.handle_key(key(KeyCode::Enter));
        assert!(app.blackjack_session.is_some());
        app.handle_key(key(KeyCode::Char('h')));
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::StageSelect);

        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("gambling"),
                stage_id: StageId::new("roulette-1"),
            },
            Box::new({
                let store = MemoryHistoryStore::default();
                store.add_won(1).expect("starting wallet");
                store
            }),
        );
        app.handle_key(key(KeyCode::Enter));
        assert!(app.roulette_session.is_some());
        app.handle_key(key(KeyCode::Char('1')));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::Playing);
        assert!(app.roulette_is_spinning());
        app.handle_key(key(KeyCode::Char('2')));
        assert_eq!(
            app.roulette_session
                .as_ref()
                .expect("roulette session")
                .choice(),
            Some(RouletteColor::Red)
        );
        app.roulette_spin_until = Some(Instant::now() - Duration::from_secs(1));
        app.on_tick();
        assert!(app.gambling_feedback.is_some());
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::StageSelect);

        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("gambling"),
                stage_id: StageId::new("roulette-1"),
            },
            Box::new({
                let store = MemoryHistoryStore::default();
                store.add_won(2).expect("starting wallet");
                store
            }),
        );
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('2')));
        app.handle_key(key(KeyCode::Enter));
        assert!(app.roulette_is_spinning());
        app.roulette_spin_until = Some(Instant::now() - Duration::from_secs(1));
        app.on_tick();
        assert!(app.gambling_feedback.is_some());
        let wallet_after_result = app.wallet_won();
        app.gambling_feedback_until = Some(Instant::now() - Duration::from_secs(1));
        app.on_tick();
        assert!(app.gambling_feedback.is_none());
        assert!(
            app.roulette_session
                .as_ref()
                .expect("next roulette session")
                .result_color()
                .is_none()
        );
        assert_eq!(app.wallet_won(), wallet_after_result - 1);

        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("gambling"),
                stage_id: StageId::new("typing-mine"),
            },
            Box::new(MemoryHistoryStore::default()),
        );
        let sentence = {
            app.handle_key(key(KeyCode::Enter));
            app.typing_session
                .as_ref()
                .expect("typing session")
                .sentence()
                .to_string()
        };
        for character in sentence.chars() {
            app.handle_key(key(KeyCode::Char(character)));
        }
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.wallet_won(), 1);
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);

        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("breakout"),
                stage_id: StageId::new("classic-1"),
            },
            Box::new(MemoryHistoryStore::default()),
        );
        app.handle_key(key(KeyCode::Enter));
        assert!(app.breakout_session.is_some());
        app.handle_key(key(KeyCode::Left));
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);
    }
}
