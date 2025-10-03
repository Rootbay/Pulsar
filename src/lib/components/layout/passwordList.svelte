<script lang="ts">
  import { createEventDispatcher, tick, onDestroy } from 'svelte';
  import TagIcon from "../ui/TagIcon.svelte";
  import Favicon from "../ui/Favicon.svelte";
  import { iconPaths } from "$lib/icons";
  import { invoke } from '@tauri-apps/api/core';
  import { selectedTag, filterCategory } from '$lib/stores';
  import type { PasswordItem } from '../../../routes/+layout.ts';
  import { Search, X } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuTrigger
  } from '$lib/components/ui/context-menu';
  export let items: PasswordItem[] = [];
  export let buttons: any[] = [];
  export let selectedId: number | null = null;

  $: tagMap = new Map(buttons.map(b => [b.text, { color: b.color, icon: b.icon }]));

  function getFallback(item: any) {
    const tagNames = item.tags ? item.tags.split(',').map((tag: string) => tag.trim()) : [];
    if (tagNames.length > 0) {
        const firstTag = tagMap.get(tagNames[0]);
        if (firstTag) {
            return { icon: firstTag.icon, color: firstTag.color };
        }
    }
    return { icon: iconPaths.default, color: 'var(--sidebar-border)' };
  }

  let isResizing = false;
  let navWidth = 360;
  let startX = 0;
  let searchTerm = '';
  

  const dispatch = createEventDispatcher<{
    select: PasswordItem;
    createEntry: void;
    editEntry: PasswordItem;
    removeEntry: PasswordItem;
  }>();

  $: itemsCount = items.length;

  $: filteredItems = items.filter((item) => {
    const matchesTag =
      $selectedTag === null ||
      (item.tags &&
        (typeof item.tags === 'string'
          ? item.tags.split(',').map((tag: string) => tag.trim())
          : item.tags
        ).includes($selectedTag));

    if (!matchesTag) {
      return false;
    }

    if (searchTerm === '') {
      return true;
    }

    const normalizedTerm = searchTerm.toLowerCase();
    const titleMatch = item.title?.toLowerCase().includes(normalizedTerm) ?? false;
    const usernameMatch = item.username?.toLowerCase().includes(normalizedTerm) ?? false;
    return titleMatch || usernameMatch;
  });

  function getTagNames(item: any): string[] {
    return item?.tags ? (typeof item.tags === 'string' ? item.tags.split(',').map((t: string) => t.trim()) : item.tags) : [];
  }

  function isPinned(item: any): boolean {
    const tags = getTagNames(item).map((t) => t.toLowerCase());
    return tags.includes('pinned') || tags.includes('pin');
  }

  function isSameDay(a: Date, b: Date): boolean {
    return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
  }

  function isToday(dateStr: string): boolean {
    const d = new Date(dateStr);
    const now = new Date();
    return isSameDay(d, now);
  }

  function isYesterday(dateStr: string): boolean {
    const d = new Date(dateStr);
    const y = new Date();
    y.setDate(y.getDate() - 1);
    return isSameDay(d, y);
  }

  $: sectionedItems = (() => {
    const pinned: any[] = [];
    const today: any[] = [];
    const yesterday: any[] = [];
    const earlier: any[] = [];
    for (const item of filteredItems) {
      if (isPinned(item)) {
        pinned.push(item);
        continue;
      }
      if (item?.updated_at && isToday(item.updated_at)) {
        today.push(item);
      } else if (item?.updated_at && isYesterday(item.updated_at)) {
        yesterday.push(item);
      } else {
        earlier.push(item);
      }
    }
    return [
      { title: 'Pinned', items: pinned },
      { title: 'Today', items: today },
      { title: 'Yesterday', items: yesterday },
      { title: 'Earlier', items: earlier },
    ];
  })();

  $: {
    const visibleIds = new Set(filteredItems.map((i) => i.id));
    if (filteredItems.length === 0) {
      selectedItemId = null;
    } else if (selectedItemId === null || !visibleIds.has(selectedItemId)) {
      const firstSection = sectionedItems.find((s) => s.items.length > 0);
      if (firstSection) {
        const firstItem = firstSection.items[0];
        selectItem(firstItem);
      }
    }
  }

  function handleSearchInput(event: Event) {
    searchTerm = (event.target as HTMLInputElement).value;
  }

  function clearSearch() {
    searchTerm = '';
  }

  let selectedItemId: number | null = null;

  let showSkeleton = false;
  let skeletonTimer: any = null;
  let lastSkeletonKey = '';
  $: currentSkeletonKey = `${$selectedTag ?? 'all'}|${$filterCategory}`;
  $: if (currentSkeletonKey !== lastSkeletonKey) {
    lastSkeletonKey = currentSkeletonKey;
    (async () => {
      await tick();
      const count = filteredItems.length;
      if (count > 0) {
        showSkeleton = true;
        clearTimeout(skeletonTimer);
        skeletonTimer = setTimeout(() => {
          showSkeleton = false;
        }, 200);
      }
    })();
  }

  onDestroy(() => {
    clearTimeout(skeletonTimer);
  });

  function handleCreateEntry() {
    dispatch('createEntry');
  }

  function handleEditEntry(item: PasswordItem) {
    dispatch('editEntry', item);
  }

  function handleRemoveEntry(item: PasswordItem) {
    dispatch('removeEntry', item);
  }

  async function handlePinToggle(item: PasswordItem) {
    const id = item.id;
    const parts = getTagNames(item);
    const lower = parts.map((t) => t.toLowerCase());
    const alreadyPinnedIdx = lower.findIndex((t) => t === 'pinned' || t === 'pin');
    let newTagsArr: string[];
    if (alreadyPinnedIdx >= 0) {
      newTagsArr = parts.filter((t, i) => lower[i] !== 'pinned' && lower[i] !== 'pin');
    } else {
      newTagsArr = ['Pinned', ...parts];
    }
    const newTags = newTagsArr.join(',');
    try {
      await invoke('update_password_item_tags', { id, tags: newTags });
      const idx = items.findIndex((it) => it.id === id);
      if (idx !== -1) {
        items = [
          ...items.slice(0, idx),
          { ...items[idx], tags: newTags },
          ...items.slice(idx + 1)
        ];
      }
    } catch (e) {
      console.error('Failed to toggle pin:', e);
      alert(`Failed to ${alreadyPinnedIdx >= 0 ? 'unpin' : 'pin'} item: ${e}`);
    }
  }
  function startResize(e: MouseEvent) {
    isResizing = true;
    startX = e.clientX;
    const navElement = document.querySelector('.passwordList') as HTMLElement;
    if (navElement) {
      navWidth = navElement.offsetWidth;
    }
    window.addEventListener('mousemove', resize);
    window.addEventListener('mouseup', stopResize);
  }

  function resize(e: MouseEvent) {
    if (!isResizing) return;
    const newWidth = Math.max(200, Math.min(600, navWidth + (e.clientX - startX)));
    document.documentElement.style.setProperty('--passwordList-width', `${newWidth}px`);
  }

  function stopResize() {
    isResizing = false;
    window.removeEventListener('mousemove', resize);
    window.removeEventListener('mouseup', stopResize);
  }

  $: if (selectedId !== null && selectedId !== selectedItemId) {
    selectedItemId = selectedId;
  }

  function selectItem(item: any) {
    selectedItemId = item.id;
    dispatch('select', item);
  }

