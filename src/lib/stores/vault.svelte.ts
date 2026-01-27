import { settings } from './appSettings.svelte';
import { defaultVaultSettings, type VaultSettings } from '../config/settings';
import type { PasswordItem, PasswordItemOverview } from '../types/password';
import { callBackend } from '../utils/backend';
import { appState } from './appState.svelte';

class VaultStore {
  #items = $state<PasswordItemOverview[]>([]);
  #isLoading = $state(false);
  #activeVaultId = $state<string | null>(null);
  #searchTerm = $state('');
  #loadPromise: Promise<void> | null = null;

  constructor() {
    $effect.root(() => {
      $effect(() => {
        if (appState.isLocked) {
          this.#items = [];
          this.#activeVaultId = null;
          this.#searchTerm = '';
        }
      });
    });
  }

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
    const existing = settings.state.vaultSettingsById[id];
    return { ...defaultVaultSettings, ...existing };
  }

  filteredItems = $derived.by(() => {
    const normalizedSearch = this.#searchTerm.trim().toLowerCase();
    const tag = appState.selectedTag;
    const category = appState.filterCategory;
    const RECENT_DAY_WINDOW = 7;
    const DAY_IN_MS = 24 * 60 * 60 * 1000;
    const PIN_TAG_NAMES = ['pinned', 'pin'];

    return this.#items.filter((item) => {
      if (tag !== null) {
        const itemTags = item.tags?.split(',').map((t) => t.trim().toLowerCase()) ?? [];
        if (!itemTags.includes(tag.toLowerCase())) return false;
      }

      if (category === 'recent') {
        const itemTags = item.tags?.split(',').map((t) => t.trim().toLowerCase()) ?? [];
        const isPinned = itemTags.some((t) => PIN_TAG_NAMES.includes(t));

        if (!isPinned) {
          const updatedAt = item.updated_at ? new Date(item.updated_at).getTime() : 0;
          const now = Date.now();
          if (now - updatedAt > RECENT_DAY_WINDOW * DAY_IN_MS) return false;
        }
      }

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
        this.#items = await callBackend<PasswordItemOverview[]>('get_password_overviews');
      } catch (error) {
        console.error('Failed to load vault items:', error);
      } finally {
        this.#isLoading = false;
        this.#loadPromise = null;
      }
    })();

    return this.#loadPromise;
  }

  async getItemDetails(id: number): Promise<PasswordItem | null> {
    const existing = this.#items.find((i) => i.id === id);
    if (existing && 'password' in existing) {
      return existing as PasswordItem;
    }

    const fullItem = await callBackend<PasswordItem | null>('get_password_item_by_id', { id });
    if (fullItem) {
      this.updateItem(fullItem);
    }
    return fullItem;
  }

  selectVault(vaultId: string, defaults: Partial<VaultSettings> = {}) {
    this.#activeVaultId = vaultId;
    const existing = settings.state.vaultSettingsById[vaultId];
    settings.state.vaultSettingsById[vaultId] = {
      ...defaultVaultSettings,
      ...existing,
      ...defaults
    };
    settings.save();
  }

  updateSettings(updater: (settings: VaultSettings) => VaultSettings) {
    const id = this.#activeVaultId;
    if (!id) return;

    const existing = settings.state.vaultSettingsById[id];
    settings.state.vaultSettingsById[id] = updater({
      ...defaultVaultSettings,
      ...existing
    });
    settings.save();
  }

  clearVault(vaultId: string) {
    if (vaultId in settings.state.vaultSettingsById) {
      const newMap = { ...settings.state.vaultSettingsById };
      delete newMap[vaultId];
      settings.state.vaultSettingsById = newMap;
      settings.save();
    }
    if (this.#activeVaultId === vaultId) {
      this.#activeVaultId = null;
      this.#items = [];
    }
  }

  setItems(items: PasswordItemOverview[]) {
    this.#items = items;
  }

  updateItem(updatedItem: PasswordItemOverview) {
    const index = this.#items.findIndex((item) => item.id === updatedItem.id);
    if (index !== -1) {
      this.#items[index] = updatedItem;
    }
  }

  removeItem(id: number) {
    this.#items = this.#items.filter((item) => item.id !== id);
  }
}

export const vaultStore = new VaultStore();

export const vaultSettings = {
  get value() {
    return vaultStore.settings;
  },
  selectVault: (id: string, def?: Partial<VaultSettings>) => vaultStore.selectVault(id, def),
  update: (updater: (s: VaultSettings) => VaultSettings) => vaultStore.updateSettings(updater),
  clear: (id: string) => vaultStore.clearVault(id),
  getCurrentVaultId: () => vaultStore.activeVaultId
};
