<script lang="ts">
  import { appState, vaultStore } from '$lib/stores';
  import { tagStore, type TagInput, type TagButton } from '$lib/stores/tags.svelte';
  import AppSidebar from '$lib/components/layout/sidebar.svelte';
  import PasswordList from '$lib/components/layout/passwordList.svelte';
  import PasswordDetail from '$lib/components/layout/passwordDetail.svelte';
  import CreatePasswordPopup from '$lib/components/CreatePasswordPopup.svelte';
  import Popup from '$lib/components/ui/Popup.svelte';
  import DeleteTagPopup from '$lib/components/DeleteTagPopup.svelte';
  import DeletePasswordPopup from '$lib/components/DeletePasswordPopup.svelte';
  import { SidebarProvider } from '$lib/components/ui/sidebar';
  import Settings from '../settings/general/+page.svelte';
  import { callBackend } from '$lib/utils/backend';
  import type { PasswordItem, PasswordItemOverview } from '$lib/types/password';
  import { onMount, tick } from 'svelte';
  import { toast } from '$lib/components/ui/sonner';
  import { profileSettings } from '$lib/stores/profile.svelte';

  type Button = TagInput;

  let { children } = $props();

  let selectedPasswordItem = $state<PasswordItem | null>(null);
  let showCreatePasswordPopup = $state(false);
  let showPopup = $state(false);
  let showDeleteTagPopup = $state(false);
  let showDeletePasswordPopup = $state(false);
  let tagToDelete = $state<TagButton | null>(null);
  let itemToDelete = $state<PasswordItemOverview | null>(null);
  let popupMode = $state<'create' | 'edit'>('create');
  let popupTag = $state<Button | null>(null);
  const buttons = $derived(tagStore.tags);
  let displayColor = $state('#94a3b8');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let passwordListRef = $state<any>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let passwordDetailRef = $state<any>(null);

  $effect(() => {
    if (selectedPasswordItem && selectedPasswordItem.tags) {
      const tags = selectedPasswordItem.tags.split(',').map((t) => t.trim()).filter(Boolean);
      if (tags.length > 0) {
        const button = buttons.find((b) => b.text === tags[0]);
        displayColor = button ? button.color : '#94a3b8';
      } else {
        displayColor = '#94a3b8';
      }
    } else {
      displayColor = '#94a3b8';
    }
  });

  async function loadPasswordItems() {
    if (appState.isLocked) {
      return;
    }

    try {
      await vaultStore.loadItems();
      await tagStore.refresh();
    } catch (error) {
      console.error('Failed to load vault items:', error);
    }
  }

  function handleVaultRefreshEvent() {
    void loadPasswordItems();
  }

  $effect(() => {
    if (appState.requestedItemId !== null) {
      const id = appState.requestedItemId;
      appState.requestedItemId = null;

      const item = vaultStore.items.find((i) => i.id === id);
      if (item) {
        handlePasswordSelected(item);
      } else {
        appState.selectedTag = null;
        appState.filterCategory = 'all';
        vaultStore.searchTerm = '';

        tick().then(async () => {
          await vaultStore.loadItems();
          const reloadedItem = vaultStore.items.find((i) => i.id === id);
          if (reloadedItem) {
            handlePasswordSelected(reloadedItem);
          }
        });
      }
    }
  });

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
    if (!appState.isLocked) {
      loadPasswordItems();
      profileSettings.load();
    } else {
      selectedPasswordItem = null;
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

        if (appState.selectedTag === oldTag.text) {
          appState.selectedTag = updatedTag.text;
        }
      }
    } catch (error) {
      console.error('Failed to save tag:', error);
      toast.error('Failed to save tag.');
      return;
    }

    await vaultStore.loadItems();

    if (selectedPasswordItem) {
      const refreshedItem = vaultStore.items.find((item) => item.id === selectedPasswordItem?.id);
      if (refreshedItem) {
        await handlePasswordSelected(refreshedItem);
      } else {
        selectedPasswordItem = null;
      }
    }
  }

  function handleTagDeleteRequested(detail: Button) {
    if (!detail.id) {
      return;
    }
    tagToDelete = detail as TagButton;
    showDeleteTagPopup = true;
  }

  async function confirmDeleteTag() {
    if (!tagToDelete || !tagToDelete.id) return;

    try {
      await tagStore.remove(tagToDelete.id);
      toast.success('Tag deleted.');
    } catch (error) {
      console.error('Failed to delete tag:', error);
      toast.error('Failed to delete tag.');
      return;
    }

    if (appState.selectedTag === tagToDelete.text) {
      appState.selectedTag = null;
    }

    showDeleteTagPopup = false;
    tagToDelete = null;

    await tick();
    await vaultStore.loadItems();
    await tagStore.refresh();

    if (selectedPasswordItem) {
      const refreshedItem = vaultStore.items.find((item) => item.id === selectedPasswordItem?.id);
      if (refreshedItem) {
        await handlePasswordSelected(refreshedItem);
      } else {
        selectedPasswordItem = null;
      }
    }
  }

  function handleCreateEntry() {
    showCreatePasswordPopup = true;
  }

  async function handlePasswordSaved(newId?: number) {
    showCreatePasswordPopup = false;
    await vaultStore.loadItems();

    if (newId) {
      const newItem = vaultStore.items.find((item) => item.id === newId);
      if (newItem) {
        await handlePasswordSelected(newItem);
        return;
      }
    }

    if (vaultStore.items.length) {
      let newest = vaultStore.items[0];

      for (const item of vaultStore.items) {
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

      await handlePasswordSelected(newest);
    }
  }

  function handleCloseCreatePasswordPopup() {
    showCreatePasswordPopup = false;
  }

  async function handlePasswordSelected(item: PasswordItemOverview | null) {
    if (!item) {
      selectedPasswordItem = null;
      return;
    }
    try {
      const fullItem = await vaultStore.getItemDetails(item.id);
      selectedPasswordItem = fullItem;
    } catch (err) {
      console.error('Failed to fetch password details', err);
      toast.error('Failed to load item details');
    }
  }

  async function handlePasswordEditRequested(item: PasswordItemOverview) {
    if (!item || appState.isLocked) {
      return;
    }

    const fullItem = await vaultStore.getItemDetails(item.id);
    if (!fullItem) return;

    selectedPasswordItem = fullItem;

    await tick();

    await passwordListRef?.focusItem?.(item.id);
    passwordDetailRef?.enterEditMode?.();
  }

  let disableContextEdit = $derived(appState.isLocked || vaultStore.items.length === 0);

  async function handleRemoveEntry(itemToRemove: PasswordItemOverview) {
    if (!itemToRemove) return;
    itemToDelete = itemToRemove;
    showDeletePasswordPopup = true;
  }

  async function confirmDeleteEntry() {
    if (!itemToDelete) return;
    
    try {
      await callBackend('delete_password_item', { id: itemToDelete.id });
      vaultStore.removeItem(itemToDelete.id);

      if (selectedPasswordItem && selectedPasswordItem.id === itemToDelete.id) {
        selectedPasswordItem = null;
      }
      toast.success('Password entry deleted.');
    } catch (error) {
      console.error('Failed to remove password entry:', error);
      toast.error('Failed to delete password entry.');
    } finally {
      showDeletePasswordPopup = false;
      itemToDelete = null;
    }
  }
</script>

<div class="app-container-wrapper">
  <div class="main-app-view">
    <div class="layout">
      <SidebarProvider class="min-h-full w-auto">
        <AppSidebar
          {buttons}
          totalItemCount={vaultStore.totalItemCount}
          favoritesCount={vaultStore.favoritesCount}
          onopenPopup={openPopup}
          ontagDeleteRequested={handleTagDeleteRequested}
        />
      </SidebarProvider>

      <PasswordList
        items={vaultStore.items}
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
          const idx = vaultStore.items.findIndex((item) => item.id === id);
          if (idx !== -1) {
            const updatedItem = { ...vaultStore.items[idx], tags };
            vaultStore.updateItem(updatedItem);
          }
        }}
        bind:this={passwordDetailRef}
      >
        {@render children?.()}
      </PasswordDetail>
    </div>
  </div>
</div>

{#if appState.showSettingsPopup}
  <div
    class="bg-background/80 fixed inset-0 z-50 flex items-center justify-center p-4 backdrop-blur-sm sm:p-8"
  >
    <div class="max-h-full w-full max-w-4xl overflow-y-auto rounded-2xl shadow-2xl">
      <Settings />
    </div>
  </div>
{/if}

{#if showPopup}
  <Popup onclose={closePopup} onsave={handleSave} mode={popupMode} tag={popupTag} />
{/if}

{#if showDeleteTagPopup && tagToDelete}
  <DeleteTagPopup
    tag={tagToDelete}
    onclose={() => (showDeleteTagPopup = false)}
    onconfirm={confirmDeleteTag}
  />
{/if}

{#if showDeletePasswordPopup && itemToDelete}
  <DeletePasswordPopup
    item={itemToDelete}
    onclose={() => (showDeletePasswordPopup = false)}
    onconfirm={confirmDeleteEntry}
  />
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
