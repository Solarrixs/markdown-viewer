import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import {
  sectionItems, currentSection, selectedIndex, activeFilePath,
  openTabs, activeTabIndex, fileContent, fileDiff, editMode, showDiff,
  alwaysOnTop, savedIndicator, editText, selfSaveInFlight, toasts,
  splitPath, splitContent, splitDiff, activeSplit,
} from './stores';
import type { InboxItem, Section, DiffResult } from './stores';

function extractFilename(path: string): string {
  return path.split('/').pop() ?? 'file';
}

let toastId = 0;
export function showToast(message: string, undoAction?: () => Promise<void>) {
  const id = ++toastId;
  toasts.update(t => [...t.slice(-2), { id, message, undoAction: undoAction ?? null, timestamp: Date.now() }]);
  setTimeout(() => dismissToast(id), 4000);
}
export function dismissToast(id: number) {
  toasts.update(t => t.filter(toast => toast.id !== id));
}

export interface OpenFileParams {
  path: string;
  filename: string;
  additions?: number;
  deletions?: number;
}

let refreshInFlight = false;
let refreshQueued = false;

/** @internal Reset concurrency guard state (for tests only) */
export function _resetRefreshState() {
  refreshInFlight = false;
  refreshQueued = false;
}

export async function refreshItems() {
  // If a refresh is already in flight, queue one more (but only one)
  if (refreshInFlight) {
    refreshQueued = true;
    return;
  }
  refreshInFlight = true;
  try {
    const items = await invoke<InboxItem[]>('get_inbox_items', { filter: get(currentSection) });
    sectionItems.set(items ?? []);
  } catch (e) {
    console.error('Failed to refresh items:', e);
  } finally {
    refreshInFlight = false;
    if (refreshQueued) {
      refreshQueued = false;
      refreshItems();
    }
  }
}

let savedIndicatorTimer: ReturnType<typeof setTimeout>;

export async function saveContent(path: string, content: string) {
  selfSaveInFlight.set(true);
  await invoke('save_file', { path, content });
  fileContent.set(content);
  savedIndicator.set(true);
  clearTimeout(savedIndicatorTimer);
  savedIndicatorTimer = setTimeout(() => savedIndicator.set(false), 1500);
  // Clear self-save flag after a short delay to let watcher event pass
  setTimeout(() => selfSaveInFlight.set(false), 500);
}

export async function saveIfDirty() {
  const path = get(activeFilePath);
  if (!path || !get(editMode)) return;
  const current = get(editText);
  if (current === get(fileContent)) return;
  try {
    await saveContent(path, current);
  } catch (e) {
    console.error('Auto-save failed:', e);
  }
}

export async function switchSection(section: Section) {
  await saveIfDirty();
  currentSection.set(section);
  selectedIndex.set(0);
  refreshItems();
}

export async function openFile(item: OpenFileParams, index?: number) {
  if (index !== undefined) {
    selectedIndex.set(index);
  }
  activeFilePath.set(item.path);
  editMode.set(false);
  showDiff.set(false);

  // Add to tabs if not already open
  const tabs = [...get(openTabs)];
  const existingIdx = tabs.findIndex(t => t.path === item.path);
  if (existingIdx === -1) {
    tabs.push({
      path: item.path,
      filename: item.filename,
      additions: item.additions ?? 0,
      deletions: item.deletions ?? 0,
    });
    openTabs.set(tabs);
    activeTabIndex.set(tabs.length - 1);
  } else {
    activeTabIndex.set(existingIdx);
  }

  // Load content immediately, mark as read in background
  await loadFileContent(item.path);
  invoke('mark_as_read', { path: item.path }).catch(e => console.error('Failed to mark as read:', e));
}

export async function openFileDialog() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Markdown', extensions: ['md', 'markdown', 'txt'] }],
  });
  if (typeof selected === 'string') {
    await openFilePath(selected);
  }
}

export async function openFilePath(path: string) {
  const result = await invoke<OpenFileParams>('ensure_file_tracked', { path });
  await openFile(result);
}

export async function archiveFile() {
  const path = get(activeFilePath);
  if (!path) return;
  const item = get(sectionItems).find(i => i.path === path);
  const filename = item?.filename ?? extractFilename(path);
  const previousStatus = item?.status ?? 'read';
  const previousReminderTime = item?.reminder_time ?? null;

  await invoke('mark_as_archived', { path });
  showToast(`Archived ${filename}`, async () => {
    await invoke('restore_file', { path, status: previousStatus, reminderTime: previousReminderTime });
    await refreshItems();
    await openFile({ path, filename });
  });
  advanceAfterRemoval(path);
}

export async function archiveByPath(path: string) {
  await closeTabByPath(path);
  await invoke('mark_as_archived', { path });
  await refreshItems();
}

export async function togglePin() {
  const path = get(activeFilePath);
  if (!path) return;
  const item = get(sectionItems).find(i => i.path === path);
  const filename = item?.filename ?? extractFilename(path);
  const newState = await invoke<boolean>('toggle_pin', { path });
  showToast(`${newState ? 'Pinned' : 'Unpinned'} ${filename}`, async () => {
    await invoke('toggle_pin', { path });
    await refreshItems();
  });
  refreshItems();
}

