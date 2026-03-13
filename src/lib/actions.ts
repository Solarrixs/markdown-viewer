import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import {
  inboxItems, currentSection, selectedIndex, activeFilePath,
  openTabs, activeTabIndex, fileContent, fileDiff, editMode, showDiff,
} from './stores';
import type { InboxItem, Section } from './stores';

export async function refreshItems() {
  try {
    const items = await invoke<InboxItem[]>('get_inbox_items', { filter: get(currentSection) });
    inboxItems.set(items);
  } catch (e) {
    console.error('Failed to refresh items:', e);
  }
}

export function switchSection(section: Section) {
  currentSection.set(section);
  selectedIndex.set(0);
  refreshItems();
}

export async function openFile(item: InboxItem, index?: number) {
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
      additions: item.additions,
      deletions: item.deletions,
    });
    openTabs.set(tabs);
    activeTabIndex.set(tabs.length - 1);
  } else {
    activeTabIndex.set(existingIdx);
  }

  // Load content and mark as read in parallel
  await Promise.all([
    loadFileContent(item.path),
    invoke('mark_as_read', { path: item.path }).catch(() => {}),
  ]);
}

export async function archiveFile() {
  const path = get(activeFilePath);
  if (!path) return;
  await invoke('mark_as_archived', { path });
  await refreshItems();
  const items = get(inboxItems);
  if (items.length > 0) {
    const newIdx = Math.min(get(selectedIndex), items.length - 1);
    selectedIndex.set(newIdx);
    await openFile(items[newIdx]);
  } else {
    activeFilePath.set(null);
  }
}

export async function togglePin() {
  const path = get(activeFilePath);
  if (!path) return;
  const item = get(inboxItems).find(i => i.path === path);
  await invoke('pin_file', { path, pinned: !item?.pinned });
  await refreshItems();
}

export async function toggleAlwaysOnTop(): Promise<boolean> {
  try {
    return await invoke<boolean>('toggle_always_on_top');
  } catch (e) {
    console.error('Failed to toggle always-on-top:', e);
    return false;
  }
}

export async function loadFileContent(path: string) {
  const [contentResult, diffResult] = await Promise.allSettled([
    invoke<string>('get_file_content', { path }),
    invoke<any>('get_file_diff', { path }),
  ]);
  fileContent.set(contentResult.status === 'fulfilled' ? contentResult.value : `Error loading file: ${(contentResult as PromiseRejectedResult).reason}`);
  fileDiff.set(diffResult.status === 'fulfilled' ? diffResult.value : null);
}

export function closeActiveTab() {
  const tabs = [...get(openTabs)];
  const idx = get(activeTabIndex);
  if (tabs.length === 0) return;
  tabs.splice(idx, 1);
  openTabs.set(tabs);

  if (tabs.length === 0) {
    activeFilePath.set(null);
    activeTabIndex.set(0);
    fileContent.set('');
    fileDiff.set(null);
  } else {
    const newIdx = Math.min(idx, tabs.length - 1);
    switchTab(newIdx);
  }
}

export async function switchTab(index: number) {
  activeTabIndex.set(index);
  const tab = get(openTabs)[index];
  if (!tab) return;
  activeFilePath.set(tab.path);
  editMode.set(false);
  showDiff.set(false);
  await loadFileContent(tab.path);
}
