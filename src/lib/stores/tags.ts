import { writable } from 'svelte/store';
import { callBackend } from '$lib/utils/backend';

export interface TagButton {
  id: number;
  text: string;
  icon: string;
  color: string;
}

export type TagInput = Omit<TagButton, 'id'> & { id?: number };

function createTagStore() {
  const { subscribe, set, update } = writable<TagButton[]>([]);
  let refreshPromise: Promise<void> | null = null;

  const refresh = async () => {
    if (refreshPromise) {
      return refreshPromise;
    }

    refreshPromise = (async () => {
      try {
        const tags = await callBackend<TagButton[]>('get_buttons');
        set(tags);
      } finally {
        refreshPromise = null;
      }
    })();

    return refreshPromise;
  };

  const create = async (tag: TagInput) => {
    await callBackend('save_button', {
      text: tag.text,
      icon: tag.icon,
      color: tag.color
    });
    await refresh();
  };

  const updateTag = async (tag: TagButton) => {
    await callBackend('update_button', {
      id: tag.id,
      text: tag.text,
      icon: tag.icon,
      color: tag.color
    });
    update((items) => items.map((item) => (item.id === tag.id ? tag : item)));
  };

  const remove = async (id: number) => {
    await callBackend('delete_button', { id });
    update((items) => items.filter((item) => item.id !== id));
  };

  const renameTagAcrossItems = async (oldTag: string, newTag: string) => {
    await callBackend('rename_tag_in_password_items', { old_tag: oldTag, new_tag: newTag });
  };

  const removeTagAcrossItems = async (tag: string) => {
    await callBackend('remove_tag_from_password_items', { tag });
  };

  return {
    subscribe,
    refresh,
    create,
    updateTag,
    remove,
    renameTagAcrossItems,
    removeTagAcrossItems
  };
}

export const tagStore = createTagStore();
