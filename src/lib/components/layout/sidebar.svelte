<script lang="ts">
import { createEventDispatcher } from 'svelte';
import Icon from "../ui/Icon.svelte";
import { iconPaths } from "$lib/icons";
import { selectedTag, filterCategory, showSettingsPopup } from '$lib/stores';
import ContextMenu from '../ui/ContextMenu.svelte';
import { invoke } from '@tauri-apps/api/core';

  interface Button { id: number; text: string; icon: string; color: string }
  export let buttons: Button[] = [];

  const dispatch = createEventDispatcher();

  let contextMenuVisible = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let contextMenuButton: Button | null = null;

  function handleOpenPopup() {
      dispatch('openPopup', { mode: 'create' });
  }

  function handleShowAll() {
      selectedTag.set(null);
      filterCategory.set('all');
  }

  function handleTagClick(tagText: string) {
      selectedTag.set(tagText);
  }

  function handleContextMenu(event: MouseEvent, button: Button) {
      event.preventDefault();
      contextMenuVisible = true;
      contextMenuX = event.clientX;
      contextMenuY = event.clientY;
      contextMenuButton = button;
  }

  function closeContextMenu() {
      contextMenuVisible = false;
      contextMenuButton = null;
  }

  function handleEdit() {
      if (contextMenuButton) {
          dispatch('openPopup', { mode: 'edit', tag: contextMenuButton });
      }
      closeContextMenu();
  }

  async function handleRemove() {
      if (contextMenuButton) {
          try {
              await invoke('delete_button', { id: contextMenuButton.id });
              dispatch('tagDeleted', { id: contextMenuButton.id, text: contextMenuButton.text });
          } catch (error) {
              console.error('Failed to delete tag:', error);
          }
      }
      closeContextMenu();
  }
</script>

<aside class="sidebar">
	<img src="/svelte.svg" alt="Logo" class="logo-icon" />
	<section class="navButtonContainer">
		<button class="asideButton show-all-button" aria-label="Show All Items" on:click={handleShowAll} class:selected={$selectedTag === null && $filterCategory === 'all'}>
			  <Icon path={iconPaths.globe} size="22" viewBox="0 0 44 44" color="#fff" />
		</button>
		
		<section class="asideButtonsContainer">
			{#each buttons as button (button.id)}
			<button 
                class="asideButton" 
                aria-label={button.text} 
                style="border-color: #1A1A1C; background-color: transparent;" 
                class:selected={$selectedTag === button.text}
                on:click={() => handleTagClick(button.text)}
                on:contextmenu={(e) => handleContextMenu(e, button)}
            >
				<Icon
                    path={button.icon}
                    color={button.color}
                    size="22"
                    viewBox="0 0 44 44"
                />
			</button>
			{/each}
		</section>

		<button id="newButton" on:click={handleOpenPopup} aria-label="Add New Item">
			  <Icon path={iconPaths.plus} size="23" color="currentColor" />
		</button>
	</section>

	<button on:click={() => showSettingsPopup.set(true)} class="settings-button" aria-label="Settings">
			<Icon path={iconPaths.settings} size="22" viewBox="0 0 44 44" color="#fff" />
  </button>
</aside>

{#if contextMenuVisible}
    <ContextMenu 
        x={contextMenuX} 
        y={contextMenuY} 
        on:close={closeContextMenu}
        on:edit={handleEdit}
        on:remove={handleRemove}
    />
{/if}

<style>
  .sidebar {
    background-color: var(--sidepanel-bg);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 15px 0;
    height: 100%;
    overflow-y: auto;
    box-sizing: border-box;
  }

  .logo-icon {
    width: 24px;
    height: 24px;
  }

  #newButton {
    height: 40px;
    width: 40px;
    border-radius: 20px;
    border: none;
    cursor: pointer;
    background: var(--btn-nav-border);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  #newButton :global(svg) {
    width: 23px;
    height: 23px;
    color: var(--white);
	opacity: 40%;
  }

  .asideButtonsContainer {
    display: flex;
    flex-direction: column;
  }

  .navButtonContainer {
    display: flex;
    flex-direction: column;
    margin-top: 25px;
    align-items: center;
    flex-grow: 1;
  }

  .show-all-button {
    margin-bottom: 25px;
  }

  .asideButton {
    width: 46px;
    height: 46px;
    border-radius: 8px;
    border: 3px solid;
    border-color: var(--btn-nav-border);
    background-color: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    margin-bottom: 15px;
  }

  .asideButton :global(svg) {
    width: 22px;
    height: 22px;
    display: block;
    opacity: 40%;
    transition: opacity 200ms ease;
  }

  /* Hover effect: increase icon opacity on hover */
  .asideButton:hover :global(svg) {
    opacity: 100%;
  }

  /* Selected effect: keep icon opaque when selected */
  .asideButton.selected :global(svg) {
    opacity: 100%;
  }

  .settings-button {
    width: 46px;
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    border: none;
    margin-top: auto;
    margin-bottom: 0;
    background-color: transparent;
  }

  .settings-button :global(svg) {
    opacity: 40%;
  }

  .settings-button:hover :global(svg) {
    opacity: 100%;
  }

</style>
