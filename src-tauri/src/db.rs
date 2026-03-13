use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WatchedFolder {
    pub id: i64,
    pub path: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnorePattern {
    pub id: i64,
    pub pattern: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub status: String,
    pub pinned: bool,
    pub reminder_time: Option<String>,
    pub last_modified: Option<String>,
    pub last_seen_hash: Option<String>,
    pub created_at: String,
}

pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(&db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn db_path() -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("com.engram.markinbox");
        path.push("markinbox.db");
        path
    }

    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS watched_folders (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                active BOOLEAN DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS ignore_patterns (
                id INTEGER PRIMARY KEY,
                pattern TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                status TEXT DEFAULT 'unread',
                pinned BOOLEAN DEFAULT 0,
                reminder_time TEXT,
                last_modified TEXT,
                last_seen_hash TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );",
        )?;

        // Seed default ignore patterns
        let defaults = vec![
            "_CONTEXT.md",
            "_INDEX.md",
            "*INDEX*",
            ".obsidian/*",
            "*.csv",
            "*.json",
            "*.py",
        ];
        for pattern in defaults {
            conn.execute(
                "INSERT OR IGNORE INTO ignore_patterns (pattern) VALUES (?1)",
                params![pattern],
            )?;
        }

        Ok(())
    }

    // -- Watched Folders --

    pub fn add_watched_folder(&self, path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO watched_folders (path) VALUES (?1)",
            params![path],
        )?;
        Ok(())
    }

    pub fn get_watched_folders(&self) -> Result<Vec<WatchedFolder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached("SELECT id, path, active FROM watched_folders WHERE active = 1")?;
        let rows = stmt.query_map([], |row| {
            Ok(WatchedFolder {
                id: row.get(0)?,
                path: row.get(1)?,
                active: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn remove_watched_folder(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM watched_folders WHERE id = ?1", params![id])?;
        Ok(())
    }

    // -- Ignore Patterns --

    pub fn get_ignore_patterns_with_ids(&self) -> Result<Vec<IgnorePattern>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached("SELECT id, pattern FROM ignore_patterns")?;
        let rows = stmt.query_map([], |row| {
            Ok(IgnorePattern {
                id: row.get(0)?,
                pattern: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    pub fn add_ignore_pattern(&self, pattern: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO ignore_patterns (pattern) VALUES (?1)",
            params![pattern],
        )?;
        Ok(())
    }

    pub fn remove_ignore_pattern(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM ignore_patterns WHERE id = ?1", params![id])?;
        Ok(())
    }

    // -- Files --

    const FILE_COLUMNS: &'static str = "id, path, status, pinned, reminder_time, last_modified, last_seen_hash, created_at";

    fn row_to_file_record(row: &rusqlite::Row) -> rusqlite::Result<FileRecord> {
        Ok(FileRecord {
            id: row.get(0)?,
            path: row.get(1)?,
            status: row.get(2)?,
            pinned: row.get(3)?,
            reminder_time: row.get(4)?,
            last_modified: row.get(5)?,
            last_seen_hash: row.get(6)?,
            created_at: row.get(7)?,
        })
    }

    pub fn upsert_file(&self, path: &str, last_modified: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO files (path, status, last_modified)
             VALUES (?1, 'unread', ?2)
             ON CONFLICT(path) DO UPDATE SET
                status = CASE WHEN status = 'archived' THEN 'archived' ELSE 'unread' END,
                last_modified = ?2",
            params![path, last_modified],
        )?;
        Ok(())
    }

    pub fn upsert_file_if_missing(&self, path: &str, last_modified: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO files (path, status, last_modified) VALUES (?1, 'unread', ?2)",
            params![path, last_modified],
        )?;
        Ok(())
    }

    pub fn get_files_by_status(&self, filter: &str) -> Result<Vec<FileRecord>> {
        let conn = self.conn.lock().unwrap();
        let (where_clause, order) = match filter {
            "inbox" => ("WHERE status IN ('unread', 'read') AND pinned = 0 AND reminder_time IS NULL", "ORDER BY last_modified DESC"),
            "archive" => ("WHERE status = 'archived'", "ORDER BY last_modified DESC"),
            "pinned" => ("WHERE pinned = 1 AND status != 'archived' AND reminder_time IS NULL", "ORDER BY last_modified DESC"),
            "reminders" => ("WHERE reminder_time IS NOT NULL AND status != 'archived'", "ORDER BY reminder_time ASC"),
            _ => ("", "ORDER BY last_modified DESC"),
        };
        let sql = format!("SELECT {} FROM files {} {}", Self::FILE_COLUMNS, where_clause, order);
        let mut stmt = conn.prepare_cached(&sql)?;
        let rows = stmt.query_map([], Self::row_to_file_record)?;
        rows.collect()
    }

    pub fn mark_status(&self, path: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE files SET status = ?2 WHERE path = ?1",
            params![path, status],
        )?;
        Ok(())
    }

    pub fn mark_as_read(&self, path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE files SET status = 'read' WHERE path = ?1 AND status = 'unread'",
            params![path],
        )?;
        Ok(())
    }

    pub fn toggle_pinned(&self, path: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let new_state: bool = conn.query_row(
            "UPDATE files SET pinned = NOT pinned WHERE path = ?1 RETURNING pinned",
            params![path],
            |row| row.get(0),
        )?;
        Ok(new_state)
    }

    pub fn fire_reminder(&self, path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE files SET status = 'unread', reminder_time = NULL WHERE path = ?1",
            params![path],
        )?;
        Ok(())
    }

    pub fn set_reminder(&self, path: &str, time: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE files SET reminder_time = ?2 WHERE path = ?1",
            params![path, time],
        )?;
        Ok(())
    }

    pub fn get_due_reminders(&self) -> Result<Vec<FileRecord>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let sql = format!(
            "SELECT {} FROM files WHERE reminder_time IS NOT NULL AND reminder_time <= ?1 AND status != 'archived'",
            Self::FILE_COLUMNS
        );
        let mut stmt = conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params![now], Self::row_to_file_record)?;
        rows.collect()
    }

    pub fn search_files(&self, query: &str) -> Result<Vec<FileRecord>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let sql = format!(
            "SELECT {} FROM files WHERE path LIKE ?1 ORDER BY last_modified DESC LIMIT 20",
            Self::FILE_COLUMNS
        );
        let mut stmt = conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params![pattern], Self::row_to_file_record)?;
        rows.collect()
    }
}
