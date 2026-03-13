import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

// Mock @tauri-apps/api/core
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock @tauri-apps/plugin-dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

import {
  refreshItems,
  _resetRefreshState,
  saveContent,
  saveIfDirty,
  switchSection,
  openFile,
  archiveFile,
  togglePin,
  loadFileContent,
  closeTabByIndex,
  closeActiveTab,
  switchTab,
  reopenLastClosedTab,
  setReminderAndArchive,
  archiveByPath,
  showToast,
  dismissToast,
  openInSplit,
  closeSplit,
  toggleAlwaysOnTop,
} from './actions';
import {
  sectionItems,
  currentSection,
  selectedIndex,
  activeFilePath,
  openTabs,
  activeTabIndex,
  fileContent,
  fileDiff,
  editMode,
  showDiff,
  editText,
  selfSaveInFlight,
  toasts,
  alwaysOnTop,
  savedIndicator,
  splitPath,
  splitContent,
  splitDiff,
  activeSplit,
} from './stores';
import type { InboxItem, DiffResult } from './stores';

function makeItem(overrides: Partial<InboxItem> = {}): InboxItem {
  return {
    path: '/test/file.md',
    filename: 'file.md',
    status: 'read',
    pinned: false,
    reminder_time: null,
    last_modified: null,
    additions: 0,
    deletions: 0,
    ...overrides,
  };
}

const mockDiff: DiffResult = {
  content: 'diff content',
  hunks: [],
  additions: 1,
  deletions: 0,
};

function resetStores() {
  currentSection.set('inbox');
  sectionItems.set([]);
  selectedIndex.set(0);
  activeFilePath.set(null);
  openTabs.set([]);
  activeTabIndex.set(0);
  fileContent.set('');
  editText.set('');
  fileDiff.set(null);
  editMode.set(false);
  showDiff.set(false);
  selfSaveInFlight.set(false);
  toasts.set([]);
  alwaysOnTop.set(false);
  savedIndicator.set(false);
  splitPath.set(null);
  splitContent.set('');
  splitDiff.set(null);
  activeSplit.set('left');
}

beforeEach(() => {
  vi.useFakeTimers();
  resetStores();
  _resetRefreshState();
  mockInvoke.mockReset();
  // Default fallback so background fire-and-forget invoke calls don't crash.
  // Return [] for get_inbox_items since refreshItems sets it on sectionItems store.
  mockInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === 'get_inbox_items') return [];
    return undefined;
  });
});

afterEach(() => {
  vi.useRealTimers();
});

// ─── Toast ──────────────────────────────────────────────────────────────

describe('showToast / dismissToast', () => {
  it('adds a toast to the store', () => {
    showToast('hello');
    const t = get(toasts);
    expect(t).toHaveLength(1);
    expect(t[0].message).toBe('hello');
    expect(t[0].undoAction).toBeNull();
  });

  it('auto-dismisses after 4 seconds', () => {
    showToast('temp');
    expect(get(toasts)).toHaveLength(1);
    vi.advanceTimersByTime(4000);
    expect(get(toasts)).toHaveLength(0);
  });

  it('limits to 3 toasts (keeps last 2 + new)', () => {
    showToast('a');
    showToast('b');
    showToast('c');
    showToast('d');
    const t = get(toasts);
    expect(t.length).toBeLessThanOrEqual(3);
  });

  it('dismissToast removes specific toast', () => {
    showToast('keep');
    showToast('remove');
    const id = get(toasts)[1].id;
    dismissToast(id);
    expect(get(toasts)).toHaveLength(1);
    expect(get(toasts)[0].message).toBe('keep');
  });

  it('stores undo action when provided', () => {
    const undo = vi.fn().mockResolvedValue(undefined);
    showToast('with undo', undo);
    expect(get(toasts)[0].undoAction).toBe(undo);
  });
});

// ─── refreshItems ────────────────────────────────────────────────────────

