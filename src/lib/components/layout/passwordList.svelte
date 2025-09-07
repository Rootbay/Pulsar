<script lang="ts">
  import { onMount, createEventDispatcher, tick, onDestroy } from 'svelte';
  import Icon from "../ui/Icon.svelte";
  import TagIcon from "../ui/TagIcon.svelte";
  import Favicon from "../ui/Favicon.svelte";
  import { iconPaths } from "$lib/icons";
  import { invoke } from '@tauri-apps/api/core';

  import { selectedTag, filterCategory } from '$lib/stores';

  export let items: any[] = [];
  export let buttons: any[] = [];
  export let selectedId: number | null = null; // external selection control

  $: tagMap = new Map(buttons.map(b => [b.text, { color: b.color, icon: b.icon }]));

  function getFallback(item: any) {
    const tagNames = item.tags ? item.tags.split(',').map((tag: string) => tag.trim()) : [];
    if (tagNames.length > 0) {
        const firstTag = tagMap.get(tagNames[0]);
        if (firstTag) {
            return { icon: firstTag.icon, color: firstTag.color };
        }
    }
    return { icon: iconPaths.default, color: '#ccc' };
  }

  let isResizing = false;
  let navWidth = 360;
  let startX = 0;
  let searchTerm = '';
  

  const dispatch = createEventDispatcher();

  $: itemsCount = items.length;

  $: filteredItems = items.filter(item => {
    const matchesTag = $selectedTag === null || (item.tags && (typeof item.tags === 'string' ? item.tags.split(',').map((tag: string) => tag.trim()) : item.tags).includes($selectedTag));
    const matchesSearch = searchTerm === '' || item.title.toLowerCase().includes(searchTerm.toLowerCase()) || item.username.toLowerCase().includes(searchTerm.toLowerCase());
    return matchesTag && matchesSearch;
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

  // Auto-select the first visible item when the list changes due to
  // sidebar tag/filter changes or search updates. If the current
  // selection is no longer visible, pick the first item in the first
  // non-empty section; otherwise leave the selection as-is.
  $: {
    const visibleIds = new Set(filteredItems.map((i) => i.id));
    if (filteredItems.length === 0) {
      // Clear selection when nothing is visible
      selectedItemId = null;
    } else if (selectedItemId === null || !visibleIds.has(selectedItemId)) {
      const firstSection = sectionedItems.find((s) => s.items.length > 0);
      if (firstSection) {
        const firstItem = firstSection.items[0];
        // Use existing selection flow to update parent and local state
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

  let showContextMenu = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let selectedItem: any = null;
  let isCreateEnabled = false;
  let isEditRemoveEnabled = false;
  let selectedItemId: number | null = null;

  // Skeleton loading when switching via sidebar (tags/filter)
  let showSkeleton = false;
  let skeletonTimer: any = null;
  let lastSkeletonKey = '';
  $: currentSkeletonKey = `${$selectedTag ?? 'all'}|${$filterCategory}`;
  $: if (currentSkeletonKey !== lastSkeletonKey) {
    lastSkeletonKey = currentSkeletonKey;
    (async () => {
      await tick(); // ensure filteredItems/sections updated for new selection
      const count = filteredItems.length;
      if (count > 0) {
        showSkeleton = true;
        clearTimeout(skeletonTimer);
        skeletonTimer = setTimeout(() => {
          showSkeleton = false;
        }, 200); // shorter animation window
      }
    })();
  }

  onDestroy(() => {
    clearTimeout(skeletonTimer);
  });

  function handleContextMenu(event: MouseEvent, item: any = null) {
    event.preventDefault();
    selectedItem = item;
    isCreateEnabled = item === null;
    isEditRemoveEnabled = item !== null;

    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    showContextMenu = true;
  }

  function closeContextMenu() {
    showContextMenu = false;
    selectedItem = null;
  }

  function handleCreateEntry() {
    dispatch('createEntry');
    closeContextMenu();
  }

  function handleEditEntry() {
    if (selectedItem) {
      dispatch('editEntry', selectedItem);
    }
    closeContextMenu();
  }

  function handleRemoveEntry() {
    if (selectedItem) {
      dispatch('removeEntry', selectedItem);
    }
    closeContextMenu();
  }

  async function handlePinToggle() {
    if (!selectedItem) return;
    const id = selectedItem.id;
    const parts = getTagNames(selectedItem);
    const lower = parts.map((t) => t.toLowerCase());
    const alreadyPinnedIdx = lower.findIndex((t) => t === 'pinned' || t === 'pin');
    let newTagsArr: string[];
    if (alreadyPinnedIdx >= 0) {
      // Unpin: remove any pin/pinned tags
      newTagsArr = parts.filter((t, i) => lower[i] !== 'pinned' && lower[i] !== 'pin');
    } else {
      // Pin: add canonical 'Pinned' tag at the start
      newTagsArr = ['Pinned', ...parts];
    }
    const newTags = newTagsArr.join(',');
    try {
      await invoke('update_password_item_tags', { id, tags: newTags });
      // Update local view so UI reflects immediately
      selectedItem = { ...selectedItem, tags: newTags };
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
    } finally {
      closeContextMenu();
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

  function handleGlobalMouseDown(e: MouseEvent) {
    // Don't close if clicking inside the context menu itself
    const target = e.target as HTMLElement;
    if (target && target.closest('.context-menu')) return;
    // Only close menu on left-clicks so right-click keeps it open
    if (e.button === 0) closeContextMenu();
  }

  onMount(() => {
    window.addEventListener('mousedown', handleGlobalMouseDown);
    return () => {
      window.removeEventListener('mousedown', handleGlobalMouseDown);
    };
  });
</script>

<nav class="passwordList">
  <div class="navInner" aria-busy={showSkeleton} on:contextmenu={(e) => { const t = e.target as HTMLElement; if (!t.closest('.item')) handleContextMenu(e, null); }}>
    <div class="topControls" role="region" aria-label="Navigation controls">
      <div class="searchContainer">
        <button class="searchBtn" aria-label="Search" title="Search">
          <Icon
            path={iconPaths.search}
            size="20"
            viewBox="0 0 40 40"
            color="currentColor"
          />
        </button>
        <input type="text" placeholder="Search..." class="searchInput" bind:value={searchTerm} on:input={handleSearchInput} />
        {#if searchTerm}
          <button class="clearSearchBtn" on:click={clearSearch} aria-label="Clear search">
            <Icon path={iconPaths.clear} size="16" viewBox="0 0 48 48" color="currentColor" />
          </button>
        {/if}
      </div>

      <div class="segButtons" role="tablist" aria-label="Filter tabs">
                <button class="segBtn all" role="tab" aria-selected={$filterCategory === 'all'} on:click={() => filterCategory.set('all')}>All <span class="count">({itemsCount})</span></button>
        <button class="segBtn recent" role="tab" aria-selected={$filterCategory === 'recent'} on:click={() => filterCategory.set('recent')}>Recently</button>
      </div>
    </div>

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
          <ul class="itemList" role="list" on:contextmenu={(e) => handleContextMenu(e, null)}>
            {#if showSkeleton}
              {#each Array(section.items.length) as _, i}
                <li class="item" aria-hidden="true">
                  <a class="itemLink">
                    <div class="itemLeft">
                      <div class="skeleton-avatar" aria-hidden="true"></div>
                      <div class="itemTexts">
                        <div class="skeleton-line title" aria-hidden="true"></div>
                        <div class="skeleton-line desc" aria-hidden="true"></div>
                      </div>
                    </div>
                  </a>
                </li>
              {/each}
            {:else}
              {#each section.items as item (item.id)}
                {@const fallback = getFallback(item)}
                <li class="item" class:selected={selectedItemId === item.id} role="listitem" on:contextmenu|stopPropagation={(e) => handleContextMenu(e, item)}>
                  <a
                    href={item.url}
                    class="itemLink"
                    on:click|preventDefault={() => selectItem(item)}
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
                      <TagIcon tagNames={item.tags ? item.tags.split(',').map((tag: string) => tag.trim()) : []} {tagMap} />
                    </div>
                  </a>
                </li>
              {/each}
            {/if}
          </ul>
        {/if}
      {/each}
    {/if}
  </div>
  <button class="resizer" on:mousedown={startResize} aria-label="Resize sidebar"></button>

  {#if showContextMenu}
    <div
      class="context-menu"
      role="menu"
      tabindex="0"
      style="left: {contextMenuX}px; top: {contextMenuY}px;"
      on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { closeContextMenu(); } }}
    >
      <button type="button" on:click={handleCreateEntry} disabled={!isCreateEnabled}>Create Entry</button>
      <button type="button" on:click={handlePinToggle} disabled={!isEditRemoveEnabled}>{isPinned(selectedItem) ? 'Unpin' : 'Pin'}</button>
      <button type="button" on:click={handleEditEntry} disabled={!isEditRemoveEnabled}>Edit Entry</button>
      <button type="button" on:click={handleRemoveEntry} disabled={!isEditRemoveEnabled}>Remove Entry</button>
    </div>
  {/if}
</nav>

<style>
  .passwordList {
    width: var(--passwordList-width);
    min-width: 150px;
    position: relative;
    background: var(--sidepanel-gradient);
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .navInner {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    flex-grow: 1;
    /* Make the whole list area scroll instead of each section */
    overflow-y: auto;
    /* Ensure flex child can actually shrink to allow scrolling */
    min-height: 0;
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
    background: #202024;
    border-radius: 16px;
    height: 32px;
    width: 100%;
    padding: 0 5px;
    box-sizing: border-box;
  }

  .searchBtn {
    border: none;
    background: transparent;
    color: var(--white);
    cursor: pointer;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    z-index: 1;
    padding-right: 8px;
  }

  .searchBtn :global(svg) {
    opacity: 0.8;
  }

  .searchInput {
    flex-grow: 1;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--white);
    padding: 0;
    font-size: 14px;
    caret-color: currentColor; 
    outline: none;
  }

  .clearSearchBtn {
    border: none;
    background: transparent;
    color: var(--white);
    cursor: pointer;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    z-index: 1;
    padding: 0 5px;
  }

  .segButtons {
    display: flex;
    gap: 12px;
    width: 100%;
  }

  .segBtn {
    height: 32px;
    padding: 0 12px;
    border-radius: 7px;
    border: 2px solid var(--btn-nav-border);
    background: var(--near-black);
    color: rgba(247, 219, 209, 0.5);
    font-size: 13px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: background-image 220ms ease, color 220ms ease;
  }

  .segBtn[aria-selected="true"] {
    background: var(--near-black);
    color: transparent;
    background-image: linear-gradient(to right, #F7DBD1, #C587CB);
    -webkit-background-clip: text;
    background-clip: text;
  }

  .sectionTitle {
    margin-left: 12px;
    font-size: 12px;
    color: #BDBDBD;
    margin-top: 0;
  }

  .itemList {
    margin-top: 8px;
    list-style: none;
    /* Tighten spacing between consecutive lists */
    padding: 0 0 6px 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    /* Do not grow; avoids large empty space between sections */
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
    background-color: #444;
    border-radius: 10px;
    border: 2px solid transparent;
    background-clip: padding-box;
  }

  .itemList::-webkit-scrollbar-thumb:hover {
    background-color: #555;
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
    background: #17171B;
  }

  .item.selected {
    background: #17171B;
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
    color: #FFFFFF;
    line-height: 1;
    user-select: none;
  }

  .itemDesc {
    margin-top: 3px;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.5);
    line-height: 1;
    user-select: none;
  }

  .itemTags {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  /* Skeleton loading styles */
  @keyframes skeleton-shimmer {
    0% { background-position: -160px 0; }
    100% { background-position: 160px 0; }
  }

  .navInner[aria-busy="true"] {
    pointer-events: none;
  }

  .skeleton-avatar {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
    background-size: 400px 100%;
    animation: skeleton-shimmer 0.8s ease-in-out infinite;
  }

  .skeleton-line {
    height: 12px;
    border-radius: 6px;
    background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
    background-size: 400px 100%;
    animation: skeleton-shimmer 0.8s ease-in-out infinite;
  }

  .skeleton-line.title {
    width: 140px;
    margin-bottom: 8px;
  }

  .skeleton-line.desc {
    width: 100px;
    opacity: 0.6;
  }

  /* No skeleton for tag icons per request */

  .no-items-message {
    padding: 20px;
    text-align: center;
    color: #aaa;
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

  .context-menu {
    position: fixed;
    background-color: var(--near-black);
    border: 1px solid var(--btn-nav-border);
    border-radius: 8px;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
    z-index: 200;
    display: flex;
    flex-direction: column;
    padding: 5px 0;
  }

  .context-menu:hover {
    background-color: #17171A;
  }

  .context-menu button {
    background: none;
    border: none;
    color: var(--white);
    padding: 8px 15px;
    text-align: left;
    cursor: pointer;
    width: 100%;
    white-space: nowrap;
  }

  .context-menu button:hover:not(:disabled) {
    background-color: #3a3a3a;
  }

  .context-menu button:disabled {
    color: #666;
    cursor: not-allowed;
  }
</style>
