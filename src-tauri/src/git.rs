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

/// Get the diff for a specific file in a specific commit (vs its parent).
/// `repo_path` is the repo workdir, `commit_oid` is the commit hash,
/// `file_path` is the relative path within the repo.
pub fn get_commit_file_diff(repo_path: &str, commit_oid: &str, file_path: &str) -> Option<DiffResult> {
    let repo = Repository::open(repo_path).ok()?;
    let oid = git2::Oid::from_str(commit_oid).ok()?;
    let commit = repo.find_commit(oid).ok()?;
    let tree = commit.tree().ok()?;
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(file_path);

    let diff = repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&tree),
        Some(&mut diff_opts),
    ).ok()?;

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
    }).ok()?;

    // Read the file content at this commit
    let entry = tree.get_path(std::path::Path::new(file_path)).ok()?;
    let blob = repo.find_blob(entry.id()).ok()?;
    let content = String::from_utf8_lossy(blob.content()).to_string();

    Some(DiffResult {
        content,
        hunks,
        additions,
        deletions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── Helper: create a temp git repo with an initial commit ──

    fn init_repo_with_commit(dir: &Path) -> Repository {
        let repo = Repository::init(dir).expect("git init failed");

        // Configure user for commits
        let mut config = repo.config().expect("failed to get config");
        config
            .set_str("user.name", "Test User")
            .expect("set user.name");
        config
            .set_str("user.email", "test@example.com")
            .expect("set user.email");

        repo
    }

    fn commit_all(repo: &Repository, message: &str) {
        let mut index = repo.index().expect("failed to get index");
        index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .expect("add_all failed");
        index.write().expect("index write failed");
        let tree_oid = index.write_tree().expect("write_tree failed");
        let tree = repo.find_tree(tree_oid).expect("find_tree failed");
        let sig = repo.signature().expect("signature failed");

        let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
            .expect("commit failed");
    }

    // ── extract_filename tests ──

    #[test]
    fn test_extract_filename_simple() {
        assert_eq!(extract_filename("/home/user/docs/note.md"), "note.md");
    }

    #[test]
    fn test_extract_filename_nested() {
        assert_eq!(
            extract_filename("/a/b/c/d/deep_file.txt"),
            "deep_file.txt"
        );
    }

    #[test]
    fn test_extract_filename_root_file() {
        assert_eq!(extract_filename("/file.md"), "file.md");
    }

    #[test]
    fn test_extract_filename_no_extension() {
        assert_eq!(extract_filename("/home/user/Makefile"), "Makefile");
    }

    #[test]
    fn test_extract_filename_empty_string() {
        // Path::new("").file_name() is None
        assert_eq!(extract_filename(""), "");
    }

    #[test]
    fn test_extract_filename_trailing_slash() {
        // Path::new("/foo/bar/") treats "bar" as the last component
        assert_eq!(extract_filename("/foo/bar/"), "bar");
    }

    // ── DiffHunk / DiffResult struct construction ──

    #[test]
    fn test_diff_hunk_construction() {
        let hunk = DiffHunk {
            old_start: 10,
            new_start: 12,
            new_lines: 3,
            content: "added line".to_string(),
            change_type: "added".to_string(),
        };
        assert_eq!(hunk.old_start, 10);
        assert_eq!(hunk.new_start, 12);
        assert_eq!(hunk.new_lines, 3);
        assert_eq!(hunk.content, "added line");
        assert_eq!(hunk.change_type, "added");
    }

    #[test]
    fn test_diff_result_construction() {
        let result = DiffResult {
            content: "hello world".to_string(),
            hunks: vec![
                DiffHunk {
                    old_start: 1,
                    new_start: 1,
                    new_lines: 1,
                    content: "old line".to_string(),
                    change_type: "removed".to_string(),
                },
                DiffHunk {
                    old_start: 0,
                    new_start: 1,
                    new_lines: 1,
                    content: "new line".to_string(),
                    change_type: "added".to_string(),
                },
            ],
            additions: 1,
            deletions: 1,
        };
        assert_eq!(result.additions, 1);
        assert_eq!(result.deletions, 1);
        assert_eq!(result.hunks.len(), 2);
        assert_eq!(result.hunks[0].change_type, "removed");
        assert_eq!(result.hunks[1].change_type, "added");
    }

    #[test]
    fn test_diff_result_empty_hunks() {
        let result = DiffResult {
            content: "unchanged file".to_string(),
            hunks: vec![],
            additions: 0,
            deletions: 0,
        };
        assert!(result.hunks.is_empty());
        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 0);
    }

    #[test]
    fn test_diff_stats_default() {
        let stats = DiffStats::default();
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 0);
    }

    #[test]
    fn test_diff_result_serialization() {
        let result = DiffResult {
            content: "test".to_string(),
            hunks: vec![DiffHunk {
                old_start: 1,
                new_start: 2,
                new_lines: 1,
                content: "line".to_string(),
                change_type: "added".to_string(),
            }],
            additions: 1,
            deletions: 0,
        };
        let json = serde_json::to_string(&result).expect("serialize failed");
        let deserialized: DiffResult =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.additions, 1);
        assert_eq!(deserialized.hunks.len(), 1);
        assert_eq!(deserialized.hunks[0].content, "line");
    }

    // ── get_file_diff with real temp repos ──

    #[test]
    fn test_get_file_diff_no_changes() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_path = tmp.path().join("test.md");
        fs::write(&file_path, "# Hello\n").expect("write file");
        commit_all(&repo, "initial commit");

        // No changes since commit — diff should be None (no deltas)
        let result = get_file_diff(file_path.to_str().unwrap());
        assert!(
            result.is_none() || result.as_ref().map_or(false, |r| r.hunks.is_empty()),
            "Expected no diff hunks for an unchanged file"
        );
    }

    #[test]
    fn test_get_file_diff_with_additions() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_path = tmp.path().join("test.md");
        fs::write(&file_path, "line1\n").expect("write file");
        commit_all(&repo, "initial commit");

        // Modify the file
        fs::write(&file_path, "line1\nline2\nline3\n").expect("modify file");

        let result = get_file_diff(file_path.to_str().unwrap());
        assert!(result.is_some(), "Expected a diff result");
        let diff = result.unwrap();
        assert!(diff.additions >= 2, "Expected at least 2 additions");
        assert_eq!(diff.deletions, 0);
        assert!(!diff.hunks.is_empty());

        let added_hunks: Vec<_> = diff
            .hunks
            .iter()
            .filter(|h| h.change_type == "added")
            .collect();
        assert!(
            added_hunks.len() >= 2,
            "Expected at least 2 added hunks, got {}",
            added_hunks.len()
        );
    }

    #[test]
    fn test_get_file_diff_with_deletions() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_path = tmp.path().join("test.md");
        fs::write(&file_path, "line1\nline2\nline3\n").expect("write file");
        commit_all(&repo, "initial commit");

        // Remove lines
        fs::write(&file_path, "line1\n").expect("modify file");

        let result = get_file_diff(file_path.to_str().unwrap());
        assert!(result.is_some());
        let diff = result.unwrap();
        assert!(diff.deletions >= 2, "Expected at least 2 deletions");

        let removed_hunks: Vec<_> = diff
            .hunks
            .iter()
            .filter(|h| h.change_type == "removed")
            .collect();
        assert!(removed_hunks.len() >= 2);
    }

    #[test]
    fn test_get_file_diff_with_modifications() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_path = tmp.path().join("test.md");
        fs::write(&file_path, "original line\n").expect("write file");
        commit_all(&repo, "initial commit");

        fs::write(&file_path, "modified line\n").expect("modify file");

        let result = get_file_diff(file_path.to_str().unwrap());
        assert!(result.is_some());
        let diff = result.unwrap();
        // A modification shows as 1 deletion + 1 addition
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 1);
    }

    #[test]
    fn test_get_file_diff_content_matches_current_file() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_path = tmp.path().join("test.md");
        fs::write(&file_path, "before\n").expect("write file");
        commit_all(&repo, "initial commit");

        let new_content = "after editing\n";
        fs::write(&file_path, new_content).expect("modify file");

        let result = get_file_diff(file_path.to_str().unwrap());
        assert!(result.is_some());
        assert_eq!(result.unwrap().content, new_content);
    }

    // ── Edge case: file not in a git repo ──

    #[test]
    fn test_get_file_diff_not_in_repo() {
        let tmp = TempDir::new().expect("tmpdir");
        // No git init — just a bare directory
        let file_path = tmp.path().join("orphan.md");
        fs::write(&file_path, "no repo here\n").expect("write file");

        let result = get_file_diff(file_path.to_str().unwrap());
        assert!(result.is_none(), "Expected None for file outside a git repo");
    }

    // ── Edge case: new file with no commit history ──

    #[test]
    fn test_get_file_diff_new_file_no_commits() {
        let tmp = TempDir::new().expect("tmpdir");
        let _repo = init_repo_with_commit(tmp.path());
        // Repo exists but has no commits

        let file_path = tmp.path().join("brand_new.md");
        fs::write(&file_path, "brand new content\n").expect("write file");

        // HEAD doesn't exist yet, so get_file_diff should return None
        let result = get_file_diff(file_path.to_str().unwrap());
        assert!(
            result.is_none(),
            "Expected None for file in repo with no commits"
        );
    }

    // ── Edge case: binary-ish file ──

    #[test]
    fn test_get_file_diff_binary_file() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_path = tmp.path().join("image.png");
        // Write some bytes that look binary (contain null bytes)
        fs::write(&file_path, b"\x89PNG\r\n\x1a\n\x00\x00\x00").expect("write file");
        commit_all(&repo, "add binary file");

        // Modify the binary file
        fs::write(&file_path, b"\x89PNG\r\n\x1a\n\x00\x00\x01\xff").expect("modify file");

        let result = get_file_diff(file_path.to_str().unwrap());
        // Binary files may return None (read_to_string fails) or a result with
        // no meaningful hunks. Either is acceptable.
        if let Some(diff) = result {
            // If it did parse, hunks may be empty for binary content
            let _ = diff.hunks;
        }
    }

    // ── Edge case: nonexistent file ──

    #[test]
    fn test_get_file_diff_nonexistent_file() {
        let result = get_file_diff("/tmp/this_file_does_not_exist_at_all.md");
        assert!(result.is_none());
    }

    // ── get_batch_diff_stats tests ──

    #[test]
    fn test_get_batch_diff_stats_empty_input() {
        let result = get_batch_diff_stats(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_batch_diff_stats_with_changes() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_a = tmp.path().join("a.md");
        let file_b = tmp.path().join("b.md");
        fs::write(&file_a, "aaa\n").expect("write a");
        fs::write(&file_b, "bbb\n").expect("write b");
        commit_all(&repo, "initial commit");

        // Modify both files
        fs::write(&file_a, "aaa\nnew line in a\n").expect("modify a");
        fs::write(&file_b, "changed b\n").expect("modify b");

        let paths = vec![
            file_a.to_string_lossy().to_string(),
            file_b.to_string_lossy().to_string(),
        ];
        let stats = compute_batch_diff_stats(&paths);

        let stats_a = stats.get(&file_a.to_string_lossy().to_string());
        assert!(stats_a.is_some(), "Expected stats for file a");
        assert!(
            stats_a.unwrap().additions >= 1,
            "Expected additions in file a"
        );

        let stats_b = stats.get(&file_b.to_string_lossy().to_string());
        assert!(stats_b.is_some(), "Expected stats for file b");
        // b was changed: 1 deletion + 1 addition
        assert_eq!(stats_b.unwrap().additions, 1);
        assert_eq!(stats_b.unwrap().deletions, 1);
    }

    #[test]
    fn test_compute_batch_diff_stats_file_not_in_repo() {
        let tmp = TempDir::new().expect("tmpdir");
        // No git init
        let file_path = tmp.path().join("orphan.md");
        fs::write(&file_path, "no repo\n").expect("write file");

        let paths = vec![file_path.to_string_lossy().to_string()];
        let stats = compute_batch_diff_stats(&paths);
        assert!(stats.is_empty(), "Expected empty stats for non-repo file");
    }

    #[test]
    fn test_compute_batch_diff_stats_no_changes() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_path = tmp.path().join("stable.md");
        fs::write(&file_path, "no changes\n").expect("write file");
        commit_all(&repo, "initial commit");

        let paths = vec![file_path.to_string_lossy().to_string()];
        let stats = compute_batch_diff_stats(&paths);
        // No working-tree changes, so the file shouldn't appear in the diff
        assert!(
            stats.is_empty() || {
                let s = stats.get(&file_path.to_string_lossy().to_string());
                s.map_or(true, |s| s.additions == 0 && s.deletions == 0)
            },
            "Expected no diff stats for unchanged file"
        );
    }

    // ── DiffHunk change_type values ──

    #[test]
    fn test_diff_hunk_change_types_are_correct_strings() {
        let tmp = TempDir::new().expect("tmpdir");
        let repo = init_repo_with_commit(tmp.path());

        let file_path = tmp.path().join("test.md");
        fs::write(&file_path, "line1\nline2\nline3\n").expect("write");
        commit_all(&repo, "initial");

        // Replace line2, keep line1 and line3 as context
        fs::write(&file_path, "line1\nreplaced\nline3\n").expect("modify");

        let result = get_file_diff(file_path.to_str().unwrap()).expect("diff should exist");

        let change_types: Vec<&str> = result.hunks.iter().map(|h| h.change_type.as_str()).collect();
        // Should only contain valid change types
        for ct in &change_types {
            assert!(
                ["added", "removed", "context"].contains(ct),
                "Unexpected change_type: {}",
                ct
            );
        }
        // Should have at least one of each for a replacement with context
        assert!(change_types.contains(&"added"));
        assert!(change_types.contains(&"removed"));
        assert!(change_types.contains(&"context"));
    }
}
