use git2::{DiffOptions, Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

/// TTL cache for batch diff stats to avoid recomputing on every refresh.
static DIFF_CACHE: Mutex<Option<DiffCache>> = Mutex::new(None);

struct DiffCache {
    stats: HashMap<String, DiffStats>,
    created_at: Instant,
}

const DIFF_CACHE_TTL_SECS: u64 = 5;

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

/// Batch diff stats for multiple files, grouped by repo.
/// Results are cached with a 5-second TTL to avoid recomputing on every refresh.
pub fn get_batch_diff_stats(file_paths: &[String]) -> HashMap<String, DiffStats> {
    if file_paths.is_empty() {
        return HashMap::new();
    }

    // Check TTL cache
    {
        let cache = DIFF_CACHE.lock().unwrap();
        if let Some(ref c) = *cache {
            if c.created_at.elapsed().as_secs() < DIFF_CACHE_TTL_SECS {
                return c.stats.clone();
            }
        }
    }

    let result = compute_batch_diff_stats(file_paths);

    // Update cache
    {
        let mut cache = DIFF_CACHE.lock().unwrap();
        *cache = Some(DiffCache {
            stats: result.clone(),
            created_at: Instant::now(),
        });
    }

    result
}

/// Compute diff stats by grouping files by their git repo, then running
/// one diff per repo.
fn compute_batch_diff_stats(file_paths: &[String]) -> HashMap<String, DiffStats> {
    let mut result = HashMap::new();

    // Group file paths by their repo workdir
    let mut repos: HashMap<String, Vec<String>> = HashMap::new();
    for fp in file_paths {
        let p = Path::new(fp);
        if let Ok(repo) = Repository::discover(p) {
            if let Some(workdir) = repo.workdir() {
                let key = workdir.to_string_lossy().to_string();
                repos.entry(key).or_default().push(fp.clone());
            }
        }
    }

    // Run one diff per repo
    for (workdir_str, paths) in &repos {
        let workdir = Path::new(workdir_str);
        let repo = match Repository::open(workdir) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let head = match repo.head().and_then(|h| h.peel_to_commit()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let head_tree = match head.tree() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let diff = match repo.diff_tree_to_workdir_with_index(Some(&head_tree), None) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let path_set: std::collections::HashSet<&String> = paths.iter().collect();
        let workdir_path = workdir.to_path_buf();

        for i in 0..diff.deltas().len() {
            if let Some(delta) = diff.get_delta(i) {
                let rel_path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path());
                if let Some(rel) = rel_path {
                    let abs_path = workdir_path.join(rel).to_string_lossy().to_string();
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