describe('refreshItems', () => {
  it('fetches items for current section and updates store', async () => {
    const items = [makeItem()];
    mockInvoke.mockResolvedValueOnce(items);
    await refreshItems();
    expect(mockInvoke).toHaveBeenCalledWith('get_inbox_items', { filter: 'inbox' });
    expect(get(sectionItems)).toEqual(items);
  });

  it('uses the current section value', async () => {
    currentSection.set('archive');
    mockInvoke.mockResolvedValueOnce([]);
    await refreshItems();
    expect(mockInvoke).toHaveBeenCalledWith('get_inbox_items', { filter: 'archive' });
  });

  it('does not throw on invoke failure', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('backend error'));
    await expect(refreshItems()).resolves.toBeUndefined();
  });

  it('preserves existing items on failure', async () => {
    const items = [makeItem()];
    sectionItems.set(items);
    mockInvoke.mockRejectedValueOnce(new Error('fail'));
    await refreshItems();
    expect(get(sectionItems)).toEqual(items);
  });
});

// ─── saveContent ─────────────────────────────────────────────────────────

describe('saveContent', () => {
  it('invokes save_file and updates stores', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await saveContent('/test.md', 'new content');
    expect(mockInvoke).toHaveBeenCalledWith('save_file', { path: '/test.md', content: 'new content' });
    expect(get(fileContent)).toBe('new content');
    expect(get(savedIndicator)).toBe(true);
    expect(get(selfSaveInFlight)).toBe(true);
  });

  it('clears savedIndicator after 1500ms', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await saveContent('/test.md', 'x');
    expect(get(savedIndicator)).toBe(true);
    vi.advanceTimersByTime(1500);
    expect(get(savedIndicator)).toBe(false);
  });

  it('clears selfSaveInFlight after 500ms', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await saveContent('/test.md', 'x');
    expect(get(selfSaveInFlight)).toBe(true);
    vi.advanceTimersByTime(500);
    expect(get(selfSaveInFlight)).toBe(false);
  });
});

// ─── saveIfDirty ─────────────────────────────────────────────────────────

