<svelte:options runes />

<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import type { PasswordItem } from '$lib/types/password';
  import { iconPaths } from '$lib/icons';
  import { dndzone } from 'svelte-dnd-action';
  import { flip } from 'svelte/animate';
  import { Plus } from '@lucide/svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  interface Props {
    selectedPasswordItem: PasswordItem | null;
    isEditing: boolean;
    buttons: { text: string; color: string; icon: string; id?: number }[];

    onReorderPending?: (detail: { tags: string }) => void;
    onTagAdded?: () => void;
    onTagRemoved?: () => void;
  }

  let {
    selectedPasswordItem,
    isEditing = $bindable(false),
    buttons,
    onReorderPending,
    onTagAdded,
    onTagRemoved
  }: Props = $props();

  let showChooser = $state(false);

  const REMOVE_ANIM_MS = 140;
  const ADD_ANIM_MS = 140;

  let removingTags = $state(new SvelteSet<string>());
  let recentlyAdded = $state(new SvelteSet<string>());
  
  // Initialize dndTags once from props. parent handles reset via {#key}
  const initialTags = (selectedPasswordItem?.tags || '')
    .split(',')
    .map(t => t.trim())
    .filter(Boolean);
  
  // Use a Set to ensure uniqueness for dndzone IDs
  const uniqueInitialTags = [...new Set(initialTags)];
  let dndTags = $state<{ id: string; text: string }[]>(uniqueInitialTags.map(t => ({ id: t, text: t })));
  
  let isDragging = $state(false);
  let justDragged = $state(false);

  // Derived state for what is actually currently "active"
  const workingTags = $derived(dndTags.map((t) => t.text));
  const remainingTags = $derived(buttons.filter((button) => !workingTags.includes(button.text)));
  const hasRemainingTags = $derived(remainingTags.length > 0);

  // Auto-close chooser logic - more defensive to avoid potential loops
  $effect(() => {
    if ((!hasRemainingTags || !isEditing) && showChooser) {
      showChooser = false;
    }
  });

  function getTagColor(tag: string): string {
    const btn = buttons.find((button) => button.text === tag);
    return btn ? btn.color : '#94a3b8';
  }

  function getTagIcon(tag: string): string {
    const btn = buttons.find((button) => button.text === tag);
    return btn?.icon || iconPaths.default;
  }

  async function addExistingTag(tagToAdd: string) {
    if (!selectedPasswordItem || workingTags.includes(tagToAdd)) return;

    recentlyAdded.add(tagToAdd);
    
    dndTags = [...dndTags, { id: tagToAdd, text: tagToAdd }];
    
    await tick();
    onReorderPending?.({ tags: dndTags.map(t => t.text).join(',') });

    setTimeout(() => {
      recentlyAdded.delete(tagToAdd);
    }, ADD_ANIM_MS + 60);

    onTagAdded?.();
  }

  async function removeTag(tagToRemove: string) {
    if (!selectedPasswordItem || isDragging || removingTags.has(tagToRemove)) return;

    if (justDragged) {
      justDragged = false;
      return;
    }

    removingTags.add(tagToRemove);

    setTimeout(async () => {
      // Check if tag is still in dndTags (it might have been removed by re-keying or other means)
      if (dndTags.some(t => t.text === tagToRemove)) {
        dndTags = dndTags.filter((tag) => tag.text !== tagToRemove);
        await tick();
        onReorderPending?.({ tags: dndTags.map(t => t.text).join(',') });
      }
      removingTags.delete(tagToRemove);
      onTagRemoved?.();
    }, REMOVE_ANIM_MS);
  }

  function handleConsider(e: CustomEvent<{ items: { id: string; text: string }[] }>) {
    isDragging = true;
    dndTags = e.detail.items;
  }

  async function handleFinalize(e: CustomEvent<{ items: { id: string; text: string }[] }>) {
    dndTags = e.detail.items;
    justDragged = true;
    setTimeout(() => (justDragged = false), 50);
    await tick();
    onReorderPending?.({ tags: dndTags.map(t => t.text).join(',') });
    isDragging = false;
  }
</script>

