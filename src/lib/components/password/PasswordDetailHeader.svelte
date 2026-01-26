<svelte:options runes />

<script lang="ts">
  import type { PasswordItem } from '$lib/types/password';
  import TagList from './TagList.svelte';
  import { Ellipsis } from '@lucide/svelte';
  import { iconPaths } from '$lib/icons';
  import Favicon from '../ui/Favicon.svelte';

  interface TagOption {
    id: number;
    text: string;
    icon: string;
    color: string;
  }

  const primaryTagName = $derived.by(() => {
    const raw = selectedPasswordItem?.tags ?? '';
    const first = raw
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean)[0];
    return first || null;
  });

  const primaryTagIconPath = $derived.by(() => {
    const name = primaryTagName;
    if (!name) return iconPaths.default;
    const btn = (buttons || []).find((b: TagOption) => b.text === name);
    return btn?.icon || iconPaths.default;
  });

  interface Props {
    selectedPasswordItem: PasswordItem | null;
    isEditing: boolean;
    displayColor: string;
    buttons: TagOption[];

    onEnterEditMode?: () => void;
    onSave?: () => void;
    onRemoveEntry?: (id: number | undefined) => void;
    onTagsReorderedPending?: (detail: { tags: string }) => void;
  }

  let {
    selectedPasswordItem = $bindable<PasswordItem | null>(),
    isEditing = $bindable(false),
    displayColor,
    buttons,
    onEnterEditMode,
    onSave,
    onRemoveEntry,
    onTagsReorderedPending
  }: Props = $props();

  let showMoreDropdown = $state(false);

  let prevColor: string | null = $state(null);
  let pulse = $state(false);

  $effect(() => {
    if (selectedPasswordItem) {
      if (prevColor === null) {
        prevColor = displayColor;
      } else if (prevColor !== displayColor) {
        pulse = false;
        setTimeout(() => {
          pulse = true;
          prevColor = displayColor;
          setTimeout(() => (pulse = false), 360);
        }, 0);
      }
    }
  });

  function enterEditMode() {
    onEnterEditMode?.();
  }
</script>

{#if selectedPasswordItem}
  <div class="detail-header" style="--display-color: {displayColor};">
    <div class="title-and-tags">
      <div class="title-container">
        <Favicon
          url={selectedPasswordItem.url || undefined}
          title={selectedPasswordItem.title}
          fallbackIcon={primaryTagIconPath}
          fallbackColor={displayColor}
          size={30}
          useStroke={true}
        />
        <h2 class="header-title">
          {selectedPasswordItem.title}
        </h2>
        <span class="color-pulse-bg" aria-hidden="true" class:pulsing={pulse}></span>
      </div>

      <TagList
        bind:selectedPasswordItem
        bind:isEditing
        {buttons}
        onReorderPending={onTagsReorderedPending}
      />
    </div>

    <div class="detail-actions">
      <button class="edit-button" onclick={isEditing ? () => onSave?.() : enterEditMode}>
        {isEditing ? 'Save' : 'Modify'}
      </button>
      <div class="more-dropdown-container">
        <button class="more-button" onclick={() => (showMoreDropdown = !showMoreDropdown)}>
          <Ellipsis class="size-6" />
        </button>
        {#if showMoreDropdown}
          <div class="more-dropdown-menu">
            <button onclick={() => onRemoveEntry?.(selectedPasswordItem?.id)}>
              Delete Entry
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .title-container {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 5px;
  }

  /* .title-image removed */

  .detail-header h2 {
    margin: 0;
    font-weight: 400;
    font-size: 18px;
    transition: color 260ms ease;
  }

  .title-and-tags {
    display: flex;
    flex-direction: column;
  }

  .title-container {
    position: relative;
  }
  /* Ensure favicon (and its fallback icon) sits above the pulse background */
  .title-container :global(.itemImgContainer) {
    position: relative;
    z-index: 1;
  }

  .color-pulse-bg {
    position: absolute;
    left: -6px;
    top: -6px;
    width: 30px;
    height: 30px;
    border-radius: 12px;
    background: var(--display-color);
    filter: blur(8px);
    opacity: 0;
    pointer-events: none;
    transform: scale(0.85);
    transition:
      opacity 360ms ease,
      transform 360ms ease,
      background-color 260ms ease;
    z-index: 0;
  }
  .color-pulse-bg.pulsing {
    opacity: 0.25;
    transform: scale(1.05);
  }

  .detail-actions {
    display: flex;
    align-items: center;
  }

  .edit-button {
    background-color: var(--near-black);
    border: 2px solid var(--btn-nav-border);
    border-radius: 13.5px;
    font-family: inherit;
    font-style: normal;
    font-weight: 500;
    font-size: 0.8rem;
    color: rgba(247, 219, 209, 0.5);
    cursor: pointer;
    width: 68px;
    height: 27px;
    margin-right: 10px;
  }

  .edit-button:hover {
    background-color: #17171b;
  }

  .more-dropdown-container {
    position: relative;
    display: inline-block;
  }

  .more-button {
    background: none;
    border: none;
    padding: 4px 0 0 0;
    color: #f7dbd1;
    width: 24px;
    height: 24px;
    cursor: pointer;
  }

  .more-button:hover {
    color: #ffbfa8;
  }

  .more-dropdown-menu {
    position: absolute;
    top: 100%;
    right: 0;
    background-color: var(--near-black);
    border: 2px solid var(--btn-nav-border);
    border-radius: 8px;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
    z-index: 200;
    display: flex;
    flex-direction: column;
    padding: 5px 0;
    min-width: 150px;
  }

  .more-dropdown-menu button {
    background: none;
    border: none;
    color: var(--white);
    padding: 8px 15px;
    text-align: left;
    cursor: pointer;
    width: 100%;
    white-space: nowrap;
  }

  .more-dropdown-menu button:hover {
    background-color: #3a3a3a;
  }
</style>
