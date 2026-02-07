<script lang="ts">
  import { iconPaths } from '$lib/icons';
  import { appState } from '$lib/stores';
  import Icon from '$lib/components/ui/Icon.svelte';
  import { Globe, Plus, Settings, Star } from '@lucide/svelte';
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger
  } from '$lib/components/ui/context-menu';
  import {
    Sidebar as SidebarRoot,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupContent,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem
  } from '$lib/components/ui/sidebar';

  interface ButtonOption {
    id: number;
    text: string;
    icon: string;
    color: string;
    count?: number;
  }

  const DEFAULT_TAG_COLOR = 'var(--sidebar-border)';

  interface Props {
    buttons?: ButtonOption[];
    totalItemCount?: number;
    favoritesCount?: number;
    onopenPopup?: (detail: { mode: 'create' | 'edit'; tag?: ButtonOption }) => void;
    ontagDeleteRequested?: (button: ButtonOption) => void;
  }

  let { buttons = [], totalItemCount = 0, favoritesCount = 0, onopenPopup, ontagDeleteRequested }: Props = $props();

  function handleOpenPopup() {
    onopenPopup?.({ mode: 'create' });
  }

  function handleShowAll() {
    appState.selectedTag = null;
    appState.filterCategory = 'all';
  }

  function handleShowFavorites() {
    appState.selectedTag = null;
    appState.filterCategory = 'favorites';
  }

  function handleTagClick(tagText: string) {
    appState.selectedTag = tagText;
  }

  function handleEdit(button: ButtonOption) {
    onopenPopup?.({ mode: 'edit', tag: button });
  }

  async function handleRemove(button: ButtonOption) {
    ontagDeleteRequested?.(button);
  }

  function handleCreateVault() {
    console.log('Create vault clicked');
  }

  import { createResizeController } from './password-list/resizeController';
  import { settings } from '$lib/stores/appSettings.svelte';
  import { onDestroy } from 'svelte';

  function handleKeyDown(e: KeyboardEvent) {
    const step = e.shiftKey ? 50 : 10;
    const currentWidth = settings.state.appearance.sidebarWidth || 240;

    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      resizeController.setWidth(currentWidth - step);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      resizeController.setWidth(currentWidth + step);
    }
  }

  const resizeController = createResizeController({
    cssVar: '--sidebar-width',
    selector: '.app-sidebar',
    minWidth: 150,
    maxWidth: 400,
    onResizeEnd: (width) => {
      settings.state.appearance.sidebarWidth = width;
      settings.save();
    }
  });

  onDestroy(() => {
    resizeController.destroy();
  });
</script>

