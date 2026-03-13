use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::db::Database;

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

pub fn start_watcher(app_handle: AppHandle, db: Arc<Database>) {
    std::thread::spawn(move || {
        let folders = db.get_watched_folders().unwrap_or_default();
        let raw_patterns = db.get_ignore_patterns().unwrap_or_default();
        let ignore = CompiledIgnore::from_patterns(&raw_patterns);

        let db_clone = db.clone();
        let app_clone = app_handle.clone();

        let mut _watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            for path in &event.paths {
                                if let Some(path_str) = path.to_str() {
                                    if should_process(path_str, &ignore) {
                                        let modified = chrono::Utc::now()
                                            .format("%Y-%m-%dT%H:%M:%S")
                                            .to_string();
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
                        _ => {}
                    }
                }
            },
            Config::default(),
        )
        .expect("Failed to create file watcher");

        for folder in &folders {
            let path = Path::new(&folder.path);
            if path.exists() {
                if let Err(e) = _watcher.watch(path, RecursiveMode::Recursive) {
                    eprintln!("Failed to watch {}: {}", folder.path, e);
                }
            }
        }

        // Park the thread forever — watcher is event-driven internally.
        // _watcher must stay alive (not dropped) so we hold it via the binding above.
        loop {
            std::thread::park();
        }
    });
}

fn should_process(path: &str, ignore: &CompiledIgnore) -> bool {
    if !path.ends_with(".md") {
        return false;
    }
    let filename = Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
    !ignore.should_ignore(path, filename)
}
