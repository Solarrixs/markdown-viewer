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

// Bulk open modal
export const bulkOpenModalOpen = writable<boolean>(false);

// Rename trigger — incremented to signal TabBar to start rename on active tab
export const renameTrigger = writable<number>(0);

// Always on top state
export const alwaysOnTop = writable<boolean>(false);

// Saved indicator flash
export const savedIndicator = writable<boolean>(false);

// Editor text (promoted from Editor local state for auto-save)
export const editText = writable<string>('');

// Flag to suppress watcher-triggered reloads during self-initiated saves
export const selfSaveInFlight = writable<boolean>(false);

// Table of contents panel visible
export const showToc = writable<boolean>(true);

// Scroll position ratio (0-1) preserved across edit/view toggles
export const scrollRatio = writable<number>(0);

// Ticks every 60s to force re-evaluation of relative timestamps
export const tick = readable(0, (set) => {
  let count = 0;
  const interval = setInterval(() => set(++count), 60_000);
  return () => clearInterval(interval);
});

export interface ToastItem {
  id: number;
  message: string;
  undoAction: (() => Promise<void>) | null;
  timestamp: number;
}
export const toasts = writable<ToastItem[]>([]);

export const shortcutHelpOpen = writable<boolean>(false);

export const findBarOpen = writable<boolean>(false);

// Timeline / commit view
export type SidebarViewMode = 'files' | 'timeline';
export const sidebarViewMode = writable<SidebarViewMode>('files');

export interface CommitRecord {
  id: number;
  repo_path: string;
  oid: string;
  message: string;
  author: string | null;
  timestamp: string;
  files_changed: number;
  additions: number;
  deletions: number;
  session_id: string | null;
  created_at: string;
}

export interface CommitFileRecord {
  id: number;
  commit_oid: string;
  file_path: string;
  additions: number;
  deletions: number;
  status: string;
}

export interface DiffSummaryRecord {
  id: number;
  commit_oid: string;
  summary: string;
  model: string | null;
  created_at: string;
}

export interface AnnotationRecord {
  id: number;
  file_path: string;
  line_number: number;
  commit_hash: string | null;
  annotation_text: string;
  sent: boolean;
  created_at: string;
  updated_at: string;
}

export interface ReviewStatusRecord {
  id: number;
  commit_hash: string;
  status: string;
  reviewed_at: string | null;
  notes: string | null;
}

export interface SessionInfo {
  session_id: string;
  timestamp: string;
  status: 'active' | 'idle' | 'completed';
}

export const recentCommits = writable<CommitRecord[]>([]);
export const selectedCommitOid = writable<string | null>(null);
export const commitFiles = writable<CommitFileRecord[]>([]);
export const annotations = writable<AnnotationRecord[]>([]);
export const activeAnnotationLine = writable<number | null>(null);
export const reviewStatuses = writable<ReviewStatusRecord[]>([]);
export const reviewProgress = writable<{ reviewed: number; total: number }>({ reviewed: 0, total: 0 });
export const claudeSessions = writable<SessionInfo[]>([]);

export const feedbackPanelOpen = writable<boolean>(false);

export const splitPath = writable<string | null>(null);
export const splitContent = writable<string>('');
export const splitDiff = writable<DiffResult | null>(null);
export const activeSplit = writable<'left' | 'right'>('left');
