use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::arithmetic::{AnswerFormat, Operation, ProblemGenerator, boxed_generator};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameKind {
    Arithmetic,
    Snake,
    TicTacToe,
    TwentyFortyEight,
    Sudoku,
    Gambling,
    Blackjack,
    Roulette,
    Slots,
    Holdem,
    TypingPractice,
    Breakout,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameId(String);

impl GameId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StageId(String);

impl StageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

pub struct StageDefinition {
    pub id: StageId,
    pub display_name: String,
    pub operation: Operation,
    pub difficulty_order: u32,
    pub generator: Arc<dyn ProblemGenerator>,
    pub answer_format: AnswerFormat,
    pub time_limit: Duration,
    pub game_kind: GameKind,
}

impl Clone for StageDefinition {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            display_name: self.display_name.clone(),
            operation: self.operation,
            difficulty_order: self.difficulty_order,
            generator: Arc::clone(&self.generator),
            answer_format: self.answer_format,
            time_limit: self.time_limit,
            game_kind: self.game_kind,
        }
    }
}

#[derive(Clone)]
pub struct GameDefinition {
    pub id: GameId,
    pub display_name: String,
    pub kind: GameKind,
    pub stages: Vec<StageDefinition>,
}

pub trait GameModule: Send + Sync {
    fn definition(&self) -> GameDefinition;
}

#[derive(Default)]
pub struct MathGame;

impl GameModule for MathGame {
    fn definition(&self) -> GameDefinition {
        let time_limit = Duration::from_secs(30);
        let stages = vec![
            StageDefinition {
                id: StageId::new("addition-1"),
                display_name: "덧셈 · 1단계".to_string(),
                operation: Operation::Addition,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Addition, 0..=9),
                answer_format: AnswerFormat::Integer,
                time_limit,
                game_kind: GameKind::Arithmetic,
            },
            StageDefinition {
                id: StageId::new("subtraction-1"),
                display_name: "뺄셈 · 1단계".to_string(),
                operation: Operation::Subtraction,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Subtraction, 0..=9),
                answer_format: AnswerFormat::Integer,
                time_limit,
                game_kind: GameKind::Arithmetic,
            },
            StageDefinition {
                id: StageId::new("multiplication-1"),
                display_name: "곱셈 · 1단계".to_string(),
                operation: Operation::Multiplication,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Multiplication, 0..=9),
                answer_format: AnswerFormat::Integer,
                time_limit,
                game_kind: GameKind::Arithmetic,
            },
            StageDefinition {
                id: StageId::new("division-1"),
                display_name: "나눗셈 · 1단계".to_string(),
                operation: Operation::Division,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Division, 1..=9),
                answer_format: AnswerFormat::Integer,
                time_limit,
                game_kind: GameKind::Arithmetic,
            },
        ];

        GameDefinition {
            id: GameId::new("math"),
            display_name: "암산 게임".to_string(),
            kind: GameKind::Arithmetic,
            stages,
        }
    }
}

#[derive(Default)]
pub struct SnakeGame;

impl GameModule for SnakeGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("snake"),
            display_name: "스네이크".to_string(),
            kind: GameKind::Snake,
            stages: vec![StageDefinition {
                id: StageId::new("classic-1"),
                display_name: "클래식 · 1단계".to_string(),
                operation: Operation::Addition,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Addition, 0..=0),
                answer_format: AnswerFormat::Integer,
                time_limit: Duration::ZERO,
                game_kind: GameKind::Snake,
            }],
        }
    }
}

#[derive(Default)]
pub struct TicTacToeGame;

impl GameModule for TicTacToeGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("tictactoe"),
            display_name: "틱택토".to_string(),
            kind: GameKind::TicTacToe,
            stages: vec![StageDefinition {
                id: StageId::new("classic-1"),
                display_name: "클래식 · 1단계".to_string(),
                operation: Operation::Addition,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Addition, 0..=0),
                answer_format: AnswerFormat::Integer,
                time_limit: Duration::ZERO,
                game_kind: GameKind::TicTacToe,
            }],
        }
    }
}