describe('saveIfDirty', () => {
  it('does nothing when no active file', async () => {
    mockInvoke.mockReset();
    await saveIfDirty();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('does nothing when not in edit mode', async () => {
    mockInvoke.mockReset();
    activeFilePath.set('/test.md');
    editMode.set(false);
    await saveIfDirty();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('does nothing when content has not changed', async () => {
    mockInvoke.mockReset();
    activeFilePath.set('/test.md');
    editMode.set(true);
    fileContent.set('same');
    editText.set('same');
    await saveIfDirty();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('saves when content has changed', async () => {
    activeFilePath.set('/test.md');
    editMode.set(true);
    fileContent.set('old');
    editText.set('new');
    mockInvoke.mockResolvedValueOnce(undefined);
    await saveIfDirty();
    expect(mockInvoke).toHaveBeenCalledWith('save_file', { path: '/test.md', content: 'new' });
  });

  it('does not throw on save failure', async () => {
    activeFilePath.set('/test.md');
    editMode.set(true);
    fileContent.set('old');
    editText.set('new');
    mockInvoke.mockRejectedValueOnce(new Error('save error'));
    await expect(saveIfDirty()).resolves.toBeUndefined();
  });
});

// ─── switchSection ───────────────────────────────────────────────────────

describe('switchSection', () => {
  it('sets section and resets selected index', async () => {
    selectedIndex.set(5);
    mockInvoke.mockResolvedValue([]);
    await switchSection('pinned');
    expect(get(currentSection)).toBe('pinned');
    expect(get(selectedIndex)).toBe(0);
  });

  it('calls refreshItems (invokes get_inbox_items)', async () => {
    mockInvoke.mockResolvedValue([]);
    await switchSection('archive');
    // refreshItems is fire-and-forget, so we need to flush
    await vi.advanceTimersByTimeAsync(0);
    expect(mockInvoke).toHaveBeenCalledWith('get_inbox_items', { filter: 'archive' });
  });

  it('saves dirty content before switching', async () => {
    activeFilePath.set('/test.md');
    editMode.set(true);
    fileContent.set('old');
    editText.set('new');
    mockInvoke.mockResolvedValue(undefined);
    await switchSection('pinned');
    expect(mockInvoke).toHaveBeenCalledWith('save_file', { path: '/test.md', content: 'new' });
  });
});

// ─── openFile ────────────────────────────────────────────────────────────

describe('openFile', () => {
  it('sets active file path and resets edit/diff mode', async () => {
    editMode.set(true);
    showDiff.set(true);
    mockInvoke.mockResolvedValue('# content');
    await openFile({ path: '/test.md', filename: 'test.md' });
    expect(get(activeFilePath)).toBe('/test.md');
    expect(get(editMode)).toBe(false);
    expect(get(showDiff)).toBe(false);
  });

  it('adds a new tab when file not already open', async () => {
    mockInvoke.mockResolvedValue('content');
    await openFile({ path: '/a.md', filename: 'a.md' });
    const tabs = get(openTabs);
    expect(tabs).toHaveLength(1);
    expect(tabs[0].path).toBe('/a.md');
    expect(get(activeTabIndex)).toBe(0);
  });

  it('does not duplicate tabs for same path', async () => {
    openTabs.set([{ path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 }]);
    mockInvoke.mockResolvedValue('content');
    await openFile({ path: '/a.md', filename: 'a.md' });
    expect(get(openTabs)).toHaveLength(1);
    expect(get(activeTabIndex)).toBe(0);
  });

  it('sets selectedIndex when provided', async () => {
    mockInvoke.mockResolvedValue('content');
    await openFile({ path: '/a.md', filename: 'a.md' }, 3);
    expect(get(selectedIndex)).toBe(3);
  });

  it('loads file content and marks as read', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return '# Hello';
      if (cmd === 'get_file_diff') return mockDiff;
      if (cmd === 'mark_as_read') return undefined;
      return undefined;
    });
    await openFile({ path: '/a.md', filename: 'a.md' });
    expect(mockInvoke).toHaveBeenCalledWith('get_file_content', { path: '/a.md' });
    expect(mockInvoke).toHaveBeenCalledWith('mark_as_read', { path: '/a.md' });
    expect(get(fileContent)).toBe('# Hello');
  });

  it('preserves additions/deletions in tab', async () => {
    mockInvoke.mockResolvedValue('content');
    await openFile({ path: '/a.md', filename: 'a.md', additions: 5, deletions: 2 });
    const tabs = get(openTabs);
    expect(tabs[0].additions).toBe(5);
    expect(tabs[0].deletions).toBe(2);
  });
});

// ─── loadFileContent ─────────────────────────────────────────────────────

describe('loadFileContent', () => {
  it('sets fileContent and editText on success', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return '# Test';
      if (cmd === 'get_file_diff') return mockDiff;
      return undefined;
    });
    await loadFileContent('/test.md');
    expect(get(fileContent)).toBe('# Test');
    expect(get(editText)).toBe('# Test');
  });

  it('loads diff in background without blocking content', async () => {
    let diffResolve: (v: DiffResult) => void;
    const diffPromise = new Promise<DiffResult>((res) => { diffResolve = res; });

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return '# Fast';
      if (cmd === 'get_file_diff') return diffPromise;
      return undefined;
    });

    await loadFileContent('/test.md');
    // Content should be set immediately
    expect(get(fileContent)).toBe('# Fast');
    // Diff hasn't resolved yet - that's fine, loadFileContent shouldn't block on it
  });

  it('closes tab and refreshes when file does not exist', async () => {
    openTabs.set([{ path: '/gone.md', filename: 'gone.md', additions: 0, deletions: 0 }]);
    activeFilePath.set('/gone.md');
    activeTabIndex.set(0);

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') throw new Error('not found');
      if (cmd === 'get_inbox_items') return [];
      return undefined;
    });

    await loadFileContent('/gone.md');
    // Tab should be removed
    expect(get(openTabs)).toHaveLength(0);
  });

  it('sets fileDiff to null when diff fails', async () => {
    fileDiff.set(mockDiff);
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return 'content';
      if (cmd === 'get_file_diff') throw new Error('no git');
      return undefined;
    });
    await loadFileContent('/test.md');
    // Wait for the background diff promise to settle
    await vi.advanceTimersByTimeAsync(0);
    expect(get(fileDiff)).toBeNull();
  });
});

// ─── togglePin ───────────────────────────────────────────────────────────

