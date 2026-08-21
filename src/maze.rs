use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::domain::{GameId, StageId};
use crate::round::{Clock, RoundResult, RoundStatus, SystemClock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MazeDifficulty {
    Alley,
    Labyrinth,
    Abyss,
}

impl MazeDifficulty {
    pub fn from_stage_id(stage_id: &StageId) -> Self {
        match stage_id.as_str() {
            "labyrinth" => Self::Labyrinth,
            "abyss" => Self::Abyss,
            _ => Self::Alley,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Alley => "ALLEY",
            Self::Labyrinth => "LABYRINTH",
            Self::Abyss => "ABYSS",
        }
    }

    fn dimensions(self) -> (usize, usize) {
        match self {
            Self::Alley => (15, 9),
            Self::Labyrinth => (25, 13),
            Self::Abyss => (35, 17),
        }
    }
}

pub struct MazeSession {
    game_id: GameId,
    stage_id: StageId,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    clock: Arc<dyn Clock>,
    difficulty: MazeDifficulty,
    width: usize,
    height: usize,
    walls: Vec<bool>,
    visited: Vec<bool>,
    player: usize,
    goal: usize,
    steps: u32,
    optimal_steps: u32,
    game_over: bool,
}

impl MazeSession {
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
        let difficulty = MazeDifficulty::from_stage_id(&stage_id);
        let (width, height) = difficulty.dimensions();
        let mut walls = vec![true; width * height];
        carve_maze(&mut walls, width, height, seed);
        let player = width + 1;
        let goal = (height - 2) * width + width - 2;
        let optimal_steps = shortest_path(&walls, width, height, player, goal);
        let mut visited = vec![false; width * height];
        visited[player] = true;
        Self {
            game_id,
            stage_id,
            started_at: Utc::now(),
            started_instant: clock.now(),
            clock,
            difficulty,
            width,
            height,
            walls,
            visited,
            player,
            goal,
            steps: 0,
            optimal_steps,
            game_over: false,
        }
    }

    pub fn difficulty(&self) -> MazeDifficulty {
        self.difficulty
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn player(&self) -> usize {
        self.player
    }

    pub fn goal(&self) -> usize {
        self.goal
    }

    pub fn steps(&self) -> u32 {
        self.steps
    }

    pub fn optimal_steps(&self) -> u32 {
        self.optimal_steps
    }

    pub fn is_wall(&self, index: usize) -> bool {
        self.walls.get(index).copied().unwrap_or(true)
    }

    pub fn was_visited(&self, index: usize) -> bool {
        self.visited.get(index).copied().unwrap_or(false)
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn elapsed(&self) -> Duration {
        self.clock
            .now()
            .saturating_duration_since(self.started_instant)
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        let x = (self.player % self.width) as i32 + dx;
        let y = (self.player / self.width) as i32 + dy;
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return None;
        }
        let next = y as usize * self.width + x as usize;
        if self.walls[next] {
            return None;
        }
        self.player = next;
        self.visited[next] = true;
        self.steps += 1;
        if self.player == self.goal {
            self.game_over = true;
            return Some(self.result(RoundStatus::Completed));
        }
        None
    }

    pub fn finish_abandoned(&mut self) -> Option<RoundResult> {
        if self.game_over {
            return None;
        }
        self.game_over = true;
        Some(self.result(RoundStatus::Abandoned))
    }

    fn result(&self, status: RoundStatus) -> RoundResult {
        let completed = self.player == self.goal;
        let efficiency = if self.steps == 0 {
            0.0
        } else {
            (f64::from(self.optimal_steps) / f64::from(self.steps)).min(1.0)
        };
        let score = if completed {
            (efficiency * 1000.0).round() as u32
        } else {
            0
        };
        RoundResult {
            game_id: self.game_id.clone(),
            stage_id: self.stage_id.clone(),
            app_version: crate::APP_VERSION.to_string(),
            started_at: self.started_at,
            actual_play_time_ms: self.elapsed().as_millis() as u64,
            correct_answers: u32::from(completed),
            attempts: self.steps,
            accuracy: efficiency,
            best_streak: self.steps,
            score,
            status,
        }
    }
}

fn carve_maze(walls: &mut [bool], width: usize, height: usize, seed: u64) {
    let mut rng = StdRng::seed_from_u64(seed);
    let start = width + 1;
    walls[start] = false;
    let mut stack = vec![start];
    while let Some(&current) = stack.last() {
        let x = current % width;
        let y = current / width;
        let mut directions = [(0i32, -2i32), (2, 0), (0, 2), (-2, 0)];
        directions.shuffle(&mut rng);
        let next = directions.into_iter().find_map(|(dx, dy)| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx <= 0 || nx >= width as i32 - 1 || ny <= 0 || ny >= height as i32 - 1 {
                return None;
            }
            let destination = ny as usize * width + nx as usize;
            walls[destination].then_some((destination, dx, dy))
        });
        if let Some((destination, dx, dy)) = next {
            let middle_x = (x as i32 + dx / 2) as usize;
            let middle_y = (y as i32 + dy / 2) as usize;
            walls[middle_y * width + middle_x] = false;
            walls[destination] = false;
            stack.push(destination);
        } else {
            stack.pop();
        }
    }
}

fn shortest_path(walls: &[bool], width: usize, height: usize, start: usize, goal: usize) -> u32 {
    let mut distances = vec![u32::MAX; walls.len()];
    distances[start] = 0;
    let mut queue = VecDeque::from([start]);
    while let Some(current) = queue.pop_front() {
        if current == goal {
            return distances[current];
        }
        let x = current % width;
        let y = current / width;
        for (dx, dy) in [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                continue;
            }
            let next = ny as usize * width + nx as usize;
            if !walls[next] && distances[next] == u32::MAX {
                distances[next] = distances[current] + 1;
                queue.push_back(next);
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_maze_has_a_path_to_the_goal_at_every_stage() {
        for stage in ["alley", "labyrinth", "abyss"] {
            for seed in 0..24 {
                let session =
                    MazeSession::with_seed(GameId::new("maze"), StageId::new(stage), seed);
                assert!(session.optimal_steps() > 0, "{stage}/{seed}");
                assert!(!session.is_wall(session.player()));
                assert!(!session.is_wall(session.goal()));
            }
        }
    }

    #[test]
    fn player_cannot_walk_through_outer_wall() {
        let mut session = MazeSession::with_seed(GameId::new("maze"), StageId::new("alley"), 4);
        let start = session.player();
        session.move_player(-1, 0);
        assert_eq!(session.player(), start);
        assert_eq!(session.steps(), 0);
    }

    #[test]
    fn same_seed_builds_the_same_maze() {
        let left = MazeSession::with_seed(GameId::new("maze"), StageId::new("labyrinth"), 99);
        let right = MazeSession::with_seed(GameId::new("maze"), StageId::new("labyrinth"), 99);
        assert_eq!(left.walls, right.walls);
    }
}
