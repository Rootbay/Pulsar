<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { iconPaths } from '$lib/icons';
  import { filterCategory, selectedTag, showSettingsPopup } from '$lib/stores';
  import Icon from '../ui/Icon.svelte';
  import { Globe, Plus, Settings } from '@lucide/svelte';
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
  import { Button } from '$lib/components/ui/button';

  interface ButtonOption {
    id: number;
    text: string;
    icon: string;
    color: string;
  }

  const DEFAULT_TAG_COLOR = 'var(--sidebar-border)';

  export let buttons: ButtonOption[] = [];

  const dispatch = createEventDispatcher();

  function handleOpenPopup() {
    dispatch('openPopup', { mode: 'create' });
  }

  function handleShowAll() {
    selectedTag.set(null);
    filterCategory.set('all');
  }

  function handleTagClick(tagText: string) {
    selectedTag.set(tagText);
  }

  function handleEdit(button: ButtonOption) {
    dispatch('openPopup', { mode: 'edit', tag: button });
  }

  async function handleRemove(button: ButtonOption) {
    try {
      await invoke('delete_button', { id: button.id });
      dispatch('tagDeleted', {
        id: button.id,
        text: button.text
      });
    } catch (error) {
      console.error('Failed to delete tag:', error);
    }
  }
</script>

<SidebarRoot
  collapsible="none"
  class="border-r border-border/50 bg-sidebar/90 backdrop-blur"
  style="--sidebar-width: 5.75rem;"
>
  <SidebarHeader class="flex items-center justify-center py-5">
    <img src="/svelte.svg" alt="Pulsar logo" class="size-8" />
  </SidebarHeader>

  <SidebarContent class="flex flex-1 flex-col items-center gap-6 px-2 pb-6">
    <SidebarGroup class="w-full">
      <SidebarGroupContent>
        <SidebarMenu class="items-center gap-3">
          <SidebarMenuItem>
            <SidebarMenuButton
              size="lg"
              aria-label="Show all items"
              tooltipContent="All items"
              isActive={$selectedTag === null && $filterCategory === 'all'}
              class="h-12 w-12 justify-center rounded-xl text-sidebar-foreground/80 transition hover:text-sidebar-foreground"
              onclick={handleShowAll}
            >
              <Globe style="w-5 h-5" className="size-5 opacity-70 transition-opacity group-data-[active=true]/menu-item:opacity-100" />
              <span class="sr-only">All items</span>
            </SidebarMenuButton>
          </SidebarMenuItem>

          {#each buttons as button (button.id)}
            <SidebarMenuItem>
              <ContextMenu>
                <ContextMenuTrigger>
                  <SidebarMenuButton
                    size="lg"
                    aria-label={button.text}
                    tooltipContent={button.text}
                    isActive={$selectedTag === button.text}
                    class="group h-12 w-12 justify-center rounded-xl text-sidebar-foreground/70 transition hover:text-sidebar-foreground"
                    onclick={() => handleTagClick(button.text)}
                  >
                    <span
                      class="size-2.5 rounded-full"
                      style={`background: ${button.color || DEFAULT_TAG_COLOR};`}
                      aria-hidden="true"
                    ></span>
                    <Icon
                      path={button.icon}
                      size="22"
                      viewBox="0 0 44 44"
                      color={button.color || DEFAULT_TAG_COLOR}
                      className="size-5 opacity-75 transition-opacity group-data-[active=true]/menu-item:opacity-100"
                    />
                    <span class="sr-only">{button.text}</span>
                  </SidebarMenuButton>
                </ContextMenuTrigger>
                <ContextMenuContent class="w-44">
                  <ContextMenuItem onSelect={() => handleEdit(button)}>Edit Tag</ContextMenuItem>
                  <ContextMenuItem
                    class="text-destructive focus:text-destructive data-[highlighted]:bg-destructive/10"
                    onSelect={() => handleRemove(button)}
                  >Remove Tag</ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            </SidebarMenuItem>
          {/each}
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>

    <Button
      variant="outline"
      size="icon"
      class="size-12 rounded-full border-dashed border-border/60 text-muted-foreground transition hover:border-primary/50 hover:text-primary"
      aria-label="Add new tag"
      onclick={handleOpenPopup}
    >
      <Plus style="w-5 h-5" />
    </Button>
  </SidebarContent>

  <SidebarFooter class="mt-auto w-full border-t border-border/40 pt-4">
    <SidebarMenu class="items-center">
      <SidebarMenuItem>
        <SidebarMenuButton
          size="lg"
          aria-label="Open settings"
          tooltipContent="Settings"
          class="h-12 w-12 justify-center rounded-xl text-sidebar-foreground/70 transition hover:text-sidebar-foreground"
          onclick={() => showSettingsPopup.set(true)}
        >
          <Settings style="w-5 h-5" className="size-5 opacity-70 transition-opacity group-hover/menu-item:opacity-100" />
          <span class="sr-only">Settings</span>
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  </SidebarFooter>
</SidebarRoot>

