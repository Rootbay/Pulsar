<svelte:options runes />

<script lang="ts">
  import { onDestroy, tick } from 'svelte';
  import { SvelteDate } from 'svelte/reactivity';
  import TagIcon from '../ui/TagIcon.svelte';
  import Favicon from '../ui/Favicon.svelte';
  import { iconPaths } from '$lib/icons';
  import { callBackend } from '$lib/utils/backend';
  import { selectedTag, filterCategory } from '$lib/stores';
  import type { FilterCategory } from '$lib/stores';
  import type { PasswordItem } from '$lib/types/password';
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

  type TagButton = {
    id?: number;
    text: string;
    color: string;
    icon: string;
  };

  type TagMeta = {
    icon: string;
    color: string;
  };

  type SectionTitle = 'Pinned' | 'Today' | 'Yesterday' | 'Earlier';

  interface ItemSection {
    title: SectionTitle;
    items: PasswordItem[];
  }

  interface Props {
    items: PasswordItem[];
    buttons: TagButton[];
    selectedId: number | null;
    disableEdit?: boolean;
    onselect?: (item: PasswordItem) => void;
    oncreateEntry?: () => void;
    oneditEntry?: (item: PasswordItem) => void;
    onremoveEntry?: (item: PasswordItem) => void;
  }

  let {
    items = $bindable<PasswordItem[]>([]),
    buttons,
    selectedId = null,
    disableEdit = false,
    onselect,
    oncreateEntry,
    oneditEntry,
    onremoveEntry
  }: Props = $props();

  const defaultFallback: TagMeta = {
    icon: iconPaths.default,
    color: 'var(--sidebar-border)'
  };

  const sectionOrder: SectionTitle[] = ['Pinned', 'Today', 'Yesterday', 'Earlier'];
  const RECENT_DAY_WINDOW = 7;
  const DAY_IN_MS = 24 * 60 * 60 * 1000;
  const PIN_TAG_NAMES = new Set(['pinned', 'pin']);
  const RECENT_FILTER: FilterCategory = 'recent';

  let searchTerm = $state('');
  let selectedItemId = $state<number | null>(null);
  let showSkeleton = $state(false);
  let skeletonTimer: ReturnType<typeof setTimeout> | null = null;
  let lastSkeletonKey = '';
  let navInnerRef: HTMLElement | null = null;
  let highlightTimer: ReturnType<typeof setTimeout> | null = null;

  const itemsCount = $derived(items.length);

  const tagMap = $derived.by(
    () =>
      new Map<string, TagMeta>(
        buttons.map((button) => [
          button.text,
          { color: button.color, icon: button.icon || iconPaths.default }
        ])
      )
  );

  const filteredItems = $derived.by(() => {
    const normalizedSearchTerm = searchTerm.trim().toLowerCase();

    return items.filter((item) => {
      const tagNames = getTagNames(item);

      if ($selectedTag !== null && !tagNames.includes($selectedTag)) {
        return false;
      }

      if ($filterCategory === RECENT_FILTER && !isRecent(item, tagNames)) {
        return false;
      }

      if (!normalizedSearchTerm) {
        return true;
      }

      const title = item.title?.toLowerCase() ?? '';
      const username = item.username?.toLowerCase() ?? '';
      const tags = item.tags?.toLowerCase() ?? '';
      const url = item.url?.toLowerCase() ?? '';

      return (
        title.includes(normalizedSearchTerm) ||
        username.includes(normalizedSearchTerm) ||
        tags.includes(normalizedSearchTerm) ||
        url.includes(normalizedSearchTerm)
      );
    });
  });

  const sectionedItems = $derived.by(() => partitionItems(filteredItems));

  const currentSkeletonKey = $derived(() => `${$selectedTag ?? 'all'}|${$filterCategory}`);

  let lastDispatchedId: number | null = null;
  $effect(() => {
    const visibleIds = new Set(filteredItems.map((item) => item.id));

    let desiredId: number | null = null;
    if (selectedId !== null && visibleIds.has(selectedId)) {
      desiredId = selectedId;
    } else {
      const nextItem = sectionedItems.find((section) => section.items.length > 0)?.items[0];
      desiredId = nextItem ? nextItem.id : null;
    }

    Promise.resolve().then(() => {
      selectedItemId = desiredId;
      if (desiredId != null && lastDispatchedId !== desiredId) {
        const item = filteredItems.find((i) => i.id === desiredId);
        if (item) {
          onselect?.(item);
          lastDispatchedId = desiredId;
        }
      }
    });
  });

  $effect(() => {
    const skeletonKey = currentSkeletonKey();
    if (skeletonKey === lastSkeletonKey) {
      return;
    }

    lastSkeletonKey = skeletonKey;

    if (filteredItems.length === 0) {
      showSkeleton = false;
      if (skeletonTimer) {
        clearTimeout(skeletonTimer);
        skeletonTimer = null;
      }
      return;
    }

    tick().then(() => {
      if (filteredItems.length === 0) {
        return;
      }

      showSkeleton = true;

      if (skeletonTimer) {
        clearTimeout(skeletonTimer);
      }

      skeletonTimer = setTimeout(() => {
        showSkeleton = false;
        skeletonTimer = null;
      }, 200);
    });
  });

  onDestroy(() => {
    if (skeletonTimer) {
      clearTimeout(skeletonTimer);
      skeletonTimer = null;
    }

    if (highlightTimer) {
      clearTimeout(highlightTimer);
      highlightTimer = null;
    }
  });

  function clearEditingHighlight() {
    if (!navInnerRef) return;

    const highlighted = navInnerRef.querySelectorAll<HTMLElement>('.editing-target');
    highlighted.forEach((element) => element.classList.remove('editing-target'));
  }

  export async function focusItem(id: number | null | undefined) {
    if (id == null) {
      return;
    }

    await tick();

    if (!navInnerRef) {
      return;
    }

    clearEditingHighlight();

    const target = navInnerRef.querySelector<HTMLElement>(`[data-item-id="${id}"]`);
    if (!target) {
      return;
    }

    target.classList.add('editing-target');
    target.scrollIntoView({ behavior: 'smooth', block: 'center' });

    if (highlightTimer) {
      clearTimeout(highlightTimer);
    }

    highlightTimer = setTimeout(() => {
      target.classList.remove('editing-target');
      highlightTimer = null;
    }, 800);
  }

  function partitionItems(list: PasswordItem[]): ItemSection[] {
    const buckets: Record<SectionTitle, PasswordItem[]> = {
      Pinned: [],
      Today: [],
      Yesterday: [],
      Earlier: []
    };

    for (const item of list) {
      if (isPinned(item)) {
        buckets.Pinned.push(item);
      } else if (item.updated_at && isToday(item.updated_at)) {
        buckets.Today.push(item);
      } else if (item.updated_at && isYesterday(item.updated_at)) {
        buckets.Yesterday.push(item);
      } else {
        buckets.Earlier.push(item);
      }
    }

    return sectionOrder.map((title) => ({
      title,
      items: buckets[title]
    }));
  }

  function getTagNames(item: PasswordItem): string[] {
    return item.tags
      ? item.tags
          .split(',')
          .map((tag) => tag.trim())
          .filter(Boolean)
      : [];
  }

  function hasPinnedTag(tagNames: string[]): boolean {
    return tagNames.some((tag) => PIN_TAG_NAMES.has(tag.toLowerCase()));
  }

  function isPinned(item: PasswordItem): boolean {
    return hasPinnedTag(getTagNames(item));
  }

  function toDate(dateStr: string): Date | null {
    const date = new SvelteDate(dateStr);
    return Number.isNaN(date.getTime()) ? null : date;
  }

  function isSameDay(a: Date, b: Date): boolean {
    return (
      a.getFullYear() === b.getFullYear() &&
      a.getMonth() === b.getMonth() &&
      a.getDate() === b.getDate()
    );
  }

  function isToday(dateStr: string): boolean {
    const date = toDate(dateStr);
    if (!date) {
      return false;
    }
    const now = new SvelteDate();
    return isSameDay(date, now);
  }

  function isYesterday(dateStr: string): boolean {
    const date = toDate(dateStr);
    if (!date) {
      return false;
    }
    const yesterday = new SvelteDate();
    yesterday.setDate(yesterday.getDate() - 1);
    return isSameDay(date, yesterday);
  }

  function isWithinDays(dateStr: string, windowInDays: number): boolean {
    const date = toDate(dateStr);
    if (!date) {
      return false;
    }
    const now = new SvelteDate();
    const diff = now.getTime() - date.getTime();
    if (diff < 0) {
      return false;
    }
    return diff <= windowInDays * DAY_IN_MS;
  }

  function isRecent(item: PasswordItem, tagNames: string[]): boolean {
    if (hasPinnedTag(tagNames)) {
      return true;
    }

    return item.updated_at ? isWithinDays(item.updated_at, RECENT_DAY_WINDOW) : false;
  }

  function getFallback(item: PasswordItem): TagMeta {
    const tagNames = getTagNames(item);
    if (tagNames.length > 0) {
      const firstTagMeta = tagMap.get(tagNames[0]);
      if (firstTagMeta) {
        return firstTagMeta;
      }
    }
    return defaultFallback;
  }

  function clearSearch() {
    searchTerm = '';
  }

  async function handlePinToggle(item: PasswordItem) {
    const id = item.id;
    const tagNames = getTagNames(item);
    const pinnedIndex = tagNames.findIndex((tag) => PIN_TAG_NAMES.has(tag.toLowerCase()));

    const nextTags =
      pinnedIndex >= 0
        ? tagNames.filter((_, index) => index !== pinnedIndex)
        : ['Pinned', ...tagNames];

    const updatedTags = nextTags.join(',');

    try {
      await callBackend('update_password_item_tags', { id, tags: updatedTags });
      const itemIndex = items.findIndex((candidate) => candidate.id === id);
      if (itemIndex !== -1) {
        items = [
          ...items.slice(0, itemIndex),
          { ...items[itemIndex], tags: updatedTags },
          ...items.slice(itemIndex + 1)
        ];
      }
    } catch (error) {
      console.error('Failed to toggle pin:', error);
      alert(`Failed to ${pinnedIndex >= 0 ? 'unpin' : 'pin'} item: ${error}`);
    }
  }

  function skeletonPlaceholders(count: number): number[] {
    return Array.from({ length: count }, (_, index) => index);
  }

  function selectItem(item: PasswordItem) {
    selectedItemId = item.id;
    onselect?.(item);
  }

  function handleCreateEntry() {
    oncreateEntry?.();
  }

  function handleEditEntry(item: PasswordItem) {
    if (disableEdit) {
      return;
    }

    oneditEntry?.(item);
  }

  function handleRemoveEntry(item: PasswordItem) {
    onremoveEntry?.(item);
  }
