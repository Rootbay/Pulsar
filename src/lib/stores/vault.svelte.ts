import { settingsManager } from './appSettings.svelte';
import { defaultVaultSettings, type VaultSettings } from '../config/settings';
import type { PasswordItem } from '../types/password';
import { callBackend } from '../utils/backend';
import { appState } from './appState.svelte';

class VaultStore {
  #items = $state<PasswordItem[]>([]);
  #isLoading = $state(false);
  #activeVaultId = $state<string | null>(null);
  #searchTerm = $state('');
  #loadPromise: Promise<void> | null = null;

  get items() {
    return this.#items;
  }

  get isLoading() {
    return this.#isLoading;
  }

  get activeVaultId() {
    return this.#activeVaultId;
  }

  get searchTerm() {
    return this.#searchTerm;
  }

  set searchTerm(value: string) {
    this.#searchTerm = value;
  }

  get settings(): VaultSettings {
    const id = this.#activeVaultId;
    if (!id) return { ...defaultVaultSettings };
    const existing = settingsManager.current.vaultSettingsById[id];
    return { ...defaultVaultSettings, ...existing };
  }

  /**
   * Filtered and searched items based on appState and searchTerm.
   */
  filteredItems = $derived.by(() => {
    const normalizedSearch = this.#searchTerm.trim().toLowerCase();
    const tag = appState.selectedTag;
    const category = appState.filterCategory;
    const RECENT_DAY_WINDOW = 7;
    const DAY_IN_MS = 24 * 60 * 60 * 1000;
    const PIN_TAG_NAMES = ['pinned', 'pin'];

    return this.#items.filter((item) => {
      // 1. Tag filtering
      if (tag !== null) {
        const itemTags = item.tags?.split(',').map(t => t.trim().toLowerCase()) ?? [];
        if (!itemTags.includes(tag.toLowerCase())) return false;
      }

      // 2. Category/Recent filtering
      if (category === 'recent') {
        const itemTags = item.tags?.split(',').map(t => t.trim().toLowerCase()) ?? [];
        const isPinned = itemTags.some(t => PIN_TAG_NAMES.includes(t));
        
        if (!isPinned) {
          const updatedAt = item.updated_at ? new Date(item.updated_at).getTime() : 0;
          const now = Date.now();
          if (now - updatedAt > RECENT_DAY_WINDOW * DAY_IN_MS) return false;
        }
      }

      // 3. Search filtering
      if (normalizedSearch) {
        const title = item.title?.toLowerCase() ?? '';
        const username = item.username?.toLowerCase() ?? '';
        const tags = item.tags?.toLowerCase() ?? '';
        const url = item.url?.toLowerCase() ?? '';
        
        return (
          title.includes(normalizedSearch) ||
          username.includes(normalizedSearch) ||
          tags.includes(normalizedSearch) ||
          url.includes(normalizedSearch)
        );
      }

      return true;
    });
  });

  async loadItems() {
    if (appState.isLocked) {
      this.#items = [];
      return;
    }

    if (this.#loadPromise) return this.#loadPromise;

    this.#isLoading = true;
    this.#loadPromise = (async () => {
      try {
        this.#items = await callBackend<PasswordItem[]>('get_password_items');
      } catch (error) {
        console.error('Failed to load vault items:', error);
      } finally {
        this.#isLoading = false;
        this.#loadPromise = null;
      }
    })();

    return this.#loadPromise;
  }

  selectVault(vaultId: string, defaults: Partial<VaultSettings> = {}) {
    this.#activeVaultId = vaultId;
    settingsManager.update((settings) => {
      const existing = settings.vaultSettingsById[vaultId];
      settings.vaultSettingsById = {
        ...settings.vaultSettingsById,
        [vaultId]: {
          ...defaultVaultSettings,
          ...existing,
          ...defaults
        }
      };
    });
  }

  updateSettings(updater: (settings: VaultSettings) => VaultSettings) {
    const id = this.#activeVaultId;
    if (!id) return;
    settingsManager.update((settings) => {
      const existing = settings.vaultSettingsById[id];
      settings.vaultSettingsById = {
        ...settings.vaultSettingsById,
        [id]: updater({
          ...defaultVaultSettings,
          ...existing
        })
      };
    });
  }

  clearVault(vaultId: string) {
    settingsManager.update((settings) => {
      if (!(vaultId in settings.vaultSettingsById)) return;
      const newMap = { ...settings.vaultSettingsById };
      delete newMap[vaultId];
      settings.vaultSettingsById = newMap;
    });
    if (this.#activeVaultId === vaultId) {
      this.#activeVaultId = null;
      this.#items = [];
    }
  }

  setItems(items: PasswordItem[]) {
    this.#items = items;
  }

  updateItem(updatedItem: PasswordItem) {
    const index = this.#items.findIndex(item => item.id === updatedItem.id);
    if (index !== -1) {
      this.#items[index] = updatedItem;
    }
  }

  removeItem(id: number) {
    this.#items = this.#items.filter(item => item.id !== id);
  }
}

export const vaultStore = new VaultStore();

// Legacy compatibility wrapper
export const vaultSettings = {
  subscribe(fn: (value: VaultSettings) => void) {
    return $effect.root(() => {
      $effect(() => {
        fn(vaultStore.settings);
      });
    });
  },
  get value() {
    return vaultStore.settings;
  },
  selectVault: (id: string, def?: Partial<VaultSettings>) => vaultStore.selectVault(id, def),
  update: (updater: (s: VaultSettings) => VaultSettings) => vaultStore.updateSettings(updater),
  clear: (id: string) => vaultStore.clearVault(id),
  getCurrentVaultId: () => vaultStore.activeVaultId
};