#[derive(Default)]
pub struct TwentyFortyEightGame;

impl GameModule for TwentyFortyEightGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("2048"),
            display_name: "2048".to_string(),
            kind: GameKind::TwentyFortyEight,
            stages: vec![StageDefinition {
                id: StageId::new("classic-1"),
                display_name: "클래식 · 1단계".to_string(),
                operation: Operation::Addition,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Addition, 0..=0),
                answer_format: AnswerFormat::Integer,
                time_limit: Duration::ZERO,
                game_kind: GameKind::TwentyFortyEight,
            }],
        }
    }
}

#[derive(Default)]
pub struct SudokuGame;

impl GameModule for SudokuGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("sudoku"),
            display_name: "스도쿠".to_string(),
            kind: GameKind::Sudoku,
            stages: vec![StageDefinition {
                id: StageId::new("classic-1"),
                display_name: "클래식 · 1단계".to_string(),
                operation: Operation::Addition,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Addition, 0..=0),
                answer_format: AnswerFormat::Integer,
                time_limit: Duration::ZERO,
                game_kind: GameKind::Sudoku,
            }],
        }
    }
}

#[derive(Default)]
pub struct GamblingGame;

impl GameModule for GamblingGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("gambling"),
            display_name: "도박장".to_string(),
            kind: GameKind::Gambling,
            stages: vec![
                StageDefinition {
                    id: StageId::new("blackjack-1"),
                    display_name: "블랙잭 · 1단계".to_string(),
                    operation: Operation::Addition,
                    difficulty_order: 1,
                    generator: boxed_generator(Operation::Addition, 0..=0),
                    answer_format: AnswerFormat::Integer,
                    time_limit: Duration::ZERO,
                    game_kind: GameKind::Blackjack,
                },
                StageDefinition {
                    id: StageId::new("roulette-1"),
                    display_name: "룰렛 · 1단계".to_string(),
                    operation: Operation::Addition,
                    difficulty_order: 1,
                    generator: boxed_generator(Operation::Addition, 0..=0),
                    answer_format: AnswerFormat::Integer,
                    time_limit: Duration::ZERO,
                    game_kind: GameKind::Roulette,
                },
                StageDefinition {
                    id: StageId::new("slots-1"),
                    display_name: "슬롯머신 · 1단계".to_string(),
                    operation: Operation::Addition,
                    difficulty_order: 1,
                    generator: boxed_generator(Operation::Addition, 0..=0),
                    answer_format: AnswerFormat::Integer,
                    time_limit: Duration::ZERO,
                    game_kind: GameKind::Slots,
                },
                StageDefinition {
                    id: StageId::new("holdem-1"),
                    display_name: "텍사스 홀덤 · AI 대전".to_string(),
                    operation: Operation::Addition,
                    difficulty_order: 1,
                    generator: boxed_generator(Operation::Addition, 0..=0),
                    answer_format: AnswerFormat::Integer,
                    time_limit: Duration::ZERO,
                    game_kind: GameKind::Holdem,
                },
                StageDefinition {
                    id: StageId::new("typing-mine"),
                    display_name: "타자 채굴".to_string(),
                    operation: Operation::Addition,
                    difficulty_order: 1,
                    generator: boxed_generator(Operation::Addition, 0..=0),
                    answer_format: AnswerFormat::Integer,
                    time_limit: Duration::ZERO,
                    game_kind: GameKind::TypingPractice,
                },
            ],
        }
    }
}

#[derive(Default)]
pub struct BreakoutGame;

impl GameModule for BreakoutGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("breakout"),
            display_name: "벽돌깨기".to_string(),
            kind: GameKind::Breakout,
            stages: vec![StageDefinition {
                id: StageId::new("classic-1"),
                display_name: "클래식 · 1단계".to_string(),
                operation: Operation::Addition,
                difficulty_order: 1,
                generator: boxed_generator(Operation::Addition, 0..=0),
                answer_format: AnswerFormat::Integer,
                time_limit: Duration::ZERO,
                game_kind: GameKind::Breakout,
            }],
        }
    }
}

