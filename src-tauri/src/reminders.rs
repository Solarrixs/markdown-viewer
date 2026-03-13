use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::db::Database;
use crate::git;

pub fn start_reminder_loop(app_handle: AppHandle, db: Arc<Database>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(60));

        match db.get_due_reminders() {
            Ok(due_files) => {
                for file in due_files {
                    let filename = git::extract_filename(&file.path);

                    // Mark as unread and clear reminder atomically
                    if let Err(e) = db.fire_reminder(&file.path) {
                        eprintln!("Failed to update reminder for {}: {}", file.path, e);
                        continue;
                    }

                    // Send notification
                    #[cfg(desktop)]
                    {
                        use tauri_plugin_notification::NotificationExt;
                        let _ = app_handle
                            .notification()
                            .builder()
                            .title("MarkInbox Reminder")
                            .body(format!("Time to review: {}", filename))
                            .show();
                    }

                    // Emit event to frontend
                    let _ = app_handle.emit("reminder-fired", &file.path);
                }
            }
            Err(e) => eprintln!("Reminder check error: {}", e),
        }
    });
}
