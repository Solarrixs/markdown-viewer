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
    pub conn: Mutex<Connection>,
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
        let mut stmt = conn.prepare("SELECT id, path, active FROM watched_folders WHERE active = 1")?;
        let rows = stmt.query_map([], |row| {
            Ok(WatchedFolder {
                id: row.get(0)?,
                path: row.get(1)?,
                active: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    // -- Ignore Patterns --

    pub fn get_ignore_patterns(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT pattern FROM ignore_patterns")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
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
                status = 'unread',
                last_modified = ?2",
            params![path, last_modified],
        )?;
        Ok(())
    }

    pub fn get_files_by_status(&self, filter: &str) -> Result<Vec<FileRecord>> {
        let conn = self.conn.lock().unwrap();
        let (where_clause, order) = match filter {
            "inbox" => ("WHERE status = 'unread'", "ORDER BY last_modified DESC"),
            "pinned" => ("WHERE pinned = 1", "ORDER BY last_modified DESC"),
            "reminders" => ("WHERE reminder_time IS NOT NULL", "ORDER BY reminder_time ASC"),
            _ => ("", "ORDER BY last_modified DESC"),
        };
        let sql = format!("SELECT {} FROM files {} {}", Self::FILE_COLUMNS, where_clause, order);
        let mut stmt = conn.prepare(&sql)?;
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

    pub fn set_pinned(&self, path: &str, pinned: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE files SET pinned = ?2 WHERE path = ?1",
            params![path, pinned],
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
            "SELECT {} FROM files WHERE reminder_time IS NOT NULL AND reminder_time <= ?1 AND status != 'unread'",
            Self::FILE_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
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
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![pattern], Self::row_to_file_record)?;
        rows.collect()
    }
}
