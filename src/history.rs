use std::cmp::Ordering;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{GameId, StageId};
use crate::round::RoundResult;

pub type PlayRecord = RoundResult;

pub const HISTORY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("기록 파일을 읽을 수 없습니다: {0}")]
    Io(#[from] io::Error),
    #[error("기록 파일 형식이 손상되었습니다: {0}")]
    Json(#[from] serde_json::Error),
    #[error("지원하지 않는 기록 스키마 버전입니다: {0}")]
    UnsupportedSchema(u32),
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryFile {
    schema_version: u32,
    records: Vec<PlayRecord>,
    #[serde(default)]
    wallet_won: u64,
}

pub trait HistoryStore: Send {
    fn load_all(&self) -> Result<Vec<PlayRecord>, HistoryError>;
    fn add(&self, record: PlayRecord) -> Result<(), HistoryError>;
    fn best_for_stage(
        &self,
        game_id: &GameId,
        stage_id: &StageId,
    ) -> Result<Option<PlayRecord>, HistoryError>;
    fn recent(&self, limit: usize) -> Result<Vec<PlayRecord>, HistoryError>;
    fn wallet_balance(&self) -> Result<u64, HistoryError>;
    fn add_won(&self, amount: u64) -> Result<(), HistoryError>;
    fn spend_won(&self, amount: u64) -> Result<bool, HistoryError>;
}

#[derive(Clone, Debug)]
pub struct JsonHistoryStore {
    path: PathBuf,
}

impl Default for JsonHistoryStore {
    fn default() -> Self {
        let path = ProjectDirs::from("com", "cli-game", "cli-game")
            .map(|dirs| dirs.data_dir().join("history.json"))
            .unwrap_or_else(|| PathBuf::from(".cli-game").join("history.json"));
        Self { path }
    }
}

impl JsonHistoryStore {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn ensure_file_exists(&self) -> Result<(), HistoryError> {
        if self.path.exists() {
            return Ok(());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.write_file(&HistoryFile {
            schema_version: HISTORY_SCHEMA_VERSION,
            records: Vec::new(),
            wallet_won: 0,
        })
    }

    fn read_file(&self) -> Result<HistoryFile, HistoryError> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let history: HistoryFile = serde_json::from_reader(reader)?;
        if history.schema_version != HISTORY_SCHEMA_VERSION {
            return Err(HistoryError::UnsupportedSchema(history.schema_version));
        }
        Ok(history)
    }

    fn write_file(&self, history: &HistoryFile) -> Result<(), HistoryError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary_path =
            self.path
                .with_extension(format!("json.tmp.{}.{}", std::process::id(), nonce));

        let write_result = (|| -> Result<(), HistoryError> {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary_path)?;
            serde_json::to_writer_pretty(&mut file, history)?;
            file.write_all(b"\n")?;
            file.flush()?;
            file.sync_all()?;
            fs::rename(&temporary_path, &self.path)?;
            Ok(())
        })();

        if write_result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        write_result
    }
}

impl HistoryStore for JsonHistoryStore {
    fn load_all(&self) -> Result<Vec<PlayRecord>, HistoryError> {
        self.ensure_file_exists()?;
        Ok(self.read_file()?.records)
    }

    fn add(&self, record: PlayRecord) -> Result<(), HistoryError> {
        self.ensure_file_exists()?;
        let mut history = self.read_file()?;
        history.records.push(record);
        self.write_file(&history)
    }

    fn best_for_stage(
        &self,
        game_id: &GameId,
        stage_id: &StageId,
    ) -> Result<Option<PlayRecord>, HistoryError> {
        let records = self.load_all()?;
        Ok(records
            .into_iter()
            .filter(|record| &record.game_id == game_id && &record.stage_id == stage_id)
            .max_by(compare_records))
    }

    fn recent(&self, limit: usize) -> Result<Vec<PlayRecord>, HistoryError> {
        let mut records = self.load_all()?;
        records.sort_by_key(|record| std::cmp::Reverse(record.started_at));
        records.truncate(limit);
        Ok(records)
    }

    fn wallet_balance(&self) -> Result<u64, HistoryError> {
        self.ensure_file_exists()?;
        Ok(self.read_file()?.wallet_won)
    }

    fn add_won(&self, amount: u64) -> Result<(), HistoryError> {
        self.ensure_file_exists()?;
        let mut history = self.read_file()?;
        history.wallet_won = history.wallet_won.saturating_add(amount);
        self.write_file(&history)
    }

    fn spend_won(&self, amount: u64) -> Result<bool, HistoryError> {
        self.ensure_file_exists()?;
        let mut history = self.read_file()?;
        if history.wallet_won < amount {
            return Ok(false);
        }
        history.wallet_won -= amount;
        self.write_file(&history)?;
        Ok(true)
    }
}

