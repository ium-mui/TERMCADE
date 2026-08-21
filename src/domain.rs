use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    Minesweeper,
    ConnectFour,
    MemoryMatch,
    Maze,
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

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CatalogError {
    #[error("게임 ID는 비어 있을 수 없습니다")]
    EmptyGameId,
    #[error("게임 '{game_id}'의 표시 이름은 비어 있을 수 없습니다")]
    EmptyGameName { game_id: String },
    #[error("게임 '{game_id}'에는 스테이지가 하나 이상 필요합니다")]
    EmptyStages { game_id: String },
    #[error("게임 ID '{game_id}'가 중복되었습니다")]
    DuplicateGameId { game_id: String },
    #[error("게임 '{game_id}'의 스테이지 ID는 비어 있을 수 없습니다")]
    EmptyStageId { game_id: String },
    #[error("게임 '{game_id}'의 스테이지 ID '{stage_id}'가 중복되었습니다")]
    DuplicateStageId { game_id: String, stage_id: String },
    #[error("게임 '{game_id}'의 스테이지 '{stage_id}' 표시 이름은 비어 있을 수 없습니다")]
    EmptyStageName { game_id: String, stage_id: String },
    #[error("게임 '{game_id}'의 스테이지 '{stage_id}' 종류가 부모 게임과 일치하지 않습니다")]
    StageKindMismatch { game_id: String, stage_id: String },
    #[error("도박장 스테이지 '{stage_id}'는 구체적인 게임 종류를 가져야 합니다")]
    NestedGamblingStage { stage_id: String },
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
            stages: vec![
                arcade_stage("relaxed", "릴랙스 · 느린 속도", 1, GameKind::Snake),
                arcade_stage("classic-1", "클래식 · 표준 속도", 2, GameKind::Snake),
                arcade_stage("turbo", "터보 · 고속", 3, GameKind::Snake),
            ],
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
            stages: vec![
                arcade_stage("easy", "초급 · 단서 46+", 1, GameKind::Sudoku),
                arcade_stage("classic-1", "클래식 · 단서 40+", 2, GameKind::Sudoku),
                arcade_stage("hard", "고급 · 단서 32+", 3, GameKind::Sudoku),
            ],
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

fn arcade_stage(
    id: &str,
    name: &str,
    difficulty_order: u32,
    game_kind: GameKind,
) -> StageDefinition {
    StageDefinition {
        id: StageId::new(id),
        display_name: name.to_string(),
        operation: Operation::Addition,
        difficulty_order,
        generator: boxed_generator(Operation::Addition, 0..=0),
        answer_format: AnswerFormat::Integer,
        time_limit: Duration::ZERO,
        game_kind,
    }
}

#[derive(Default)]
pub struct MinesweeperGame;

impl GameModule for MinesweeperGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("minesweeper"),
            display_name: "지뢰찾기".to_string(),
            kind: GameKind::Minesweeper,
            stages: vec![
                arcade_stage("beginner", "초급 · 9×9 · 지뢰 10", 1, GameKind::Minesweeper),
                arcade_stage(
                    "intermediate",
                    "중급 · 16×12 · 지뢰 30",
                    2,
                    GameKind::Minesweeper,
                ),
                arcade_stage("expert", "고급 · 24×16 · 지뢰 70", 3, GameKind::Minesweeper),
            ],
        }
    }
}

#[derive(Default)]
pub struct ConnectFourGame;

impl GameModule for ConnectFourGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("connect-four"),
            display_name: "커넥트 포".to_string(),
            kind: GameKind::ConnectFour,
            stages: vec![
                arcade_stage("easy", "AI 대전 · 쉬움", 1, GameKind::ConnectFour),
                arcade_stage("normal", "AI 대전 · 보통", 2, GameKind::ConnectFour),
                arcade_stage("hard", "AI 대전 · 어려움", 3, GameKind::ConnectFour),
            ],
        }
    }
}

#[derive(Default)]
pub struct MemoryMatchGame;

impl GameModule for MemoryMatchGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("memory"),
            display_name: "카드 짝맞추기".to_string(),
            kind: GameKind::MemoryMatch,
            stages: vec![
                arcade_stage("small", "스몰 · 6쌍", 1, GameKind::MemoryMatch),
                arcade_stage("classic", "클래식 · 8쌍", 2, GameKind::MemoryMatch),
                arcade_stage("grand", "그랜드 · 12쌍", 3, GameKind::MemoryMatch),
            ],
        }
    }
}

#[derive(Default)]
pub struct MazeGame;

impl GameModule for MazeGame {
    fn definition(&self) -> GameDefinition {
        GameDefinition {
            id: GameId::new("maze"),
            display_name: "미로 탈출".to_string(),
            kind: GameKind::Maze,
            stages: vec![
                arcade_stage("alley", "골목 · 15×9", 1, GameKind::Maze),
                arcade_stage("labyrinth", "미궁 · 25×13", 2, GameKind::Maze),
                arcade_stage("abyss", "심연 · 35×17", 3, GameKind::Maze),
            ],
        }
    }
}

#[derive(Clone)]
pub struct GameCatalog {
    games: Vec<GameDefinition>,
}

