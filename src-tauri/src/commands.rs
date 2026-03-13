use crate::db::Database;
use crate::git;
use serde::Serialize;
use std::sync::Arc;
use tauri::{Manager, State};

#[derive(Debug, Serialize)]
pub struct InboxItem {
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
pub struct SearchResult {
    pub path: String,
    pub filename: String,
}

#[tauri::command]
pub fn get_inbox_items(
    db: State<'_, Arc<Database>>,
    filter: &str,
) -> Result<Vec<InboxItem>, String> {
    let records = db.get_files_by_status(filter).map_err(|e| e.to_string())?;

    // Batch diff: one repo open + one diff for all files
    let paths: Vec<String> = records.iter().map(|r| r.path.clone()).collect();
    let diff_stats = git::get_batch_diff_stats(&paths);

    Ok(records
        .into_iter()
        .map(|record| {
            let stats = diff_stats
                .get(&record.path)
                .cloned()
                .unwrap_or_default();
            InboxItem {
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
    db.mark_status(path, "read").map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_as_archived(db: State<'_, Arc<Database>>, path: &str) -> Result<(), String> {
    db.mark_status(path, "archived").map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pin_file(db: State<'_, Arc<Database>>, path: &str, pinned: bool) -> Result<(), String> {
    db.set_pinned(path, pinned).map_err(|e| e.to_string())
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
pub fn add_watched_folder(db: State<'_, Arc<Database>>, path: &str) -> Result<(), String> {
    db.add_watched_folder(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_watched_folders(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<crate::db::WatchedFolder>, String> {
    db.get_watched_folders().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_files(
    db: State<'_, Arc<Database>>,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let records = db.search_files(query).map_err(|e| e.to_string())?;
    Ok(records
        .into_iter()
        .map(|r| SearchResult {
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
    Ok(new_state)
}