describe('togglePin', () => {
  it('does nothing when no active file', async () => {
    mockInvoke.mockReset();
    await togglePin();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('invokes toggle_pin and shows toast', async () => {
    activeFilePath.set('/test.md');
    sectionItems.set([makeItem({ path: '/test.md', filename: 'test.md' })]);
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'toggle_pin') return true;
      if (cmd === 'get_inbox_items') return [];
      return undefined;
    });
    await togglePin();
    expect(mockInvoke).toHaveBeenCalledWith('toggle_pin', { path: '/test.md' });
    const t = get(toasts);
    expect(t).toHaveLength(1);
    expect(t[0].message).toBe('Pinned test.md');
  });

  it('shows Unpinned when toggle returns false', async () => {
    activeFilePath.set('/test.md');
    sectionItems.set([makeItem({ path: '/test.md', filename: 'test.md' })]);
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'toggle_pin') return false;
      if (cmd === 'get_inbox_items') return [];
      return undefined;
    });
    await togglePin();
    expect(get(toasts)[0].message).toBe('Unpinned test.md');
  });

  it('toast has undo action that re-toggles', async () => {
    activeFilePath.set('/test.md');
    sectionItems.set([makeItem({ path: '/test.md' })]);
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'toggle_pin') return true;
      if (cmd === 'get_inbox_items') return [];
      return undefined;
    });
    await togglePin();
    const undo = get(toasts)[0].undoAction;
    expect(undo).not.toBeNull();
    if (undo) {
      await undo();
      // Should have called toggle_pin again
      const toggleCalls = mockInvoke.mock.calls.filter(c => c[0] === 'toggle_pin');
      expect(toggleCalls.length).toBeGreaterThanOrEqual(2);
    }
  });
});

// ─── archiveFile ─────────────────────────────────────────────────────────

describe('archiveFile', () => {
  it('does nothing when no active file', async () => {
    mockInvoke.mockReset();
    await archiveFile();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('invokes mark_as_archived and shows toast', async () => {
    activeFilePath.set('/test.md');
    sectionItems.set([makeItem({ path: '/test.md', filename: 'test.md' })]);
    openTabs.set([{ path: '/test.md', filename: 'test.md', additions: 0, deletions: 0 }]);
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'mark_as_archived') return undefined;
      if (cmd === 'get_inbox_items') return [];
      return undefined;
    });
    await archiveFile();
    expect(mockInvoke).toHaveBeenCalledWith('mark_as_archived', { path: '/test.md' });
    expect(get(toasts)[0].message).toBe('Archived test.md');
  });

  it('uses extracted filename when item not in sectionItems', async () => {
    activeFilePath.set('/some/path/notes.md');
    sectionItems.set([]);
    openTabs.set([{ path: '/some/path/notes.md', filename: 'notes.md', additions: 0, deletions: 0 }]);
    activeTabIndex.set(0);
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'mark_as_archived') return undefined;
      if (cmd === 'get_inbox_items') return [];
      if (cmd === 'get_file_content') return 'content';
      if (cmd === 'get_file_diff') return null;
      return undefined;
    });
    await archiveFile();
    expect(get(toasts)[0].message).toBe('Archived notes.md');
  });
});

// ─── archiveByPath ───────────────────────────────────────────────────────

describe('archiveByPath', () => {
  it('closes tab, archives, and refreshes', async () => {
    openTabs.set([{ path: '/test.md', filename: 'test.md', additions: 0, deletions: 0 }]);
    activeTabIndex.set(0);
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'mark_as_archived') return undefined;
      if (cmd === 'get_inbox_items') return [];
      return undefined;
    });
    await archiveByPath('/test.md');
    expect(mockInvoke).toHaveBeenCalledWith('mark_as_archived', { path: '/test.md' });
    expect(mockInvoke).toHaveBeenCalledWith('get_inbox_items', expect.any(Object));
  });
});

// ─── setReminderAndArchive ───────────────────────────────────────────────

describe('setReminderAndArchive', () => {
  it('sets reminder, archives, shows toast', async () => {
    activeFilePath.set('/test.md');
    sectionItems.set([makeItem({ path: '/test.md', filename: 'test.md' })]);
    openTabs.set([{ path: '/test.md', filename: 'test.md', additions: 0, deletions: 0 }]);
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'set_reminder') return undefined;
      if (cmd === 'mark_as_archived') return undefined;
      if (cmd === 'get_inbox_items') return [];
      return undefined;
    });
    await setReminderAndArchive('/test.md', '2026-03-14T10:00:00Z');
    expect(mockInvoke).toHaveBeenCalledWith('set_reminder', { path: '/test.md', time: '2026-03-14T10:00:00Z' });
    expect(mockInvoke).toHaveBeenCalledWith('mark_as_archived', { path: '/test.md' });
    expect(get(toasts)[0].message).toBe('Reminder set for test.md');
  });
});

