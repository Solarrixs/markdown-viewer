import { writable } from 'svelte/store';

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

export type Section = 'inbox' | 'pinned' | 'reminders';

// Current section
export const currentSection = writable<Section>('inbox');

// All inbox items for current section
export const inboxItems = writable<InboxItem[]>([]);

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