{#if selectedPasswordItem}
  <div class="tags-container">
    <div
      class="selected-tags"
      class:dragging={isDragging}
      use:dndzone={{
        items: dndTags,
        flipDurationMs: 300,
        dragDisabled: !isEditing,
        dropFromOthersDisabled: true,
        dropTargetStyle: { outline: 'none' },
        dropTargetClasses: []
      }}
      onconsider={handleConsider}
      onfinalize={handleFinalize}
    >
      {#each dndTags as item (item.id)}
        <div
          animate:flip={{ duration: 300 }}
          class="tag-item"
          class:editing={isEditing}
          class:removing={removingTags.has(item.text)}
          class:added={recentlyAdded.has(item.text)}
          style="--tag-color: {getTagColor(item.text)};"
          role={isEditing ? 'button' : undefined}
          aria-label={isEditing ? `Remove ${item.text}` : undefined}
          title={isEditing ? 'Click to remove' : undefined}
          onclick={() => {
            if (isEditing) removeTag(item.text);
          }}
        >
          <div class="tag-bg"></div>
          <span class="tag-content">
            <Icon path={getTagIcon(item.text)} size="14" color="currentColor" viewBox="0 0 48 48" />
            {item.text}
          </span>
        </div>
      {/each}
    </div>

    {#if isEditing && hasRemainingTags}
      <button
        type="button"
        class="add-toggle-btn"
        aria-expanded={showChooser}
        onclick={() => (showChooser = !showChooser)}
      >
        <Plus class="size-4" />
        <span>Add tag</span>
      </button>
      {#if showChooser}
        <div class="available-tags">
          <div class="chips chips-available">
            {#each remainingTags as tag (tag.id)}
              <button
                type="button"
                class="chooserChip"
                style="--tag-color: {tag.color};"
                onclick={() => addExistingTag(tag.text)}
              >
                <Icon path={tag.icon} size="14" color="currentColor" viewBox="0 0 48 48" />
                <span>{tag.text}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .tags-container {
    display: flex;
    gap: 12px;
    margin-top: 5px;
    justify-content: space-between;
    align-items: center;
    outline: none;
    -webkit-tap-highlight-color: transparent;
  }

  .selected-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    outline: none !important;
  }

  .selected-tags.dragging {
    flex-wrap: nowrap;
    overflow-x: auto;
    overscroll-behavior: contain;
    scrollbar-width: none;
  }
  .selected-tags.dragging::-webkit-scrollbar {
    display: none;
  }

  .tag-item {
    position: relative;
    display: flex;
    align-items: center;
    border-radius: 9px;
    padding: 2px 6px;
    color: var(--tag-color);
    font-size: 12px;
    font-weight: 500;
    overflow: hidden;
    transition: transform 120ms ease, box-shadow 120ms ease;
    touch-action: none;
    transform-origin: center;
    will-change: transform, box-shadow;
  }

  .tag-bg {
    position: absolute;
    inset: 0;
    border-radius: 9px;
    background: var(--tag-color);
    opacity: 0.3;
  }

  .tag-content {
    position: relative;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .tag-item :global(svg) {
    width: 16px;
    height: 16px;
    fill: currentColor;
  }

  .tag-item.editing {
    cursor: pointer;
  }

  .tag-item.editing:hover {
    outline: 1px solid #ef4444;
  }

  .tag-item.removing {
    animation: tagRemoveSlide 140ms ease-in-out forwards;
    pointer-events: none;
  }
  
  .tag-item.added {
    animation: tagAddSlide 140ms ease-out;
  }

  @keyframes tagRemoveSlide {
    0% { transform: translateX(0); opacity: 1; }
    100% { transform: translateX(6px); opacity: 0; }
  }
  
  @keyframes tagAddSlide {
    0% { transform: translateX(6px); opacity: 0; }
    100% { transform: translateX(0); opacity: 1; }
  }

  .add-toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 10px;
    border: 1px solid var(--btn-nav-border);
    background: #111;
    color: #c9ceda;
    cursor: pointer;
    font-size: 12px;
  }

  .add-toggle-btn:hover {
    filter: brightness(1.1);
  }

  .available-tags {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    margin-left: 4px;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }
  
  .chooserChip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 10px;
    border: 1px solid var(--btn-nav-border);
    background: #111;
    color: var(--tag-color);
    cursor: pointer;
    font-size: 12px;
    transition: transform 120ms ease;
  }

  .chooserChip:hover {
    filter: brightness(1.05);
    outline: 1px solid #22c55e;
  }
</style>