impl Default for GameCatalog {
    fn default() -> Self {
        Self::try_from_modules([
            Box::new(MathGame) as Box<dyn GameModule>,
            Box::new(SnakeGame) as Box<dyn GameModule>,
            Box::new(TicTacToeGame) as Box<dyn GameModule>,
            Box::new(TwentyFortyEightGame) as Box<dyn GameModule>,
            Box::new(SudokuGame) as Box<dyn GameModule>,
            Box::new(MinesweeperGame) as Box<dyn GameModule>,
            Box::new(ConnectFourGame) as Box<dyn GameModule>,
            Box::new(MemoryMatchGame) as Box<dyn GameModule>,
            Box::new(MazeGame) as Box<dyn GameModule>,
            Box::new(GamblingGame) as Box<dyn GameModule>,
            Box::new(BreakoutGame) as Box<dyn GameModule>,
        ])
        .expect("built-in game catalog must be valid")
    }
}

impl GameCatalog {
    pub fn from_modules<I>(modules: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn GameModule>>,
    {
        Self::try_from_modules(modules).expect("game catalog must be valid")
    }

    pub fn try_from_modules<I>(modules: I) -> Result<Self, CatalogError>
    where
        I: IntoIterator<Item = Box<dyn GameModule>>,
    {
        let mut catalog = Self { games: Vec::new() };
        for module in modules {
            catalog.try_register(module.as_ref())?;
        }
        Ok(catalog)
    }

    pub fn register(&mut self, module: &dyn GameModule) {
        self.try_register(module)
            .expect("registered game definition must be valid");
    }

    pub fn try_register(&mut self, module: &dyn GameModule) -> Result<(), CatalogError> {
        let definition = module.definition();
        validate_game_definition(&definition)?;
        if self.games.iter().any(|game| game.id == definition.id) {
            return Err(CatalogError::DuplicateGameId {
                game_id: definition.id.to_string(),
            });
        }
        self.games.push(definition);
        Ok(())
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

fn validate_game_definition(game: &GameDefinition) -> Result<(), CatalogError> {
    let game_id = game.id.as_str();
    if game_id.trim().is_empty() {
        return Err(CatalogError::EmptyGameId);
    }
    if game.display_name.trim().is_empty() {
        return Err(CatalogError::EmptyGameName {
            game_id: game_id.to_string(),
        });
    }
    if game.stages.is_empty() {
        return Err(CatalogError::EmptyStages {
            game_id: game_id.to_string(),
        });
    }

    let mut stage_ids = HashSet::with_capacity(game.stages.len());
    for stage in &game.stages {
        let stage_id = stage.id.as_str();
        if stage_id.trim().is_empty() {
            return Err(CatalogError::EmptyStageId {
                game_id: game_id.to_string(),
            });
        }
        if !stage_ids.insert(stage_id) {
            return Err(CatalogError::DuplicateStageId {
                game_id: game_id.to_string(),
                stage_id: stage_id.to_string(),
            });
        }
        if stage.display_name.trim().is_empty() {
            return Err(CatalogError::EmptyStageName {
                game_id: game_id.to_string(),
                stage_id: stage_id.to_string(),
            });
        }
        if game.kind == GameKind::Gambling {
            if stage.game_kind == GameKind::Gambling {
                return Err(CatalogError::NestedGamblingStage {
                    stage_id: stage_id.to_string(),
                });
            }
        } else if stage.game_kind != game.kind {
            return Err(CatalogError::StageKindMismatch {
                game_id: game_id.to_string(),
                stage_id: stage_id.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_contains_all_builtin_games_and_stages() {
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
        assert_eq!(
            snake
                .stages
                .iter()
                .map(|stage| stage.id.as_str())
                .collect::<Vec<_>>(),
            ["relaxed", "classic-1", "turbo"]
        );
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
        assert_eq!(
            catalog
                .find_game_by_str("sudoku")
                .expect("sudoku")
                .stages
                .len(),
            3
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
        for (id, kind) in [
            ("minesweeper", GameKind::Minesweeper),
            ("connect-four", GameKind::ConnectFour),
            ("memory", GameKind::MemoryMatch),
            ("maze", GameKind::Maze),
        ] {
            let game = catalog.find_game_by_str(id).expect("new arcade game");
            assert_eq!(game.kind, kind);
            assert_eq!(game.stages.len(), 3);
            assert_eq!(
                game.stages
                    .iter()
                    .map(|stage| stage.difficulty_order)
                    .collect::<Vec<_>>(),
                [1, 2, 3]
            );
        }
    }

    #[test]
    fn catalog_rejects_duplicate_game_ids() {
        let result = GameCatalog::try_from_modules([
            Box::new(MathGame) as Box<dyn GameModule>,
            Box::new(MathGame) as Box<dyn GameModule>,
        ]);
        assert_eq!(
            result.err(),
            Some(CatalogError::DuplicateGameId {
                game_id: "math".to_string(),
            })
        );
    }

    #[test]
    fn catalog_rejects_duplicate_stage_ids() {
        struct DuplicateStages;

        impl GameModule for DuplicateStages {
            fn definition(&self) -> GameDefinition {
                let mut definition = SnakeGame.definition();
                definition.stages.push(definition.stages[0].clone());
                definition
            }
        }

        let result =
            GameCatalog::try_from_modules([Box::new(DuplicateStages) as Box<dyn GameModule>]);
        assert_eq!(
            result.err(),
            Some(CatalogError::DuplicateStageId {
                game_id: "snake".to_string(),
                stage_id: "relaxed".to_string(),
            })
        );
    }
}
