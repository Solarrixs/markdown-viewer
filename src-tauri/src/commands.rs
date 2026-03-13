use crate::db::Database;
use crate::git;
use crate::watcher::WatcherHandle;
use serde::Serialize;
use std::sync::Arc;
use tauri::{Emitter, Manager, State};

#[derive(Debug, Serialize)]
pub struct FileListItem {
    pub path: String,
    pub filename: String,
    pub status: String,
    pub pinned: bool,
    pub reminder_time: Option<String>,
    pub last_modified: Option<String>,
    pub additions: i32,
    pub deletions: i32,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub filename: String,
}

#[tauri::command]
pub fn get_inbox_items(
    db: State<'_, Arc<Database>>,
    filter: &str,
) -> Result<Vec<FileListItem>, String> {
    let records: Vec<_> = db.get_files_by_status(filter)
        .map_err(|e| e.to_string())?
        .into_iter()
        // TODO: Replace per-record Path::exists() with soft-delete via watcher events (WARN-8)
        .filter(|r| std::path::Path::new(&r.path).exists())
        .collect();

    // Batch diff: one repo open + one diff for existing files only
    let paths: Vec<String> = records.iter().map(|r| r.path.clone()).collect();
    let diff_stats = git::get_batch_diff_stats(&paths);

    Ok(records
        .into_iter()
        .map(|record| {
            let stats = diff_stats
                .get(&record.path)
                .cloned()
                .unwrap_or_default();
            record_to_file_list_item(record, stats.additions, stats.deletions)
        })
        .collect())
}

