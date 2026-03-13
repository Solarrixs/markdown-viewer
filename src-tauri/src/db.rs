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

    /// Create an in-memory database for testing purposes.
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
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

    pub fn get_file_paths_by_statuses(&self, statuses: Option<&[&str]>) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let (sql, params_vec): (String, Vec<String>) = match statuses {
            Some(s) if !s.is_empty() => {
                let placeholders: Vec<String> = (1..=s.len()).map(|i| format!("?{}", i)).collect();
                (
                    format!("SELECT path FROM files WHERE status IN ({})", placeholders.join(", ")),
                    s.iter().map(|x| x.to_string()).collect(),
                )
            }
            _ => (
                "SELECT path FROM files WHERE status != 'archived'".to_string(),
                vec![],
            ),
        };
        let mut stmt = conn.prepare_cached(&sql)?;
        if params_vec.is_empty() {
            stmt.query_map([], |row| row.get(0))?.collect()
        } else {
            let params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
            stmt.query_map(params.as_slice(), |row| row.get(0))?.collect()
        }
    }

    pub fn restore_file(&self, path: &str, status: &str, reminder_time: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE files SET status = ?2, reminder_time = ?3 WHERE path = ?1",
            params![path, status, reminder_time],
        )?;
        Ok(())
    }

    pub fn rename_file_path(&self, old_path: &str, new_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE files SET path = ?2 WHERE path = ?1",
            params![old_path, new_path],
        )?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Initialization
    // ---------------------------------------------------------------

    #[test]
    fn test_new_in_memory_creates_tables() {
        let db = Database::new_in_memory().unwrap();
        // Verify all three tables exist by querying them
        let conn = db.conn.lock().unwrap();
        conn.execute_batch("SELECT 1 FROM watched_folders LIMIT 1").unwrap();
        conn.execute_batch("SELECT 1 FROM ignore_patterns LIMIT 1").unwrap();
        conn.execute_batch("SELECT 1 FROM files LIMIT 1").unwrap();
    }

    #[test]
    fn test_initialize_seeds_default_ignore_patterns() {
        let db = Database::new_in_memory().unwrap();
        let patterns = db.get_ignore_patterns_with_ids().unwrap();
        // Should have exactly 7 default patterns
        assert_eq!(patterns.len(), 7);
        let pattern_strings: Vec<&str> = patterns.iter().map(|p| p.pattern.as_str()).collect();
        assert!(pattern_strings.contains(&"_CONTEXT.md"));
        assert!(pattern_strings.contains(&"*.csv"));
        assert!(pattern_strings.contains(&".obsidian/*"));
    }

    #[test]
    fn test_initialize_is_idempotent() {
        let db = Database::new_in_memory().unwrap();
        // Call initialize again — should not duplicate seed patterns
        db.initialize().unwrap();
        let patterns = db.get_ignore_patterns_with_ids().unwrap();
        assert_eq!(patterns.len(), 7);
    }

    // ---------------------------------------------------------------
    // Watched Folders
    // ---------------------------------------------------------------

    #[test]
    fn test_add_and_get_watched_folder() {
        let db = Database::new_in_memory().unwrap();
        db.add_watched_folder("/home/user/notes").unwrap();
        let folders = db.get_watched_folders().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].path, "/home/user/notes");
        assert!(folders[0].active);
    }

    #[test]
    fn test_add_duplicate_watched_folder_is_ignored() {
        let db = Database::new_in_memory().unwrap();
        db.add_watched_folder("/home/user/notes").unwrap();
        db.add_watched_folder("/home/user/notes").unwrap();
        let folders = db.get_watched_folders().unwrap();
        assert_eq!(folders.len(), 1);
    }

    #[test]
    fn test_add_multiple_watched_folders() {
        let db = Database::new_in_memory().unwrap();
        db.add_watched_folder("/path/a").unwrap();
        db.add_watched_folder("/path/b").unwrap();
        db.add_watched_folder("/path/c").unwrap();
        let folders = db.get_watched_folders().unwrap();
        assert_eq!(folders.len(), 3);
    }

    #[test]
    fn test_remove_watched_folder() {
        let db = Database::new_in_memory().unwrap();
        db.add_watched_folder("/home/user/notes").unwrap();
        let folders = db.get_watched_folders().unwrap();
        let id = folders[0].id;

        db.remove_watched_folder(id).unwrap();
        let folders = db.get_watched_folders().unwrap();
        assert_eq!(folders.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_watched_folder_succeeds() {
        let db = Database::new_in_memory().unwrap();
        // Should not error — DELETE with no matching rows is fine
        db.remove_watched_folder(9999).unwrap();
    }

    #[test]
    fn test_get_watched_folders_returns_only_active() {
        let db = Database::new_in_memory().unwrap();
        db.add_watched_folder("/active").unwrap();
        db.add_watched_folder("/inactive").unwrap();

        // Manually deactivate one
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE watched_folders SET active = 0 WHERE path = '/inactive'",
                [],
            )
            .unwrap();
        }

        let folders = db.get_watched_folders().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].path, "/active");
    }

    #[test]
    fn test_get_watched_folders_empty() {
        let db = Database::new_in_memory().unwrap();
        let folders = db.get_watched_folders().unwrap();
        assert!(folders.is_empty());
    }

    // ---------------------------------------------------------------
    // Ignore Patterns
    // ---------------------------------------------------------------

    #[test]
    fn test_add_ignore_pattern() {
        let db = Database::new_in_memory().unwrap();
        let before = db.get_ignore_patterns_with_ids().unwrap().len();
        db.add_ignore_pattern("*.tmp").unwrap();
        let after = db.get_ignore_patterns_with_ids().unwrap();
        assert_eq!(after.len(), before + 1);
        assert!(after.iter().any(|p| p.pattern == "*.tmp"));
    }

    #[test]
    fn test_add_duplicate_ignore_pattern_is_ignored() {
        let db = Database::new_in_memory().unwrap();
        let before = db.get_ignore_patterns_with_ids().unwrap().len();
        db.add_ignore_pattern("*.csv").unwrap(); // already seeded
        let after = db.get_ignore_patterns_with_ids().unwrap().len();
        assert_eq!(before, after);
    }

    #[test]
    fn test_remove_ignore_pattern() {
        let db = Database::new_in_memory().unwrap();
        db.add_ignore_pattern("*.tmp").unwrap();
        let patterns = db.get_ignore_patterns_with_ids().unwrap();
        let tmp = patterns.iter().find(|p| p.pattern == "*.tmp").unwrap();

        db.remove_ignore_pattern(tmp.id).unwrap();
        let patterns = db.get_ignore_patterns_with_ids().unwrap();
        assert!(!patterns.iter().any(|p| p.pattern == "*.tmp"));
    }

    #[test]
    fn test_remove_nonexistent_ignore_pattern_succeeds() {
        let db = Database::new_in_memory().unwrap();
        db.remove_ignore_pattern(99999).unwrap();
    }

    // ---------------------------------------------------------------
    // Files — upsert
    // ---------------------------------------------------------------

    #[test]
    fn test_upsert_file_creates_new_record() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/hello.md", "2025-01-01T00:00:00").unwrap();
        let files = db.get_files_by_status("inbox").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "/notes/hello.md");
        assert_eq!(files[0].status, "unread");
        assert!(!files[0].pinned);
    }

    #[test]
    fn test_upsert_file_updates_existing_unread() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/hello.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/hello.md", "2025-01-02T00:00:00").unwrap();

        let files = db.get_files_by_status("inbox").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].last_modified.as_deref(), Some("2025-01-02T00:00:00"));
        assert_eq!(files[0].status, "unread");
    }

    #[test]
    fn test_upsert_file_preserves_archived_status() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/hello.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/hello.md", "archived").unwrap();

        // Re-upsert should keep archived
        db.upsert_file("/notes/hello.md", "2025-01-02T00:00:00").unwrap();
        let files = db.get_files_by_status("archive").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, "archived");
        // But last_modified should be updated
        assert_eq!(files[0].last_modified.as_deref(), Some("2025-01-02T00:00:00"));
    }

    #[test]
    fn test_upsert_file_resets_read_to_unread() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/hello.md", "2025-01-01T00:00:00").unwrap();
        db.mark_as_read("/notes/hello.md").unwrap();

        // Re-upsert should reset to unread (since file was modified)
        db.upsert_file("/notes/hello.md", "2025-01-02T00:00:00").unwrap();
        let files = db.get_files_by_status("inbox").unwrap();
        assert_eq!(files[0].status, "unread");
    }

    #[test]
    fn test_upsert_file_if_missing_creates_new() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file_if_missing("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        let files = db.get_files_by_status("inbox").unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_upsert_file_if_missing_does_not_overwrite() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.mark_as_read("/notes/a.md").unwrap();

        // Should not overwrite — file already exists
        db.upsert_file_if_missing("/notes/a.md", "2025-02-01T00:00:00").unwrap();
        let files = db.get_files_by_status("inbox").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, "read");
        assert_eq!(files[0].last_modified.as_deref(), Some("2025-01-01T00:00:00"));
    }

    // ---------------------------------------------------------------
    // Files — status & filtering
    // ---------------------------------------------------------------

    #[test]
    fn test_mark_status() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        let inbox = db.get_files_by_status("inbox").unwrap();
        assert!(inbox.is_empty());
        let archive = db.get_files_by_status("archive").unwrap();
        assert_eq!(archive.len(), 1);
    }

    #[test]
    fn test_mark_status_nonexistent_path() {
        let db = Database::new_in_memory().unwrap();
        // Should succeed (0 rows updated)
        db.mark_status("/nonexistent.md", "archived").unwrap();
    }

    #[test]
    fn test_mark_as_read() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.mark_as_read("/notes/a.md").unwrap();

        let files = db.get_files_by_status("inbox").unwrap();
        assert_eq!(files[0].status, "read");
    }

    #[test]
    fn test_mark_as_read_only_affects_unread() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        // mark_as_read should NOT change archived status
        db.mark_as_read("/notes/a.md").unwrap();
        let archive = db.get_files_by_status("archive").unwrap();
        assert_eq!(archive.len(), 1);
        assert_eq!(archive[0].status, "archived");
    }

    #[test]
    fn test_get_files_by_status_inbox_excludes_pinned_and_reminders() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/normal.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/pinned.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/reminder.md", "2025-01-01T00:00:00").unwrap();

        db.toggle_pinned("/notes/pinned.md").unwrap();
        db.set_reminder("/notes/reminder.md", Some("2025-06-01T00:00:00")).unwrap();

        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].path, "/notes/normal.md");
    }

    #[test]
    fn test_get_files_by_status_pinned() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/b.md", "2025-01-01T00:00:00").unwrap();
        db.toggle_pinned("/notes/a.md").unwrap();

        let pinned = db.get_files_by_status("pinned").unwrap();
        assert_eq!(pinned.len(), 1);
        assert_eq!(pinned[0].path, "/notes/a.md");
    }

    #[test]
    fn test_get_files_by_status_pinned_excludes_archived() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.toggle_pinned("/notes/a.md").unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        let pinned = db.get_files_by_status("pinned").unwrap();
        assert!(pinned.is_empty());
    }

    #[test]
    fn test_get_files_by_status_pinned_excludes_reminders() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.toggle_pinned("/notes/a.md").unwrap();
        db.set_reminder("/notes/a.md", Some("2025-06-01T00:00:00")).unwrap();

        let pinned = db.get_files_by_status("pinned").unwrap();
        assert!(pinned.is_empty());
    }

    #[test]
    fn test_get_files_by_status_reminders() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.set_reminder("/notes/a.md", Some("2025-06-01T00:00:00")).unwrap();

        let reminders = db.get_files_by_status("reminders").unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].path, "/notes/a.md");
    }

    #[test]
    fn test_get_files_by_status_reminders_excludes_archived() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.set_reminder("/notes/a.md", Some("2025-06-01T00:00:00")).unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        let reminders = db.get_files_by_status("reminders").unwrap();
        assert!(reminders.is_empty());
    }

    #[test]
    fn test_get_files_by_status_archive() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/b.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        let archive = db.get_files_by_status("archive").unwrap();
        assert_eq!(archive.len(), 1);
        assert_eq!(archive[0].path, "/notes/a.md");
    }

    #[test]
    fn test_get_files_by_status_unknown_filter_returns_all() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/b.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        let all = db.get_files_by_status("everything").unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_get_files_by_status_empty() {
        let db = Database::new_in_memory().unwrap();
        let files = db.get_files_by_status("inbox").unwrap();
        assert!(files.is_empty());
    }

    // ---------------------------------------------------------------
    // Files — pin
    // ---------------------------------------------------------------

    #[test]
    fn test_toggle_pinned() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();

        let pinned = db.toggle_pinned("/notes/a.md").unwrap();
        assert!(pinned);

        let unpinned = db.toggle_pinned("/notes/a.md").unwrap();
        assert!(!unpinned);
    }

    #[test]
    fn test_toggle_pinned_nonexistent_path_errors() {
        let db = Database::new_in_memory().unwrap();
        let result = db.toggle_pinned("/nonexistent.md");
        assert!(result.is_err());
    }

    // ---------------------------------------------------------------
    // Files — reminders
    // ---------------------------------------------------------------

    #[test]
    fn test_set_reminder() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.set_reminder("/notes/a.md", Some("2025-06-15T09:00:00")).unwrap();

        let reminders = db.get_files_by_status("reminders").unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].reminder_time.as_deref(), Some("2025-06-15T09:00:00"));
    }

    #[test]
    fn test_clear_reminder() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.set_reminder("/notes/a.md", Some("2025-06-15T09:00:00")).unwrap();
        db.set_reminder("/notes/a.md", None).unwrap();

        let reminders = db.get_files_by_status("reminders").unwrap();
        assert!(reminders.is_empty());
    }

    #[test]
    fn test_fire_reminder() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.mark_as_read("/notes/a.md").unwrap();
        db.set_reminder("/notes/a.md", Some("2025-06-15T09:00:00")).unwrap();

        db.fire_reminder("/notes/a.md").unwrap();

        // Should move back to inbox as unread with no reminder
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].status, "unread");
        assert!(inbox[0].reminder_time.is_none());
    }

    #[test]
    fn test_get_due_reminders_returns_past_reminders() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/past.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/future.md", "2025-01-01T00:00:00").unwrap();

        // Set one in the past and one far in the future
        db.set_reminder("/notes/past.md", Some("2000-01-01T00:00:00")).unwrap();
        db.set_reminder("/notes/future.md", Some("2099-12-31T23:59:59")).unwrap();

        let due = db.get_due_reminders().unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].path, "/notes/past.md");
    }

    #[test]
    fn test_get_due_reminders_excludes_archived() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.set_reminder("/notes/a.md", Some("2000-01-01T00:00:00")).unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        let due = db.get_due_reminders().unwrap();
        assert!(due.is_empty());
    }

    #[test]
    fn test_get_due_reminders_empty() {
        let db = Database::new_in_memory().unwrap();
        let due = db.get_due_reminders().unwrap();
        assert!(due.is_empty());
    }

    // ---------------------------------------------------------------
    // Files — get_file_paths_by_statuses
    // ---------------------------------------------------------------

    #[test]
    fn test_get_file_paths_by_statuses_specific() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/unread.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/read.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/archived.md", "2025-01-01T00:00:00").unwrap();
        db.mark_as_read("/notes/read.md").unwrap();
        db.mark_status("/notes/archived.md", "archived").unwrap();

        let paths = db.get_file_paths_by_statuses(Some(&["unread"])).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "/notes/unread.md");

        let paths = db.get_file_paths_by_statuses(Some(&["unread", "read"])).unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_get_file_paths_by_statuses_none_excludes_archived() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/b.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/b.md", "archived").unwrap();

        let paths = db.get_file_paths_by_statuses(None).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "/notes/a.md");
    }

    #[test]
    fn test_get_file_paths_by_statuses_empty_slice_excludes_archived() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        let empty: &[&str] = &[];
        let paths = db.get_file_paths_by_statuses(Some(empty)).unwrap();
        assert!(paths.is_empty() || !paths.contains(&"/notes/a.md".to_string()));
    }

    // ---------------------------------------------------------------
    // Files — restore
    // ---------------------------------------------------------------

    #[test]
    fn test_restore_file_from_archive() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        db.restore_file("/notes/a.md", "read", None).unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].status, "read");
    }

    #[test]
    fn test_restore_file_with_reminder() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.mark_status("/notes/a.md", "archived").unwrap();

        db.restore_file("/notes/a.md", "unread", Some("2025-12-01T00:00:00")).unwrap();
        let reminders = db.get_files_by_status("reminders").unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].reminder_time.as_deref(), Some("2025-12-01T00:00:00"));
    }

    #[test]
    fn test_restore_nonexistent_file_succeeds() {
        let db = Database::new_in_memory().unwrap();
        // 0 rows updated, no error
        db.restore_file("/nonexistent.md", "unread", None).unwrap();
    }

    // ---------------------------------------------------------------
    // Files — search
    // ---------------------------------------------------------------

    #[test]
    fn test_search_files_by_path() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/project-alpha/readme.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/project-beta/readme.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/other/todo.md", "2025-01-01T00:00:00").unwrap();

        let results = db.search_files("project").unwrap();
        assert_eq!(results.len(), 2);

        let results = db.search_files("alpha").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_files_no_match() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();

        let results = db.search_files("zzzznonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_files_empty_query_returns_all() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/a.md", "2025-01-01T00:00:00").unwrap();
        db.upsert_file("/notes/b.md", "2025-01-01T00:00:00").unwrap();

        let results = db.search_files("").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_files_limit_20() {
        let db = Database::new_in_memory().unwrap();
        for i in 0..25 {
            db.upsert_file(&format!("/notes/file{}.md", i), "2025-01-01T00:00:00").unwrap();
        }

        let results = db.search_files("file").unwrap();
        assert_eq!(results.len(), 20);
    }

    #[test]
    fn test_search_files_case_insensitive_like() {
        let db = Database::new_in_memory().unwrap();
        db.upsert_file("/notes/README.md", "2025-01-01T00:00:00").unwrap();

        // SQLite LIKE is case-insensitive for ASCII by default
        let results = db.search_files("readme").unwrap();
        assert_eq!(results.len(), 1);
    }

    // ---------------------------------------------------------------
    // Integration: combined workflows
    // ---------------------------------------------------------------

    #[test]
    fn test_full_file_lifecycle() {
        let db = Database::new_in_memory().unwrap();

        // File arrives (unread in inbox)
        db.upsert_file("/notes/meeting.md", "2025-01-01T00:00:00").unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].status, "unread");

        // User reads it
        db.mark_as_read("/notes/meeting.md").unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox[0].status, "read");

        // User pins it
        db.toggle_pinned("/notes/meeting.md").unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert!(inbox.is_empty()); // pinned files not in inbox
        let pinned = db.get_files_by_status("pinned").unwrap();
        assert_eq!(pinned.len(), 1);

        // User archives it
        db.mark_status("/notes/meeting.md", "archived").unwrap();
        let pinned = db.get_files_by_status("pinned").unwrap();
        assert!(pinned.is_empty());
        let archive = db.get_files_by_status("archive").unwrap();
        assert_eq!(archive.len(), 1);

        // User restores it
        db.restore_file("/notes/meeting.md", "unread", None).unwrap();
        // Still pinned from before, but now unarchived
        let pinned = db.get_files_by_status("pinned").unwrap();
        assert_eq!(pinned.len(), 1);
    }

    #[test]
    fn test_reminder_lifecycle() {
        let db = Database::new_in_memory().unwrap();

        db.upsert_file("/notes/followup.md", "2025-01-01T00:00:00").unwrap();
        db.mark_as_read("/notes/followup.md").unwrap();

        // Set reminder — should move out of inbox into reminders
        db.set_reminder("/notes/followup.md", Some("2025-06-01T09:00:00")).unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert!(inbox.is_empty());
        let reminders = db.get_files_by_status("reminders").unwrap();
        assert_eq!(reminders.len(), 1);

        // Fire reminder — should move back to inbox as unread
        db.fire_reminder("/notes/followup.md").unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].status, "unread");
        let reminders = db.get_files_by_status("reminders").unwrap();
        assert!(reminders.is_empty());
    }
}
