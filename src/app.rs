use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::blackjack::BlackjackSession;
use crate::breakout::BreakoutSession;
use crate::casino::{CardHandPhase, CasinoState};
use crate::cli::CliRoute;
use crate::connect_four::ConnectFourSession;
use crate::domain::{GameCatalog, GameId, GameKind, StageId};
use crate::game_2048::Game2048Session;
use crate::game_session::GameSession;
use crate::history::{HistoryStore, PlayRecord};
use crate::holdem::{HoldemAction, HoldemSession};
use crate::maze::MazeSession;
use crate::memory_match::MemoryMatchSession;
use crate::minesweeper::MinesweeperSession;
use crate::roulette::{RouletteColor, RouletteSession};
use crate::round::{RoundResult, RoundSession, SubmissionOutcome};
use crate::slots::SlotsSession;
use crate::snake::{Direction, SnakeSession};
use crate::sudoku::SudokuSession;
use crate::tictactoe::TicTacToeSession;
use crate::typing::{TypingPracticeSession, TypingSubmission};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    GameSelect,
    StageSelect,
    Ready,
    Playing,
    Result,
}

pub struct App {
    catalog: GameCatalog,
    screen: Screen,
    selected_game: usize,
    selected_stage: usize,
    active_session: Option<GameSession>,
    wallet_won: u64,
    casino: CasinoState,
    gambling_feedback: Option<String>,
    roulette_spin_until: Option<Instant>,
    roulette_spin_frame: usize,
    result: Option<RoundResult>,
    status_message: Option<String>,
    should_quit: bool,
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
            active_session: None,
            wallet_won,
            casino: CasinoState::default(),
            gambling_feedback: None,
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

    pub fn catalog(&self) -> &GameCatalog {
        &self.catalog
    }

    pub fn selected_game_index(&self) -> usize {
        self.selected_game
    }

    pub fn selected_stage_index(&self) -> usize {
        self.selected_stage
    }

    pub fn result(&self) -> Option<&RoundResult> {
        self.result.as_ref()
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn gambling_feedback(&self) -> Option<&str> {
        self.gambling_feedback.as_deref()
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
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

    pub fn gambling_bet(&self) -> u64 {
        self.casino.bet()
    }

    pub fn gambling_bet_input(&self) -> &str {
        self.casino.bet_input()
    }

    pub fn gambling_total_wager(&self) -> u64 {
        self.casino.total_wager()
    }

    pub fn gambling_bet_committed(&self) -> bool {
        self.casino.bet_committed()
    }

    pub fn card_hand_betting_open(&self) -> bool {
        self.casino.card_phase() != CardHandPhase::Covered
    }

    pub fn card_hand_started(&self) -> bool {
        self.casino.card_phase() == CardHandPhase::Playing
    }

    pub fn active_game_kind(&self) -> Option<GameKind> {
        self.active_session.as_ref().map(GameSession::kind)
    }

    pub fn round_session(&self) -> Option<&RoundSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_arithmetic)
    }

    pub fn snake_session(&self) -> Option<&SnakeSession> {
        self.active_session.as_ref().and_then(GameSession::as_snake)
    }

