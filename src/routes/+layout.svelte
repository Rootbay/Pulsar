<script lang="ts">
  import { isDatabaseLoaded, needsPasswordSetup, isLocked, showSettingsPopup } from '$lib/stores';
  import '../app.css';
  
  import DatabaseSelection from '$lib/components/auth/DatabaseSelection.svelte';
  import SetMasterPassword from '$lib/components/auth/SetMasterPassword.svelte';
  import Login from "../lib/components/auth/Login.svelte";
  import Totp from "../lib/components/auth/totp.svelte";
  import Popup from "../lib/components/ui/Popup.svelte";
  import Sidebar from "../lib/components/layout/sidebar.svelte";
  import PasswordList from "../lib/components/layout/passwordList.svelte";
  import PasswordDetail from "../lib/components/layout/passwordDetail.svelte";
  import CreatePasswordPopup from "../lib/components/CreatePasswordPopup.svelte";
  import Settings from "./settings/+page.svelte";
  

  import { invoke } from "@tauri-apps/api/core";
  import type { PasswordItem } from "./+layout.ts";
  import { securitySettings } from '$lib/stores/security';
  import { appearanceSettings } from '$lib/stores/appearance';

  $: if (typeof document !== 'undefined') {
    const htmlElement = document.documentElement;
    const currentTheme = $appearanceSettings.theme;

    htmlElement.classList.remove('theme-light', 'theme-dark', 'theme-system');

    if (currentTheme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      htmlElement.classList.add(prefersDark ? 'theme-dark' : 'theme-light');
    } else {
      htmlElement.classList.add(`theme-${currentTheme}`);
    }
  }

  interface Button {
    id: number;
    text: string;
    icon: string;
    color: string;
  }

  let passwordItems: PasswordItem[] = [];
  let selectedPasswordItem: PasswordItem | null = null;
  let showCreatePasswordPopup = false;
  let showPopup = false;
  let popupMode: 'create' | 'edit' = 'create';
  let popupTag: any = null;
  let buttons: Button[] = [];
  let displayColor: string = '#94a3b8';
  

  $: {
    if (selectedPasswordItem && selectedPasswordItem.tags) {
      const firstTag = selectedPasswordItem.tags.split(',')[0].trim();
      const button = buttons.find(b => b.text === firstTag);
      displayColor = button ? button.color : '#94a3b8';
    } else {
      displayColor = '#94a3b8';
    }
  }

  $: needsTotp = !$isLocked && $securitySettings.useTotp && !passwordItems.length;

  async function loadPasswordItems() {
    if ($isLocked) {
      passwordItems = [];
      return;
    }
    try {
      passwordItems = await invoke('get_password_items');
      buttons = await invoke('get_buttons');
    } catch (error) {
      console.error("Failed to load vault items:", error);
    }
  }

  $: if (!$isLocked) {
    loadPasswordItems();
  }

  function handleChangeDatabaseRequest() {
    isDatabaseLoaded.set(false);
    isLocked.set(true);
  }

  async function handleLock() {
    try {
      await invoke("lock");
      isLocked.set(true);
      selectedPasswordItem = null;
      passwordItems = [];
      buttons = [];
    } catch (error) {
      console.error("Lock failed:", error);
    }
  }

  function openPopup(event: CustomEvent) {
    popupMode = event.detail.mode;
    popupTag = event.detail.tag || null;
    showPopup = true;
  }

  async function closePopup() {
    showPopup = false;
  }

  async function handleSave(event: CustomEvent) {
    const { mode, updatedTag } = event.detail;

    if (mode === 'create') {
        buttons = await invoke('get_buttons');
    } else if (mode === 'edit') {
        const oldTag = popupTag;
        buttons = buttons.map(b => b.id === updatedTag.id ? updatedTag : b);

        if (oldTag && oldTag.text !== updatedTag.text) {
            const itemsToUpdate = passwordItems.filter(item => 
                item.tags && item.tags.split(',').map(t => t.trim()).includes(oldTag.text)
            );

            for (const item of itemsToUpdate) {
                if (item.tags) {
                    let tags = item.tags.split(',').map(t => t.trim());
                    tags = tags.map(t => t === oldTag.text ? updatedTag.text : t);
                    const newTags = tags.join(',');
                    
                    item.tags = newTags;

                    try {
                        await invoke('update_password_item_tags', { id: item.id, tags: newTags });
                    } catch (error) {
                        console.error(`Failed to update tags for item ${item.id}:`, error);
                    }
                }
            }
            passwordItems = [...passwordItems];
        }
    }
  }

  async function handleTagDeleted(event: CustomEvent) {
    const { id, text } = event.detail;
    buttons = buttons.filter(b => b.id !== id);

    const itemsToUpdate = passwordItems.filter(item => item.tags && item.tags.split(',').map(t => t.trim()).includes(text));

    for (const item of itemsToUpdate) {
        if (item.tags) {
            let tags = item.tags.split(',').map(t => t.trim());
            tags = tags.filter(t => t !== text);
            const newTags = tags.join(',');
            
            item.tags = newTags;

            try {
                await invoke('update_password_item_tags', { id: item.id, tags: newTags });
            } catch (error) {
                console.error(`Failed to update tags for item ${item.id}:`, error);
            }
        }
    }

    passwordItems = await invoke('get_password_items');

    if (selectedPasswordItem) {
      const refreshedItem = passwordItems.find(i => i.id === selectedPasswordItem?.id);
      selectedPasswordItem = refreshedItem ?? null;
    }
  }

  

  function handleCreateEntry() {
    showCreatePasswordPopup = true;
  }

  async function handlePasswordSaved() {
    showCreatePasswordPopup = false;
    passwordItems = await invoke('get_password_items');
    // Auto-select the newly created item: pick the most recent by created_at or highest id as fallback
    if (passwordItems && passwordItems.length) {
      let newest = passwordItems[0];
      for (const item of passwordItems) {
        try {
          if (new Date(item.created_at) > new Date(newest.created_at)) {
            newest = item;
          }
        } catch (_) {
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

  function handleTagCreated() {
    
    loadPasswordItems();
  }

  function handlePasswordSelected(event: CustomEvent) {
    selectedPasswordItem = event.detail;
  }

  async function handleRemoveEntry(event: CustomEvent) {
    const itemToRemove = event.detail;
    if (!itemToRemove) return;

    try {
      await invoke("delete_password_item", { id: itemToRemove.id });
      passwordItems = passwordItems.filter(i => i.id !== itemToRemove.id);
      if (selectedPasswordItem && selectedPasswordItem.id === itemToRemove.id) {
        selectedPasswordItem = null;
      }
    } catch (error) {
      console.error("Failed to remove password entry:", error);
      alert(`Error: ${error}`);
    }
  }
</script>

{#if !$isDatabaseLoaded}
  <DatabaseSelection />
{:else if $needsPasswordSetup}
  <SetMasterPassword />
{:else if $isLocked}
  <Login on:changeDatabase={handleChangeDatabaseRequest} />
{:else if needsTotp}
  <Totp on:loginSuccess={loadPasswordItems} />
{:else}
  <div class="app-container-wrapper">
    <div class="main-app-view">
      <div class="layout">
        <Sidebar {buttons} on:openPopup={openPopup} on:tagDeleted={handleTagDeleted} on:lock={handleLock} />
        <PasswordList items={passwordItems} on:createEntry={handleCreateEntry} on:select={handlePasswordSelected} {buttons} on:removeEntry={handleRemoveEntry} selectedId={selectedPasswordItem?.id ?? null} />
        <PasswordDetail
          bind:selectedPasswordItem
          {displayColor}
          {buttons}
          on:removeEntry={handleRemoveEntry}
          on:tagsSaved={(e) => {
            const { id, tags } = e.detail;
            const idx = passwordItems.findIndex(i => i.id === id);
            if (idx !== -1) {
              passwordItems[idx] = { ...passwordItems[idx], tags };
              passwordItems = [...passwordItems];
            }
          }}
        >
          <slot />
        </PasswordDetail>
      </div>
    </div>
  </div>

  {#if $showSettingsPopup}
    <Settings on:close={() => showSettingsPopup.set(false)} />
  {/if}

  {#if showPopup}
    <Popup on:close={closePopup} on:save={handleSave} mode={popupMode} tag={popupTag} />
  {/if}

  {#if showCreatePasswordPopup}
    <CreatePasswordPopup on:passwordSaved={handlePasswordSaved} on:close={handleCloseCreatePasswordPopup} on:tagCreated={handleTagCreated} />
  {/if}
{/if}

<style>
  :global(html, body) {
    margin:0;
    padding:0;
    height:100%;
    min-height:100vh;
    overflow:hidden;
    width:100%;
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

  :global(input[type="checkbox"]), 
  :global(input[type="radio"]) {
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

  :global(input[type="radio"]) {
    border-radius: 50%;
  }

  :global(input[type="checkbox"]:checked),
  :global(input[type="radio"]:checked) {
    background-color: var(--accent-color);
    border-color: var(--accent-color);
  }

  :global(input[type="checkbox"]:checked::before) {
    content: '✔';
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

  :global(input[type="radio"]:checked::before) {
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

  :global(input[type="checkbox"]:focus),
  :global(input[type="radio"]:focus),
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
