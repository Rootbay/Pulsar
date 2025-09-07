<script lang="ts">
    import type { PasswordItem } from '../../../routes/+layout.ts';
    import Icon from '../ui/Icon.svelte';
    import { iconPaths } from '$lib/icons';
    import { invoke } from '@tauri-apps/api/core';
    import { createEventDispatcher } from 'svelte';
    import { dndzone } from 'svelte-dnd-action';
    import { flip } from 'svelte/animate';

    const dispatch = createEventDispatcher();

    export let selectedPasswordItem: PasswordItem | null;
    export let isEditing: boolean;
    
   export let buttons: any[];

    let showChooser = false;

    // Animation helpers
    const REMOVE_ANIM_MS = 140;
    const ADD_ANIM_MS = 140;
    let removingTags: Set<string> = new Set();
    let recentlyAdded: Set<string> = new Set();
    let dndTags: { id: string; text: string }[] = [];
    let justDragged = false;
    let suppressSync = false;
    let isDragging = false;
    let pendingReorder: string | null = null;
    let workingTags: string[] = [];
    $: workingTags = (() => {
        const base = (pendingReorder !== null
            ? pendingReorder
            : (dndTags.length ? dndTags.map(t => t.text).join(',') : (selectedPasswordItem?.tags ?? '')));
        return base.split(',').map((t: string) => t.trim()).filter(Boolean);
    })();
    $: {
        // While dragging or when a pending reorder exists, keep local order
        if (suppressSync || pendingReorder !== null) {
            // skip syncing from selectedPasswordItem.tags
        } else {
            const list = selectedPasswordItem?.tags?.split(',').map(t => t.trim()).filter(Boolean) ?? [];
            const needsSync = dndTags.length !== list.length || dndTags.some((t, i) => t.text !== list[i]);
            if (needsSync) {
                // use stable ids equal to tag text (tags are unique)
                dndTags = list.map((t) => ({ id: t, text: t }));
            }
        }
    }

    // Clear local pending state when exiting edit mode
    $: if (!isEditing) {
        pendingReorder = null;
    }
    // removed text search per request

    function getTagColor(tag: string) {
        const button = buttons.find(b => b.text === tag);
        return button ? button.color : '#94a3b8';
    }

    function getTagIcon(tag: string) {
        const button = buttons.find(b => b.text === tag);
        return button ? button.icon : iconPaths.default;
    }

    // Only allow adding existing tags from `buttons`
    async function addExistingTag(tagToAdd: string) {
        if (!selectedPasswordItem) return;

        const tags = workingTags;
        const next = tagToAdd.trim();
        if (!next) return;
        if (!buttons.some(b => b.text === next)) {
            alert('Only existing tags can be added.');
            return;
        }
        if (tags.includes(next)) {
            return; // already present, no-op
        }

        // mark as recently added to play entry animation
        recentlyAdded.add(next);
        const updatedTags = [...tags, next].join(',');
        // enter pending mode (do not persist yet)
        pendingReorder = updatedTags;
        if (!dndTags.some(t => t.text === next)) {
            dndTags = [...dndTags, { id: next, text: next }];
        }
        dispatch('reorderPending', { tags: updatedTags });
        // clear highlight after animation
        setTimeout(() => { recentlyAdded.delete(next); }, ADD_ANIM_MS + 60);
        dispatch('tagAdded');
    }

    async function removeTag(tagToRemove: string) {
        if (!selectedPasswordItem) return;
        if (isDragging) return;
        if (justDragged) { justDragged = false; return; }
        if (removingTags.has(tagToRemove)) return;

        const tags = workingTags;
        const updatedTags = tags.filter(tag => tag !== tagToRemove).join(',');

        // Immediately mark pending new order so Save can persist even if clicked quickly
        pendingReorder = updatedTags;
        dispatch('reorderPending', { tags: updatedTags });

        // play removal animation, then update UI list
        removingTags.add(tagToRemove);
        setTimeout(() => {
            dndTags = dndTags.filter((t) => t.text !== tagToRemove);
            dispatch('tagRemoved');
            removingTags.delete(tagToRemove);
        }, REMOVE_ANIM_MS);
    }
</script>

