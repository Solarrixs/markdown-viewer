import { derived, get, readable } from 'svelte/store';
import type { Readable } from 'svelte/store';
import {
  activeFilePath, sectionItems, currentSection, editMode, commandPaletteOpen,
  reminderPickerOpen, settingsOpen, shortcutHelpOpen, showDiff, showToc,
  sidebarVisible, renameTrigger, bulkOpenModalOpen, findBarOpen, toasts,
  selectedIndex,
} from './stores';
import type { InboxItem, Section } from './stores';
import {
  togglePin, openInFinder, openInTerminal, copyPath, deleteFile, archiveFile,
  openFileDialog, saveIfDirty, reopenLastClosedTab, closeActiveTab,
  openInSplit, closeSplit, toggleAlwaysOnTop, dismissToast,
  switchSection,
} from './actions';

// ---------------------------------------------------------------------------
// Core Types
// ---------------------------------------------------------------------------

export interface FeatureDef {
  id: string;
  label: string | ((ctx: FeatureContext) => string);
  icon: string;

  toolbar?: {
    group: ToolbarGroup;
    order: number;
    side?: 'left' | 'right';
  };
  commandPalette?: {
    category?: string;
  };
  shortcut?: {
    key: string;
    chord?: string;
    scope: 'global' | 'navigation';
  };

  action: (ctx: FeatureContext) => void | Promise<void>;

  when?: (ctx: FeatureContext) => boolean;
  active?: (ctx: FeatureContext) => boolean;
}

export interface FeatureContext {
  currentItem: InboxItem | null;
  currentSection: Section;
  activeFilePath: string | null;
  isEditing: boolean;
  commandPaletteOpen: boolean;
}

export type ToolbarEntry = ResolvedFeature | { separator: true };

export interface ResolvedFeature {
  id: string;
  resolvedLabel: string;
  icon: string;
  isActive: boolean;
  shortcutHint: string | null;
  toolbar?: FeatureDef['toolbar'];
  commandPalette?: FeatureDef['commandPalette'];
  def: FeatureDef;
}

// ---------------------------------------------------------------------------
// Section Types
// ---------------------------------------------------------------------------

export interface SectionDef {
  id: Section;
  label: string;
  order: number;
  chord?: string;
}

// ---------------------------------------------------------------------------
// Toolbar Groups (explicit ordering)
// ---------------------------------------------------------------------------

type ToolbarGroup = 'note-actions' | 'external' | 'view';

const TOOLBAR_GROUP_ORDER: Record<ToolbarGroup, number> = {
  'note-actions': 0,
  'external': 1,
  'view': 2,
};

// ---------------------------------------------------------------------------
// Common Predicates
// ---------------------------------------------------------------------------

const whenFileOpen = (ctx: FeatureContext) => ctx.activeFilePath !== null;
const whenFileOpenNotEditing = (ctx: FeatureContext) => ctx.activeFilePath !== null && !ctx.isEditing;

// ---------------------------------------------------------------------------
// Section Definitions (defined first — features are generated from these)
// ---------------------------------------------------------------------------

export const sections: SectionDef[] = [
  { id: 'inbox', label: 'Inbox', order: 1, chord: 'i' },
  { id: 'pinned', label: 'Pinned', order: 2, chord: 'p' },
  { id: 'reminders', label: 'Reminders', order: 3, chord: 'r' },
  { id: 'archive', label: 'Archive', order: 4, chord: 'a' },
];

const SECTION_ICONS: Record<Section, string> = {
  inbox: '📥',
  pinned: '📌',
  reminders: '⏰',
  archive: '🗃️',
};

// Generate go-to features from sections to avoid duplication
const sectionFeatures: FeatureDef[] = sections.map(s => ({
  id: `go-to-${s.id}`,
  label: `Go to ${s.label}`,
  icon: SECTION_ICONS[s.id],
  commandPalette: { category: 'Navigation' },
  ...(s.chord ? { shortcut: { key: s.chord, chord: 'g', scope: 'navigation' as const } } : {}),
  action: () => switchSection(s.id),
}));