export async function openInFinder() {
  const path = get(activeFilePath);
  if (path) await invoke('open_in_finder', { path });
}

export async function openInTerminal() {
  const path = get(activeFilePath);
  if (path) await invoke('open_in_terminal', { path });
}

export async function copyPath() {
  const path = get(activeFilePath);
  if (path) {
    await navigator.clipboard.writeText(path);
    showToast(`Copied ${path}`);
  }
}

export async function toggleAlwaysOnTop(): Promise<boolean> {
  try {
    const newState = await invoke<boolean>('toggle_always_on_top');
    alwaysOnTop.set(newState);
    return newState;
  } catch (e) {
    console.error('Failed to toggle always-on-top:', e);
    return false;
  }
}

export async function loadFileContent(path: string) {
  // Load content immediately, don't wait for diff
  try {
    const content = await invoke<string>('get_file_content', { path });
    fileContent.set(content);
    editText.set(content);
  } catch (_) {
    // File no longer exists — close its tab and refresh
    await closeTabByPath(path);
    await refreshItems();
    return;
  }

  // Load diff in background — don't block on it
  invoke<DiffResult>('get_file_diff', { path })
    .then(diff => fileDiff.set(diff))
    .catch(() => fileDiff.set(null));
}

const recentlyClosedTabs: OpenFileParams[] = [];

function clearActiveFile() {
  activeFilePath.set(null);
  activeTabIndex.set(0);
  fileContent.set('');
  editText.set('');
  fileDiff.set(null);
  editMode.set(false);
  showDiff.set(false);
}

export async function closeTabByIndex(index: number) {
  const tabs = [...get(openTabs)];
  if (index < 0 || index >= tabs.length) return;

  await saveIfDirty();

  const closed = tabs[index];
  recentlyClosedTabs.push({ path: closed.path, filename: closed.filename });
  if (recentlyClosedTabs.length > 20) recentlyClosedTabs.shift();

  tabs.splice(index, 1);
  openTabs.set(tabs);

  if (tabs.length === 0) {
    clearActiveFile();
  } else if (get(activeTabIndex) === index || get(activeTabIndex) >= tabs.length) {
    const newIdx = Math.min(index, tabs.length - 1);
    await switchTab(newIdx);
  } else if (get(activeTabIndex) > index) {
    // Active tab shifted left
    activeTabIndex.update(i => i - 1);
  }
}

async function advanceAfterRemoval(removedPath: string) {
  await closeTabByPath(removedPath);
  await refreshItems();
  const items = get(sectionItems);
  if (items.length > 0) {
    const newIdx = Math.min(get(selectedIndex), items.length - 1);
    selectedIndex.set(newIdx);
    await openFile(items[newIdx]);
  } else {
    clearActiveFile();
  }
}

async function closeTabByPath(path: string) {
  const idx = get(openTabs).findIndex(t => t.path === path);
  if (idx !== -1) {
    await closeTabByIndex(idx);
  } else if (get(activeFilePath) === path) {
    clearActiveFile();
  }
}

export async function closeActiveTab() {
  await closeTabByIndex(get(activeTabIndex));
}

export async function reopenLastClosedTab() {
  const item = recentlyClosedTabs.pop();
  if (!item) return;
  await openFile(item);
}

export async function setReminderAndArchive(path: string, time: string) {
  const item = get(sectionItems).find(i => i.path === path);
  const filename = item?.filename ?? extractFilename(path);
  const previousStatus = item?.status ?? 'read';

  await invoke('set_reminder', { path, time });
  await invoke('mark_as_archived', { path });
  showToast(`Reminder set for ${filename}`, async () => {
    await invoke('restore_file', { path, status: previousStatus, reminderTime: null });
    await refreshItems();
    await openFile({ path, filename });
  });
  advanceAfterRemoval(path);
}

export async function switchTab(index: number) {
  await saveIfDirty();
  activeTabIndex.set(index);
  const tab = get(openTabs)[index];
  if (!tab) return;
  const pathChanged = tab.path !== get(activeFilePath);
  activeFilePath.set(tab.path);
  editMode.set(false);
  showDiff.set(false);
  if (pathChanged) {
    await loadFileContent(tab.path);
  }
}

export async function openInSplit(item: OpenFileParams) {
  splitPath.set(item.path);
  activeSplit.set('right');
  try {
    const content = await invoke<string>('get_file_content', { path: item.path });
    splitContent.set(content);
  } catch (e) {
    console.error('Failed to load split content:', e);
    return;
  }
  invoke<DiffResult>('get_file_diff', { path: item.path })
    .then(diff => splitDiff.set(diff))
    .catch(() => splitDiff.set(null));
}

export function closeSplit() {
  splitPath.set(null);
  splitContent.set('');
  splitDiff.set(null);
  activeSplit.set('left');
}