#[derive(Clone)]
pub struct GameCatalog {
    games: Vec<GameDefinition>,
}

impl Default for GameCatalog {
    fn default() -> Self {
        Self::from_modules([
            Box::new(MathGame) as Box<dyn GameModule>,
            Box::new(SnakeGame) as Box<dyn GameModule>,
            Box::new(TicTacToeGame) as Box<dyn GameModule>,
            Box::new(TwentyFortyEightGame) as Box<dyn GameModule>,
            Box::new(SudokuGame) as Box<dyn GameModule>,
            Box::new(GamblingGame) as Box<dyn GameModule>,
            Box::new(BreakoutGame) as Box<dyn GameModule>,
        ])
    }
}

impl GameCatalog {
    pub fn from_modules<I>(modules: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn GameModule>>,
    {
        Self {
            games: modules
                .into_iter()
                .map(|module| module.definition())
                .collect(),
        }
    }

    pub fn register(&mut self, module: &dyn GameModule) {
        self.games.push(module.definition());
    }

    pub fn games(&self) -> &[GameDefinition] {
        &self.games
    }

    pub fn find_game(&self, id: &GameId) -> Option<&GameDefinition> {
        self.games.iter().find(|game| &game.id == id)
    }

    pub fn find_game_by_str(&self, id: &str) -> Option<&GameDefinition> {
        self.games.iter().find(|game| game.id.as_str() == id)
    }

    pub fn find_stage(&self, game_id: &GameId, stage_id: &StageId) -> Option<&StageDefinition> {
        self.find_game(game_id)
            .and_then(|game| game.stages.iter().find(|stage| &stage.id == stage_id))
    }

    pub fn find_stage_by_str(
        &self,
        game_id: &str,
        stage_id: &str,
    ) -> Option<(&GameDefinition, &StageDefinition)> {
        self.find_game_by_str(game_id).and_then(|game| {
            game.stages
                .iter()
                .find(|stage| stage.id.as_str() == stage_id)
                .map(|stage| (game, stage))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_contains_the_initial_games() {
        let catalog = GameCatalog::default();
        let game = catalog.find_game_by_str("math").expect("math game");
        let ids: Vec<_> = game.stages.iter().map(|stage| stage.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "addition-1",
                "subtraction-1",
                "multiplication-1",
                "division-1"
            ]
        );
        assert!(
            game.stages
                .iter()
                .all(|stage| stage.time_limit == Duration::from_secs(30))
        );
        let snake = catalog.find_game_by_str("snake").expect("snake game");
        assert_eq!(snake.kind, GameKind::Snake);
        assert_eq!(snake.stages[0].id.as_str(), "classic-1");
        assert_eq!(
            catalog
                .find_game_by_str("tictactoe")
                .expect("tictactoe")
                .kind,
            GameKind::TicTacToe
        );
        assert_eq!(
            catalog.find_game_by_str("2048").expect("2048").kind,
            GameKind::TwentyFortyEight
        );
        assert_eq!(
            catalog.find_game_by_str("sudoku").expect("sudoku").kind,
            GameKind::Sudoku
        );
        let gambling = catalog.find_game_by_str("gambling").expect("gambling");
        assert_eq!(gambling.kind, GameKind::Gambling);
        assert_eq!(
            gambling
                .stages
                .iter()
                .map(|stage| stage.id.as_str())
                .collect::<Vec<_>>(),
            [
                "blackjack-1",
                "roulette-1",
                "slots-1",
                "holdem-1",
                "typing-mine",
            ]
        );
        assert_eq!(gambling.stages[0].game_kind, GameKind::Blackjack);
        assert_eq!(gambling.stages[1].game_kind, GameKind::Roulette);
        assert_eq!(gambling.stages[2].game_kind, GameKind::Slots);
        assert_eq!(gambling.stages[3].game_kind, GameKind::Holdem);
        assert_eq!(gambling.stages[4].game_kind, GameKind::TypingPractice);
        assert_eq!(
            catalog.find_game_by_str("breakout").expect("breakout").kind,
            GameKind::Breakout
        );
    }
}