// ---------------------------------------------------------------------------
// Feature Definitions
// ---------------------------------------------------------------------------

export const features: FeatureDef[] = [
  // --- Toolbar: note-actions group ---
  {
    id: 'pin',
    label: (ctx) => ctx.currentItem?.pinned ? 'Unpin' : 'Pin',
    icon: '📌',
    toolbar: { group: 'note-actions', order: 1 },
    commandPalette: { category: 'Note' },
    shortcut: { key: 'p', scope: 'navigation' },
    action: () => togglePin(),
    active: (ctx) => ctx.currentItem?.pinned ?? false,
    when: whenFileOpen,
  },
  {
    id: 'rename',
    label: 'Rename',
    icon: '✏️',
    toolbar: { group: 'note-actions', order: 2 },
    commandPalette: { category: 'Note' },
    shortcut: { key: 'r', scope: 'navigation' },
    action: () => renameTrigger.update(n => n + 1),
    when: whenFileOpen,
  },
  {
    id: 'remind',
    label: 'Remind',
    icon: '⏰',
    toolbar: { group: 'note-actions', order: 3 },
    commandPalette: { category: 'Note' },
    shortcut: { key: 'h', scope: 'navigation' },
    action: () => reminderPickerOpen.set(true),
    when: whenFileOpen,
  },
  {
    id: 'archive',
    label: 'Archive',
    icon: '📥',
    commandPalette: { category: 'Note' },
    shortcut: { key: 'e', scope: 'navigation' },
    action: () => archiveFile(),
    when: whenFileOpen,
  },
  {
    id: 'delete',
    label: 'Delete',
    icon: '🗑️',
    commandPalette: { category: 'Note' },
    shortcut: { key: 'd', scope: 'navigation' },
    action: () => deleteFile(),
    when: whenFileOpen,
  },

  // --- Toolbar: external group ---
  {
    id: 'open-in-finder',
    label: 'Finder',
    icon: '📁',
    toolbar: { group: 'external', order: 1 },
    commandPalette: { category: 'File' },
    shortcut: { key: 'f', scope: 'navigation' },
    action: () => openInFinder(),
    when: whenFileOpen,
  },
  {
    id: 'open-in-terminal',
    label: 'Terminal',
    icon: '▶',
    toolbar: { group: 'external', order: 2 },
    commandPalette: { category: 'File' },
    shortcut: { key: 't', scope: 'navigation' },
    action: () => openInTerminal(),
    when: whenFileOpen,
  },
  {
    id: 'copy-path',
    label: 'Copy Path',
    icon: '📋',
    toolbar: { group: 'external', order: 3 },
    commandPalette: { category: 'File' },
    shortcut: { key: 'c', chord: 'g', scope: 'navigation' },
    action: () => copyPath(),
    when: whenFileOpen,
  },

  // --- Toolbar: view group ---
  {
    id: 'toc',
    label: 'TOC',
    icon: '☰',
    toolbar: { group: 'view', order: 1 },
    commandPalette: { category: 'View' },
    action: () => showToc.update(v => !v),
    active: () => get(showToc),
    when: whenFileOpenNotEditing,
  },

  // --- Command palette + shortcut only (no toolbar) ---
  {
    id: 'open-file',
    label: 'Open File...',
    icon: '📂',
    commandPalette: { category: 'File' },
    shortcut: { key: 'Cmd+o', scope: 'global' },
    action: () => openFileDialog(),
  },
  {
    id: 'open-multiple',
    label: 'Open Multiple Files...',
    icon: '📂',
    commandPalette: { category: 'File' },
    action: () => bulkOpenModalOpen.set(true),
  },
  {
    id: 'toggle-edit',
    label: (ctx) => ctx.isEditing ? 'Exit Edit Mode' : 'Edit Mode',
    icon: '✍️',
    commandPalette: { category: 'View' },
    shortcut: { key: 'Cmd+e', scope: 'global' },
    action: async (ctx) => {
      if (ctx.isEditing) {
        await saveIfDirty();
        editMode.set(false);
      } else {
        editMode.set(true);
      }
    },
    when: whenFileOpen,
  },
  {
    id: 'toggle-diff',
    label: 'Toggle Diff View',
    icon: '±',
    commandPalette: { category: 'View' },
    shortcut: { key: 'Cmd+d', scope: 'global' },
    action: () => showDiff.update(v => !v),
    when: whenFileOpenNotEditing,
  },
  {
    id: 'toggle-sidebar',
    label: 'Toggle Sidebar',
    icon: '◧',
    commandPalette: { category: 'View' },
    shortcut: { key: 'Cmd+\\', scope: 'global' },
    action: () => sidebarVisible.update(v => !v),
  },
  {
    id: 'toggle-always-on-top',
    label: 'Toggle Always on Top',
    icon: '📌',
    commandPalette: { category: 'View' },
    action: () => toggleAlwaysOnTop(),
  },
  {
    id: 'open-settings',
    label: 'Open Settings',
    icon: '⚙️',
    commandPalette: { category: 'App' },
    shortcut: { key: 'Cmd+,', scope: 'global' },
    action: () => settingsOpen.set(true),
  },
  {
    id: 'command-palette',
    label: 'Command Palette',
    icon: '🔍',
    shortcut: { key: 'Cmd+k', scope: 'global' },
    action: () => commandPaletteOpen.set(true),
  },
  {
    id: 'close-tab',
    label: 'Close Tab',
    icon: '✕',
    commandPalette: { category: 'Tab' },
    shortcut: { key: 'Cmd+w', scope: 'global' },
    action: () => closeActiveTab(),
  },
  {
    id: 'reopen-tab',
    label: 'Reopen Last Closed Tab',
    icon: '↩️',
    commandPalette: { category: 'Tab' },
    shortcut: { key: 'Cmd+Shift+t', scope: 'global' },
    action: () => reopenLastClosedTab(),
  },
  {
    id: 'find',
    label: 'Find in File',
    icon: '🔍',
    shortcut: { key: 'Cmd+f', scope: 'global' },
    action: () => findBarOpen.set(true),
    when: whenFileOpenNotEditing,
  },
  {
    id: 'split-open',
    label: 'Split View: Open Right',
    icon: '⊞',
    commandPalette: { category: 'View' },
    shortcut: { key: 'Cmd+Enter', scope: 'global' },
    action: () => {
      const items = get(sectionItems);
      const idx = get(selectedIndex);
      if (items[idx]) openInSplit(items[idx]);
    },
  },
  {
    id: 'split-close',
    label: 'Split View: Close',
    icon: '⊟',
    commandPalette: { category: 'View' },
    action: () => closeSplit(),
  },

  // --- Shortcut-only (no toolbar, no palette) ---
  {
    id: 'shortcut-help',
    label: 'Keyboard Shortcuts',
    icon: '⌨️',
    shortcut: { key: '?', scope: 'navigation' },
    action: () => shortcutHelpOpen.update(v => !v),
  },
  {
    id: 'undo-last',
    label: 'Undo',
    icon: '↩️',
    shortcut: { key: 'z', scope: 'navigation' },
    action: () => {
      const t = get(toasts);
      const last = [...t].reverse().find(toast => toast.undoAction);
      if (last) {
        last.undoAction!();
        dismissToast(last.id);
      }
    },
  },

  // --- Section navigation (generated from sections array) ---
  ...sectionFeatures,
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatShortcutHint(shortcut: FeatureDef['shortcut']): string {
  if (!shortcut) return '';
  if (shortcut.chord) {
    return `${shortcut.chord.toUpperCase()} ${shortcut.key.toUpperCase()}`;
  }
  return shortcut.key
    .replace(/Cmd\+/i, '⌘')
    .replace(/Shift\+/i, '⇧')
    .replace(/Backspace/i, '⌫')
    .replace(/Enter/i, '↩')
    .toUpperCase();
}

function resolveFeature(f: FeatureDef, ctx: FeatureContext): ResolvedFeature {
  return {
    id: f.id,
    resolvedLabel: typeof f.label === 'function' ? f.label(ctx) : f.label,
    icon: f.icon,
    isActive: f.active ? f.active(ctx) : false,
    shortcutHint: f.shortcut ? formatShortcutHint(f.shortcut) : null,
    toolbar: f.toolbar,
    commandPalette: f.commandPalette,
    def: f,
  };
}

function normalizeShortcut(shortcut: NonNullable<FeatureDef['shortcut']>): string {
  if (shortcut.chord) return `chord:${shortcut.chord}+${shortcut.key}`.toLowerCase();
  return shortcut.key.toLowerCase();
}

// ---------------------------------------------------------------------------
// Reactive Context
// ---------------------------------------------------------------------------

export const featureContext: Readable<FeatureContext> = derived(
  [activeFilePath, sectionItems, currentSection, editMode, commandPaletteOpen],
  ([$path, $items, $section, $editing, $paletteOpen]) => ({
    currentItem: $items.find(i => i.path === $path) ?? null,
    currentSection: $section,
    activeFilePath: $path,
    isEditing: $editing,
    commandPaletteOpen: $paletteOpen,
  })
);

// ---------------------------------------------------------------------------
// Derived Stores for Consumers
// ---------------------------------------------------------------------------

export const toolbarFeatures: Readable<ToolbarEntry[]> = derived(
  featureContext,
  (ctx) => {
    const visible = features
      .filter(f => f.toolbar && (!f.when || f.when(ctx)))
      .map(f => resolveFeature(f, ctx))
      .sort((a, b) => {
        const ga = TOOLBAR_GROUP_ORDER[a.toolbar!.group];
        const gb = TOOLBAR_GROUP_ORDER[b.toolbar!.group];
        if (ga !== gb) return ga - gb;
        return a.toolbar!.order - b.toolbar!.order;
      });

    // Inject separators between groups
    const result: ToolbarEntry[] = [];
    let lastGroup: string | null = null;
    for (const f of visible) {
      if (lastGroup !== null && f.toolbar!.group !== lastGroup) {
        result.push({ separator: true });
      }
      result.push(f);
      lastGroup = f.toolbar!.group;
    }
    return result;
  }
);

export const paletteFeatures: Readable<ResolvedFeature[]> = derived(
  featureContext,
  (ctx) => features
    .filter(f => f.commandPalette && (!f.when || f.when(ctx)))
    .map(f => resolveFeature(f, ctx))
);

// Shortcut and chord maps include ALL features (no `when` filtering).
// `when` is for UI visibility (toolbar/palette); shortcuts should always be
// available so features can toggle themselves (e.g., Cmd+E to exit edit mode).
// These are static since the features array doesn't change at runtime.

function buildShortcutMap(): Map<string, FeatureDef> {
  const map = new Map<string, FeatureDef>();
  for (const f of features) {
    if (f.shortcut && !f.shortcut.chord) {
      map.set(normalizeShortcut(f.shortcut), f);
    }
  }
  return map;
}

function buildChordMap(): Map<string, Map<string, FeatureDef>> {
  const map = new Map<string, Map<string, FeatureDef>>();
  for (const f of features) {
    if (f.shortcut?.chord) {
      const prefix = f.shortcut.chord.toLowerCase();
      if (!map.has(prefix)) map.set(prefix, new Map());
      map.get(prefix)!.set(f.shortcut.key.toLowerCase(), f);
    }
  }
  return map;
}

const _shortcutMap = buildShortcutMap();
const _chordMap = buildChordMap();

export const shortcutMap = readable(_shortcutMap);
export const chordMap = readable(_chordMap);

// ---------------------------------------------------------------------------
// Action Execution
// ---------------------------------------------------------------------------

export function executeFeature(feature: ResolvedFeature | FeatureDef, ctx: FeatureContext) {
  const def = 'def' in feature ? feature.def : feature;
  def.action(ctx);
}
