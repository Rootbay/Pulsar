<svelte:options runes />

<script lang="ts">
  import type { PasswordItem } from '$lib/types/password';
  import TagList from './TagList.svelte';
  import { Ellipsis } from '@lucide/svelte';
  import { iconPaths } from '$lib/icons';
  import Favicon from '../ui/Favicon.svelte';
  import { Button } from '$lib/components/ui/button';

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
    if (!name) return iconPaths.globe;
    const btn = (buttons || []).find((b: TagOption) => b.text === name);
    return btn?.icon || iconPaths.globe;
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
    if (selectedPasswordItem && prevColor !== displayColor) {
      const oldColor = prevColor;
      prevColor = displayColor;
      
      if (oldColor !== null && !pulse) {
        pulse = true;
        setTimeout(() => {
          pulse = false;
        }, 600); // Match animation duration
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
        />
        <h2 class="header-title">
          {selectedPasswordItem.title}
        </h2>
        <span class="color-pulse-bg" aria-hidden="true" class:pulsing={pulse}></span>
      </div>

      {#key selectedPasswordItem ? `${selectedPasswordItem.id}-${selectedPasswordItem.tags}` : 'none'}
        <TagList
          {selectedPasswordItem}
          bind:isEditing
          {buttons}
          onReorderPending={onTagsReorderedPending}
        />
      {/key}
    </div>

    <div class="flex items-center gap-2">
      {#if isEditing}
        <Button size="sm" onclick={() => onSave?.()}>Save</Button>
      {/if}
      
      <div class="relative">
        <Button 
          variant="ghost" 
          size="icon" 
          class="h-10 w-10 text-muted-foreground hover:text-foreground"
          onclick={() => (showMoreDropdown = !showMoreDropdown)}
        >
          <Ellipsis class="size-6" />
        </Button>
        
        {#if showMoreDropdown}
          <div 
            class="fixed inset-0 z-40" 
            role="presentation"
            onclick={() => showMoreDropdown = false} 
          ></div>
          <div class="absolute right-0 top-full mt-1 z-50 min-w-[140px] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md">
            <button
              class="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
              onclick={() => {
                enterEditMode();
                showMoreDropdown = false;
              }}
            >
              Edit Entry
            </button>
            <button
              class="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-destructive/10 hover:text-destructive text-destructive"
              onclick={() => {
                onRemoveEntry?.(selectedPasswordItem?.id);
                showMoreDropdown = false;
              }}
            >
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

  .title-container :global(.itemImgContainer) {
    position: relative;
    z-index: 1;
  }

  .color-pulse-bg {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%) scale(0.85);
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--display-color);
    filter: blur(8px);
    opacity: 0;
    pointer-events: none;
    z-index: 0;
  }
  .color-pulse-bg.pulsing {
    animation: smoothGlowPulse 600ms ease-in-out forwards;
  }

  @keyframes smoothGlowPulse {
    0% { opacity: 0; transform: translateY(-50%) scale(0.85); }
    30% { opacity: 0.45; transform: translateY(-50%) scale(1.25); }
    100% { opacity: 0; transform: translateY(-50%) scale(1.05); }
  }
</style>
