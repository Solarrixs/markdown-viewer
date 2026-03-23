mod commands;
mod commits;
mod db;
mod git;
mod reminders;
mod sessions;
mod summarize;
pub mod watcher;

use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{CheckMenuItemBuilder, Menu, MenuBuilder, MenuItem, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    App, Emitter, Manager,
};

/// Build the application menu bar.
fn build_menu(app: &App) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    // Custom menu items
    let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, Some("super+,"))?;
    let save_item = MenuItem::with_id(app, "save", "Save", true, Some("super+S"))?;
    let close_tab_item =
        MenuItem::with_id(app, "close_tab", "Close Tab", true, Some("super+W"))?;
    let edit_mode_item =
        MenuItem::with_id(app, "edit_mode", "Edit Mode", true, Some("super+E"))?;
    let diff_view_item =
        MenuItem::with_id(app, "diff_view", "Show Changes", true, Some("super+D"))?;
    let toggle_sidebar_item =
        MenuItem::with_id(app, "toggle_sidebar", "Toggle Sidebar", true, Some("super+\\"))?;
    let command_palette_item =
        MenuItem::with_id(app, "command_palette", "Command Palette", true, Some("super+K"))?;
    let always_on_top_item =
        CheckMenuItemBuilder::with_id("always_on_top", "Always on Top").build(app)?;
    let view_changes_item =
        MenuItem::with_id(app, "diff_view_history", "View File Changes", true, None::<&str>)?;
    let help_item =
        MenuItem::with_id(app, "about_help", "Mark In Box Help", true, None::<&str>)?;

    // Mark In Box (app menu)
    let app_submenu = SubmenuBuilder::with_id(app, "app", "Mark In Box")
        .about(Some(tauri::menu::AboutMetadata {
            name: Some("Mark In Box".into()),
            version: Some("0.1.0".into()),
            copyright: Some("Created by Maxx Yung".into()),
            ..Default::default()
        }))
        .separator()
        .item(&settings_item)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    let open_file_item =
        MenuItem::with_id(app, "open_file", "Open File...", true, Some("super+O"))?;

    // File
    let file_submenu = SubmenuBuilder::with_id(app, "file", "File")
        .item(&open_file_item)
        .separator()
        .item(&save_item)
        .separator()
        .item(&close_tab_item)
        .build()?;

    // Edit (predefined items — critical for copy/paste to work on macOS)
    let edit_submenu = SubmenuBuilder::with_id(app, "edit", "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    // View
    let view_submenu = SubmenuBuilder::with_id(app, "view", "View")
        .item(&edit_mode_item)
        .item(&diff_view_item)
        .separator()
        .item(&toggle_sidebar_item)
        .separator()
        .item(&command_palette_item)
        .separator()
        .fullscreen()
        .build()?;

    // History
    let history_submenu = SubmenuBuilder::with_id(app, "history", "History")
        .item(&view_changes_item)
        .build()?;

    // Window
    let window_submenu = SubmenuBuilder::with_id(app, "window", "Window")
        .minimize()
        .maximize()
        .separator()
        .item(&always_on_top_item)
        .separator()
        .fullscreen()
        .build()?;

    // Help
    let help_submenu = SubmenuBuilder::with_id(app, "help", "Help")
        .item(&help_item)
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&app_submenu)
        .item(&file_submenu)
        .item(&edit_submenu)
        .item(&view_submenu)
        .item(&history_submenu)
        .item(&window_submenu)
        .item(&help_submenu)
        .build()?;

    Ok(menu)
}

/// Build the system tray icon and menu.
fn build_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItemBuilder::with_id("show", "Show Mark In Box").build(app)?;
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
        .tooltip("Mark In Box")
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
}

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
        .plugin(tauri_plugin_dialog::init())
        .manage(database.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_inbox_items,
            commands::get_file_content,
            commands::get_file_diff,
            commands::mark_as_read,
            commands::delete_file,
            commands::mark_as_archived,
            commands::toggle_pin,
            commands::set_reminder,
            commands::save_file,
            commands::add_watched_folder,
            commands::get_watched_folders,
            commands::remove_watched_folder,
            commands::search_files,
            commands::toggle_always_on_top,
            commands::get_ignore_patterns,
            commands::add_ignore_pattern,
            commands::remove_ignore_pattern,
            commands::ensure_file_tracked,
            commands::ensure_files_tracked,
            commands::rename_file,
            commands::open_in_finder,
            commands::open_in_terminal,
            commands::restore_file,
            commands::search_file_contents,
            commands::get_setting,
            commands::set_setting,
            commands::get_recent_commits,
            commands::get_commit_files,
            commands::save_annotation,
            commands::delete_annotation,
            commands::get_annotations,
            commands::get_unsent_annotations,
            commands::mark_annotations_sent,
            commands::set_review_status,
            commands::get_review_statuses,
            commands::get_review_progress,
            commands::get_diff_summary,
            commands::get_commit_file_diff,
            summarize::trigger_summarize,
            sessions::list_claude_sessions,
            sessions::send_feedback_to_session,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // Start background services
            let watcher_handle = watcher::start_watcher(handle.clone(), db_for_builder.clone());
            app.manage(watcher_handle);
            reminders::start_reminder_loop(handle.clone(), db_for_builder.clone());
            commits::start_commit_poller(handle.clone(), db_for_builder.clone());

            // Prune DB entries for files deleted while the app was closed (background)
            let db_prune = db_for_builder.clone();
            std::thread::spawn(move || {
                match db_prune.prune_missing_files() {
                    Ok(pruned) if pruned > 0 => eprintln!("Pruned {} missing files from database", pruned),
                    Err(e) => eprintln!("Failed to prune missing files: {}", e),
                    _ => {}
                }
            });

            // Build menu bar
            let app_menu = build_menu(app)?;
            app.set_menu(app_menu)?;

            // Menu event handler
            app.on_menu_event(move |app_handle, event| {
                let id = event.id().as_ref();
                match id {
                    "always_on_top" => {
                        let _ = commands::toggle_always_on_top(app_handle.clone());
                    }
                    "diff_view_history" => {
                        let _ = app_handle.emit("diff_view", ());
                    }
                    "settings" | "save" | "close_tab" | "edit_mode" | "diff_view"
                    | "toggle_sidebar" | "command_palette" | "open_file" => {
                        let _ = app_handle.emit(id, ());
                    }
                    _ => {}
                }
            });

            // Build system tray
            build_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
