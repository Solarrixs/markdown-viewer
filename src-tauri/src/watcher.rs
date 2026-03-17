use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;

use crate::db::Database;

/// Supported file extensions for markdown/text files.
pub const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown", "txt"];

struct CompiledIgnore {
    globs: Vec<glob::Pattern>,
    exact: Vec<String>,
}

impl CompiledIgnore {
    fn from_patterns(patterns: &[String]) -> Self {
        let mut globs = Vec::new();
        let mut exact = Vec::new();
        for p in patterns {
            if p.contains('*') {
                if let Ok(g) = glob::Pattern::new(p) {
                    globs.push(g);
                }
            } else {
                exact.push(p.clone());
            }
        }
        CompiledIgnore { globs, exact }
    }

    fn should_ignore(&self, path: &str, filename: &str) -> bool {
        if self.exact.iter().any(|e| e == filename) {
            return true;
        }
        self.globs
            .iter()
            .any(|g| g.matches(filename) || g.matches(path))
    }
}

pub struct WatcherHandle {
    restart_signal: Arc<Notify>,
}

impl WatcherHandle {
    pub fn restart(&self) {
        self.restart_signal.notify_one();
    }
}

pub fn start_watcher(app_handle: AppHandle, db: Arc<Database>) -> WatcherHandle {
    let restart_signal = Arc::new(Notify::new());
    let signal_clone = restart_signal.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for watcher");

        rt.block_on(async move {
            loop {
                let folders = db.get_watched_folders().unwrap_or_default();
                let raw_patterns: Vec<String> = db.get_ignore_patterns_with_ids()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| p.pattern)
                    .collect();
                let ignore = CompiledIgnore::from_patterns(&raw_patterns);

                let db_clone = db.clone();
                let app_clone = app_handle.clone();

                let watcher_result = RecommendedWatcher::new(
                    move |res: Result<Event, notify::Error>| {
                        if let Ok(event) = res {
                            match event.kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    for path in &event.paths {
                                        if let Some(path_str) = path.to_str() {
                                            if should_process(path_str, &ignore) {
                                                let modified = get_file_mtime_string(path_str);
                                                if let Err(e) =
                                                    db_clone.upsert_file(path_str, &modified)
                                                {
                                                    eprintln!("DB upsert error: {}", e);
                                                }
                                                let _ = app_clone.emit("file-changed", path_str);
                                            }
                                        }
                                    }
                                }
                                EventKind::Remove(_) => {
                                    for path in &event.paths {
                                        if let Some(path_str) = path.to_str() {
                                            if should_process(path_str, &ignore) {
                                                let _ = db_clone.delete_file(path_str);
                                                let _ = app_clone.emit("file-removed", path_str);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    },
                    Config::default(),
                );

                match watcher_result {
                    Ok(mut watcher) => {
                        for folder in &folders {
                            let path = Path::new(&folder.path);
                            if path.exists() {
                                if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                                    eprintln!("Failed to watch {}: {}", folder.path, e);
                                }
                            }
                        }

                        // Wait for restart signal
                        signal_clone.notified().await;
                        // Watcher is dropped here, then loop restarts with fresh config
                        drop(watcher);
                    }
                    Err(e) => {
                        eprintln!("Failed to create file watcher: {}", e);
                        // Wait before retrying
                        signal_clone.notified().await;
                    }
                }
            }
        });
    });

    WatcherHandle { restart_signal }
}

pub fn get_file_mtime_string(path: &str) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.to_rfc3339()
        })
        .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339())
}

fn should_process(path: &str, ignore: &CompiledIgnore) -> bool {
    let has_valid_ext = MARKDOWN_EXTENSIONS.iter().any(|ext| path.ends_with(&format!(".{}", ext)));
    if !has_valid_ext {
        return false;
    }
    let filename = Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
    !ignore.should_ignore(path, filename)
}
