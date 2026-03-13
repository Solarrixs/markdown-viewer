<script lang="ts">
  import { get } from 'svelte/store';
  import {
    editMode, showDiff, showToc, commandPaletteOpen, reminderPickerOpen,
    sidebarVisible, selectedIndex, sectionItems, activeFilePath,
    openTabs, activeTabIndex, settingsOpen, shortcutHelpOpen, toasts, findBarOpen,
    renameTrigger,
  } from './stores';
  import { openFile, switchSection, archiveFile, togglePin, closeActiveTab, switchTab, saveIfDirty, openFileDialog, openInFinder, openInTerminal, copyPath, reopenLastClosedTab, dismissToast, openInSplit } from './actions';

  let pendingChord = false;
  let chordTimeout: ReturnType<typeof setTimeout>;

  async function openSelected() {
    const item = $sectionItems[$selectedIndex];
    if (!item) return;
    await openFile(item);
  }

  function handleKeydown(e: KeyboardEvent) {
    const meta = e.metaKey || e.ctrlKey;

    if (meta && e.key === 'o') {
      e.preventDefault();
      openFileDialog();
      return;
    }
    if (meta && e.key === 'k') {
      e.preventDefault();
      commandPaletteOpen.set(true);
      return;
    }
    if (meta && e.key === ',') {
      e.preventDefault();
      settingsOpen.set(true);
      return;
    }
    if (meta && e.key === 'Enter') {
      e.preventDefault();
      const item = $sectionItems[$selectedIndex];
      if (item) openInSplit(item);
      return;
    }
    if (meta && e.key === 'e') {
      e.preventDefault();
      if ($activeFilePath) {
        if ($editMode) {
          saveIfDirty().then(() => editMode.set(false));
        } else {
          editMode.set(true);
        }
      }
      return;
    }
    if (meta && e.key === 'd') {
      e.preventDefault();
      if ($activeFilePath && !$editMode) showDiff.update(v => !v);
      return;
    }
    if (meta && e.key === '\\') {
      e.preventDefault();
      sidebarVisible.update(v => !v);
      return;
    }
    if (meta && e.shiftKey && e.key === 't') {
      e.preventDefault();
      reopenLastClosedTab();
      return;
    }
    if (meta && e.key === 'w') {
      e.preventDefault();
      closeActiveTab();
      return;
    }

    if (meta && e.key === 'f') {
      if (!$editMode && $activeFilePath) {
        e.preventDefault();
        findBarOpen.set(true);
      }
      return;
    }

    if (meta && e.key >= '1' && e.key <= '9') {
      e.preventDefault();
      const idx = parseInt(e.key) - 1;
      if (idx < $openTabs.length && idx !== $activeTabIndex) {
        switchTab(idx);
      }
      return;
    }

    if ($editMode || $commandPaletteOpen || $reminderPickerOpen || $settingsOpen || $shortcutHelpOpen) return;

    if (pendingChord) {
      pendingChord = false;
      clearTimeout(chordTimeout);
      switch (e.key.toLowerCase()) {
        case 'i': switchSection('inbox'); break;
        case 'p': switchSection('pinned'); break;
        case 'r': switchSection('reminders'); break;
        case 'a': switchSection('archive'); break;
      }
      return;
    }

    switch (e.key.toLowerCase()) {
      case 'j':
        e.preventDefault();
        selectedIndex.update(i => Math.min(i + 1, $sectionItems.length - 1));
        break;
      case 'k':
        e.preventDefault();
        selectedIndex.update(i => Math.max(i - 1, 0));
        break;
      case 'enter':
        e.preventDefault();
        openSelected();
        break;
      case 'e':
        e.preventDefault();
        archiveFile();
        break;
      case 'p':
        e.preventDefault();
        togglePin();
        break;
      case 'h':
        e.preventDefault();
        if ($activeFilePath) reminderPickerOpen.set(true);
        break;
      case 'f':
        e.preventDefault();
        openInFinder();
        break;
      case 't':
        e.preventDefault();
        openInTerminal();
        break;
      case 'c':
        e.preventDefault();
        copyPath();
        break;
      case 'r':
        e.preventDefault();
        if ($activeFilePath) renameTrigger.update(n => n + 1);
        break;
      case 'o':
        e.preventDefault();
        if ($activeFilePath && !$editMode && !$showDiff) showToc.update(v => !v);
        break;
      case 'g':
        pendingChord = true;
        chordTimeout = setTimeout(() => { pendingChord = false; }, 500);
        break;
      case '?':
        e.preventDefault();
        shortcutHelpOpen.update(v => !v);
        break;
      case 'z':
        e.preventDefault();
        {
          const t = get(toasts);
          const last = [...t].reverse().find(toast => toast.undoAction);
          if (last) {
            last.undoAction!();
            dismissToast(last.id);
          }
        }
        break;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<slot />