#[tauri::command]
pub fn get_file_content(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_diff(path: &str) -> Result<git::DiffResult, String> {
    git::get_file_diff(path).ok_or_else(|| "No diff available".to_string())
}

#[tauri::command]
pub fn mark_as_read(db: State<'_, Arc<Database>>, path: &str) -> Result<(), String> {
    db.mark_as_read(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_as_archived(db: State<'_, Arc<Database>>, path: &str) -> Result<(), String> {
    db.mark_status(path, "archived").map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_pin(db: State<'_, Arc<Database>>, path: &str) -> Result<bool, String> {
    db.toggle_pinned(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_reminder(db: State<'_, Arc<Database>>, path: &str, time: &str) -> Result<(), String> {
    db.set_reminder(path, Some(time)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_watched_folder(
    db: State<'_, Arc<Database>>,
    watcher: State<'_, WatcherHandle>,
    path: &str,
) -> Result<(), String> {
    db.add_watched_folder(path).map_err(|e| e.to_string())?;
    watcher.restart();
    Ok(())
}

#[tauri::command]
pub fn get_watched_folders(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<crate::db::WatchedFolder>, String> {
    db.get_watched_folders().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_watched_folder(
    db: State<'_, Arc<Database>>,
    watcher: State<'_, WatcherHandle>,
    id: i64,
) -> Result<(), String> {
    db.remove_watched_folder(id).map_err(|e| e.to_string())?;
    watcher.restart();
    Ok(())
}

#[tauri::command]
pub fn search_files(
    db: State<'_, Arc<Database>>,
    query: &str,
) -> Result<Vec<FileInfo>, String> {
    let records = db.search_files(query).map_err(|e| e.to_string())?;
    Ok(records
        .into_iter()
        .map(|r| FileInfo {
            filename: git::extract_filename(&r.path),
            path: r.path,
        })
        .collect())
}

#[tauri::command]
pub fn toggle_always_on_top(app: tauri::AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Window not found")?;
    let current = window.is_always_on_top().map_err(|e| e.to_string())?;
    let new_state = !current;
    window
        .set_always_on_top(new_state)
        .map_err(|e| e.to_string())?;

    // Sync the menu checkbox with the new state
    if let Some(menu) = app.menu() {
        if let Some(item) = menu.get("always_on_top") {
            if let Some(check_item) = item.as_check_menuitem() {
                let _ = check_item.set_checked(new_state);
            }
        }
    }

    // Emit event so frontend store stays in sync
    let _ = app.emit("always-on-top-changed", new_state);

    Ok(new_state)
}

// -- Settings commands --

#[tauri::command]
pub fn get_ignore_patterns(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<crate::db::IgnorePattern>, String> {
    db.get_ignore_patterns_with_ids().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_ignore_pattern(
    db: State<'_, Arc<Database>>,
    watcher: State<'_, WatcherHandle>,
    pattern: &str,
) -> Result<(), String> {
    db.add_ignore_pattern(pattern).map_err(|e| e.to_string())?;
    watcher.restart();
    Ok(())
}

#[tauri::command]
pub fn remove_ignore_pattern(
    db: State<'_, Arc<Database>>,
    watcher: State<'_, WatcherHandle>,
    id: i64,
) -> Result<(), String> {
    db.remove_ignore_pattern(id).map_err(|e| e.to_string())?;
    watcher.restart();
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ContentSearchResult {
    pub path: String,
    pub filename: String,
    pub line_number: usize,
    pub context: String,
}

#[tauri::command]
pub fn search_file_contents(
    db: State<'_, Arc<Database>>,
    query: &str,
    statuses: Option<Vec<String>>,
) -> Result<Vec<ContentSearchResult>, String> {
    let status_refs: Option<Vec<&str>> = statuses.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect());
    let paths = db
        .get_file_paths_by_statuses(status_refs.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(search_contents_in_paths(paths, query, 20))
}

// -- Open arbitrary file --

/// Validate a path is a readable markdown file, track it in the DB, and return FileInfo.
fn validate_and_track(db: &Database, path: &str) -> Result<FileInfo, String> {
    let meta = std::fs::metadata(path).map_err(|_| format!("File not found: {}", path))?;
    if !meta.is_file() {
        return Err(format!("Not a file: {}", path));
    }
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !crate::watcher::MARKDOWN_EXTENSIONS.contains(&ext) {
        return Err(format!("Only markdown files (.md, .markdown, .txt) can be opened: {}", path));
    }
    let mtime = crate::watcher::get_file_mtime_string(path);
    db.upsert_file_if_missing(path, &mtime)
        .map_err(|e| e.to_string())?;
    Ok(FileInfo {
        filename: git::extract_filename(path),
        path: path.to_string(),
    })
}

#[tauri::command]
pub fn ensure_file_tracked(
    db: State<'_, Arc<Database>>,
    path: &str,
) -> Result<FileInfo, String> {
    validate_and_track(&db, path)
}

#[tauri::command]
pub fn ensure_files_tracked(
    db: State<'_, Arc<Database>>,
    paths: Vec<String>,
) -> Result<Vec<FileInfo>, String> {
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for path in &paths {
        match validate_and_track(&db, path) {
            Ok(info) => results.push(info),
            Err(e) => errors.push(e),
        }
    }

    if results.is_empty() && !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    Ok(results)
}

#[tauri::command]
pub fn rename_file(
    db: State<'_, Arc<Database>>,
    old_path: &str,
    new_name: &str,
) -> Result<FileInfo, String> {
    let old = std::path::Path::new(old_path);
    if !old.exists() {
        return Err(format!("File not found: {}", old_path));
    }
    let new_path = old.parent()
        .ok_or("No parent directory")?
        .join(new_name);
    if new_path.exists() {
        return Err(format!("File already exists: {}", new_path.display()));
    }
    // Validate extension
    let ext = new_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !crate::watcher::MARKDOWN_EXTENSIONS.contains(&ext) {
        return Err("Filename must end with .md, .markdown, or .txt".to_string());
    }
    std::fs::rename(old_path, &new_path).map_err(|e| e.to_string())?;
    let new_path_str = new_path.to_string_lossy().to_string();
    db.rename_file_path(old_path, &new_path_str).map_err(|e| e.to_string())?;
    Ok(FileInfo {
        filename: git::extract_filename(&new_path_str),
        path: new_path_str,
    })
}

// -- External app commands --

#[tauri::command]
pub fn open_in_finder(path: &str) -> Result<(), String> {
    std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_in_terminal(path: &str) -> Result<(), String> {
    let parent = std::path::Path::new(path)
        .parent()
        .ok_or("No parent directory")?;
    std::process::Command::new("open")
        .arg("-a")
        .arg("Ghostty")
        .arg(parent)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn restore_file(db: State<'_, Arc<Database>>, path: &str, status: &str, reminder_time: Option<&str>) -> Result<(), String> {
    db.restore_file(path, status, reminder_time).map_err(|e| e.to_string())
}

// ── Testable helper functions ──────────────────────────────────────────────

/// Search file contents for a query string, returning matches with context.
/// Extracted from the `search_file_contents` command for testability.
pub(crate) fn search_contents_in_paths(
    paths: Vec<String>,
    query: &str,
    max_results: usize,
) -> Vec<ContentSearchResult> {
    if query.len() < 2 {
        return vec![];
    }

    let lower_query = query.to_lowercase();
    let mut results = Vec::new();

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let filename = crate::git::extract_filename(&path);
            for (line_idx, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&lower_query) {
                    results.push(ContentSearchResult {
                        filename,
                        path: path.clone(),
                        line_number: line_idx + 1,
                        context: line.trim().chars().take(120).collect(),
                    });
                    break;
                }
            }
        }
        if results.len() >= max_results {
            break;
        }
    }

    results
}

/// Build a FileListItem from a FileRecord and optional diff stats.
/// Extracted from `get_inbox_items` for testability.
pub(crate) fn record_to_file_list_item(
    record: crate::db::FileRecord,
    additions: i32,
    deletions: i32,
) -> FileListItem {
    FileListItem {
        filename: git::extract_filename(&record.path),
        path: record.path,
        status: record.status,
        pinned: record.pinned,
        reminder_time: record.reminder_time,
        last_modified: record.last_modified,
        additions,
        deletions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    /// Create a fresh in-memory database for testing.
    fn test_db() -> Database {
        Database::new_in_memory().expect("Failed to create in-memory database")
    }

    // Note: extract_filename tests live in git.rs where the function is defined.
    // Database CRUD tests live in db.rs. This module tests command-specific helpers only.

    // ════════════════════════════════════════════════════════════════════════
    //  Helper function tests
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_record_to_file_list_item() {
        let record = crate::db::FileRecord {
            id: 1,
            path: "/home/user/notes/test.md".to_string(),
            status: "unread".to_string(),
            pinned: false,
            reminder_time: None,
            last_modified: Some("2024-01-01T00:00:00Z".to_string()),
            last_seen_hash: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let item = record_to_file_list_item(record, 5, 3);
        assert_eq!(item.filename, "test.md");
        assert_eq!(item.path, "/home/user/notes/test.md");
        assert_eq!(item.status, "unread");
        assert!(!item.pinned);
        assert_eq!(item.additions, 5);
        assert_eq!(item.deletions, 3);
        assert!(item.reminder_time.is_none());
    }

    #[test]
    fn test_record_to_file_list_item_with_reminder() {
        let record = crate::db::FileRecord {
            id: 2,
            path: "/notes/reminder.md".to_string(),
            status: "read".to_string(),
            pinned: true,
            reminder_time: Some("2025-06-01T10:00:00Z".to_string()),
            last_modified: None,
            last_seen_hash: Some("abc123".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let item = record_to_file_list_item(record, 0, 0);
        assert_eq!(item.filename, "reminder.md");
        assert!(item.pinned);
        assert_eq!(item.reminder_time, Some("2025-06-01T10:00:00Z".to_string()));
    }

    // Note: Database CRUD tests live in db.rs. Only command-specific tests here.

    // ════════════════════════════════════════════════════════════════════════
    //  File I/O command tests (using temp files)
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_get_file_content() {
        let dir = std::env::temp_dir().join("markinbox_test_get_content");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.md");
        std::fs::write(&file_path, "# Hello\nWorld").unwrap();

        let content = get_file_content(file_path.to_str().unwrap()).unwrap();
        assert_eq!(content, "# Hello\nWorld");

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_get_file_content_nonexistent() {
        let result = get_file_content("/nonexistent/path/file.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_file() {
        let dir = std::env::temp_dir().join("markinbox_test_save");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("output.md");

        save_file(file_path.to_str().unwrap(), "# Saved content").unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "# Saved content");

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_save_file_overwrites() {
        let dir = std::env::temp_dir().join("markinbox_test_save_overwrite");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("output.md");

        save_file(file_path.to_str().unwrap(), "first").unwrap();
        save_file(file_path.to_str().unwrap(), "second").unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "second");

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    // ════════════════════════════════════════════════════════════════════════
    //  Content search helper tests
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_search_contents_in_paths_short_query_returns_empty() {
        let results = search_contents_in_paths(vec![], "a", 20);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_contents_in_paths_finds_match() {
        let dir = std::env::temp_dir().join("markinbox_test_search_content");
        std::fs::create_dir_all(&dir).unwrap();

        let file1 = dir.join("notes.md");
        std::fs::write(&file1, "# Meeting Notes\nDiscuss project deadline\nAction items").unwrap();

        let file2 = dir.join("todo.md");
        std::fs::write(&file2, "# Todo\nBuy groceries\nClean house").unwrap();

        let paths = vec![
            file1.to_string_lossy().to_string(),
            file2.to_string_lossy().to_string(),
        ];

        let results = search_contents_in_paths(paths, "deadline", 20);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "notes.md");
        assert_eq!(results[0].line_number, 2);
        assert!(results[0].context.contains("deadline"));

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_search_contents_case_insensitive() {
        let dir = std::env::temp_dir().join("markinbox_test_search_case");
        std::fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.md");
        std::fs::write(&file, "IMPORTANT: Do not forget").unwrap();

        let paths = vec![file.to_string_lossy().to_string()];
        let results = search_contents_in_paths(paths, "important", 20);
        assert_eq!(results.len(), 1);

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_search_contents_max_results() {
        let dir = std::env::temp_dir().join("markinbox_test_search_max");
        std::fs::create_dir_all(&dir).unwrap();

        let mut paths = Vec::new();
        for i in 0..5 {
            let file = dir.join(format!("file{}.md", i));
            std::fs::write(&file, "matching content here").unwrap();
            paths.push(file.to_string_lossy().to_string());
        }

        let results = search_contents_in_paths(paths, "matching", 3);
        assert_eq!(results.len(), 3);

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_search_contents_first_match_per_file_only() {
        let dir = std::env::temp_dir().join("markinbox_test_search_first");
        std::fs::create_dir_all(&dir).unwrap();

        let file = dir.join("multi.md");
        std::fs::write(&file, "line one has keyword\nline two has keyword\nline three has keyword").unwrap();

        let paths = vec![file.to_string_lossy().to_string()];
        let results = search_contents_in_paths(paths, "keyword", 20);
        // Should only return the first match per file
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 1);

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_search_contents_skips_unreadable_files() {
        let paths = vec!["/nonexistent/file.md".to_string()];
        let results = search_contents_in_paths(paths, "anything", 20);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_contents_context_truncated_to_120_chars() {
        let dir = std::env::temp_dir().join("markinbox_test_search_truncate");
        std::fs::create_dir_all(&dir).unwrap();

        let file = dir.join("long.md");
        let long_line = format!("keyword {}", "x".repeat(200));
        std::fs::write(&file, &long_line).unwrap();

        let paths = vec![file.to_string_lossy().to_string()];
        let results = search_contents_in_paths(paths, "keyword", 20);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].context.len(), 120);

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    // ════════════════════════════════════════════════════════════════════════
    //  Integration-style: DB + command logic combined
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_full_file_lifecycle() {
        let db = test_db();

        // 1. File discovered by watcher (upsert)
        db.upsert_file("/test/lifecycle.md", "2024-01-01T00:00:00Z").unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].status, "unread");

        // 2. User opens file (mark as read)
        db.mark_as_read("/test/lifecycle.md").unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox[0].status, "read");

        // 3. User pins file
        let pinned = db.toggle_pinned("/test/lifecycle.md").unwrap();
        assert!(pinned);
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 0); // removed from inbox
        let pinned_files = db.get_files_by_status("pinned").unwrap();
        assert_eq!(pinned_files.len(), 1);

        // 4. User unpins and archives
        db.toggle_pinned("/test/lifecycle.md").unwrap();
        db.mark_status("/test/lifecycle.md", "archived").unwrap();
        let archive = db.get_files_by_status("archive").unwrap();
        assert_eq!(archive.len(), 1);

        // 5. User restores from archive
        db.restore_file("/test/lifecycle.md", "unread", None).unwrap();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].status, "unread");
    }

    #[test]
    fn test_reminder_lifecycle() {
        let db = test_db();

        // File arrives, user sets a reminder
        db.upsert_file("/test/remind.md", "2024-01-01T00:00:00Z").unwrap();
        db.set_reminder("/test/remind.md", Some("2020-06-01T10:00:00")).unwrap();

        // File should be in reminders section, not inbox
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 0);
        let reminders = db.get_files_by_status("reminders").unwrap();
        assert_eq!(reminders.len(), 1);

        // Reminder fires (past due)
        let due = db.get_due_reminders().unwrap();
        assert_eq!(due.len(), 1);
        db.fire_reminder("/test/remind.md").unwrap();

        // File returns to inbox as unread
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].status, "unread");
        assert!(inbox[0].reminder_time.is_none());
    }

    #[test]
    fn test_search_files_returns_file_info_shape() {
        let db = test_db();
        db.upsert_file("/home/user/notes/important.md", "2024-01-01T00:00:00Z").unwrap();

        let records = db.search_files("important").unwrap();
        // Simulate what the search_files command does
        let infos: Vec<FileInfo> = records
            .into_iter()
            .map(|r| FileInfo {
                filename: git::extract_filename(&r.path),
                path: r.path,
            })
            .collect();

        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].filename, "important.md");
        assert_eq!(infos[0].path, "/home/user/notes/important.md");
    }

    // ════════════════════════════════════════════════════════════════════════
    //  Edge cases
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_toggle_pin_nonexistent_file_errors() {
        let db = test_db();
        let result = db.toggle_pinned("/nonexistent/file.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_mark_status_nonexistent_file_is_noop() {
        let db = test_db();
        // Should not error, just affects 0 rows
        db.mark_status("/nonexistent/file.md", "archived").unwrap();
    }

    #[test]
    fn test_empty_inbox() {
        let db = test_db();
        let inbox = db.get_files_by_status("inbox").unwrap();
        assert_eq!(inbox.len(), 0);
    }

    #[test]
    fn test_unknown_filter_returns_all_files() {
        let db = test_db();
        db.upsert_file("/test/a.md", "2024-01-01T00:00:00Z").unwrap();
        db.upsert_file("/test/b.md", "2024-01-01T00:00:00Z").unwrap();
        db.mark_status("/test/b.md", "archived").unwrap();

        // Unknown filter uses empty WHERE clause, so returns all
        let all = db.get_files_by_status("unknown_filter").unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_many_files_search_limited_to_20() {
        let db = test_db();
        for i in 0..30 {
            db.upsert_file(&format!("/test/file_{}.md", i), "2024-01-01T00:00:00Z").unwrap();
        }

        let results = db.search_files("file").unwrap();
        assert_eq!(results.len(), 20);
    }
}
