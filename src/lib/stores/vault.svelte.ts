import { settings } from './appSettings.svelte';
import { defaultVaultSettings, type VaultSettings } from '../config/settings';
import type { PasswordItem, PasswordItemOverview } from '../types/password';
import { callBackend } from '../utils/backend';
import { appState } from './appState.svelte';
import { tagStore } from './tags.svelte';

class VaultStore {
  #items = $state<PasswordItemOverview[]>([]);
  #isLoading = $state(false);
  #activeVaultId = $state<string | null>(null);
  #searchTerm = $state('');
  #searchTimeout: ReturnType<typeof setTimeout> | null = null;
  #limit = 50;
  #offset = 0;
  #hasMore = $state(true);
  #totalItemCount = $state(0);
  #favoritesCount = $state(0);
  #currentRequestId = 0;

  constructor() {
    $effect.root(() => {
      $effect(() => {
        if (appState.isLocked) {
          this.#items = [];
          this.#activeVaultId = null;
          this.#searchTerm = '';
          this.#offset = 0;
          this.#hasMore = true;
          this.#totalItemCount = 0;
          this.#favoritesCount = 0;
          this.#currentRequestId++;
        }
      });

      $effect(() => {
        const _tag = appState.selectedTag;
        const _category = appState.filterCategory;
        const _query = this.#searchTerm;

        if (this.#searchTimeout) clearTimeout(this.#searchTimeout);
        this.#searchTimeout = setTimeout(() => {
          this.loadItems();
        }, 150);
      });
    });
  }

  get items() {
    return this.#items;
  }

  get totalItemCount() {
    return this.#totalItemCount;
  }

  get favoritesCount() {
    return this.#favoritesCount;
  }

  get isLoading() {
    return this.#isLoading;
  }

  get hasMore() {
    return this.#hasMore;
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

  async #fetchItems() {
    const tag = appState.selectedTag;
    const category = appState.filterCategory;
    const tagObj = tagStore.tags.find((t) => t.text === tag);

    return await callBackend<PasswordItemOverview[]>('search_password_items', {
      query: this.#searchTerm,
      tagId: tagObj?.id ?? null,
      category: category === 'all' ? null : category,
      limit: this.#limit,
      offset: this.#offset
    });
  }

  async refreshCounts() {
    if (appState.isLocked) return;
    try {
      const [total, favorites] = await Promise.all([
        callBackend<number>('get_total_items_count'),
        callBackend<number>('get_favorites_count').catch(() => 0)
      ]);
      this.#totalItemCount = total;
      this.#favoritesCount = favorites;
    } catch (error) {
      console.error('Failed to get counts:', error);
    }
  }

  async loadItems() {
    if (appState.isLocked) {
      this.#items = [];
      return;
    }

    const requestId = ++this.#currentRequestId;
    this.#isLoading = true;
    this.#offset = 0;
    this.#hasMore = true;
    try {
      const results = await this.#fetchItems();
      if (this.#currentRequestId !== requestId) return;
      this.#items = results;
      this.#hasMore = results.length === this.#limit;
      this.#offset = results.length;
      await this.refreshCounts();
    } catch (error) {
      console.error('Failed to load vault items:', error);
    } finally {
      if (this.#currentRequestId === requestId) {
        this.#isLoading = false;
      }
    }
  }

  async loadMore() {
    if (this.#isLoading || !this.#hasMore || appState.isLocked) return;

    const requestId = this.#currentRequestId;
    this.#isLoading = true;
    try {
      const results = await this.#fetchItems();
      if (this.#currentRequestId !== requestId) return;
      if (results.length > 0) {
        this.#items = [...this.#items, ...results];
        this.#offset += results.length;
      }
      this.#hasMore = results.length === this.#limit;
    } catch (error) {
      console.error('Failed to load more vault items:', error);
    } finally {
      if (this.#currentRequestId === requestId) {
        this.#isLoading = false;
      }
    }
  }

  async getItemDetails(id: number): Promise<PasswordItem | null> {
    const existing = this.#items.find((i) => i.id === id);
    if (existing && 'password' in existing) {
      return existing as PasswordItem;
    }

    const fullItem = await callBackend<PasswordItem | null>('get_password_item_by_id', { id });
    if (fullItem) {
      const overview: PasswordItemOverview = {
        id: fullItem.id,
        category: fullItem.category,
        title: fullItem.title,
        description: fullItem.description,
        img: fullItem.img,
        tags: fullItem.tags,
        username: fullItem.username,
        url: fullItem.url,
        created_at: fullItem.created_at,
        updated_at: fullItem.updated_at,
        color: fullItem.color
      };
      this.updateItem(overview);
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
