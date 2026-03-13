import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import {
  sectionItems, currentSection, selectedIndex, activeFilePath,
  openTabs, activeTabIndex, fileContent, fileDiff, editMode, showDiff,
  alwaysOnTop, savedIndicator, editText, selfSaveInFlight,
} from './stores';
import type { InboxItem, Section, DiffResult } from './stores';

export interface OpenFileParams {
  path: string;
  filename: string;
  additions?: number;
  deletions?: number;
}

export async function refreshItems() {
  try {
    const items = await invoke<InboxItem[]>('get_inbox_items', { filter: get(currentSection) });
    sectionItems.set(items);
  } catch (e) {
    console.error('Failed to refresh items:', e);
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
  await refreshItems();
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

  // Load content and mark as read in parallel
  await Promise.all([
    loadFileContent(item.path),
    invoke('mark_as_read', { path: item.path }).catch(e => console.error('Failed to mark as read:', e)),
  ]);
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

  await closeTabByPath(path);
  await invoke('mark_as_archived', { path });
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

export async function archiveByPath(path: string) {
  await closeTabByPath(path);
  await invoke('mark_as_archived', { path });
  await refreshItems();
}

export async function togglePin() {
  const path = get(activeFilePath);
  if (!path) return;
  await invoke('toggle_pin', { path });
  await refreshItems();
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
  if (path) await navigator.clipboard.writeText(path);
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
  const [contentResult, diffResult] = await Promise.allSettled([
    invoke<string>('get_file_content', { path }),
    invoke<DiffResult>('get_file_diff', { path }),
  ]);

  if (contentResult.status === 'fulfilled') {
    fileContent.set(contentResult.value);
    editText.set(contentResult.value);
  } else {
    // File no longer exists — close its tab and refresh
    await closeTabByPath(path);
    await refreshItems();
    return;
  }
  fileDiff.set(diffResult.status === 'fulfilled' ? diffResult.value : null);
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

async function closeTabByPath(path: string) {
  await saveIfDirty();
  const idx = get(openTabs).findIndex(t => t.path === path);
  if (idx !== -1) {
    const tabs = [...get(openTabs)];
    tabs.splice(idx, 1);
    openTabs.set(tabs);
    if (tabs.length === 0) {
      clearActiveFile();
    } else if (get(activeTabIndex) >= tabs.length) {
      await switchTab(tabs.length - 1);
    } else if (get(activeTabIndex) > idx) {
      activeTabIndex.update(i => i - 1);
    }
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
  await invoke('set_reminder', { path, time });
  await invoke('mark_as_archived', { path });
  await refreshItems();
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