{#if selectedPasswordItem}
  <div class="tags-container">
    <div class="selected-tags" class:dragging={isDragging}
         use:dndzone={{
            items: dndTags,
            flipDurationMs: 300,
            dragDisabled: !isEditing,
            dropFromOthersDisabled: true,
            // kill the default yellow drop-target outline from svelte-dnd-action
            dropTargetStyle: { outline: 'none' },
            dropTargetClasses: []
         }}
         on:consider={(e) => { isDragging = true; suppressSync = true; dndTags = e.detail.items; }}
         on:finalize={(e) => {
            dndTags = e.detail.items;
            justDragged = true;
            setTimeout(() => (justDragged = false), 50);
            const newOrder = dndTags.map(t => t.text).join(',');
            pendingReorder = newOrder;
            // notify parent that a reorder is pending save
            dispatch('reorderPending', { tags: newOrder });
            suppressSync = false;
            isDragging = false;
         }}
    >
      {#each dndTags as item (item.id)}
        <div animate:flip={{ duration: 300 }}
             class="tag-item"
             class:editing={isEditing}
             class:removing={removingTags.has(item.text)}
             class:added={recentlyAdded.has(item.text)}
             style="--tag-color: {getTagColor(item.text)};"
             role={isEditing ? 'button' : undefined}
             aria-label={isEditing ? `Remove ${item.text}` : undefined}
             title={isEditing ? 'Click to remove' : undefined}
             on:click={() => { if (isEditing) removeTag(item.text); }}
        >
          <div class="tag-bg"></div>
          <span class="tag-content">
            <Icon path={getTagIcon(item.text)} size="14" color="currentColor" />
            {item.text}
            <!-- drag handle removed; whole tag is draggable in edit mode -->
          </span>
        </div>
      {/each}
    </div>

    {#if isEditing}
      <button type="button" class="add-toggle-btn" aria-expanded={showChooser} on:click={() => (showChooser = !showChooser)}>
        <Icon path={iconPaths.plus} size="14" color="currentColor" />
        <span>Add tag</span>
      </button>
      {#if showChooser}
        <div class="available-tags">
          <div class="chips chips-available">
            {#each (buttons || []).filter(b => !workingTags.includes(b.text)) as tag (tag.id)}
              <button type="button" class="chooserChip" style="--tag-color: {tag.color};" on:click={() => addExistingTag(tag.text)}>
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
        /* prevent any UA focus outline on container */
        outline: none;
        -webkit-tap-highlight-color: transparent;
    }
    .tags-container:focus,
    .tags-container:focus-visible,
    .tags-container:focus-within { outline: none !important; box-shadow: none !important; border: none !important; }

    .tags-wrapper {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
    }
    .selected-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
        outline: none !important;
    }
    /* Lock wrapping during drag so movement stays in a straight line */
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
    /* removed drag-handle styles */

    .tag-item :global(svg) {
        width: 16px;
        height: 16px;
        fill: currentColor;
    }

    /* Remove interaction: click tag to remove (edit mode only) */
    .tag-item { user-select: none; }
    .tag-item.editing { cursor: pointer; }
    /* Remove hover border/animation to avoid conflict with drag */
    /* Normal hover styling (active when not dragging) */
    .tag-item.editing:hover {
        outline: 1px solid #ef4444;
        box-shadow: 0 0 0 1px rgba(239, 68, 68, 0.15) inset;
    }
    /* Disable hover effects and transitions while dragging to prevent jitter */
    /* While dragging, suppress hover visuals to avoid jitter */
    .selected-tags.dragging .tag-item.editing:hover {
        animation: none;
        box-shadow: none;
        outline: none;
    }
    .selected-tags.dragging .tag-item.editing:hover::after { display: none; }
    .selected-tags.dragging .tag-item { transition: none; }
    /* Remove focus outlines on the drag rail */
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
    /* Hover overlay (disabled during drag via rule above) */
    .tag-item.editing:hover::after {
        content: '';
        position: absolute;
        inset: 0;
        background: rgba(239, 68, 68, 0.08);
        pointer-events: none;
    }

    /* Suppress hover border when actively pressing to start a drag */
    .tag-item.editing:active { outline: none !important; box-shadow: none !important; }
    /* Also suppress any default focus ring that may flash before drag starts */
    .tag-item:focus,
    .tag-item:focus-visible { outline: none !important; box-shadow: none !important; }
    /* Extra safety: suppress all outlines/box-shadows on tags while dragging */
    .selected-tags.dragging .tag-item,
    .selected-tags.dragging .tag-item:hover,
    .selected-tags.dragging .tag-item:active,
    .selected-tags.dragging .tag-item:focus { outline: none !important; box-shadow: none !important; }

    /* Remove any default outlines from svelte-dnd-action helper elements */
    :global(.svelte-dnd-action-ghost),
    :global(.svelte-dnd-action-dragged),
    :global(.svelte-dnd-action-placeholder),
    :global(.svelte-dnd-action-dropzone),
    :global(.dnd-shadow) {
        outline: none !important;
        box-shadow: none !important;
        border: none !important;
    }

    /* removed hover corner circle hint */

    .add-tag-button {
        background: transparent;
        border: none;
        color: #888;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2px;
    }

    .add-tag-button:hover {
        color: var(--white);
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

    /* Available tag chooser (fits inline with tags) */
    .available-tags {
        display: flex;
        align-items: center;
        gap: 10px;
        flex-wrap: wrap;
        margin-left: 4px;
    }

    .chips { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
    .chips-available { }

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
 
