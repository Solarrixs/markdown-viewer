<script lang="ts">
  import { get } from 'svelte/store';
  import { shortcutMap, chordMap, featureContext, executeFeature } from './registry';
  import {
    selectedIndex, sectionItems, openTabs, activeTabIndex,
    editMode, commandPaletteOpen, reminderPickerOpen, settingsOpen, shortcutHelpOpen,
  } from './stores';
  import { openFile, switchTab } from './actions';

  let pendingChord = false;
  let pendingChordKey = '';
  let chordTimeout: ReturnType<typeof setTimeout>;

  async function openSelected() {
    const item = $sectionItems[$selectedIndex];
    if (!item) return;
    await openFile(item);
  }

  function handleKeydown(e: KeyboardEvent) {
    const meta = e.metaKey || e.ctrlKey;
    const ctx = get(featureContext);

    // 1. Cmd+1-9 tab switching (parametric, stays hardcoded)
    if (meta && e.key >= '1' && e.key <= '9') {
      e.preventDefault();
      const idx = parseInt(e.key) - 1;
      if (idx < $openTabs.length && idx !== $activeTabIndex) {
        switchTab(idx);
      }
      return;
    }

    // 2. Global shortcuts from registry (fire even during edit/modal)
    if (meta) {
      let shortcutKey = 'cmd+' + e.key.toLowerCase();
      if (e.shiftKey) shortcutKey = 'cmd+shift+' + e.key.toLowerCase();
      const feature = $shortcutMap.get(shortcutKey);
      if (feature && feature.shortcut?.scope === 'global') {
        e.preventDefault();
        executeFeature(feature, ctx);
        return;
      }
    }

    // 3. Guard: suppress navigation shortcuts during edit/modal
    if ($editMode || $commandPaletteOpen || $reminderPickerOpen || $settingsOpen || $shortcutHelpOpen) return;

    // 4. Chord handling
    if (pendingChord) {
      pendingChord = false;
      clearTimeout(chordTimeout);
      const chordFeatures = $chordMap.get(pendingChordKey);
      if (chordFeatures) {
        const feature = chordFeatures.get(e.key.toLowerCase());
        if (feature) {
          e.preventDefault();
          executeFeature(feature, ctx);
        }
      }
      return;
    }

    // 5. Check for chord prefix keys
    for (const prefix of $chordMap.keys()) {
      if (e.key.toLowerCase() === prefix) {
        pendingChord = true;
        pendingChordKey = prefix;
        chordTimeout = setTimeout(() => { pendingChord = false; }, 500);
        return;
      }
    }

    // 6. Hardcoded navigation (j/k/Enter — not features)
    switch (e.key.toLowerCase()) {
      case 'j':
        e.preventDefault();
        selectedIndex.update(i => Math.min(i + 1, $sectionItems.length - 1));
        return;
      case 'k':
        e.preventDefault();
        selectedIndex.update(i => Math.max(i - 1, 0));
        return;
      case 'enter':
        e.preventDefault();
        openSelected();
        return;
    }

    // 7. Navigation shortcuts from registry
    const feature = $shortcutMap.get(e.key.toLowerCase());
    if (feature && feature.shortcut?.scope === 'navigation') {
      e.preventDefault();
      executeFeature(feature, ctx);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<slot />