<SidebarRoot collapsible="none" class="app-sidebar bg-sidebar backdrop-blur relative">
  <SidebarHeader class="flex items-center justify-center py-5">
    <img src="/logo.png" alt="Pulsar Logo" class="h-8 w-8" />
  </SidebarHeader>

  <SidebarContent class="flex flex-1 flex-col gap-2 px-2 pb-6">
    <SidebarGroup class="w-full">
      <SidebarGroupContent>
        <SidebarMenu class="gap-1">
          <SidebarMenuItem>
            <SidebarMenuButton
              size="lg"
              aria-label="Show all items"
              tooltipContent="All items"
              isActive={appState.selectedTag === null && appState.filterCategory === 'all'}
              style="--tag-color: var(--sidebar-border);"
              class="h-[46px] w-full cursor-pointer justify-start pl-[12px] rounded-[5px] transition text-sidebar-foreground hover:bg-sidebar-border/30 data-[active=true]:bg-sidebar-border/50"
              onclick={handleShowAll}
            >
              <div class="flex h-[32px] w-[32px] shrink-0 items-center justify-center rounded-md bg-[color-mix(in_oklch,var(--tag-color)_15%,transparent)]">
                <Globe size="14" />
              </div>
              <span class="ml-0 font-medium">All items</span>
              {#if totalItemCount > 0}
                <span class="ml-auto mr-[8px] flex h-[20px] items-center justify-center rounded-full bg-sidebar-foreground/10 px-3 text-sm">
                  {totalItemCount}
                </span>
              {/if}
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              size="lg"
              aria-label="Show favorites"
              tooltipContent="Favorites"
              isActive={appState.selectedTag === null && appState.filterCategory === 'favorites'}
              style="--tag-color: var(--sidebar-border);"
              class="h-[46px] w-full cursor-pointer justify-start pl-[12px] rounded-[5px] transition text-sidebar-foreground hover:bg-sidebar-border/30 data-[active=true]:bg-sidebar-border/50"
              onclick={handleShowFavorites}
            >
              <div class="flex h-[32px] w-[32px] shrink-0 items-center justify-center rounded-md bg-[color-mix(in_oklch,var(--tag-color)_15%,transparent)]">
                <Star size="14" />
              </div>
              <span class="ml-0 font-medium">Favorites</span>
              {#if favoritesCount > 0}
                <span class="ml-auto mr-[8px] flex h-[20px] items-center justify-center rounded-full bg-sidebar-foreground/10 px-3 text-sm">
                  {favoritesCount}
                </span>
              {/if}
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>

    <SidebarGroup class="w-full">
      <div class="relative z-10 mb-2 flex h-[24px] items-center justify-between pl-[12px] pr-[8px]">
        <span class="text-xs font-bold leading-none tracking-wider text-muted-foreground">VAULT</span>
        <button
          onclick={handleOpenPopup}
          class="flex items-center justify-center text-muted-foreground transition-colors hover:text-foreground"
          aria-label="Add Tag"
        >
          <Plus size="14" />
        </button>
      </div>
      <SidebarGroupContent>
        <SidebarMenu class="gap-1">
          {#each buttons as button (button.id)}
            <SidebarMenuItem>
              <ContextMenu>
                <ContextMenuTrigger>
                  <SidebarMenuButton
                    size="lg"
                    aria-label={button.text}
                    tooltipContent={button.text}
                    isActive={appState.selectedTag === button.text}
                    style={`--tag-color: ${button.color || DEFAULT_TAG_COLOR};`}
                    class="group h-[50px] w-full cursor-pointer justify-start pl-[12px] rounded-[5px] transition text-sidebar-foreground hover:bg-sidebar-border/30 data-[active=true]:bg-sidebar-border/50"
                    onclick={() => handleTagClick(button.text)}
                  >
                    <div class="flex h-[32px] w-[32px] shrink-0 items-center justify-center rounded-md bg-[color-mix(in_oklch,var(--tag-color)_15%,transparent)]">
                      <Icon
                        path={button.icon || iconPaths.default}
                        size="12"
                        viewBox="0 0 44 44"
                        color="var(--tag-color)"
                        className="opacity-100"
                      />
                    </div>
                    <span class="ml-0 font-medium truncate">{button.text}</span>
                    {#if button.count && button.count > 0}
                      <span class="ml-auto mr-[8px] flex h-[20px] items-center justify-center rounded-full bg-sidebar-foreground/10 px-3 text-sm">
                        {button.count}
                      </span>
                    {/if}
                  </SidebarMenuButton>
                </ContextMenuTrigger>
                <ContextMenuContent class="w-44">
                  <ContextMenuItem onSelect={() => handleEdit(button)}>Edit Tag</ContextMenuItem>
                  <ContextMenuItem
                    class="text-destructive focus:text-destructive data-highlighted:bg-destructive/10"
                    onSelect={() => handleRemove(button)}>Remove Tag</ContextMenuItem
                  >
                </ContextMenuContent>
              </ContextMenu>
            </SidebarMenuItem>
          {/each}
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
  </SidebarContent>
  <SidebarFooter class="border-border/40 mt-auto w-full border-t px-6">
    <SidebarMenu class="items-center">
      <SidebarMenuItem class="w-full cursor-pointer">
          <SidebarMenuButton
          size="lg"
          aria-label="Open settings"
          tooltipContent="Settings"
          style="cursor: pointer; --tag-color: var(--sidebar-border);"
          class="h-[46px] w-full cursor-pointer justify-start pl-[12px] rounded-[5px] transition text-sidebar-foreground hover:bg-sidebar-border/30"
        >
          {#snippet child({ props })}
            <a href="/settings/general" {...props} class="flex w-full items-center">
              <div class="flex h-[32px] w-[32px] shrink-0 items-center justify-center rounded-md bg-[color-mix(in_oklch,var(--tag-color)_15%,transparent)]">
                <Settings
                  size="22"
                  className="opacity-70 transition-opacity group-hover/menu-item:opacity-100"
                />
              </div>
              <span class="ml-0 font-medium">Settings</span>
            </a>
          {/snippet}
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  </SidebarFooter>
  <div
    class="resize-handle"
    role="slider"
    aria-label="Resize sidebar"
    aria-valuenow={settings.state.appearance.sidebarWidth || 240}
    aria-valuemin={150}
    aria-valuemax={400}
    tabindex="0"
    onmousedown={(e) => resizeController.start(e)}
    onkeydown={handleKeyDown}
  ></div>
</SidebarRoot>

<style>
  .resize-handle {
    position: absolute;
    top: 0;
    right: 0;
    width: 4px;
    height: 100%;
    cursor: col-resize;
    z-index: 10;
    transition: background-color 0.2s;
  }

  .resize-handle:hover,
  .resize-handle:active {
    background-color: var(--primary);
    opacity: 0.5;
  }
</style>
