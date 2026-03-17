use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

use crate::db::Database;

#[derive(Debug, Serialize, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub timestamp: String,
    pub status: String, // "active", "idle", "completed"
}

/// List Claude Code sessions for a project directory.
/// If `project_dir` is None, attempts to detect from watched folders.
pub fn list_sessions(db: &Database, project_dir: Option<&str>) -> Result<Vec<SessionInfo>, String> {
    let dirs_to_check: Vec<String> = match project_dir {
        Some(d) => vec![d.to_string()],
        None => {
            // Use watched folders to find project dirs
            db.get_watched_folders()
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|f| f.path)
                .collect()
        }
    };

    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let claude_projects_dir = home.join(".claude").join("projects");

    if !claude_projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut all_sessions = Vec::new();

    for dir in &dirs_to_check {
        // Encode the path as Claude does: replace / with -, strip leading -
        let encoded = dir.trim_end_matches('/').replace('/', "-");
        let encoded = encoded.strip_prefix('-').unwrap_or(&encoded);
        let sessions_dir = claude_projects_dir.join(encoded);

        if !sessions_dir.exists() {
            continue;
        }

        let entries = std::fs::read_dir(&sessions_dir).map_err(|e| e.to_string())?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            if let Some(info) = parse_session_file(&path) {
                all_sessions.push(info);
            }
        }
    }

    // Sort by timestamp descending (most recent first)
    all_sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(all_sessions)
}

fn parse_session_file(path: &PathBuf) -> Option<SessionInfo> {
    let content = std::fs::read_to_string(path).ok()?;
    let first_line = content.lines().next()?;
    let val: serde_json::Value = serde_json::from_str(first_line).ok()?;

    let session_id = val.get("sessionId")?.as_str()?.to_string();

    // Determine session timestamp from file metadata
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?;
    let mtime_dt: chrono::DateTime<chrono::Utc> = mtime.into();
    let timestamp = mtime_dt.to_rfc3339();

    // Determine status based on how recently modified
    let age_secs = chrono::Utc::now().signed_duration_since(mtime_dt).num_seconds();
    let status = if age_secs < 300 {
        "active"
    } else if age_secs < 3600 {
        "idle"
    } else {
        "completed"
    };

    Some(SessionInfo {
        session_id,
        timestamp,
        status: status.to_string(),
    })
}

#[tauri::command]
pub fn list_claude_sessions(
    db: tauri::State<'_, Arc<Database>>,
    project_dir: Option<&str>,
) -> Result<Vec<SessionInfo>, String> {
    list_sessions(&db, project_dir)
}

#[tauri::command]
pub fn send_feedback_to_session(
    session_id: &str,
    feedback_text: &str,
) -> Result<(), String> {
    // Shell out to Claude CLI
    let output = std::process::Command::new("claude")
        .arg("--resume")
        .arg(session_id)
        .arg("--print")
        .arg(format!("Review feedback from MarkInbox:\n\n{}", feedback_text))
        .output()
        .map_err(|e| format!("Failed to run claude CLI: {}. Is it installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Claude CLI error: {}", stderr));
    }

    Ok(())
}
