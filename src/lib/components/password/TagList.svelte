<svelte:options runes />

<script lang="ts">
  import type { PasswordItem } from '$lib/types/password';
  import Icon from '../ui/Icon.svelte';
  import { iconPaths } from '$lib/icons';
  import { invoke } from '@tauri-apps/api/core';
  import { dndzone } from 'svelte-dnd-action';
  import { flip } from 'svelte/animate';

  interface Props {
    selectedPasswordItem: PasswordItem | null;
    isEditing: boolean;
    buttons: { text: string; color: string; icon: string; id?: string }[];

    onReorderPending?: (detail: { tags: string }) => void;
    onTagAdded?: () => void;
    onTagRemoved?: () => void;
  }

  let {
    selectedPasswordItem = $bindable<PasswordItem | null>(),
    isEditing = $bindable(false),
    buttons,
    onReorderPending,
    onTagAdded,
    onTagRemoved
  }: Props = $props();

  let showChooser = $state(false);

  const REMOVE_ANIM_MS = 140;
  const ADD_ANIM_MS = 140;

  let removingTags = $state(new Set<string>());
  let recentlyAdded = $state(new Set<string>());
  let dndTags = $state<{ id: string; text: string }[]>([]);
  let justDragged = $state(false);
  let suppressSync = $state(false);
  let isDragging = $state(false);
  let pendingReorder = $state<string | null>(null);

  let workingTags = $state<string[]>([]);

  $effect(() => {
    const base = pendingReorder !== null
      ? pendingReorder
      : (dndTags.length
          ? dndTags.map((t) => t.text).join(',')
          : (selectedPasswordItem?.tags ?? ''));

    workingTags = base
      .split(',')
      .map((t) => t.trim())
      .filter((value) => value.length > 0);
  });

  $effect(() => {
    if (suppressSync || pendingReorder !== null) {
      return;
    }
    const list = selectedPasswordItem?.tags
      ?.split(',')
      .map((t: string) => t.trim())
      .filter((value: string) => value.length > 0) ?? [];

    const needsSync =
      dndTags.length !== list.length ||
      dndTags.some((t, i) => t.text !== list[i]);

    if (needsSync) {
      dndTags = list.map((t: string) => ({ id: t, text: t }));
    }
  });

  $effect(() => {
    if (!isEditing) {
      pendingReorder = null;
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

  function addExistingTag(tagToAdd: string) {
    if (!selectedPasswordItem) return;
    const tags = [...workingTags];
    const next = tagToAdd.trim();
    if (!next) return;
    if (!buttons.some((button) => button.text === next)) {
      alert('Only existing tags can be added.');
      return;
    }
    if (tags.includes(next)) {
      return;
    }

    recentlyAdded.add(next);
    const updatedTags = [...tags, next].join(',');
    pendingReorder = updatedTags;

    if (!dndTags.some((tag) => tag.text === next)) {
      dndTags = [...dndTags, { id: next, text: next }];
    }

    onReorderPending?.({ tags: updatedTags });
    setTimeout(() => {
      recentlyAdded.delete(next);
    }, ADD_ANIM_MS + 60);

    onTagAdded?.();
  }

  function removeTag(tagToRemove: string) {
    if (!selectedPasswordItem) return;
    if (isDragging) return;
    if (justDragged) {
      justDragged = false;
      return;
    }
    if (removingTags.has(tagToRemove)) return;

    const tags = [...workingTags];
    const updatedTags = tags.filter((tag) => tag !== tagToRemove).join(',');
    pendingReorder = updatedTags;
    onReorderPending?.({ tags: updatedTags });

    removingTags.add(tagToRemove);
    setTimeout(() => {
      dndTags = dndTags.filter((tag) => tag.text !== tagToRemove);
      onTagRemoved?.();
      removingTags.delete(tagToRemove);
    }, REMOVE_ANIM_MS);
  }

  function handleConsider(e: CustomEvent<{ items: { id: string; text: string }[] }>) {
    isDragging = true;
    suppressSync = true;
    dndTags = e.detail.items;
  }

  function handleFinalize(e: CustomEvent<{ items: { id: string; text: string }[] }>) {
    dndTags = e.detail.items;
    justDragged = true;
    setTimeout(() => (justDragged = false), 50);
    const newOrder = dndTags.map(t => t.text).join(',');
    pendingReorder = newOrder;
    onReorderPending?.({ tags: newOrder });
    suppressSync = false;
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
            <Icon path={getTagIcon(item.text)} size="14" color="currentColor" />
            {item.text}
          </span>
        </div>
      {/each}
    </div>

    {#if isEditing}
      <button
        type="button"
        class="add-toggle-btn"
        aria-expanded={showChooser}
        onclick={() => (showChooser = !showChooser)}
      >
        <Icon path={iconPaths.plus} size="14" color="currentColor" />
        <span>Add tag</span>
      </button>
      {#if showChooser}
        <div class="available-tags">
          <div class="chips chips-available">
            {#each buttons.filter((button) => !workingTags.includes(button.text)) as tag (tag.id)}
              <button
                type="button"
                class="chooserChip"
                style="--tag-color: {tag.color};"
                onclick={() => addExistingTag(tag.text)}
              >
                <Icon path={tag.icon} size="14" color="currentColor" />
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
    .tags-container:focus,
    .tags-container:focus-visible,
    .tags-container:focus-within { outline: none !important; box-shadow: none !important; border: none !important; }

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
    .selected-tags.dragging::-webkit-scrollbar { display: none; }

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
        transition: outline-color 120ms ease, transform 120ms ease, box-shadow 120ms ease;
        touch-action: none;
        -webkit-tap-highlight-color: transparent;
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

    .tag-item { user-select: none; }
    .tag-item.editing { cursor: pointer; }

    .tag-item.editing:hover {
        outline: 1px solid #ef4444;
        box-shadow: 0 0 0 1px rgba(239, 68, 68, 0.15) inset;
    }

    .selected-tags.dragging .tag-item.editing:hover {
        animation: none;
        box-shadow: none;
        outline: none;
    }
    .selected-tags.dragging .tag-item.editing:hover::after { display: none; }
    .selected-tags.dragging .tag-item { transition: none; }
    .selected-tags:focus, .selected-tags:focus-visible, .selected-tags:focus-within { outline: none !important; box-shadow: none !important; }
    .tag-item.removing { animation: tagRemoveSlide 140ms ease-in-out forwards; pointer-events: none; }
    .tag-item.removing::before {
        content: '';
        position: absolute;
        top: 50%; left: 50%;
        width: 16px; height: 16px;
        border-radius: 999px;
        background: rgba(239,68,68,0.20);
        border: 1px solid #ef4444;
        transform: translate(-50%, -50%) scale(0.2);
        z-index: 1;
        animation: tagRemoveCircle 140ms ease-out forwards;
        pointer-events: none;
    }
    .tag-item.added { animation: tagAddSlide 140ms ease-out; }
    .tag-item.editing:hover::after {
        content: '';
        position: absolute;
        inset: 0;
        background: rgba(239, 68, 68, 0.08);
        pointer-events: none;
    }

    .tag-item.editing:active { outline: none !important; box-shadow: none !important; }
    .tag-item:focus,
    .tag-item:focus-visible { outline: none !important; box-shadow: none !important; }
    .selected-tags.dragging .tag-item,
    .selected-tags.dragging .tag-item:hover,
    .selected-tags.dragging .tag-item:active,
    .selected-tags.dragging .tag-item:focus { outline: none !important; box-shadow: none !important; }

    :global(.svelte-dnd-action-ghost),
    :global(.svelte-dnd-action-dragged),
    :global(.svelte-dnd-action-placeholder),
    :global(.svelte-dnd-action-dropzone),
    :global(.dnd-shadow) {
        outline: none !important;
        box-shadow: none !important;
        border: none !important;
    }

    .add-toggle-btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 4px 8px;
        border-radius: 10px;
        border: 1px solid var(--btn-nav-border);
        background: var(--near-black);
        color: #c9ceda;
        cursor: pointer;
        font-size: 12px;
    }

    .add-toggle-btn:hover { filter: brightness(1.1); }

    .available-tags {
        display: flex;
        align-items: center;
        gap: 10px;
        flex-wrap: wrap;
        margin-left: 4px;
    }

    .chips { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
    .chooserChip {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 4px 8px;
        border-radius: 10px;
        border: 1px solid var(--btn-nav-border);
        background: var(--near-black);
        color: var(--tag-color);
        cursor: pointer;
        user-select: none;
        transition: outline-color 120ms ease, transform 120ms ease, box-shadow 120ms ease;
        font-size: 12px;
    }

    .chooserChip:hover {
        filter: brightness(1.05);
        outline: 1px solid #22c55e;
        box-shadow: 0 0 0 1px rgba(34,197,94,0.15) inset;
        animation: chipHoverSlide 260ms ease-in-out 1;
    }
    @keyframes chipHoverSlide {
        0% { transform: translateX(0); }
        50% { transform: translateX(3px); }
        100% { transform: translateX(0); }
    }
    @keyframes tagRemoveSlide {
        0% { transform: translateX(0); opacity: 1; }
        100% { transform: translateX(6px); opacity: 0; }
    }
    @keyframes tagRemoveCircle {
        0% { transform: translate(-50%, -50%) scale(0.2); opacity: 0.8; }
        100% { transform: translate(-50%, -50%) scale(1.05); opacity: 0; }
    }
    @keyframes tagAddSlide {
        0% { transform: translateX(6px); opacity: 0; }
        100% { transform: translateX(0); opacity: 1; }
    }
    .chooserChip :global(svg) { width: 14px; height: 14px; display: block; }
    
</style>

