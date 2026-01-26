<script lang="ts">
  import { isLocked, showSettingsPopup, selectedTag } from '$lib/stores';
  import { tagStore, type TagInput, type TagButton } from '$lib/stores/tags';
  import AppSidebar from '$lib/components/layout/sidebar.svelte';
  import PasswordList from '$lib/components/layout/passwordList.svelte';
  import PasswordDetail from '$lib/components/layout/passwordDetail.svelte';
  import CreatePasswordPopup from '$lib/components/CreatePasswordPopup.svelte';
  import Popup from '$lib/components/ui/Popup.svelte';
  import { SidebarProvider } from '$lib/components/ui/sidebar';
  import Settings from '../settings/general/+page.svelte';
  import { callBackend } from '$lib/utils/backend';
  import type { PasswordItem } from '../+layout.ts';
  import { onMount, tick } from 'svelte';
  import { toast } from '$lib/components/ui/sonner';
  import { profileSettings } from '$lib/stores/profile';

  type Button = TagInput;

  let { children } = $props();

  const SIDEBAR_WIDTH = '86px';
  const SIDEBAR_STYLE = '--sidebar-width: ' + SIDEBAR_WIDTH + ';';

  let passwordItems = $state<PasswordItem[]>([]);
  let selectedPasswordItem = $state<PasswordItem | null>(null);
  let showCreatePasswordPopup = $state(false);
  let showPopup = $state(false);
  let popupMode = $state<'create' | 'edit'>('create');
  let popupTag = $state<Button | null>(null);
  const buttons = $derived($tagStore);
  let displayColor = $state('#94a3b8');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let passwordListRef = $state<any>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let passwordDetailRef = $state<any>(null);

  $effect(() => {
    if (selectedPasswordItem && selectedPasswordItem.tags) {
      const firstTag = selectedPasswordItem.tags.split(',')[0].trim();
      const button = buttons.find((b) => b.text === firstTag);
      displayColor = button ? button.color : '#94a3b8';
    } else {
      displayColor = '#94a3b8';
    }
  });

  async function loadPasswordItems() {
    if ($isLocked) {
      passwordItems = [];
      return;
    }

    try {
      passwordItems = await callBackend('get_password_items');
      await tagStore.refresh();
    } catch (error) {
      console.error('Failed to load vault items:', error);
    }
  }

  function handleVaultRefreshEvent() {
    void loadPasswordItems();
  }

  onMount(() => {
    if (typeof window !== 'undefined') {
      window.addEventListener('pulsar:reload-vault', handleVaultRefreshEvent);

      return () => {
        window.removeEventListener('pulsar:reload-vault', handleVaultRefreshEvent);
      };
    }

    return undefined;
  });

  $effect(() => {
    if (!$isLocked) {
      loadPasswordItems();
      profileSettings.load();
    }
  });

  function openPopup(detail: { mode: 'create' | 'edit'; tag?: Button }) {
    popupMode = detail.mode;
    popupTag = detail.tag || null;
    showPopup = true;
  }

  async function closePopup() {
    showPopup = false;
  }

  async function handleSave(detail: { mode: 'create' | 'edit'; tag: Button }) {
    const { mode, tag } = detail;

    try {
      if (mode === 'create') {
        await tagStore.create(tag);
        toast.success('Tag created.');
        return;
      }

      if (!tag.id) {
        return;
      }

      await tagStore.updateTag(tag as TagButton);
      toast.success('Tag updated.');

      const updatedTag = tag as TagButton;
      const oldTag = popupTag;
      if (oldTag && oldTag.text !== updatedTag.text) {
        await tagStore.renameTagAcrossItems(oldTag.text, updatedTag.text);
        toast.success('Tag updated across items.');

        if ($selectedTag === oldTag.text) {
          selectedTag.set(updatedTag.text);
        }
      }
    } catch (error) {
      console.error('Failed to save tag:', error);
      toast.error('Failed to save tag.');
      return;
    }

    passwordItems = await callBackend('get_password_items');

    if (selectedPasswordItem) {
      const refreshedItem = passwordItems.find((item) => item.id === selectedPasswordItem?.id);
      selectedPasswordItem = refreshedItem ?? null;
    }
  }

  async function handleTagDeleteRequested(detail: Button) {
    if (!detail.id) {
      return;
    }

    try {
      await tagStore.remove(detail.id);
      await tagStore.removeTagAcrossItems(detail.text);
      toast.success('Tag deleted.');
    } catch (error) {
      console.error('Failed to delete tag:', error);
      toast.error('Failed to delete tag.');
      return;
    }

    if ($selectedTag === detail.text) {
      selectedTag.set(null);
    }

    passwordItems = await callBackend('get_password_items');

    if (selectedPasswordItem) {
      const refreshedItem = passwordItems.find((item) => item.id === selectedPasswordItem?.id);
      selectedPasswordItem = refreshedItem ?? null;
    }
  }

  function handleCreateEntry() {
    showCreatePasswordPopup = true;
  }

  async function handlePasswordSaved() {
    showCreatePasswordPopup = false;
    passwordItems = await callBackend('get_password_items');

    if (passwordItems.length) {
      let newest = passwordItems[0];

      for (const item of passwordItems) {
        try {
          if (new Date(item.created_at) > new Date(newest.created_at)) {
            newest = item;
          }
        } catch {
          if (item.id > (newest?.id ?? -Infinity)) {
            newest = item;
          }
        }
      }

      selectedPasswordItem = newest;
    }
  }

  function handleCloseCreatePasswordPopup() {
    showCreatePasswordPopup = false;
  }

  function handlePasswordSelected(item: PasswordItem | null) {
    selectedPasswordItem = item;
  }

  async function handlePasswordEditRequested(item: PasswordItem) {
    if (!item || $isLocked) {
      return;
    }

    selectedPasswordItem = item;
    await tick();

    await passwordListRef?.focusItem?.(item.id ?? null);
    passwordDetailRef?.enterEditMode?.();
  }

  let disableContextEdit = $derived($isLocked || passwordItems.length === 0);

  async function handleRemoveEntry(itemToRemove: PasswordItem) {
    if (!itemToRemove) return;

    try {
      await callBackend('delete_password_item', { id: itemToRemove.id });
      passwordItems = passwordItems.filter((item) => item.id !== itemToRemove.id);

      if (selectedPasswordItem && selectedPasswordItem.id === itemToRemove.id) {
        selectedPasswordItem = null;
      }
    } catch (error) {
      console.error('Failed to remove password entry:', error);
    }
  }