    pub fn tictactoe_session(&self) -> Option<&TicTacToeSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_tictactoe)
    }

    pub fn game2048_session(&self) -> Option<&Game2048Session> {
        self.active_session.as_ref().and_then(GameSession::as_2048)
    }

    pub fn sudoku_session(&self) -> Option<&SudokuSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_sudoku)
    }

    pub fn blackjack_session(&self) -> Option<&BlackjackSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_blackjack)
    }

    pub fn roulette_session(&self) -> Option<&RouletteSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_roulette)
    }

    pub fn slots_session(&self) -> Option<&SlotsSession> {
        self.active_session.as_ref().and_then(GameSession::as_slots)
    }

    pub fn holdem_session(&self) -> Option<&HoldemSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_holdem)
    }

    pub fn typing_session(&self) -> Option<&TypingPracticeSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_typing)
    }

    pub fn breakout_session(&self) -> Option<&BreakoutSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_breakout)
    }

    pub fn minesweeper_session(&self) -> Option<&MinesweeperSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_minesweeper)
    }

    pub fn connect_four_session(&self) -> Option<&ConnectFourSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_connect_four)
    }

    pub fn memory_match_session(&self) -> Option<&MemoryMatchSession> {
        self.active_session
            .as_ref()
            .and_then(GameSession::as_memory_match)
    }

    pub fn maze_session(&self) -> Option<&MazeSession> {
        self.active_session.as_ref().and_then(GameSession::as_maze)
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
        let result = self.active_session.as_mut().and_then(GameSession::tick);
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
        if self.ready_is_gambling() {
            match key.code {
                KeyCode::Enter => self.start_round(),
                KeyCode::Esc => {
                    self.screen = Screen::StageSelect;
                    self.status_message = None;
                }
                KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            }
            return;
        }
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

    fn ready_is_gambling(&self) -> bool {
        self.selected_stage_definition()
            .is_some_and(|stage| is_gambling_kind(stage.game_kind))
    }

    fn handle_gambling_bet_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(character) if character.is_ascii_digit() => {
                if self.casino.push_bet_digit(character) {
                    self.status_message = None;
                }
                true
            }
            KeyCode::Backspace => {
                self.casino.backspace_bet();
                self.status_message = None;
                true
            }
            _ => false,
        }
    }

    fn handle_playing(&mut self, key: KeyEvent) {
        match self.active_game_kind() {
            Some(GameKind::Arithmetic) => self.handle_arithmetic_key(key),
            Some(GameKind::Snake) => self.handle_snake_key(key),
            Some(GameKind::TicTacToe) => self.handle_tictactoe_key(key),
            Some(GameKind::TwentyFortyEight) => self.handle_2048_key(key),
            Some(GameKind::Sudoku) => self.handle_sudoku_key(key),
            Some(GameKind::Blackjack) => self.handle_blackjack_key(key),
            Some(GameKind::Roulette) => self.handle_roulette_key(key),
            Some(GameKind::Slots) => self.handle_slots_key(key),
            Some(GameKind::Holdem) => self.handle_holdem_key(key),
            Some(GameKind::TypingPractice) => self.handle_typing_key(key),
            Some(GameKind::Breakout) => self.handle_breakout_key(key),
            Some(GameKind::Minesweeper) => self.handle_minesweeper_key(key),
            Some(GameKind::ConnectFour) => self.handle_connect_four_key(key),
            Some(GameKind::MemoryMatch) => self.handle_memory_match_key(key),
            Some(GameKind::Maze) => self.handle_maze_key(key),
            Some(GameKind::Gambling) | None => self.screen = Screen::Ready,
        }
    }

    fn handle_arithmetic_key(&mut self, key: KeyEvent) {
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_arithmetic_mut)
        else {
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
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_snake_mut)
        else {
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
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_tictactoe_mut)
        else {
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
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_2048_mut)
        else {
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
            if let Some(result) = session.move_direction(direction) {
                self.finish_with_result(result);
            }
        }
    }

    fn handle_sudoku_key(&mut self, key: KeyEvent) {
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_sudoku_mut)
        else {
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

    fn handle_gambling_feedback_key(&mut self, key: KeyEvent) -> bool {
        if self.gambling_feedback.is_some() {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => self.advance_gambling_round(),
                KeyCode::Esc | KeyCode::Char('q') => self.exit_gambling(),
                _ => {}
            }
            return true;
        }
        false
    }

    fn handle_blackjack_key(&mut self, key: KeyEvent) {
        if self.handle_gambling_feedback_key(key) {
            return;
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.exit_gambling();
            return;
        }
        if self.casino.card_phase() == CardHandPhase::Covered {
            if key.code == KeyCode::Enter {
                self.casino.open_card_betting();
                self.status_message =
                    Some("베팅 금액을 입력한 뒤 Enter로 패를 공개하세요.".to_string());
            }
            return;
        }
        if self.casino.card_phase() == CardHandPhase::Betting {
            if self.handle_gambling_bet_input(key) {
                return;
            }
            if key.code == KeyCode::Enter && self.commit_gambling_bet() {
                self.casino.start_card_hand();
                self.status_message = None;
            }
            return;
        }
        let action = match key.code {
            KeyCode::Enter | KeyCode::Char('h') => Some(true),
            KeyCode::Char('s') | KeyCode::Char(' ') => Some(false),
            _ => None,
        };
        let Some(action) = action else {
            return;
        };
        if !self.commit_gambling_bet() {
            return;
        }
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_blackjack_mut)
        else {
            self.screen = Screen::Ready;
            return;
        };
        let result = if action {
            session.hit()
        } else {
            session.stand()
        };
        if let Some(result) = result {
            self.finish_gambling_round(result);
        }
    }

    fn handle_roulette_key(&mut self, key: KeyEvent) {
        if self.handle_gambling_feedback_key(key) {
            return;
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.exit_gambling();
            return;
        }
        if self.roulette_is_spinning() {
            return;
        }
        if self.roulette_session().is_none() {
            self.screen = Screen::Ready;
            return;
        }
        if !self.casino.bet_committed() && self.handle_gambling_bet_input(key) {
            return;
        }
        match key.code {
            KeyCode::Char('r') => {
                self.active_session
                    .as_mut()
                    .and_then(GameSession::as_roulette_mut)
                    .expect("roulette session")
                    .choose(RouletteColor::Red);
            }
            KeyCode::Char('b') => {
                self.active_session
                    .as_mut()
                    .and_then(GameSession::as_roulette_mut)
                    .expect("roulette session")
                    .choose(RouletteColor::Black);
            }
            KeyCode::Char('g') => {
                self.active_session
                    .as_mut()
                    .and_then(GameSession::as_roulette_mut)
                    .expect("roulette session")
                    .choose(RouletteColor::Green);
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.start_roulette_spin(),
            _ => {}
        }
    }

    fn handle_slots_key(&mut self, key: KeyEvent) {
        if self.handle_gambling_feedback_key(key) {
            return;
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.exit_gambling();
            return;
        }
        if !self.casino.bet_committed() && self.handle_gambling_bet_input(key) {
            return;
        }
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) && !self.commit_gambling_bet() {
            return;
        }
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_slots_mut)
        else {
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

    fn handle_holdem_key(&mut self, key: KeyEvent) {
        if self.handle_gambling_feedback_key(key) {
            return;
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.exit_gambling();
            return;
        }
        if self.holdem_session().is_none() {
            self.screen = Screen::Ready;
            return;
        }
        if self.casino.card_phase() == CardHandPhase::Covered {
            if key.code == KeyCode::Enter {
                self.casino.open_card_betting();
                self.status_message =
                    Some("베팅 금액을 입력한 뒤 Enter로 카드를 공개하세요.".to_string());
            }
            return;
        }
        if self.casino.card_phase() == CardHandPhase::Betting {
            if self.handle_gambling_bet_input(key) {
                return;
            }
            if key.code == KeyCode::Enter && self.commit_gambling_bet() {
                self.casino.start_card_hand();
                self.status_message = None;
            }
            return;
        }
        let action = match key.code {
            KeyCode::Enter | KeyCode::Char('c') | KeyCode::Char(' ') => {
                Some(HoldemAction::CheckCall)
            }
            KeyCode::Char('r') => Some(HoldemAction::Raise(10)),
            KeyCode::Char('t') => Some(HoldemAction::Raise(25)),
            KeyCode::Char('y') => Some(HoldemAction::Raise(50)),
            KeyCode::Char('f') => Some(HoldemAction::Fold),
            _ => None,
        };
        let Some(action) = action else {
            return;
        };
        if !self.commit_gambling_bet() {
            return;
        }
        if let HoldemAction::Raise(amount) = action {
            if !self.add_holdem_wager(amount) {
                return;
            }
        }
        if let Some(result) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_holdem_mut)
            .and_then(|session| session.act(action))
        {
            self.finish_gambling_round(result);
        }
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            let result = self
                .active_session
                .as_mut()
                .and_then(GameSession::as_typing_mut)
                .and_then(TypingPracticeSession::finish_abandoned);
            if let Some(result) = result {
                self.finish_with_result(result);
            }
            return;
        }
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_typing_mut)
        else {
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
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_breakout_mut)
        else {
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

    fn handle_minesweeper_key(&mut self, key: KeyEvent) {
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_minesweeper_mut)
        else {
            self.screen = Screen::Ready;
            return;
        };
        let result = match key.code {
            KeyCode::Up | KeyCode::Char('w') => {
                session.move_cursor(0, -1);
                None
            }
            KeyCode::Down | KeyCode::Char('s') => {
                session.move_cursor(0, 1);
                None
            }
            KeyCode::Left | KeyCode::Char('a') => {
                session.move_cursor(-1, 0);
                None
            }
            KeyCode::Right | KeyCode::Char('d') => {
                session.move_cursor(1, 0);
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => session.reveal(),
            KeyCode::Char('f') => {
                session.toggle_flag();
                None
            }
            KeyCode::Esc | KeyCode::Char('q') => session.finish_abandoned(),
            _ => None,
        };
        if let Some(result) = result {
            self.finish_with_result(result);
        }
    }

    fn handle_connect_four_key(&mut self, key: KeyEvent) {
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_connect_four_mut)
        else {
            self.screen = Screen::Ready;
            return;
        };
        let result = match key.code {
            KeyCode::Left | KeyCode::Char('a') => {
                session.move_cursor(-1);
                None
            }
            KeyCode::Right | KeyCode::Char('d') => {
                session.move_cursor(1);
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => session.drop_disc(),
            KeyCode::Esc | KeyCode::Char('q') => session.finish_abandoned(),
            _ => None,
        };
        if let Some(result) = result {
            self.finish_with_result(result);
        }
    }

    fn handle_memory_match_key(&mut self, key: KeyEvent) {
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_memory_match_mut)
        else {
            self.screen = Screen::Ready;
            return;
        };
        let result = match key.code {
            KeyCode::Up | KeyCode::Char('w') => {
                session.move_cursor(0, -1);
                None
            }
            KeyCode::Down | KeyCode::Char('s') => {
                session.move_cursor(0, 1);
                None
            }
            KeyCode::Left | KeyCode::Char('a') => {
                session.move_cursor(-1, 0);
                None
            }
            KeyCode::Right | KeyCode::Char('d') => {
                session.move_cursor(1, 0);
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => session.select(),
            KeyCode::Esc | KeyCode::Char('q') => session.finish_abandoned(),
            _ => None,
        };
        if let Some(result) = result {
            self.finish_with_result(result);
        }
    }

    fn handle_maze_key(&mut self, key: KeyEvent) {
        let Some(session) = self
            .active_session
            .as_mut()
            .and_then(GameSession::as_maze_mut)
        else {
            self.screen = Screen::Ready;
            return;
        };
        let movement = match key.code {
            KeyCode::Up | KeyCode::Char('w') => Some((0, -1)),
            KeyCode::Down | KeyCode::Char('s') => Some((0, 1)),
            KeyCode::Left | KeyCode::Char('a') => Some((-1, 0)),
            KeyCode::Right | KeyCode::Char('d') => Some((1, 0)),
            KeyCode::Esc | KeyCode::Char('q') => {
                let result = session.finish_abandoned();
                if let Some(result) = result {
                    self.finish_with_result(result);
                }
                return;
            }
            _ => None,
        };
        if let Some((dx, dy)) = movement {
            if let Some(result) = session.move_player(dx, dy) {
                self.finish_with_result(result);
            }
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
        self.active_session = GameSession::start(game_id, stage_id, stage);
        self.result = None;
        self.status_message = None;
        self.gambling_feedback = None;
        if is_gambling_kind(game_kind) {
            self.casino.reset_round();
        }
        self.roulette_spin_until = None;
        self.roulette_spin_frame = 0;
        self.screen = Screen::Playing;
    }

    fn finish_with_result(&mut self, result: RoundResult) {
        let payout = self
            .active_session
            .as_ref()
            .map(GameSession::payout_multiplier)
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
        self.active_session = None;
        self.gambling_feedback = None;
        self.roulette_spin_until = None;
        self.roulette_spin_frame = 0;
        self.screen = Screen::Result;
    }

    fn finish_gambling_round(&mut self, result: RoundResult) {
        let payout = self.gambling_payout();
        let detail = self.gambling_round_detail();
        let mut result = result;
        result.score = payout.min(u64::from(u32::MAX)) as u32;
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
            None => format!(
                "{detail} · 총 베팅 {}원 · 배당금 +{payout}원 · Enter로 다음 판 시작",
                self.casino.total_wager()
            ),
        });
        self.status_message = None;
        self.result = None;
    }

    fn start_roulette_spin(&mut self) {
        let can_spin = self
            .roulette_session()
            .is_some_and(|session| session.choice().is_some());
        if can_spin && self.commit_gambling_bet() {
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
            .active_session
            .as_mut()
            .and_then(GameSession::as_roulette_mut)
            .and_then(RouletteSession::spin);
        if let Some(result) = result {
            self.finish_gambling_round(result);
        }
    }

    fn gambling_payout(&self) -> u64 {
        self.active_session
            .as_ref()
            .map(GameSession::payout_multiplier)
            .unwrap_or(0)
            .saturating_mul(self.casino.total_wager())
    }

    fn gambling_round_detail(&self) -> String {
        if let Some(session) = self.blackjack_session() {
            return session
                .outcome()
                .map(|outcome| format!("블랙잭 결과: {}", outcome.label()))
                .unwrap_or_else(|| "블랙잭 결과".to_string());
        }
        if let Some(session) = self.roulette_session() {
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
        if let Some(session) = self.slots_session() {
            let symbols = session.symbols();
            return format!(
                "슬롯 결과: {} {} {}",
                symbols[0].label(),
                symbols[1].label(),
                symbols[2].label()
            );
        }
        if let Some(session) = self.holdem_session() {
            return session
                .outcome()
                .map(|outcome| {
                    format!(
                        "홀덤 결과: {} · AI {} 난이도",
                        outcome.label(),
                        session.difficulty().label()
                    )
                })
                .unwrap_or_else(|| "홀덤 결과".to_string());
        }
        "도박 결과".to_string()
    }

    fn advance_gambling_round(&mut self) {
        if self.gambling_feedback.is_none() {
            return;
        }
        self.gambling_feedback = None;
        let Some((game_id, stage_id, game_kind)) =
            self.selected_game_definition().and_then(|game| {
                game.stages
                    .get(self.selected_stage)
                    .map(|stage| (game.id.clone(), stage.id.clone(), stage.game_kind))
            })
        else {
            return;
        };
        if !is_gambling_kind(game_kind) {
            return;
        }
        self.clear_sessions();
        self.casino.reset_round();
        self.active_session = GameSession::restart_casino(game_kind, game_id, stage_id);
    }

    fn exit_gambling(&mut self) {
        let result = self
            .active_session
            .as_mut()
            .and_then(GameSession::finish_abandoned);
        if let Some(result) = result {
            if let Err(error) = self.history.add(result.clone()) {
                self.status_message = Some(format!("기록을 저장하지 못했습니다: {error}"));
            } else {
                self.history_records.push(result);
            }
        }
        self.clear_sessions();
        self.gambling_feedback = None;
        self.result = None;
        self.casino.reset_round();
        self.screen = Screen::StageSelect;
    }

    fn clear_sessions(&mut self) {
        self.active_session = None;
        self.roulette_spin_until = None;
        self.roulette_spin_frame = 0;
    }

    fn commit_gambling_bet(&mut self) -> bool {
        if self.casino.bet_committed() {
            return true;
        }
        let bet = self.casino.bet();
        if bet == 0 {
            self.status_message = Some("테이블에서 베팅 금액을 먼저 입력해 주세요.".to_string());
            return false;
        }
        if !self.spend_won(bet) {
            return false;
        }
        self.casino.commit_initial_wager();
        true
    }

    fn add_holdem_wager(&mut self, amount: u32) -> bool {
        let amount = u64::from(amount);
        if amount == 0 || self.spend_won(amount) {
            self.casino.add_wager(amount);
            true
        } else {
            false
        }
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

fn is_gambling_kind(kind: GameKind) -> bool {
    matches!(
        kind,
        GameKind::Blackjack | GameKind::Roulette | GameKind::Slots | GameKind::Holdem
    )
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
        assert_eq!(app.selected_stage_index(), 1);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::Ready);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::Playing);
    }

    #[test]
    fn table_bet_is_typed_before_action_and_charged_from_wallet() {
        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("gambling"),
                stage_id: StageId::new("roulette-1"),
            },
            Box::new({
                let store = MemoryHistoryStore::default();
                store.add_won(25).expect("starting wallet");
                store
            }),
        );
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::Playing);
        assert_eq!(app.wallet_won(), 25);
        app.handle_key(key(KeyCode::Char('2')));
        app.handle_key(key(KeyCode::Char('5')));
        assert_eq!(app.gambling_bet(), 25);
        app.handle_key(key(KeyCode::Char('r')));
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.wallet_won(), 0);
        assert!(app.roulette_is_spinning());
        assert!(app.roulette_session().is_some());
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
            app.result().expect("result").status,
            crate::round::RoundStatus::Abandoned
        );

        let mut app = make_app();
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit());
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
        assert!(app.snake_session().is_some());
        app.handle_key(key(KeyCode::Up));
        assert_eq!(
            app.snake_session().expect("snake session").direction(),
            Direction::Up
        );
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);
        assert_eq!(
            app.result().expect("snake result").game_id,
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
        assert!(app.tictactoe_session().is_some());
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
        assert!(app.game2048_session().is_some());
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
        assert!(app.sudoku_session().is_some());
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
        assert!(app.blackjack_session().is_some());
        assert!(!app.card_hand_started());
        assert_eq!(app.wallet_won(), 1);
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('1')));
        app.handle_key(key(KeyCode::Enter));
        assert!(app.card_hand_started());
        assert_eq!(app.wallet_won(), 0);
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
        assert!(app.roulette_session().is_some());
        app.handle_key(key(KeyCode::Char('1')));
        app.handle_key(key(KeyCode::Char('r')));
        assert_eq!(
            app.roulette_session().expect("roulette session").choice(),
            Some(RouletteColor::Red)
        );
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.screen(), Screen::Playing);
        assert!(app.roulette_is_spinning());
        app.roulette_spin_until = Some(Instant::now() - Duration::from_secs(1));
        app.on_tick();
        assert!(app.gambling_feedback().is_some());
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
        app.handle_key(key(KeyCode::Char('1')));
        app.handle_key(key(KeyCode::Char('r')));
        app.handle_key(key(KeyCode::Enter));
        assert!(app.roulette_is_spinning());
        app.roulette_spin_until = Some(Instant::now() - Duration::from_secs(1));
        app.on_tick();
        assert!(app.gambling_feedback().is_some());
        let wallet_after_result = app.wallet_won();
        app.handle_key(key(KeyCode::Enter));
        assert!(app.gambling_feedback().is_none());
        assert!(
            app.roulette_session()
                .expect("next roulette session")
                .result_color()
                .is_none()
        );
        assert_eq!(app.wallet_won(), wallet_after_result);

        let mut app = App::new(
            GameCatalog::default(),
            CliRoute::Ready {
                game_id: GameId::new("gambling"),
                stage_id: StageId::new("holdem-1"),
            },
            Box::new({
                let store = MemoryHistoryStore::default();
                store.add_won(1).expect("starting wallet");
                store
            }),
        );
        app.handle_key(key(KeyCode::Enter));
        assert!(app.holdem_session().is_some());
        assert!(!app.card_hand_started());
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('1')));
        app.handle_key(key(KeyCode::Enter));
        assert!(app.card_hand_started());
        app.handle_key(key(KeyCode::Char('c')));
        assert!(app.holdem_session().is_some());
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::StageSelect);

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
            app.typing_session()
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
        app.handle_key(key(KeyCode::Char('1')));
        app.handle_key(key(KeyCode::Enter));
        assert!(app.breakout_session().is_some());
        app.handle_key(key(KeyCode::Left));
        app.handle_key(key(KeyCode::Esc));
        assert_eq!(app.screen(), Screen::Result);
    }
}
