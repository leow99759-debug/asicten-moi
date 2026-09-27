//! SQLite store: command history (last 500, SPEC §3.4) and key/value settings.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Result;

const HISTORY_LIMIT: i64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Status {
    Done,
    Error,
    Cancelled,
    NoInternet,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
            Self::NoInternet => "no_internet",
        }
    }

    fn parse(s: &str) -> Self {
        match s {
            "done" => Self::Done,
            "cancelled" => Self::Cancelled,
            "no_internet" => Self::NoInternet,
            _ => Self::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HistoryEntry {
    #[ts(type = "number")]
    pub id: i64,
    /// Unix time, milliseconds.
    #[ts(type = "number")]
    pub ts: i64,
    pub phrase: String,
    pub command_id: Option<String>,
    pub status: Status,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS history(
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 ts INTEGER NOT NULL,
                 phrase TEXT NOT NULL,
                 command_id TEXT,
                 status TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )?;
        Ok(Self { conn })
    }

    /// Append a history row and prune to the last 500.
    pub fn add_history(
        &self,
        ts: i64,
        phrase: &str,
        command_id: Option<&str>,
        status: Status,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO history(ts, phrase, command_id, status) VALUES (?1, ?2, ?3, ?4)",
            params![ts, phrase, command_id, status.as_str()],
        )?;
        let id = self.conn.last_insert_rowid();
        self.conn.execute(
            "DELETE FROM history WHERE id <= ?1 - ?2",
            params![id, HISTORY_LIMIT],
        )?;
        Ok(id)
    }

    /// Newest first.
    pub fn history(&self, limit: u32) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ts, phrase, command_id, status FROM history ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit], |r| {
            Ok(HistoryEntry {
                id: r.get(0)?,
                ts: r.get(1)?,
                phrase: r.get(2)?,
                command_id: r.get(3)?,
                status: Status::parse(&r.get::<_, String>(4)?),
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
                r.get(0)
            })
            .optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_newest_first_and_pruned() {
        let db = Db::open_in_memory().expect("db");
        for i in 0..510 {
            db.add_history(i, &format!("фраза {i}"), Some("cmd"), Status::Done)
                .expect("add");
        }
        db.add_history(999, "нет сети", None, Status::NoInternet)
            .expect("add");
        let all = db.history(1000).expect("history");
        assert_eq!(all.len(), 500);
        assert_eq!(all[0].phrase, "нет сети");
        assert_eq!(all[0].status, Status::NoInternet);
        assert_eq!(all[0].command_id, None);
        assert_eq!(all[1].phrase, "фраза 509");
    }

    #[test]
    fn settings_upsert() {
        let db = Db::open_in_memory().expect("db");
        assert_eq!(db.setting("k").expect("get"), None);
        db.set_setting("k", "1").expect("set");
        db.set_setting("k", "2").expect("set");
        assert_eq!(db.setting("k").expect("get").as_deref(), Some("2"));
    }

    #[test]
    fn file_db_persists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("jarvis.db");
        Db::open(&path)
            .expect("db")
            .add_history(1, "привет", None, Status::Done)
            .expect("add");
        assert_eq!(
            Db::open(&path).expect("db").history(10).expect("h").len(),
            1
        );
    }
}