</script>

<div class="app-container-wrapper">
  <div class="main-app-view">
    <div class="layout" style={SIDEBAR_STYLE}>
      <SidebarProvider class="min-h-full w-auto" style={SIDEBAR_STYLE}>
        <AppSidebar
          {buttons}
          onopenPopup={openPopup}
          ontagDeleteRequested={handleTagDeleteRequested}
        />
      </SidebarProvider>

      <PasswordList
        items={passwordItems}
        oncreateEntry={handleCreateEntry}
        onselect={handlePasswordSelected}
        oneditEntry={handlePasswordEditRequested}
        {buttons}
        onremoveEntry={handleRemoveEntry}
        selectedId={selectedPasswordItem?.id ?? null}
        disableEdit={disableContextEdit}
        bind:this={passwordListRef}
      />
      <PasswordDetail
        bind:selectedPasswordItem
        {displayColor}
        {buttons}
        onremoveEntry={handleRemoveEntry}
        ontagsSaved={(detail) => {
          const { id, tags } = detail;
          const idx = passwordItems.findIndex((item) => item.id === id);
          if (idx !== -1) {
            passwordItems[idx] = { ...passwordItems[idx], tags };
            passwordItems = [...passwordItems];
          }
        }}
        bind:this={passwordDetailRef}
      >
        {@render children?.()}
      </PasswordDetail>
    </div>
  </div>
</div>

{#if $showSettingsPopup}
  <Settings onclose={() => showSettingsPopup.set(false)} />
{/if}

{#if showPopup}
  <Popup onclose={closePopup} onsave={handleSave} mode={popupMode} tag={popupTag} />
{/if}

{#if showCreatePasswordPopup}
  <CreatePasswordPopup
    onpasswordSaved={handlePasswordSaved}
    onclose={handleCloseCreatePasswordPopup}
  />
{/if}

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    min-height: 100vh;
    overflow: hidden;
    width: 100%;
    display: flex;
    flex-direction: column;
  }

  .app-container-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .main-app-view {
    display: flex;
    flex-direction: row;
    width: 100%;
    height: 100%;
  }

  .layout {
    display: grid;
    grid-template-columns: var(--sidebar-width) var(--passwordList-width) 1fr;
    grid-template-rows: 1fr;
    height: 100%;
    width: 100%;
    position: relative;
  }

  :global(::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background-color: #3a3a3a;
    border-radius: 10px;
    border: 2px solid transparent;
    background-clip: padding-box;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background-color: #555;
  }

  :global(html) {
    scrollbar-width: thin;
    scrollbar-color: #3a3a3a transparent;
  }

  :global(input[type='checkbox']),
  :global(input[type='radio']) {
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    width: 16px;
    height: 16px;
    border: 1px solid var(--border-color);
    border-radius: 3px;
    background-color: var(--input-bg);
    cursor: pointer;
    position: relative;
    vertical-align: middle;
    margin-right: 0.5em;
    flex-shrink: 0;
  }

  :global(input[type='radio']) {
    border-radius: 50%;
  }

  :global(input[type='checkbox']:checked),
  :global(input[type='radio']:checked) {
    background-color: var(--accent-color);
    border-color: var(--accent-color);
  }

  :global(input[type='checkbox']:checked::before) {
    content: '?';
    display: block;
    color: var(--accent-text-color);
    font-size: 12px;
    line-height: 1;
    text-align: center;
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }

  :global(input[type='radio']:checked::before) {
    content: '';
    display: block;
    width: 8px;
    height: 8px;
    background-color: var(--accent-text-color);
    border-radius: 50%;
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }

  :global(input[type='checkbox']:focus),
  :global(input[type='radio']:focus),
  :global(select:focus) {
    outline: none;
    box-shadow: 0 0 0 2px var(--focus-ring-color, rgba(0, 123, 255, 0.5));
  }

  :global(select) {
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    padding: 0.5rem 2.5rem 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 0.25rem;
    background-color: var(--input-bg);
    color: var(--text-color);
    cursor: pointer;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' fill='currentColor' viewBox='0 0 24 24'><path d='M7 10l5 5 5-5z'/></svg>");
    background-repeat: no-repeat;
    background-position: right 0.75rem center;
    background-size: 1rem;
    box-sizing: border-box;
  }

  :global(select:focus) {
    outline: none;
    box-shadow: 0 0 0 2px var(--focus-ring-color);
  }
</style>