// ─── closeTabByIndex ─────────────────────────────────────────────────────

describe('closeTabByIndex', () => {
  it('does nothing for invalid index', async () => {
    mockInvoke.mockReset();
    await closeTabByIndex(-1);
    await closeTabByIndex(5);
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('removes tab and clears active file when last tab closed', async () => {
    openTabs.set([{ path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 }]);
    activeFilePath.set('/a.md');
    activeTabIndex.set(0);
    await closeTabByIndex(0);
    expect(get(openTabs)).toHaveLength(0);
    expect(get(activeFilePath)).toBeNull();
    expect(get(editMode)).toBe(false);
    expect(get(showDiff)).toBe(false);
  });

  it('switches to next tab when active tab is closed', async () => {
    openTabs.set([
      { path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 },
      { path: '/b.md', filename: 'b.md', additions: 0, deletions: 0 },
    ]);
    activeTabIndex.set(0);
    activeFilePath.set('/a.md');
    mockInvoke.mockResolvedValue('content b');
    await closeTabByIndex(0);
    expect(get(openTabs)).toHaveLength(1);
    expect(get(openTabs)[0].path).toBe('/b.md');
  });

  it('adjusts activeTabIndex when tab before active is closed', async () => {
    openTabs.set([
      { path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 },
      { path: '/b.md', filename: 'b.md', additions: 0, deletions: 0 },
      { path: '/c.md', filename: 'c.md', additions: 0, deletions: 0 },
      { path: '/d.md', filename: 'd.md', additions: 0, deletions: 0 },
    ]);
    activeTabIndex.set(2);
    activeFilePath.set('/c.md');
    mockInvoke.mockResolvedValue('content');
    // Close tab 0 (/a.md) — active is at index 2, which shifts to 1
    await closeTabByIndex(0);
    // activeTabIndex decrements since the closed tab was before it
    expect(get(activeTabIndex)).toBe(1);
  });
});

// ─── closeActiveTab ──────────────────────────────────────────────────────

describe('closeActiveTab', () => {
  it('closes the current active tab', async () => {
    openTabs.set([{ path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 }]);
    activeTabIndex.set(0);
    activeFilePath.set('/a.md');
    await closeActiveTab();
    expect(get(openTabs)).toHaveLength(0);
    expect(get(activeFilePath)).toBeNull();
  });
});

// ─── reopenLastClosedTab ─────────────────────────────────────────────────

describe('reopenLastClosedTab', () => {
  it('does nothing when no closed tabs', async () => {
    // Need a default mock in case internal code paths call invoke
    mockInvoke.mockResolvedValue(undefined);
    await reopenLastClosedTab();
    // reopenLastClosedTab pops from recentlyClosedTabs which is module-scoped;
    // if previous tests pushed items, it may call invoke. Just verify no crash.
  });

  it('reopens the last closed tab', async () => {
    // Close a tab first
    openTabs.set([{ path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 }]);
    activeTabIndex.set(0);
    activeFilePath.set('/a.md');
    await closeTabByIndex(0);
    expect(get(openTabs)).toHaveLength(0);

    // Reopen
    mockInvoke.mockResolvedValue('content');
    await reopenLastClosedTab();
    expect(get(openTabs)).toHaveLength(1);
    expect(get(openTabs)[0].path).toBe('/a.md');
  });
});

// ─── switchTab ───────────────────────────────────────────────────────────

describe('switchTab', () => {
  it('sets active tab index and loads content for new path', async () => {
    openTabs.set([
      { path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 },
      { path: '/b.md', filename: 'b.md', additions: 0, deletions: 0 },
    ]);
    activeTabIndex.set(0);
    activeFilePath.set('/a.md');
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return '# B';
      if (cmd === 'get_file_diff') return null;
      return undefined;
    });
    await switchTab(1);
    expect(get(activeTabIndex)).toBe(1);
    expect(get(activeFilePath)).toBe('/b.md');
    expect(get(editMode)).toBe(false);
    expect(get(showDiff)).toBe(false);
  });

  it('skips loading when path has not changed', async () => {
    openTabs.set([
      { path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 },
    ]);
    activeTabIndex.set(0);
    activeFilePath.set('/a.md');
    await switchTab(0);
    // Should not call get_file_content since path didn't change
    expect(mockInvoke).not.toHaveBeenCalledWith('get_file_content', expect.any(Object));
  });

  it('does nothing for invalid tab index', async () => {
    mockInvoke.mockReset();
    openTabs.set([]);
    await switchTab(0);
    // No tab at index 0, so activeFilePath should remain unchanged
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});

// ─── toggleAlwaysOnTop ───────────────────────────────────────────────────

describe('toggleAlwaysOnTop', () => {
  it('updates alwaysOnTop store on success', async () => {
    mockInvoke.mockResolvedValueOnce(true);
    const result = await toggleAlwaysOnTop();
    expect(result).toBe(true);
    expect(get(alwaysOnTop)).toBe(true);
  });

  it('returns false on failure', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('fail'));
    const result = await toggleAlwaysOnTop();
    expect(result).toBe(false);
  });
});

