mod support;

use cli_game::ui::{MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH, Renderer};
use cli_game::{GameCatalog, GameKind, RoundStatus, Screen};
use crossterm::event::KeyCode;

use support::AppHarness;

#[test]
fn menu_to_arithmetic_round_and_back_is_a_stable_flow() {
    let mut harness = AppHarness::game_select();
    harness.enter().enter().enter();

    assert_eq!(harness.app.screen(), Screen::Playing);
    assert_eq!(harness.app.active_game_kind(), Some(GameKind::Arithmetic));

    harness.escape();
    assert_eq!(harness.app.screen(), Screen::Result);
    assert_eq!(
        harness.app.result().expect("round result").status,
        RoundStatus::Abandoned
    );
}

#[test]
fn blackjack_keeps_cards_covered_until_the_wager_is_committed() {
    let mut harness = AppHarness::ready("gambling", "blackjack-1", 50);
    harness.enter();

    assert_eq!(harness.app.active_game_kind(), Some(GameKind::Blackjack));
    assert!(!harness.app.card_hand_betting_open());
    assert!(!harness.app.card_hand_started());
    assert_eq!(harness.app.wallet_won(), 50);

    harness.enter().type_ascii("20");
    assert!(harness.app.card_hand_betting_open());
    assert!(!harness.app.card_hand_started());
    assert_eq!(harness.app.wallet_won(), 50);

    harness.enter();
    assert!(harness.app.card_hand_started());
    assert_eq!(harness.app.wallet_won(), 30);
    assert_eq!(harness.persisted_wallet(), 30);
}

#[test]
fn slots_next_round_does_not_charge_until_the_next_pull() {
    let mut harness = AppHarness::ready("gambling", "slots-1", 20);
    harness.enter().type_ascii("3").enter();

    assert!(harness.app.gambling_feedback().is_some());
    let wallet_after_result = harness.app.wallet_won();
    harness.enter();

    assert!(harness.app.gambling_feedback().is_none());
    assert_eq!(harness.app.gambling_bet(), 0);
    assert_eq!(harness.app.gambling_total_wager(), 0);
    assert_eq!(harness.app.wallet_won(), wallet_after_result);
}

#[test]
fn renderer_handles_small_and_standard_terminal_sizes() {
    let harness = AppHarness::game_select();
    for (width, height) in [
        (1, 1),
        (MIN_TERMINAL_WIDTH - 1, MIN_TERMINAL_HEIGHT - 1),
        (MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT),
        (120, 40),
    ] {
        let mut renderer = Renderer::new();
        let mut output = Vec::new();
        renderer
            .draw_at(&mut output, &harness.app, width, height)
            .expect("render fixed terminal size");
        assert!(!output.is_empty());
    }
}

#[test]
fn every_builtin_stage_starts_the_declared_session_kind() {
    let catalog = GameCatalog::default();
    for game in catalog.games() {
        for stage in &game.stages {
            let mut harness = AppHarness::ready(game.id.as_str(), stage.id.as_str(), 100);
            harness.press(KeyCode::Enter);
            assert_eq!(
                harness.app.active_game_kind(),
                Some(stage.game_kind),
                "{}/{}",
                game.id,
                stage.id
            );
        }
    }
}

#[test]
fn every_new_game_stage_renders_at_minimum_and_wide_sizes() {
    let cases = [
        ("minesweeper", "beginner"),
        ("minesweeper", "intermediate"),
        ("minesweeper", "expert"),
        ("connect-four", "easy"),
        ("connect-four", "normal"),
        ("connect-four", "hard"),
        ("memory", "small"),
        ("memory", "classic"),
        ("memory", "grand"),
        ("maze", "alley"),
        ("maze", "labyrinth"),
        ("maze", "abyss"),
        ("snake", "relaxed"),
        ("snake", "classic-1"),
        ("snake", "turbo"),
        ("sudoku", "easy"),
        ("sudoku", "classic-1"),
        ("sudoku", "hard"),
    ];
    for (game_id, stage_id) in cases {
        let mut harness = AppHarness::ready(game_id, stage_id, 0);
        harness.enter();
        for (width, height) in [(MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT), (120, 40)] {
            let mut renderer = Renderer::new();
            let mut output = Vec::new();
            renderer
                .draw_at(&mut output, &harness.app, width, height)
                .expect("render new game");
            assert!(
                !output.is_empty(),
                "{game_id}/{stage_id} at {width}x{height}"
            );
        }
    }
}

#[test]
fn new_games_accept_real_key_controls_and_exit_cleanly() {
    let cases: [(&str, &str, &[KeyCode]); 4] = [
        (
            "minesweeper",
            "beginner",
            &[
                KeyCode::Right,
                KeyCode::Char('f'),
                KeyCode::Left,
                KeyCode::Enter,
            ],
        ),
        ("connect-four", "normal", &[KeyCode::Left, KeyCode::Enter]),
        (
            "memory",
            "small",
            &[
                KeyCode::Enter,
                KeyCode::Right,
                KeyCode::Enter,
                KeyCode::Enter,
            ],
        ),
        (
            "maze",
            "alley",
            &[KeyCode::Right, KeyCode::Down, KeyCode::Left],
        ),
    ];

    for (game_id, stage_id, keys) in cases {
        let mut harness = AppHarness::ready(game_id, stage_id, 0);
        harness.enter();
        for key in keys {
            harness.press(*key);
        }
        harness.escape();
        assert_eq!(harness.app.screen(), Screen::Result, "{game_id}/{stage_id}");
        assert_eq!(
            harness.app.result().expect("abandoned result").status,
            RoundStatus::Abandoned,
            "{game_id}/{stage_id}"
        );
    }
}
