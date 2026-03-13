use git2::{DiffOptions, Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffHunk {
    pub old_start: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub content: String,
    pub change_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffResult {
    pub content: String,
    pub hunks: Vec<DiffHunk>,
    pub additions: i32,
    pub deletions: i32,
}

#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    pub additions: i32,
    pub deletions: i32,
}

pub fn extract_filename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_string()
}

/// Batch diff stats for multiple files in a single repo open + diff operation.
pub fn get_batch_diff_stats(file_paths: &[String]) -> HashMap<String, DiffStats> {
    let mut result = HashMap::new();
    if file_paths.is_empty() {
        return result;
    }

    // Try to discover repo from the first file
    let first_path = Path::new(&file_paths[0]);
    let repo = match Repository::discover(first_path) {
        Ok(r) => r,
        Err(_) => return result,
    };
    let workdir = match repo.workdir() {
        Some(w) => w.to_path_buf(),
        None => return result,
    };

    let head = match repo.head().and_then(|h| h.peel_to_commit()) {
        Ok(c) => c,
        Err(_) => return result,
    };
    let head_tree = match head.tree() {
        Ok(t) => t,
        Err(_) => return result,
    };

    // One diff for the whole repo (no pathspec filter)
    let diff = match repo.diff_tree_to_workdir_with_index(Some(&head_tree), None) {
        Ok(d) => d,
        Err(_) => return result,
    };

    // Build a set of paths we care about for quick lookup
    let path_set: std::collections::HashSet<String> = file_paths.iter().cloned().collect();

    for i in 0..diff.deltas().len() {
        if let Some(delta) = diff.get_delta(i) {
            let rel_path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path());
            if let Some(rel) = rel_path {
                let abs_path = workdir.join(rel).to_string_lossy().to_string();
                if path_set.contains(&abs_path) {
                    let mut stats = DiffStats::default();
                    if let Ok(patch) = git2::Patch::from_diff(&diff, i) {
                        if let Some(patch) = patch {
                            let (_, adds, dels) = patch.line_stats().unwrap_or((0, 0, 0));
                            stats.additions = adds as i32;
                            stats.deletions = dels as i32;
                        }
                    }
                    result.insert(abs_path, stats);
                }
            }
        }
    }

    result
}

/// Full diff for a single file (used when viewing a file).
pub fn get_file_diff(file_path: &str) -> Option<DiffResult> {
    let path = Path::new(file_path);
    let repo = Repository::discover(path).ok()?;
    let workdir = repo.workdir()?;

    let content = std::fs::read_to_string(path).ok()?;
    let rel_path = path.strip_prefix(workdir).ok()?;

    let head = repo.head().ok()?;
    let head_commit = head.peel_to_commit().ok()?;
    let head_tree = head_commit.tree().ok()?;

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(rel_path.to_str()?);

    let diff = repo
        .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))
        .ok()?;

    let mut hunks = Vec::new();
    let mut additions: i32 = 0;
    let mut deletions: i32 = 0;

    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        let line_content = String::from_utf8_lossy(line.content()).to_string();

        match origin {
            '+' => {
                additions += 1;
                hunks.push(DiffHunk {
                    old_start: line.old_lineno().unwrap_or(0),
                    new_start: line.new_lineno().unwrap_or(0),
                    new_lines: 1,
                    content: line_content,
                    change_type: "added".to_string(),
                });
            }
            '-' => {
                deletions += 1;
                hunks.push(DiffHunk {
                    old_start: line.old_lineno().unwrap_or(0),
                    new_start: line.new_lineno().unwrap_or(0),
                    new_lines: 1,
                    content: line_content,
                    change_type: "removed".to_string(),
                });
            }
            ' ' => {
                hunks.push(DiffHunk {
                    old_start: line.old_lineno().unwrap_or(0),
                    new_start: line.new_lineno().unwrap_or(0),
                    new_lines: 1,
                    content: line_content,
                    change_type: "context".to_string(),
                });
            }
            _ => {}
        }
        true
    })
    .ok()?;

    Some(DiffResult {
        content,
        hunks,
        additions,
        deletions,
    })
}
