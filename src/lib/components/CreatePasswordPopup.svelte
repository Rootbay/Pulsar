<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from 'svelte';
  import Icon from './ui/Icon.svelte';

  const dispatch = createEventDispatcher();

  let title: string = '';

  let availableTags: { id: number; text: string; icon: string; color: string }[] = [];

  onMount(async () => {
    try {
      availableTags = await invoke('get_buttons');
    } catch (error) {
      console.error("Error fetching tags:", error);
    }

    popupContent?.focus();
  });

  // Tag chips state
  let tags: string[] = [];

  function addTag(tag: string) {
    if (!tags.includes(tag)) tags = [...tags, tag];
  }

  function removeTag(tag: string) {
    tags = tags.filter(t => t !== tag);
  }

  // (Optional) If tag creation is needed later,
  // this popup logic can be reintroduced.

  // Placeholder password for new entries
  const FIXED_PASSWORD = "N/A";

  async function savePassword() {
    try {
      // Use fixed password for new item
      const autoPassword = FIXED_PASSWORD;
      await invoke('save_password_item', {
        item: {
          id: 0,
          title,
          username: null,
          url: null,
          notes: null,
          password: autoPassword,
          description: null,
          img: null,
          tags: tags.length ? tags.join(',') : null,
          created_at: '',
          updated_at: '',
          color: null,
          totp_secret: null,
          custom_fields: [],
          field_order: null,
        },
      });
      dispatch('passwordSaved');
      dispatch('close');
    } catch (error) {
      console.error("Error saving password:", error);
    }
  }

  function cancel() {
    dispatch('close');
  }

  function handleContentKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      cancel();
    }
  }

  function handleBackdropKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ' ' || event.key === 'Spacebar') {
      event.preventDefault();
      cancel();
    } else if (event.key === 'Escape') {
      cancel();
    }
  }

  let popupContent: HTMLDivElement | null = null;
</script>

<div
  id="popup-backdrop"
  class="popup-backdrop"
  on:click={cancel}
  on:keydown={handleBackdropKeydown}
  role="presentation"
  tabindex="-1"
  aria-label="Create New Password Entry"
>
  <div
    class="popupContent"
    on:click|stopPropagation
    role="dialog"
    aria-modal="true"
    aria-labelledby="popup-title"
    tabindex="-1"
    on:keydown={handleContentKeydown}
    bind:this={popupContent}
  >
    <h2 id="popup-title" class="dialogTitle">New Item</h2>
    <form class="form" on:submit|preventDefault={savePassword}>
      <div class="formRow">
        <label for="title">Title</label>
        <input type="text" id="title" placeholder="Enter title" bind:value={title} required />
      </div>
      <div class="formRow">
        <label for="extraDropdown">Dropdown</label>
        <select id="extraDropdown">
          <option value="" selected disabled>—</option>
        </select>
      </div>

      <div class="tagSection">
        <div class="sectionTitle">Tags</div>
        <div class="selectedTags">
          {#each tags as tag (tag)}
            <span class="chip">
              {tag}
              <button type="button" class="chipRemove" title="Remove" on:click={() => removeTag(tag)}>×</button>
            </span>
          {/each}
          {#if tags.length === 0}
            <span class="emptyHint">No tags selected</span>
          {/if}
        </div>
        <div class="availableTags">
          {#if availableTags.length === 0}
            <span class="emptyHint">No tags available</span>
          {:else}
            {#each availableTags as tag (tag.id)}
              <button type="button" class="availableTagChip" on:click={() => addTag(tag.text)}>
                <Icon path={tag.icon} color={tag.color} viewBox='0 0 48 48' />
                <span>{tag.text}</span>
              </button>
            {/each}
          {/if}
        </div>
      </div>
      <div class="formActions">
        <button type="button" class="btnSecondary" on:click={cancel}>Cancel</button>
        <button type="submit" class="btnPrimary">Save</button>
      </div>
    </form>
  </div>
</div>



<style>
  /* Backdrop similar to ui/Popup.svelte */
  .popup-backdrop {
    position: fixed;
    inset: 0;
    background:
      radial-gradient(120% 120% at 50% -10%, rgba(255,255,255,0.06) 0%, rgba(0,0,0,0) 60%),
      rgba(6, 8, 12, 0.55);
    backdrop-filter: blur(10px) saturate(120%);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 200;
  }

  /* Modern container */
  .popupContent {
    width: min(80vw, 640px);
    border-radius: var(--radius, 12px);
    background: linear-gradient(135deg, hsl(var(--card)), hsl(var(--muted) / 0.3));
    display: flex;
    flex-direction: column;
    align-items: stretch;
    padding: 20px;
    gap: 16px;
    border: 1px solid hsl(var(--border) / 0.5);
    color: var(--white);
  }

  .dialogTitle {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    text-align: left;
  }

  .form { display: flex; flex-direction: column; gap: 14px; }
  .formRow { display: flex; flex-direction: column; gap: 6px; }
  .formRow label { font-size: 13px; color: hsl(var(--muted-foreground)); }

  .formRow input,
  .formRow select {
    width: 100%;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid rgba(80, 80, 92, 0.35);
    background: linear-gradient(180deg, #1a1b20, #14151a);
    color: var(--white);
    outline: none;
  }

  .formActions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 6px;
  }

  /* Tag chips */
  .tagSection { display: flex; flex-direction: column; gap: 8px; }
  .sectionTitle { font-size: 13px; color: hsl(var(--muted-foreground)); }
  .selectedTags { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  .availableTags { display: flex; flex-wrap: wrap; gap: 8px; }
  .emptyHint { color: hsl(var(--muted-foreground)); font-size: 12px; }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-radius: 999px;
    background: linear-gradient(180deg, #1d1e24, #16171c);
    border: 1px solid rgba(80, 80, 92, 0.35);
    font-size: 12px;
  }
  .chipRemove {
    border: none;
    background: transparent;
    color: var(--white);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0 2px;
  }

  .availableTagChip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 12px;
    border: 1px solid rgba(80, 80, 92, 0.35);
    background: linear-gradient(180deg, #1a1b20, #14151a);
    color: var(--white);
    cursor: pointer;
  }
  .availableTagChip :global(svg) { width: 18px; height: 18px; display: block; }

  .btnPrimary, .btnSecondary {
    padding: 10px 16px;
    border-radius: 12px;
    border: 1px solid rgba(80, 80, 92, 0.35);
    background: linear-gradient(180deg, #1a1b20, #14151a);
    color: var(--white);
    cursor: pointer;
  }

  .btnPrimary { border-color: rgba(120, 120, 220, 0.45); }

  @media (max-width: 560px) {
    .popupContent { padding: 16px; gap: 10px; width: min(92vw, 560px); }
  }
</style>