</script>

<nav class="passwordList">
  <ContextMenu>
    <ContextMenuTrigger>
      <div class="navInner" aria-busy={showSkeleton} role="region" aria-label="Password list" tabindex="-1">
        <div class="topControls" role="region" aria-label="Navigation controls">
          <div class="searchContainer">
            <Button
              type="button"
              class="searchBtn h-8 w-8 shrink-0 text-muted-foreground hover:text-foreground"
              variant="ghost"
              size="icon"
              aria-label="Search"
              title="Search"
            >
              <Search class="w-5 h-5" />
            </Button>
            <Input
              type="text"
              placeholder="Search..."
              class="searchInput h-8 border-0 bg-transparent px-2 text-sm shadow-none focus-visible:border-0 focus-visible:ring-0 focus-visible:ring-offset-0"
              bind:value={searchTerm}
              oninput={handleSearchInput}
            />
            {#if searchTerm}
              <Button
                type="button"
                variant="ghost"
                size="icon"
                class="clearSearchBtn h-8 w-8 shrink-0 text-muted-foreground hover:text-foreground"
                onclick={clearSearch}
                aria-label="Clear search"
              >
                <X class="w-4 h-4" />
              </Button>
            {/if}
          </div>

          <div class="segButtons" role="tablist" aria-label="Filter tabs">
            <Button
              type="button"
              class="segBtn"
              role="tab"
              variant="ghost"
              size="sm"
              aria-selected={$filterCategory === 'all'}
              onclick={() => filterCategory.set('all')}
            >
              All <span class="count">({itemsCount})</span>
            </Button>
            <Button
              type="button"
              class="segBtn"
              role="tab"
              variant="ghost"
              size="sm"
              aria-selected={$filterCategory === 'recent'}
              onclick={() => filterCategory.set('recent')}
            >
              Recently
            </Button>
          </div>
        </div>

        <ScrollArea class="navScroll">
          <div class="scrollContent">
            {#if filteredItems.length === 0}
              <ul class="itemList" role="list">
                <li class="no-items-message">
                  {#if $selectedTag}
                    No passwords found for tag "{$selectedTag}".
                  {:else}
                    No passwords found. Create a new entry or select a tag.
                  {/if}
                </li>
              </ul>
            {:else}
              {#each sectionedItems as section (section.title)}
                {#if section.items.length}
                  <div class="sectionTitle" role="heading" aria-level="2">{section.title}</div>
                  <ul class="itemList" role="list">
                    {#if showSkeleton}
                      {#each section.items as placeholder (placeholder.id)}
                        <li class="item" aria-hidden="true">
                          <div class="itemLink" role="presentation">
                            <div class="itemLeft">
                              <Skeleton class="h-7 w-7 rounded-md opacity-70" aria-hidden="true" />
                              <div class="itemTexts">
                                <Skeleton class="h-3 w-36 rounded-md opacity-80" aria-hidden="true" />
                                <Skeleton class="mt-2 h-3 w-24 rounded-md opacity-60" aria-hidden="true" />
                              </div>
                            </div>
                          </div>
                        </li>
                      {/each}
                    {:else}
                      {#each section.items as item (item.id)}
                        {@const fallback = getFallback(item)}
                        <li class="item" class:selected={selectedItemId === item.id} role="listitem">
                          <ContextMenu>
                            <ContextMenuTrigger>
                              <a
                                href={item.url}
                                class="itemLink"
                                onclick={(event: MouseEvent) => { event.preventDefault(); selectItem(item); }}
                                draggable="false"
                              >
                                <div class="itemLeft">
                                  <Favicon
                                    url={item.url}
                                    title={item.title}
                                    fallbackIcon={fallback.icon}
                                    fallbackColor={fallback.color}
                                  />
                                  <div class="itemTexts">
                                    <div class="itemTitle">{item.title}</div>
                                    <div class="itemDesc">{item.username}</div>
                                  </div>
                                </div>
                                <div class="itemTags">
                                  <TagIcon
                                    tagNames={item.tags ? item.tags.split(',').map((tag: string) => tag.trim()) : []}
                                    {tagMap}
                                  />
                                </div>
                              </a>
                            </ContextMenuTrigger>
                            <ContextMenuContent class="w-48">
                              <ContextMenuItem onSelect={() => handlePinToggle(item)}>{isPinned(item) ? 'Unpin' : 'Pin'}</ContextMenuItem>
                              <ContextMenuSeparator />
                              <ContextMenuItem onSelect={() => handleEditEntry(item)}>Edit Entry</ContextMenuItem>
                              <ContextMenuItem
                                class="text-destructive focus:text-destructive data-[highlighted]:bg-destructive/10"
                                onSelect={() => handleRemoveEntry(item)}
                              >
                                Remove Entry
                              </ContextMenuItem>
                            </ContextMenuContent>
                          </ContextMenu>
                        </li>
                      {/each}
                    {/if}
                  </ul>
                {/if}
              {/each}
            {/if}
          </div>
        </ScrollArea>
      </div>
    </ContextMenuTrigger>
    <ContextMenuContent class="w-48">
      <ContextMenuItem onSelect={handleCreateEntry}>Create Entry</ContextMenuItem>
    </ContextMenuContent>
  </ContextMenu>
  <button class="resizer" onmousedown={startResize} aria-label="Resize sidebar"></button>
</nav>

<style>
  .passwordList {
    --passwordlist-surface: color-mix(in oklch, var(--sidebar) 90%, var(--background) 10%);
    --passwordlist-elevated: color-mix(in oklch, var(--sidebar) 80%, var(--background) 20%);
    --passwordlist-hover: color-mix(in oklch, var(--sidebar) 65%, var(--background) 35%);
    --passwordlist-strong-text: var(--sidebar-foreground);
    --passwordlist-muted-text: color-mix(in oklch, var(--sidebar-foreground) 60%, transparent);
    --passwordlist-subtle-text: color-mix(in oklch, var(--sidebar-foreground) 35%, transparent);
    --passwordlist-border: var(--sidebar-border);
    --passwordlist-scroll-thumb: color-mix(in oklch, var(--sidebar-border) 75%, var(--sidebar) 25%);
    --passwordlist-skeleton-base: color-mix(in oklch, var(--passwordlist-elevated) 88%, var(--background) 12%);
    --passwordlist-skeleton-highlight: color-mix(in oklch, var(--passwordlist-elevated) 70%, var(--background) 30%);
    width: var(--passwordList-width);
    min-width: 150px;
    position: relative;
    background: var(--passwordlist-surface);
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .navInner {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    flex-grow: 1;
    min-height: 0;
    overflow: hidden;
  }

  .scrollContent {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 0 12px 12px;
    box-sizing: border-box;
  }

  .topControls {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    margin-top: 16px;
    margin-bottom: 30px;
    width: 100%;
    gap: 25px;
    padding: 0 12px;
    box-sizing: border-box;
  }

  .searchContainer {
    display: flex;
    align-items: center;
    background: var(--passwordlist-elevated);
    border-radius: 16px;
    height: 32px;
    width: 100%;
    padding: 0 5px;
    box-sizing: border-box;
  }

  .segButtons {
    display: flex;
    gap: 12px;
    width: 100%;
  }

  .sectionTitle {
    margin-left: 12px;
    font-size: 12px;
    color: var(--passwordlist-muted-text);
    margin-top: 0;
  }

  .itemList {
    margin-top: 8px;
    list-style: none;
    padding: 0 0 6px 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-grow: 0;
    overflow: visible;
  }

  .itemList::-webkit-scrollbar {
    width: 8px;
  }

  .itemList::-webkit-scrollbar-track {
    background: transparent;
  }

  .itemList::-webkit-scrollbar-thumb {
    background-color: var(--passwordlist-scroll-thumb);
    border-radius: 10px;
    border: 2px solid transparent;
    background-clip: padding-box;
  }

  .itemList::-webkit-scrollbar-thumb:hover {
    background-color: color-mix(in oklch, var(--passwordlist-scroll-thumb) 85%, var(--passwordlist-hover) 15%);
  }

  .itemList::-webkit-scrollbar-button {
    display: none;
  }

  .item {
    background: transparent;
    height: 47px;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-sizing: border-box;
    cursor: pointer;
  }

  .item:hover {
    background: var(--passwordlist-hover);
  }

  .item.selected {
    background: var(--passwordlist-hover);
  }

  .itemLink {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    height: 100%;
    text-decoration: none;
    color: inherit;
    padding-left: 12px;
    padding-right: 12px;
    padding-top: 5px;
    padding-bottom: 7px;
    box-sizing: border-box;
  }

  .itemLeft {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .itemTexts {
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .itemTitle {
    font-size: 14px;
    color: var(--passwordlist-strong-text);
    line-height: 1;
    user-select: none;
  }

  .itemDesc {
    margin-top: 3px;
    font-size: 12px;
    color: var(--passwordlist-muted-text);
    line-height: 1;
    user-select: none;
  }

  .itemTags {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .navInner[aria-busy="true"] {
    pointer-events: none;
  }

  .no-items-message {
    padding: 20px;
    text-align: center;
    color: var(--passwordlist-subtle-text);
    font-style: italic;
  }

  .resizer {
    width: 5px;
    height: 100%;
    background: transparent;
    position: absolute;
    right: 0;
    top: 0;
    cursor: ew-resize;
    z-index: 6;
    border: none;
    padding: 0;
  }

  @media (pointer: coarse) {
    .resizer { width: 12px; }
  }
</style>