</script>

<nav class="passwordList">
  <ContextMenu>
    <ContextMenuTrigger>
      <div
        class="navInner"
        bind:this={navInnerRef}
        aria-busy={showSkeleton}
        role="region"
        aria-label="Password list"
        tabindex="-1"
      >
        <div class="topControls" role="region" aria-label="Navigation controls">
          <div class="searchContainer">
            <Button
              type="button"
              class="searchBtn text-muted-foreground hover:text-foreground h-8 w-8 shrink-0"
              variant="ghost"
              size="icon"
              aria-label="Search"
              title="Search"
            >
              <Search class="h-5 w-5" />
            </Button>
            <Input
              type="text"
              placeholder="Search..."
              class="searchInput h-8 border-0 bg-transparent px-2 text-sm shadow-none focus-visible:border-0 focus-visible:ring-0 focus-visible:ring-offset-0"
              bind:value={searchTerm}
            />
            {#if searchTerm}
              <Button
                type="button"
                variant="ghost"
                size="icon"
                class="clearSearchBtn text-muted-foreground hover:text-foreground h-8 w-8 shrink-0"
                onclick={clearSearch}
                aria-label="Clear search"
              >
                <X class="h-4 w-4" />
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
              aria-selected={$filterCategory === RECENT_FILTER}
              onclick={() => filterCategory.set(RECENT_FILTER)}
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
                      {#each skeletonPlaceholders(section.items.length) as placeholderIndex (placeholderIndex)}
                        <li
                          class="item"
                          aria-hidden="true"
                          data-placeholder-index={placeholderIndex}
                        >
                          <div class="itemLink" role="presentation">
                            <div class="itemLeft">
                              <Skeleton class="h-7 w-7 rounded-md opacity-70" aria-hidden="true" />
                              <div class="itemTexts">
                                <Skeleton
                                  class="h-3 w-36 rounded-md opacity-80"
                                  aria-hidden="true"
                                />
                                <Skeleton
                                  class="mt-2 h-3 w-24 rounded-md opacity-60"
                                  aria-hidden="true"
                                />
                              </div>
                            </div>
                          </div>
                        </li>
                      {/each}
                    {:else}
                      {#each section.items as item (item.id)}
                        {@const fallback = getFallback(item)}
                        <li
                          class="item"
                          class:selected={selectedItemId === item.id}
                          data-item-id={item.id}
                          role="listitem"
                        >
                          <ContextMenu>
                            <ContextMenuTrigger>
                              <button
                                type="button"
                                class="itemLink"
                                onclick={(event: MouseEvent) => {
                                  event.preventDefault();
                                  selectItem(item);
                                }}
                                draggable="false"
                              >
                                <div class="itemLeft">
                                  <Favicon
                                    url={item.url ?? undefined}
                                    title={item.title}
                                    fallbackIcon={fallback.icon}
                                    fallbackColor={fallback.color}
                                    size={29}
                                    variant="list"
                                  />
                                  <div class="itemTexts">
                                    <div class="itemTitle">{item.title}</div>
                                    <div class="itemDesc">{item.username}</div>
                                  </div>
                                </div>
                                <div class="itemTags">
                                  <TagIcon
                                    tagNames={item.tags
                                      ? item.tags.split(',').map((tag: string) => tag.trim())
                                      : []}
                                    {tagMap}
                                  />
                                </div>
                              </button>
                            </ContextMenuTrigger>
                            <ContextMenuContent class="w-48">
                              <ContextMenuItem onSelect={() => handlePinToggle(item)}
                                >{isPinned(item) ? 'Unpin' : 'Pin'}</ContextMenuItem
                              >
                              <ContextMenuSeparator />
                              <ContextMenuItem
                                disabled={disableEdit}
                                onSelect={() => handleEditEntry(item)}
                              >
                                Edit Entry
                              </ContextMenuItem>
                              <ContextMenuItem
                                class="text-destructive focus:text-destructive data-highlighted:bg-destructive/10"
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
</nav>

<style>
  .passwordList {
    --passwordlist-surface: var(--passwordlist-base);
    --passwordlist-elevated: var(--passwordlist-item);
    --passwordlist-hover: var(--passwordlist-hover-bg);
    --passwordlist-strong-text: var(--sidebar-foreground);
    --passwordlist-muted-text: color-mix(in oklch, var(--sidebar-foreground) 60%, transparent);
    --passwordlist-subtle-text: color-mix(in oklch, var(--sidebar-foreground) 35%, transparent);
    --passwordlist-border: var(--sidebar-border);
    --passwordlist-scroll-thumb: color-mix(in oklch, var(--sidebar-border) 75%, var(--sidebar) 25%);
    --passwordlist-skeleton-base: color-mix(
      in oklch,
      var(--passwordlist-elevated) 88%,
      var(--background) 12%
    );
    --passwordlist-skeleton-highlight: color-mix(
      in oklch,
      var(--passwordlist-elevated) 70%,
      var(--background) 30%
    );
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
    margin-left: 0;
    font-size: 12px;
    color: var(--passwordlist-muted-text);
    margin-top: 0;
  }

  .sectionTitle + .itemList {
    margin-top: 0;
  }

  .itemList {
    margin-top: 8px;
    list-style: none;
    padding: 0;
    margin-left: -12px;
    margin-right: -12px;
    margin-bottom: 6px;
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
    background-color: color-mix(
      in oklch,
      var(--passwordlist-scroll-thumb) 85%,
      var(--passwordlist-hover) 15%
    );
  }

  .itemList::-webkit-scrollbar-button {
    display: none;
  }

  .item {
    background: var(--passwordlist-item);
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
    cursor: pointer;
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
    cursor: pointer;
  }

  .itemDesc {
    margin-top: 3px;
    font-size: 12px;
    color: var(--passwordlist-muted-text);
    line-height: 1;
    user-select: none;
    cursor: pointer;
  }

  .itemTags {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .navInner[aria-busy='true'] {
    pointer-events: none;
  }

  .no-items-message {
    padding: 20px;
    text-align: center;
    color: var(--passwordlist-subtle-text);
    font-style: italic;
  }
</style>