// ─── openInSplit ─────────────────────────────────────────────────────────

describe('openInSplit', () => {
  it('sets split path and loads content', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return '# Split';
      if (cmd === 'get_file_diff') return mockDiff;
      return undefined;
    });
    await openInSplit({ path: '/split.md', filename: 'split.md' });
    expect(get(splitPath)).toBe('/split.md');
    expect(get(activeSplit)).toBe('right');
    expect(get(splitContent)).toBe('# Split');
    expect(get(splitDiff)).toEqual(mockDiff);
  });

  it('sets splitDiff to null when diff fails', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return 'content';
      if (cmd === 'get_file_diff') throw new Error('no diff');
      return undefined;
    });
    await openInSplit({ path: '/split.md', filename: 'split.md' });
    expect(get(splitDiff)).toBeNull();
  });
});

// ─── closeSplit ──────────────────────────────────────────────────────────

describe('closeSplit', () => {
  it('resets all split stores', () => {
    splitPath.set('/test.md');
    splitContent.set('content');
    splitDiff.set(mockDiff);
    activeSplit.set('right');
    closeSplit();
    expect(get(splitPath)).toBeNull();
    expect(get(splitContent)).toBe('');
    expect(get(splitDiff)).toBeNull();
    expect(get(activeSplit)).toBe('left');
  });
});

// ─── Concurrency / blocking behavior ─────────────────────────────────────

describe('non-blocking behavior', () => {
  it('openFile resolves even if mark_as_read is slow', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return 'fast content';
      if (cmd === 'get_file_diff') return null;
      if (cmd === 'mark_as_read') {
        // Simulate slow backend
        return new Promise(resolve => setTimeout(resolve, 5000));
      }
      return undefined;
    });

    const promise = openFile({ path: '/test.md', filename: 'test.md' });
    // Content should be loaded but we're still waiting on mark_as_read
    vi.advanceTimersByTime(5000);
    await promise;
    expect(get(fileContent)).toBe('fast content');
  });

  it('switchSection does not block on refreshItems', async () => {
    let refreshResolved = false;
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_inbox_items') {
        return new Promise<InboxItem[]>(resolve => {
          setTimeout(() => {
            refreshResolved = true;
            resolve([]);
          }, 5000);
        });
      }
      return undefined;
    });

    await switchSection('pinned');
    // switchSection should have returned already
    expect(get(currentSection)).toBe('pinned');
    expect(get(selectedIndex)).toBe(0);
    // refreshItems hasn't resolved yet
    expect(refreshResolved).toBe(false);
  });

  it('togglePin does not block on refreshItems', async () => {
    activeFilePath.set('/test.md');
    sectionItems.set([makeItem({ path: '/test.md' })]);

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'toggle_pin') return true;
      if (cmd === 'get_inbox_items') {
        return new Promise<InboxItem[]>(resolve => {
          setTimeout(() => resolve([]), 5000);
        });
      }
      return undefined;
    });

    await togglePin();
    // Toast should be shown immediately, regardless of refreshItems status
    expect(get(toasts)).toHaveLength(1);
    expect(get(toasts)[0].message).toContain('Pinned');
  });
});

// ─── Edge cases ──────────────────────────────────────────────────────────

