use std::sync::Arc;
use git2::{Repository, Oid, Sort};
use tauri::{AppHandle, Emitter};

use crate::db::Database;
use crate::summarize;

/// Start a background loop that polls watched folders for new git commits.
/// Runs every 3 seconds, same pattern as `reminders::start_reminder_loop`.
pub fn start_commit_poller(app_handle: AppHandle, db: Arc<Database>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for commit poller");

        rt.block_on(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                if let Err(e) = poll_commits(&app_handle, &db).await {
                    eprintln!("Commit poller error: {}", e);
                }
            }
        });
    });
}

async fn poll_commits(app_handle: &AppHandle, db: &Database) -> Result<(), String> {
    let folders = db.get_watched_folders().map_err(|e| e.to_string())?;

    for folder in &folders {
        // Try to discover a git repo from the watched folder
        let repo = match Repository::discover(&folder.path) {
            Ok(r) => r,
            Err(_) => continue, // Not a git repo, skip
        };

        let workdir = match repo.workdir() {
            Some(w) => w.to_string_lossy().to_string(),
            None => continue, // Bare repo
        };

        let head = match repo.head() {
            Ok(h) => h,
            Err(_) => continue, // No HEAD (empty repo)
        };

        let head_oid = match head.target() {
            Some(oid) => oid,
            None => continue,
        };

        // Check if HEAD has changed since last poll
        let last_seen = db.get_last_seen_oid(&workdir).map_err(|e| e.to_string())?;
        if let Some(ref last) = last_seen {
            if *last == head_oid.to_string() {
                continue; // No new commits
            }
        }

        // Walk new commits
        let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
        revwalk.push(head_oid).map_err(|e| e.to_string())?;
        revwalk.set_sorting(Sort::TIME).map_err(|e| e.to_string())?;

        // If we have a last seen OID, hide it and all its ancestors
        if let Some(ref last) = last_seen {
            if let Ok(oid) = Oid::from_str(last) {
                let _ = revwalk.hide(oid);
            }
        }

        let mut new_commit_count = 0;

        for oid_result in revwalk {
            let oid = match oid_result {
                Ok(o) => o,
                Err(_) => continue,
            };

            let commit = match repo.find_commit(oid) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let message = commit.message().unwrap_or("").to_string();
            let author = commit.author().name().map(|s| s.to_string());
            let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            // Get diff stats for this commit
            let (files_changed, additions, deletions) = get_commit_stats(&repo, &commit);

            // Insert commit
            let oid_str = oid.to_string();
            if let Err(e) = db.insert_commit(
                &workdir, &oid_str, &message, author.as_deref(),
                &timestamp, files_changed, additions, deletions,
            ) {
                eprintln!("Failed to insert commit {}: {}", oid_str, e);
                continue;
            }

            // Insert per-file stats
            insert_file_stats(&repo, &commit, &oid_str, db);

            // Try session detection
            if let Some(session_id) = detect_claude_session(&workdir, commit.time().seconds()) {
                let _ = db.update_commit_session(&oid_str, &session_id);
            }

            new_commit_count += 1;

            // Limit to 50 new commits per poll to avoid overwhelming on first run
            if new_commit_count >= 50 {
                break;
            }
        }

        // Update last seen OID
        db.set_last_seen_oid(&workdir, &head_oid.to_string()).map_err(|e| e.to_string())?;

        if new_commit_count > 0 {
            let _ = app_handle.emit("new-commits", new_commit_count);

            // Auto-summarize new commits if API key is configured
            if let Ok(Some(api_key)) = db.get_setting("anthropic_api_key") {
                if !api_key.is_empty() {
                    // Summarize the HEAD commit (most recent)
                    let oid_str = head_oid.to_string();
                    if db.get_diff_summary(&oid_str).map(|s| s.is_none()).unwrap_or(false) {
                        match summarize::summarize_commit(db, &oid_str, &api_key).await {
                            Ok(_) => {}
                            Err(e) => eprintln!("Auto-summarize failed for {}: {}", &oid_str[..7], e),
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_commit_stats(repo: &Repository, commit: &git2::Commit) -> (i32, i32, i32) {
    let tree = match commit.tree() {
        Ok(t) => t,
        Err(_) => return (0, 0, 0),
    };

    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
        Ok(d) => d,
        Err(_) => return (0, 0, 0),
    };

    let stats = match diff.stats() {
        Ok(s) => s,
        Err(_) => return (0, 0, 0),
    };

    (
        stats.files_changed() as i32,
        stats.insertions() as i32,
        stats.deletions() as i32,
    )
}

fn insert_file_stats(repo: &Repository, commit: &git2::Commit, oid_str: &str, db: &Database) {
    let tree = match commit.tree() {
        Ok(t) => t,
        Err(_) => return,
    };

    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
        Ok(d) => d,
        Err(_) => return,
    };

    for i in 0..diff.deltas().len() {
        if let Some(delta) = diff.get_delta(i) {
            let file_path = delta.new_file().path()
                .or_else(|| delta.old_file().path())
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();

            let status = match delta.status() {
                git2::Delta::Added => "added",
                git2::Delta::Deleted => "deleted",
                git2::Delta::Modified => "modified",
                git2::Delta::Renamed => "renamed",
                git2::Delta::Copied => "copied",
                _ => "modified",
            };

            // Get per-file stats via patch
            let (additions, deletions) = if let Ok(Some(patch)) = git2::Patch::from_diff(&diff, i) {
                let (_, adds, dels) = patch.line_stats().unwrap_or((0, 0, 0));
                (adds as i32, dels as i32)
            } else {
                (0, 0)
            };

            let _ = db.insert_commit_file(oid_str, &file_path, additions, deletions, status);
        }
    }
}

/// Detect if a commit was made during a Claude Code session.
/// Reads `~/.claude/projects/<encoded-cwd>/` to find session files.
pub fn detect_claude_session(workdir: &str, commit_timestamp: i64) -> Option<String> {
    let home = dirs::home_dir()?;
    // Encode the workdir path as Claude does: replace / with -
    let encoded = workdir.trim_end_matches('/').replace('/', "-");
    // Remove leading dash
    let encoded = encoded.strip_prefix('-').unwrap_or(&encoded);
    let sessions_dir = home.join(".claude").join("projects").join(encoded);

    if !sessions_dir.exists() {
        return None;
    }

    let commit_time = chrono::DateTime::from_timestamp(commit_timestamp, 0)?;

    let mut best_match: Option<(String, i64)> = None; // (session_id, time_diff)

    if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            // Read first line for session metadata
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(first_line) = content.lines().next() {
                    // Parse JSON to find sessionId and timestamp
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(first_line) {
                        let session_id = val.get("sessionId")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        if let Some(ref sid) = session_id {
                            // Use file mtime as session time approximation
                            if let Ok(meta) = std::fs::metadata(&path) {
                                if let Ok(mtime) = meta.modified() {
                                    let file_time: chrono::DateTime<chrono::Utc> = mtime.into();
                                    let diff = (file_time.signed_duration_since(commit_time)).num_seconds().abs();

                                    // Within 5 minute window
                                    if diff < 300 {
                                        if best_match.as_ref().map_or(true, |(_, d)| diff < *d) {
                                            best_match = Some((sid.clone(), diff));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    best_match.map(|(sid, _)| sid)
}
