import { describe, it, expect, vi, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
  currentSection,
  sectionItems,
  selectedIndex,
  activeFilePath,
  openTabs,
  activeTabIndex,
  editMode,
  showDiff,
  commandPaletteOpen,
  reminderPickerOpen,
  sidebarVisible,
  fileContent,
  fileDiff,
  settingsOpen,
  alwaysOnTop,
  savedIndicator,
  editText,
  selfSaveInFlight,
  showToc,
  scrollRatio,
  tick,
  toasts,
  shortcutHelpOpen,
  findBarOpen,
  splitPath,
  splitContent,
  splitDiff,
  activeSplit,
} from './stores';
import type {
  InboxItem,
  DiffResult,
  DiffHunk,
  OpenTab,
  WatchedFolder,
  IgnorePattern,
  Section,
  ToastItem,
} from './stores';

// Helper: reset a writable store to a value after each test
function resetStore(store: { set: (v: any) => void }, value: any) {
  afterEach(() => store.set(value));
}

describe('stores', () => {
  // ── Interface structure tests ──────────────────────────────────────

  describe('InboxItem interface', () => {
    it('accepts a valid InboxItem object', () => {
      const item: InboxItem = {
        path: '/tmp/test.md',
        filename: 'test.md',
        status: 'unread',
        pinned: false,
        reminder_time: null,
        last_modified: '2025-01-01T00:00:00Z',
        additions: 5,
        deletions: 2,
      };
      expect(item.path).toBe('/tmp/test.md');
      expect(item.filename).toBe('test.md');
      expect(item.status).toBe('unread');
      expect(item.pinned).toBe(false);
      expect(item.reminder_time).toBeNull();
      expect(item.last_modified).toBe('2025-01-01T00:00:00Z');
      expect(item.additions).toBe(5);
      expect(item.deletions).toBe(2);
    });

    it('allows non-null reminder_time and last_modified', () => {
      const item: InboxItem = {
        path: '/a.md',
        filename: 'a.md',
        status: 'read',
        pinned: true,
        reminder_time: '2025-06-01T09:00:00Z',
        last_modified: '2025-05-31T12:00:00Z',
        additions: 0,
        deletions: 0,
      };
      expect(item.reminder_time).toBe('2025-06-01T09:00:00Z');
      expect(item.last_modified).toBe('2025-05-31T12:00:00Z');
    });
  });

  describe('DiffHunk interface', () => {
    it('accepts a valid DiffHunk object', () => {
      const hunk: DiffHunk = {
        old_start: 1,
        new_start: 1,
        new_lines: 5,
        content: '+added line',
        change_type: 'addition',
      };
      expect(hunk.old_start).toBe(1);
      expect(hunk.new_start).toBe(1);
      expect(hunk.new_lines).toBe(5);
      expect(hunk.content).toBe('+added line');
      expect(hunk.change_type).toBe('addition');
    });
  });

  describe('DiffResult interface', () => {
    it('accepts a valid DiffResult with hunks', () => {
      const diff: DiffResult = {
        content: 'full diff content',
        hunks: [
          { old_start: 1, new_start: 1, new_lines: 3, content: '+line', change_type: 'add' },
        ],
        additions: 10,
        deletions: 3,
      };
      expect(diff.content).toBe('full diff content');
      expect(diff.hunks).toHaveLength(1);
      expect(diff.additions).toBe(10);
      expect(diff.deletions).toBe(3);
    });

    it('accepts a DiffResult with empty hunks', () => {
      const diff: DiffResult = { content: '', hunks: [], additions: 0, deletions: 0 };
      expect(diff.hunks).toHaveLength(0);
    });
  });

  describe('OpenTab interface', () => {
    it('accepts a valid OpenTab object', () => {
      const tab: OpenTab = {
        path: '/tmp/note.md',
        filename: 'note.md',
        additions: 3,
        deletions: 1,
      };
      expect(tab.path).toBe('/tmp/note.md');
      expect(tab.filename).toBe('note.md');
      expect(tab.additions).toBe(3);
      expect(tab.deletions).toBe(1);
    });
  });

  describe('WatchedFolder interface', () => {
    it('accepts a valid WatchedFolder object', () => {
      const folder: WatchedFolder = { id: 1, path: '/home/docs', active: true };
      expect(folder.id).toBe(1);
      expect(folder.path).toBe('/home/docs');
      expect(folder.active).toBe(true);
    });
  });

  describe('IgnorePattern interface', () => {
    it('accepts a valid IgnorePattern object', () => {
      const pattern: IgnorePattern = { id: 42, pattern: '*.tmp' };
      expect(pattern.id).toBe(42);
      expect(pattern.pattern).toBe('*.tmp');
    });
  });

  describe('ToastItem interface', () => {
    it('accepts a valid ToastItem with undoAction', () => {
      const toast: ToastItem = {
        id: 1,
        message: 'File archived',
        undoAction: async () => {},
        timestamp: Date.now(),
      };
      expect(toast.id).toBe(1);
      expect(toast.message).toBe('File archived');
      expect(toast.undoAction).toBeInstanceOf(Function);
      expect(typeof toast.timestamp).toBe('number');
    });

    it('accepts a ToastItem with null undoAction', () => {
      const toast: ToastItem = {
        id: 2,
        message: 'Saved',
        undoAction: null,
        timestamp: 0,
      };
      expect(toast.undoAction).toBeNull();
    });
  });

  // ── Section type ───────────────────────────────────────────────────

  describe('Section type', () => {
    it('allows all four section values', () => {
      const sections: Section[] = ['inbox', 'pinned', 'reminders', 'archive'];
      expect(sections).toEqual(['inbox', 'pinned', 'reminders', 'archive']);
    });
  });

  // ── Initial values ─────────────────────────────────────────────────

  describe('initial values', () => {
    it('currentSection defaults to "inbox"', () => {
      expect(get(currentSection)).toBe('inbox');
    });

    it('sectionItems defaults to empty array', () => {
      expect(get(sectionItems)).toEqual([]);
    });

    it('selectedIndex defaults to 0', () => {
      expect(get(selectedIndex)).toBe(0);
    });

    it('activeFilePath defaults to null', () => {
      expect(get(activeFilePath)).toBeNull();
    });

    it('openTabs defaults to empty array', () => {
      expect(get(openTabs)).toEqual([]);
    });

    it('activeTabIndex defaults to 0', () => {
      expect(get(activeTabIndex)).toBe(0);
    });

    it('editMode defaults to false', () => {
      expect(get(editMode)).toBe(false);
    });

    it('showDiff defaults to false', () => {
      expect(get(showDiff)).toBe(false);
    });

    it('commandPaletteOpen defaults to false', () => {
      expect(get(commandPaletteOpen)).toBe(false);
    });

    it('reminderPickerOpen defaults to false', () => {
      expect(get(reminderPickerOpen)).toBe(false);
    });

    it('sidebarVisible defaults to true', () => {
      expect(get(sidebarVisible)).toBe(true);
    });

    it('fileContent defaults to empty string', () => {
      expect(get(fileContent)).toBe('');
    });

    it('fileDiff defaults to null', () => {
      expect(get(fileDiff)).toBeNull();
    });

    it('settingsOpen defaults to false', () => {
      expect(get(settingsOpen)).toBe(false);
    });

    it('alwaysOnTop defaults to false', () => {
      expect(get(alwaysOnTop)).toBe(false);
    });

    it('savedIndicator defaults to false', () => {
      expect(get(savedIndicator)).toBe(false);
    });

    it('editText defaults to empty string', () => {
      expect(get(editText)).toBe('');
    });

    it('selfSaveInFlight defaults to false', () => {
      expect(get(selfSaveInFlight)).toBe(false);
    });

    it('showToc defaults to true', () => {
      expect(get(showToc)).toBe(true);
    });

    it('scrollRatio defaults to 0', () => {
      expect(get(scrollRatio)).toBe(0);
    });

    it('tick starts at 0', () => {
      expect(get(tick)).toBe(0);
    });

    it('toasts defaults to empty array', () => {
      expect(get(toasts)).toEqual([]);
    });

    it('shortcutHelpOpen defaults to false', () => {
      expect(get(shortcutHelpOpen)).toBe(false);
    });

    it('findBarOpen defaults to false', () => {
      expect(get(findBarOpen)).toBe(false);
    });

    it('splitPath defaults to null', () => {
      expect(get(splitPath)).toBeNull();
    });

    it('splitContent defaults to empty string', () => {
      expect(get(splitContent)).toBe('');
    });

    it('splitDiff defaults to null', () => {
      expect(get(splitDiff)).toBeNull();
    });

    it('activeSplit defaults to "left"', () => {
      expect(get(activeSplit)).toBe('left');
    });
  });

  // ── Writable store set/get ─────────────────────────────────────────

  describe('writable stores can be set and read', () => {
    resetStore(currentSection, 'inbox');
    resetStore(sectionItems, []);
    resetStore(selectedIndex, 0);
    resetStore(activeFilePath, null);
    resetStore(openTabs, []);
    resetStore(activeTabIndex, 0);
    resetStore(editMode, false);
    resetStore(showDiff, false);
    resetStore(commandPaletteOpen, false);
    resetStore(reminderPickerOpen, false);
    resetStore(sidebarVisible, true);
    resetStore(fileContent, '');
    resetStore(fileDiff, null);
    resetStore(settingsOpen, false);
    resetStore(alwaysOnTop, false);
    resetStore(savedIndicator, false);
    resetStore(editText, '');
    resetStore(selfSaveInFlight, false);
    resetStore(showToc, true);
    resetStore(scrollRatio, 0);
    resetStore(toasts, []);
    resetStore(shortcutHelpOpen, false);
    resetStore(findBarOpen, false);
    resetStore(splitPath, null);
    resetStore(splitContent, '');
    resetStore(splitDiff, null);
    resetStore(activeSplit, 'left');

    it('currentSection can be set to each section', () => {
      for (const s of ['inbox', 'pinned', 'reminders', 'archive'] as Section[]) {
        currentSection.set(s);
        expect(get(currentSection)).toBe(s);
      }
    });

    it('sectionItems can be set with InboxItem array', () => {
      const items: InboxItem[] = [
        { path: '/a.md', filename: 'a.md', status: 'unread', pinned: false, reminder_time: null, last_modified: null, additions: 0, deletions: 0 },
        { path: '/b.md', filename: 'b.md', status: 'read', pinned: true, reminder_time: '2025-01-01', last_modified: '2025-01-01', additions: 1, deletions: 2 },
      ];
      sectionItems.set(items);
      expect(get(sectionItems)).toEqual(items);
      expect(get(sectionItems)).toHaveLength(2);
    });

    it('selectedIndex can be set', () => {
      selectedIndex.set(5);
      expect(get(selectedIndex)).toBe(5);
    });

    it('activeFilePath can be set to a string or null', () => {
      activeFilePath.set('/tmp/test.md');
      expect(get(activeFilePath)).toBe('/tmp/test.md');
      activeFilePath.set(null);
      expect(get(activeFilePath)).toBeNull();
    });

    it('openTabs can be set with OpenTab array', () => {
      const tabs: OpenTab[] = [
        { path: '/a.md', filename: 'a.md', additions: 1, deletions: 0 },
      ];
      openTabs.set(tabs);
      expect(get(openTabs)).toEqual(tabs);
    });

    it('activeTabIndex can be set', () => {
      activeTabIndex.set(3);
      expect(get(activeTabIndex)).toBe(3);
    });

    it('editMode can be toggled', () => {
      editMode.set(true);
      expect(get(editMode)).toBe(true);
      editMode.set(false);
      expect(get(editMode)).toBe(false);
    });

    it('showDiff can be toggled', () => {
      showDiff.set(true);
      expect(get(showDiff)).toBe(true);
    });

    it('commandPaletteOpen can be toggled', () => {
      commandPaletteOpen.set(true);
      expect(get(commandPaletteOpen)).toBe(true);
    });

    it('reminderPickerOpen can be toggled', () => {
      reminderPickerOpen.set(true);
      expect(get(reminderPickerOpen)).toBe(true);
    });

    it('sidebarVisible can be toggled', () => {
      sidebarVisible.set(false);
      expect(get(sidebarVisible)).toBe(false);
    });

    it('fileContent can be set', () => {
      fileContent.set('# Hello World');
      expect(get(fileContent)).toBe('# Hello World');
    });

    it('fileDiff can be set to a DiffResult', () => {
      const diff: DiffResult = {
        content: 'diff output',
        hunks: [{ old_start: 1, new_start: 1, new_lines: 2, content: '+new', change_type: 'add' }],
        additions: 1,
        deletions: 0,
      };
      fileDiff.set(diff);
      expect(get(fileDiff)).toEqual(diff);
    });

    it('fileDiff can be set to null', () => {
      fileDiff.set(null);
      expect(get(fileDiff)).toBeNull();
    });

    it('settingsOpen can be toggled', () => {
      settingsOpen.set(true);
      expect(get(settingsOpen)).toBe(true);
    });

    it('alwaysOnTop can be toggled', () => {
      alwaysOnTop.set(true);
      expect(get(alwaysOnTop)).toBe(true);
    });

    it('savedIndicator can be toggled', () => {
      savedIndicator.set(true);
      expect(get(savedIndicator)).toBe(true);
    });

    it('editText can be set', () => {
      editText.set('some markdown text');
      expect(get(editText)).toBe('some markdown text');
    });

    it('selfSaveInFlight can be toggled', () => {
      selfSaveInFlight.set(true);
      expect(get(selfSaveInFlight)).toBe(true);
    });

    it('showToc can be toggled', () => {
      showToc.set(false);
      expect(get(showToc)).toBe(false);
    });

    it('scrollRatio can be set to values between 0 and 1', () => {
      scrollRatio.set(0.5);
      expect(get(scrollRatio)).toBe(0.5);
      scrollRatio.set(1);
      expect(get(scrollRatio)).toBe(1);
    });

    it('toasts can be set with ToastItem array', () => {
      const items: ToastItem[] = [
        { id: 1, message: 'done', undoAction: null, timestamp: 100 },
      ];
      toasts.set(items);
      expect(get(toasts)).toEqual(items);
      expect(get(toasts)).toHaveLength(1);
    });

    it('shortcutHelpOpen can be toggled', () => {
      shortcutHelpOpen.set(true);
      expect(get(shortcutHelpOpen)).toBe(true);
    });

    it('findBarOpen can be toggled', () => {
      findBarOpen.set(true);
      expect(get(findBarOpen)).toBe(true);
    });

    it('splitPath can be set to a string or null', () => {
      splitPath.set('/tmp/split.md');
      expect(get(splitPath)).toBe('/tmp/split.md');
      splitPath.set(null);
      expect(get(splitPath)).toBeNull();
    });

    it('splitContent can be set', () => {
      splitContent.set('split pane content');
      expect(get(splitContent)).toBe('split pane content');
    });

    it('splitDiff can be set to DiffResult or null', () => {
      const diff: DiffResult = { content: 'x', hunks: [], additions: 0, deletions: 0 };
      splitDiff.set(diff);
      expect(get(splitDiff)).toEqual(diff);
      splitDiff.set(null);
      expect(get(splitDiff)).toBeNull();
    });

    it('activeSplit can be set to left or right', () => {
      activeSplit.set('right');
      expect(get(activeSplit)).toBe('right');
      activeSplit.set('left');
      expect(get(activeSplit)).toBe('left');
    });
  });

  // ── Writable store update() ────────────────────────────────────────

  describe('writable stores support update()', () => {
    resetStore(selectedIndex, 0);
    resetStore(sectionItems, []);
    resetStore(toasts, []);

    it('selectedIndex can be incremented via update', () => {
      selectedIndex.set(0);
      selectedIndex.update((n) => n + 1);
      expect(get(selectedIndex)).toBe(1);
    });

    it('sectionItems can be appended via update', () => {
      sectionItems.set([]);
      const newItem: InboxItem = {
        path: '/c.md', filename: 'c.md', status: 'unread', pinned: false,
        reminder_time: null, last_modified: null, additions: 0, deletions: 0,
      };
      sectionItems.update((items) => [...items, newItem]);
      expect(get(sectionItems)).toHaveLength(1);
      expect(get(sectionItems)[0].filename).toBe('c.md');
    });

    it('toasts can be filtered via update', () => {
      toasts.set([
        { id: 1, message: 'a', undoAction: null, timestamp: 0 },
        { id: 2, message: 'b', undoAction: null, timestamp: 0 },
      ]);
      toasts.update((t) => t.filter((item) => item.id !== 1));
      expect(get(toasts)).toHaveLength(1);
      expect(get(toasts)[0].id).toBe(2);
    });
  });

  // ── tick readable store ────────────────────────────────────────────

  describe('tick readable store', () => {
    it('starts at 0', () => {
      expect(get(tick)).toBe(0);
    });

    it('increments after 60 seconds', () => {
      vi.useFakeTimers();
      try {
        // Subscribe to activate the store's start function
        const values: number[] = [];
        const unsub = tick.subscribe((v) => values.push(v));

        // Advance 60s
        vi.advanceTimersByTime(60_000);
        expect(get(tick)).toBe(1);

        // Advance another 60s
        vi.advanceTimersByTime(60_000);
        expect(get(tick)).toBe(2);

        // Advance 3 more intervals
        vi.advanceTimersByTime(180_000);
        expect(get(tick)).toBe(5);

        unsub();
      } finally {
        vi.useRealTimers();
      }
    });

    it('does not increment before 60 seconds elapse within an interval', () => {
      vi.useFakeTimers();
      try {
        const unsub = tick.subscribe(() => {});
        const before = get(tick);
        vi.advanceTimersByTime(59_999);
        expect(get(tick)).toBe(before);
        unsub();
      } finally {
        vi.useRealTimers();
      }
    });

    it('stops incrementing after all subscribers unsubscribe (cleanup)', () => {
      vi.useFakeTimers();
      try {
        const values: number[] = [];
        const unsub = tick.subscribe((v) => values.push(v));

        vi.advanceTimersByTime(60_000);
        vi.advanceTimersByTime(60_000);
        const countBeforeUnsub = values.length;

        unsub();

        // After unsubscribing, advancing time should not push new values
        vi.advanceTimersByTime(180_000);
        expect(values.length).toBe(countBeforeUnsub);
      } finally {
        vi.useRealTimers();
      }
    });
  });

  // ── Subscribe mechanism ────────────────────────────────────────────

  describe('store subscriptions', () => {
    resetStore(editMode, false);

    it('subscribe is called with current value immediately', () => {
      const values: boolean[] = [];
      const unsub = editMode.subscribe((v) => values.push(v));
      expect(values).toEqual([false]);
      unsub();
    });

    it('subscribe is called on each set', () => {
      const values: boolean[] = [];
      const unsub = editMode.subscribe((v) => values.push(v));
      editMode.set(true);
      editMode.set(false);
      editMode.set(true);
      expect(values).toEqual([false, true, false, true]);
      unsub();
    });
  });
});
