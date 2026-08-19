use std::fmt;

use clap::Parser;
use thiserror::Error;

use crate::domain::{GameCatalog, GameId, StageId};

#[derive(Debug, Parser)]
#[command(
    name = "tcade",
    bin_name = "tcade",
    version,
    about = "터미널에서 즐기는 작고 강력한 아케이드",
    long_about = "게임과 스테이지를 선택해 터미널에서 플레이하는 TERMCADE입니다."
)]
pub struct Cli {
    #[arg(value_name = "COMMAND")]
    pub path: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliRoute {
    GameSelect,
    StageSelect { game_id: GameId },
    Ready { game_id: GameId, stage_id: StageId },
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CliError {
    #[error("알 수 없는 게임 '{id}'입니다. 사용 가능한 게임: {available}")]
    UnknownGame { id: String, available: String },
    #[error(
        "게임 '{game_id}'에 스테이지 '{stage_id}'가 없습니다. 사용 가능한 스테이지: {available}"
    )]
    UnknownStage {
        game_id: String,
        stage_id: String,
        available: String,
    },
    #[error("명령어가 너무 많습니다. 사용법: tcade [game [stage]]")]
    TooManyArguments,
}

pub fn resolve_route(path: &[String], catalog: &GameCatalog) -> Result<CliRoute, CliError> {
    let path = path
        .strip_prefix(&["tcade".to_string()])
        .or_else(|| path.strip_prefix(&["cli-game".to_string()]))
        .unwrap_or(path);
    match path {
        [] => Ok(CliRoute::GameSelect),
        [game_id] => {
            let game = catalog
                .find_game_by_str(game_id)
                .ok_or_else(|| CliError::UnknownGame {
                    id: game_id.clone(),
                    available: available_games(catalog),
                })?;
            Ok(CliRoute::StageSelect {
                game_id: game.id.clone(),
            })
        }
        [game_id, stage_id] => {
            let game = catalog
                .find_game_by_str(game_id)
                .ok_or_else(|| CliError::UnknownGame {
                    id: game_id.clone(),
                    available: available_games(catalog),
                })?;
            let stage = game
                .stages
                .iter()
                .find(|stage| stage.id.as_str() == stage_id)
                .ok_or_else(|| CliError::UnknownStage {
                    game_id: game_id.clone(),
                    stage_id: stage_id.clone(),
                    available: game
                        .stages
                        .iter()
                        .map(|stage| stage.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                })?;
            Ok(CliRoute::Ready {
                game_id: game.id.clone(),
                stage_id: stage.id.clone(),
            })
        }
        _ => Err(CliError::TooManyArguments),
    }
}

fn available_games(catalog: &GameCatalog) -> String {
    catalog
        .games()
        .iter()
        .map(|game| game.id.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for CliRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameSelect => write!(formatter, "game-select"),
            Self::StageSelect { game_id } => write!(formatter, "stage-select:{game_id}"),
            Self::Ready { game_id, stage_id } => write!(formatter, "ready:{game_id}:{stage_id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_paths_route_to_the_expected_screen() {
        let catalog = GameCatalog::default();
        assert_eq!(
            resolve_route(&[], &catalog).expect("game list"),
            CliRoute::GameSelect
        );
        assert_eq!(
            resolve_route(&["math".to_string()], &catalog).expect("stage list"),
            CliRoute::StageSelect {
                game_id: GameId::new("math")
            }
        );
        assert_eq!(
            resolve_route(
                &[
                    "tcade".to_string(),
                    "math".to_string(),
                    "division-1".to_string()
                ],
                &catalog,
            )
            .expect("direct stage"),
            CliRoute::Ready {
                game_id: GameId::new("math"),
                stage_id: StageId::new("division-1"),
            }
        );
        assert_eq!(
            resolve_route(&["2048".to_string(), "classic-1".to_string()], &catalog,)
                .expect("2048 stage"),
            CliRoute::Ready {
                game_id: GameId::new("2048"),
                stage_id: StageId::new("classic-1"),
            }
        );
        assert_eq!(
            resolve_route(
                &["gambling".to_string(), "blackjack-1".to_string()],
                &catalog,
            )
            .expect("blackjack stage"),
            CliRoute::Ready {
                game_id: GameId::new("gambling"),
                stage_id: StageId::new("blackjack-1"),
            }
        );
        assert_eq!(
            resolve_route(&["breakout".to_string(), "classic-1".to_string()], &catalog,)
                .expect("breakout stage"),
            CliRoute::Ready {
                game_id: GameId::new("breakout"),
                stage_id: StageId::new("classic-1"),
            }
        );
    }

    #[test]
    fn invalid_ids_include_available_choices() {
        let catalog = GameCatalog::default();
        let error = resolve_route(&["unknown".to_string()], &catalog).expect_err("unknown game");
        assert!(error.to_string().contains("math"));

        let error = resolve_route(&["math".to_string(), "unknown-stage".to_string()], &catalog)
            .expect_err("unknown stage");
        assert!(error.to_string().contains("addition-1"));
    }
}
