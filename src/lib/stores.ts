import { writable, readable } from 'svelte/store';

export interface InboxItem {
  path: string;
  filename: string;
  status: string;
  pinned: boolean;
  reminder_time: string | null;
  last_modified: string | null;
  additions: number;
  deletions: number;
}

export interface DiffHunk {
  old_start: number;
  new_start: number;
  new_lines: number;
  content: string;
  change_type: string;
}

export interface DiffResult {
  content: string;
  hunks: DiffHunk[];
  additions: number;
  deletions: number;
}

export interface OpenTab {
  path: string;
  filename: string;
  additions: number;
  deletions: number;
}

export interface WatchedFolder {
  id: number;
  path: string;
  active: boolean;
}

export interface IgnorePattern {
  id: number;
  pattern: string;
}

export type Section = 'inbox' | 'pinned' | 'reminders' | 'archive';

// Current section
export const currentSection = writable<Section>('inbox');

// All inbox items for current section
export const sectionItems = writable<InboxItem[]>([]);

// Selected item index in sidebar
export const selectedIndex = writable<number>(0);

// Currently active file path (shown in main pane)
export const activeFilePath = writable<string | null>(null);

// Open tabs
export const openTabs = writable<OpenTab[]>([]);

// Active tab index
export const activeTabIndex = writable<number>(0);

// Edit mode
export const editMode = writable<boolean>(false);

// Show diff view (Cmd+D toggle)
export const showDiff = writable<boolean>(false);

// Command palette open
export const commandPaletteOpen = writable<boolean>(false);

// Reminder picker open
export const reminderPickerOpen = writable<boolean>(false);

// Sidebar visible
export const sidebarVisible = writable<boolean>(true);

// File content cache
export const fileContent = writable<string>('');

// Diff result cache
export const fileDiff = writable<DiffResult | null>(null);

// Settings modal open
export const settingsOpen = writable<boolean>(false);

// Always on top state
export const alwaysOnTop = writable<boolean>(false);

// Saved indicator flash
export const savedIndicator = writable<boolean>(false);

// Editor text (promoted from Editor local state for auto-save)
export const editText = writable<string>('');

// Flag to suppress watcher-triggered reloads during self-initiated saves
export const selfSaveInFlight = writable<boolean>(false);

// Table of contents panel visible
export const showToc = writable<boolean>(false);

// Scroll position ratio (0-1) preserved across edit/view toggles
export const scrollRatio = writable<number>(0);

// Ticks every 60s to force re-evaluation of relative timestamps
export const tick = readable(0, (set) => {
  let count = 0;
  const interval = setInterval(() => set(++count), 60_000);
  return () => clearInterval(interval);
});
