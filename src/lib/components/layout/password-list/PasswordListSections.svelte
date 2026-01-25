<script lang="ts">
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuTrigger
  } from '$lib/components/ui/context-menu';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import TagIcon from '../../ui/TagIcon.svelte';
  import Favicon from '../../ui/Favicon.svelte';
  import type { ItemSection, TagMeta } from './utils';
  import type { PasswordItem } from '$lib/types/password';

  interface Props {
    sections?: ItemSection[];
    selectedItemId?: number | null;
    showSkeleton?: boolean;
    tagMap?: Map<string, TagMeta>;
    skeletonPlaceholders: (count: number) => number[];
    getFallback: (item: PasswordItem, tagMap: Map<string, TagMeta>) => TagMeta;
    isPinned: (item: PasswordItem) => boolean;
    onSelect?: (item: PasswordItem) => void;
    onPinToggle?: (item: PasswordItem) => void;
    onEdit?: (item: PasswordItem) => void;
    onRemove?: (item: PasswordItem) => void;
  }

  let {
    sections = [],
    selectedItemId = null,
    showSkeleton = false,
    tagMap = new Map(),
    skeletonPlaceholders,
    getFallback,
    isPinned,
    onSelect,
    onPinToggle,
    onEdit,
    onRemove
  }: Props = $props();
</script>

{#each sections as section (section.title)}
  {#if section.items.length}
    <div class="sectionTitle" role="heading" aria-level="2">{section.title}</div>
    <ul class="itemList" role="list">
      {#if showSkeleton}
        {#each skeletonPlaceholders(section.items.length) as placeholderIndex (placeholderIndex)}
          <li class="item" aria-hidden="true" data-placeholder-index={placeholderIndex}>
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
          {@const fallback = getFallback(item, tagMap)}
          <li class="item" class:selected={selectedItemId === item.id} role="listitem">
            <ContextMenu>
              <ContextMenuTrigger>
                <button
                  type="button"
                  class="itemLink"
                  onclick={(event: MouseEvent) => {
                    event.preventDefault();
                    onSelect?.(item);
                  }}
                  draggable="false"
                >
                  <div class="itemLeft">
                    <Favicon
                      url={item.url ?? undefined}
                      title={item.title}
                      fallbackIcon={fallback.icon}
                      fallbackColor={fallback.color}
                      size={30}
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
                <ContextMenuItem onSelect={() => onPinToggle?.(item)}
                  >{isPinned(item) ? 'Unpin' : 'Pin'}</ContextMenuItem
                >
                <ContextMenuSeparator />
                <ContextMenuItem onSelect={() => onEdit?.(item)}>Edit Entry</ContextMenuItem>
                <ContextMenuItem
                  class="text-destructive focus:text-destructive data-[highlighted]:bg-destructive/10"
                  onSelect={() => onRemove?.(item)}
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
