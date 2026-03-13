mod commands;
mod db;
mod git;
mod reminders;
mod watcher;

use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let database = Arc::new(db::Database::new().expect("Failed to initialize database"));

    // Add default watched folder on first run
    let _ = database.add_watched_folder(
        &dirs::home_dir()
            .unwrap_or_default()
            .join("Documents/Engram")
            .to_string_lossy(),
    );

    let db_for_builder = database.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(database.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_inbox_items,
            commands::get_file_content,
            commands::get_file_diff,
            commands::mark_as_read,
            commands::mark_as_archived,
            commands::pin_file,
            commands::set_reminder,
            commands::save_file,
            commands::add_watched_folder,
            commands::get_watched_folders,
            commands::search_files,
            commands::toggle_always_on_top,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // Start background services
            watcher::start_watcher(handle.clone(), db_for_builder.clone());
            reminders::start_reminder_loop(handle.clone(), db_for_builder.clone());

            // System tray
            let show_item = MenuItemBuilder::with_id("show", "Show MarkInbox").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let icon_bytes = include_bytes!("../icons/32x32.png");
            let icon = Image::from_bytes(icon_bytes).expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("MarkInbox")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