describe('edge cases', () => {
  it('multiple rapid openFile calls update to last file', async () => {
    mockInvoke.mockImplementation(async (cmd: string, args: Record<string, string>) => {
      if (cmd === 'get_file_content') return `content of ${args.path}`;
      if (cmd === 'get_file_diff') return null;
      return undefined;
    });

    // Simulate rapid clicks
    const p1 = openFile({ path: '/a.md', filename: 'a.md' });
    const p2 = openFile({ path: '/b.md', filename: 'b.md' });
    await Promise.all([p1, p2]);

    // Last file should be active
    expect(get(activeFilePath)).toBe('/b.md');
    // Both tabs should exist
    expect(get(openTabs)).toHaveLength(2);
  });

  it('archiveFile with no matching sectionItem uses filename from path', async () => {
    activeFilePath.set('/some/deep/path/readme.md');
    sectionItems.set([]);
    mockInvoke.mockResolvedValue(undefined);
    await archiveFile();
    await vi.advanceTimersByTimeAsync(0);
    const t = get(toasts);
    expect(t.length).toBeGreaterThanOrEqual(1);
    expect(t[0].message).toContain('readme.md');
  });

  it('closeTabByIndex saves dirty content before closing', async () => {
    openTabs.set([{ path: '/a.md', filename: 'a.md', additions: 0, deletions: 0 }]);
    activeTabIndex.set(0);
    activeFilePath.set('/a.md');
    editMode.set(true);
    fileContent.set('old');
    editText.set('modified');
    mockInvoke.mockResolvedValue(undefined);

    await closeTabByIndex(0);
    expect(mockInvoke).toHaveBeenCalledWith('save_file', { path: '/a.md', content: 'modified' });
  });
});

// ─── refreshItems concurrency guard ──────────────────────────────────────

describe('refreshItems concurrency guard', () => {
  it('coalesces concurrent refreshItems calls', async () => {
    let callCount = 0;
    let resolvers: Array<(v: InboxItem[]) => void> = [];

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_inbox_items') {
        callCount++;
        return new Promise<InboxItem[]>(resolve => {
          resolvers.push(resolve);
        });
      }
      return undefined;
    });

    // Fire 3 refreshItems calls concurrently
    const p1 = refreshItems();
    const p2 = refreshItems();
    const p3 = refreshItems();

    // Allow microtasks to settle so the first call enters invoke
    await Promise.resolve();
    await Promise.resolve();

    // Only 1 should actually invoke get_inbox_items (the rest are queued)
    expect(callCount).toBe(1);

    // Resolve the first call
    resolvers[0]([]);
    await p1;
    // Allow the queued refresh to start
    await Promise.resolve();
    await Promise.resolve();

    // The queued call should now fire (coalesced from 2 into 1)
    expect(callCount).toBe(2);

    // Resolve the second
    resolvers[1]([]);
    await Promise.resolve();
    await Promise.resolve();

    // No more queued calls
    expect(callCount).toBe(2);
  });

  it('togglePin shows feedback even during slow refresh', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_inbox_items') {
        // Slow refresh that never resolves during this test
        return new Promise<InboxItem[]>(() => {});
      }
      if (cmd === 'toggle_pin') return true;
      return undefined;
    });

    // Start a slow refresh
    refreshItems();
    await Promise.resolve();

    // togglePin should still work while refresh is pending
    activeFilePath.set('/test.md');
    sectionItems.set([makeItem({ path: '/test.md' })]);
    await togglePin();

    // Toast should appear even though refreshItems hasn't resolved
    expect(get(toasts)).toHaveLength(1);
    expect(get(toasts)[0].message).toContain('Pinned');
  });
});

// ─── openFile does not block on mark_as_read ─────────────────────────────

describe('openFile non-blocking mark_as_read', () => {
  it('loads content without waiting for mark_as_read', async () => {
    let markAsReadResolved = false;

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_file_content') return '# Content loads fast';
      if (cmd === 'get_file_diff') return null;
      if (cmd === 'mark_as_read') {
        return new Promise(resolve => {
          setTimeout(() => {
            markAsReadResolved = true;
            resolve(undefined);
          }, 10000);
        });
      }
      if (cmd === 'get_inbox_items') return [];
      return undefined;
    });

    await openFile({ path: '/test.md', filename: 'test.md' });

    // Content should be loaded immediately
    expect(get(fileContent)).toBe('# Content loads fast');
    // mark_as_read hasn't resolved yet
    expect(markAsReadResolved).toBe(false);
  });
});
