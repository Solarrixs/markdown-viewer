# MarkInbox - Markdown Viewer

## Stack
- **Frontend**: Svelte (not SvelteKit) with TypeScript
- **Backend**: Tauri v2 (Rust) with SQLite via rusqlite
- **Plugins**: tauri-plugin-shell, tauri-plugin-notification, tauri-plugin-dialog
- **Build**: Vite + @sveltejs/vite-plugin-svelte

## Architecture

### Frontend (`src/`)
- `App.svelte` — Root component, sets up event listeners for file-changed/reminder-fired
- `src/lib/stores.ts` — All app state as Svelte writable stores (no component-local state for shared data)
- `src/lib/actions.ts` — All business logic functions (openFile, switchSection, archiveFile, togglePin, saveIfDirty, etc.)
- Components: FileList, FileItem, MainPane, TabBar, NoteToolbar, Editor, RenderedMarkdown, DiffView, CommandPalette, ReminderPicker, SettingsModal, SectionTabs, KeyboardHandler

### Backend (`src-tauri/src/`)
- `lib.rs` — Tauri app setup, plugin registration, command handler registration
- `commands.rs` — All `#[tauri::command]` functions (IPC boundary)
- `db.rs` — SQLite database layer (files, watched_folders, ignore_patterns tables)
- `watcher.rs` — File system watcher with restartable `WatcherHandle` (restart signal via tokio::sync::Notify)
- `git.rs` — Git diff integration
- `reminders.rs` — Background reminder loop

### Key patterns
- Tauri commands use `State<'_, Arc<Database>>` for DB access
- Watcher restarts when watched folders or ignore patterns change (via `WatcherHandle::restart()`)
- Frontend communicates with backend via `invoke()` from `@tauri-apps/api/core`
- Sections: inbox (unread + read), pinned, reminders, archive
- Keyboard: chord system (g+i, g+p, g+r, g+a for sections), Cmd+E edit, Cmd+D diff, Cmd+K palette, Cmd+, settings

## Dev commands
```bash
npm run dev          # Start Vite dev server only
npm run build        # Build frontend
npx tauri dev        # Full Tauri dev (frontend + backend)
npx tauri build      # Production build
```

## Conventions
- Stores over props for shared state
- Actions in `actions.ts`, not in components
- Auto-save on tab switch, section switch, and edit mode toggle via `saveIfDirty()`
- Dark theme (#1a1a1a backgrounds, #5b9bd5 accent)
