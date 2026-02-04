import { callBackend } from '$lib/utils/backend';
import { appState } from './appState.svelte';

export interface TagButton {
  id: number;
  text: string;
  icon: string;
  color: string;
  count?: number;
}

export type TagInput = Omit<TagButton, 'id' | 'count'> & { id?: number };

class TagStore {
  #tags = $state<TagButton[]>([]);
  #isRefreshing = false;

  constructor() {
    $effect.root(() => {
      $effect(() => {
        if (appState.isLocked) {
          this.#tags = [];
        }
      });
    });
  }

  get tags() {
    return this.#tags;
  }

  async refresh() {
    if (this.#isRefreshing) return;
    this.#isRefreshing = true;
    try {
      const [tags, counts] = await Promise.all([
        callBackend<TagButton[]>('get_buttons'),
        callBackend<Record<number, number>>('get_tag_counts').catch(() => ({} as Record<number, number>))
      ]);
      
      this.#tags = (tags || []).map(t => ({
        ...t,
        count: counts[t.id] || 0
      }));
    } finally {
      this.#isRefreshing = false;
    }
  }

  async create(tag: TagInput) {
    await callBackend('save_button', {
      text: tag.text,
      icon: tag.icon,
      color: tag.color
    });
    await this.refresh();
  }

  async updateTag(tag: TagButton) {
    await callBackend('update_button', {
      id: tag.id,
      text: tag.text,
      icon: tag.icon,
      color: tag.color
    });
    const index = this.#tags.findIndex((t) => t.id === tag.id);
    if (index !== -1) {
      this.#tags[index] = tag;
    }
  }

  async remove(id: number) {
    await callBackend('delete_button', { id });
    this.#tags = this.#tags.filter((t) => t.id !== id);
  }

  async renameTagAcrossItems(oldTag: string, newTag: string) {
    await callBackend('rename_tag_in_password_items', { old_tag: oldTag, new_tag: newTag });
  }

  async removeTagAcrossItems(tag: string) {
    await callBackend('remove_tag_from_password_items', { tag });
  }
}

export const tagStore = new TagStore();