fn compare_records(left: &PlayRecord, right: &PlayRecord) -> Ordering {
    left.score
        .cmp(&right.score)
        .then_with(|| left.accuracy.total_cmp(&right.accuracy))
        .then_with(|| left.best_streak.cmp(&right.best_streak))
        .then_with(|| left.started_at.cmp(&right.started_at))
}

#[derive(Clone, Default)]
pub struct MemoryHistoryStore {
    records: Arc<Mutex<Vec<PlayRecord>>>,
    wallet_won: Arc<Mutex<u64>>,
}

impl MemoryHistoryStore {
    pub fn records(&self) -> Vec<PlayRecord> {
        self.records.lock().expect("history lock").clone()
    }
}

impl HistoryStore for MemoryHistoryStore {
    fn load_all(&self) -> Result<Vec<PlayRecord>, HistoryError> {
        Ok(self.records())
    }

    fn add(&self, record: PlayRecord) -> Result<(), HistoryError> {
        self.records.lock().expect("history lock").push(record);
        Ok(())
    }

    fn best_for_stage(
        &self,
        game_id: &GameId,
        stage_id: &StageId,
    ) -> Result<Option<PlayRecord>, HistoryError> {
        Ok(self
            .records()
            .into_iter()
            .filter(|record| &record.game_id == game_id && &record.stage_id == stage_id)
            .max_by(compare_records))
    }

    fn recent(&self, limit: usize) -> Result<Vec<PlayRecord>, HistoryError> {
        let mut records = self.records();
        records.sort_by_key(|record| std::cmp::Reverse(record.started_at));
        records.truncate(limit);
        Ok(records)
    }

    fn wallet_balance(&self) -> Result<u64, HistoryError> {
        Ok(*self.wallet_won.lock().expect("wallet lock"))
    }

    fn add_won(&self, amount: u64) -> Result<(), HistoryError> {
        let mut wallet = self.wallet_won.lock().expect("wallet lock");
        *wallet = wallet.saturating_add(amount);
        Ok(())
    }

    fn spend_won(&self, amount: u64) -> Result<bool, HistoryError> {
        let mut wallet = self.wallet_won.lock().expect("wallet lock");
        if *wallet < amount {
            return Ok(false);
        }
        *wallet -= amount;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::round::RoundStatus;

    fn record(score: u32, attempts: u32, streak: u32) -> PlayRecord {
        PlayRecord {
            game_id: GameId::new("math"),
            stage_id: StageId::new("addition-1"),
            app_version: "1.0.0".to_string(),
            started_at: Utc::now(),
            actual_play_time_ms: 1_000,
            correct_answers: score,
            attempts,
            accuracy: if attempts == 0 {
                0.0
            } else {
                f64::from(score) / f64::from(attempts)
            },
            best_streak: streak,
            score,
            status: RoundStatus::Completed,
        }
    }

    #[test]
    fn missing_history_file_is_created_and_loadable() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("nested").join("history.json");
        let store = JsonHistoryStore::at(&path);
        assert!(!path.exists());
        assert!(store.load_all().expect("empty history").is_empty());
        assert!(path.exists());
    }

    #[test]
    fn records_round_trip_and_best_record_uses_the_requested_tiebreakers() {
        let directory = tempfile::tempdir().expect("temp directory");
        let store = JsonHistoryStore::at(directory.path().join("history.json"));
        store.add(record(2, 4, 2)).expect("first record");
        store.add(record(3, 5, 1)).expect("second record");
        let records = store.load_all().expect("records");
        assert_eq!(records.len(), 2);
        let best = store
            .best_for_stage(&GameId::new("math"), &StageId::new("addition-1"))
            .expect("best lookup")
            .expect("best record");
        assert_eq!(best.score, 3);
    }

    #[test]
    fn corrupt_history_returns_an_error_without_overwriting_the_file() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("history.json");
        fs::write(&path, b"not json").expect("corrupt file");
        let store = JsonHistoryStore::at(&path);
        assert!(store.load_all().is_err());
        assert_eq!(fs::read(&path).expect("file bytes"), b"not json");
        assert!(store.add(record(1, 1, 1)).is_err());
        assert_eq!(fs::read(&path).expect("file bytes"), b"not json");
    }

    #[test]
    fn wallet_balance_survives_history_records_and_rejects_overspending() {
        let directory = tempfile::tempdir().expect("temp directory");
        let store = JsonHistoryStore::at(directory.path().join("history.json"));
        store.add_won(3).expect("wallet credit");
        store.add(record(1, 1, 1)).expect("record");
        assert_eq!(store.wallet_balance().expect("wallet balance"), 3);
        assert!(store.spend_won(2).expect("wallet spend"));
        assert_eq!(store.wallet_balance().expect("wallet balance"), 1);
        assert!(!store.spend_won(2).expect("rejected spend"));
        assert_eq!(store.wallet_balance().expect("wallet balance"), 1);
    }
}
