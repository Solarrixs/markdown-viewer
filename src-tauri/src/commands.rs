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
            FileListItem {
                filename: git::extract_filename(&record.path),
                path: record.path,
                status: record.status,
                pinned: record.pinned,
                reminder_time: record.reminder_time,
                last_modified: record.last_modified,
                additions: stats.additions,
                deletions: stats.deletions,
            }
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

// -- Open arbitrary file --

#[tauri::command]
pub fn ensure_file_tracked(
    db: State<'_, Arc<Database>>,
    path: &str,
) -> Result<FileInfo, String> {
    let meta = std::fs::metadata(path).map_err(|_| format!("File not found: {}", path))?;
    if !meta.is_file() {
        return Err("Path is not a file".to_string());
    }

    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !crate::watcher::MARKDOWN_EXTENSIONS.contains(&ext) {
        return Err("Only markdown files (.md, .markdown, .txt) can be opened".to_string());
    }

    let mtime = crate::watcher::get_file_mtime_string(path);
    db.upsert_file_if_missing(path, &mtime)
        .map_err(|e| e.to_string())?;

    Ok(FileInfo {
        filename: git::extract_filename(path),
        path: path.to_string(),
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
