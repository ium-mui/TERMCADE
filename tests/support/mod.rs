#![allow(dead_code)]

use cli_game::{App, CliRoute, GameCatalog, GameId, HistoryStore, MemoryHistoryStore, StageId};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Drives the real application state machine without opening a terminal.
pub struct AppHarness {
    pub app: App,
    history: MemoryHistoryStore,
}

impl AppHarness {
    pub fn game_select() -> Self {
        Self::with_route(CliRoute::GameSelect, 0)
    }

    pub fn ready(game_id: &str, stage_id: &str, wallet: u64) -> Self {
        Self::with_route(
            CliRoute::Ready {
                game_id: GameId::new(game_id),
                stage_id: StageId::new(stage_id),
            },
            wallet,
        )
    }

    pub fn with_route(route: CliRoute, wallet: u64) -> Self {
        let history = MemoryHistoryStore::default();
        history.add_won(wallet).expect("seed wallet");
        let app = App::new(GameCatalog::default(), route, Box::new(history.clone()));
        Self { app, history }
    }

    pub fn press(&mut self, code: KeyCode) -> &mut Self {
        self.app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
        self
    }

    pub fn type_ascii(&mut self, value: &str) -> &mut Self {
        for character in value.chars() {
            assert!(character.is_ascii(), "test input must be ASCII");
            self.press(KeyCode::Char(character));
        }
        self
    }

    pub fn enter(&mut self) -> &mut Self {
        self.press(KeyCode::Enter)
    }

    pub fn escape(&mut self) -> &mut Self {
        self.press(KeyCode::Esc)
    }

    pub fn persisted_wallet(&self) -> u64 {
        self.history.wallet_balance().expect("wallet balance")
    }
}
