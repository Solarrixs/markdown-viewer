use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::db::Database;

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ApiMessage>,
}

#[derive(Serialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

/// Summarize a commit's diff using the Anthropic API.
/// Returns the summary text, or an error string.
pub async fn summarize_commit(db: &Database, commit_oid: &str, api_key: &str) -> Result<String, String> {
    // Check cache first
    if let Ok(Some(existing)) = db.get_diff_summary(commit_oid) {
        return Ok(existing.summary);
    }

    // Build diff text from commit files
    let files = db.get_commit_files(commit_oid).map_err(|e| e.to_string())?;
    if files.is_empty() {
        return Err("No files found for this commit".to_string());
    }

    let mut diff_text = String::new();
    for f in &files {
        diff_text.push_str(&format!(
            "--- {}\n  status: {}, +{} -{}\n",
            f.file_path, f.status, f.additions, f.deletions
        ));
    }

    // Get commit message for context
    let commits = db.get_all_recent_commits(100).map_err(|e| e.to_string())?;
    let commit = commits.iter().find(|c| c.oid == commit_oid);
    if let Some(c) = commit {
        diff_text = format!("Commit: {}\nMessage: {}\nAuthor: {}\n\nChanged files:\n{}",
            &c.oid[..7], c.message, c.author.as_deref().unwrap_or("unknown"), diff_text);
    }

    // Truncate if too long
    if diff_text.len() > 8000 {
        diff_text.truncate(8000);
        diff_text.push_str("\n... (truncated)");
    }

    let model = "claude-haiku-4-5-20251001";

    let request = ApiRequest {
        model: model.to_string(),
        max_tokens: 300,
        system: "Summarize this code diff concisely for a reviewer. Focus on what changed and why it matters. Keep it to 2-3 sentences.".to_string(),
        messages: vec![ApiMessage {
            role: "user".to_string(),
            content: diff_text,
        }],
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }

    let api_response: ApiResponse = response.json().await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let summary = api_response.content
        .first()
        .and_then(|b| b.text.clone())
        .unwrap_or_else(|| "No summary generated".to_string());

    // Cache the result
    let _ = db.insert_diff_summary(commit_oid, &summary, Some(model));

    Ok(summary)
}

/// Tauri command to trigger summarization.
#[tauri::command]
pub async fn trigger_summarize(
    db: tauri::State<'_, Arc<Database>>,
    commit_oid: String,
) -> Result<String, String> {
    let api_key = db.get_setting("anthropic_api_key")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No Anthropic API key configured. Set it in Settings.".to_string())?;

    summarize_commit(&db, &commit_oid, &api_key).await
}
