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
  class="bg-sidebar backdrop-blur"
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
              style="--tag-color: var(--sidebar-border); --tag-hover: color-mix(in oklch, var(--sidebar-foreground) 40%, var(--sidebar-border)); --tag-active: color-mix(in oklch, var(--sidebar-foreground) 60%, var(--sidebar-border));"
              class="h-[46px] w-[46px] justify-center rounded-lg cursor-pointer text-[color:var(--tag-color)] transition hover:text-[color:var(--tag-hover)] data-[active=true]:text-[color:var(--tag-active)] hover:bg-transparent active:!bg-transparent data-[active=true]:bg-transparent shadow-[0_0_0_1px_var(--sidebar-border)] hover:shadow-[0_0_0_1px_var(--tag-hover)] active:!shadow-[0_0_0_1px_var(--tag-hover)] data-[active=true]:shadow-[0_0_0_2px_var(--tag-active)] active:!text-[color:var(--tag-hover)] transition-shadow"
              onclick={handleShowAll}
            >
              <span class="inline-flex size-5 shrink-0 items-center justify-center">
                <Globe size="20" />
              </span>
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
                    style={`--tag-color: ${button.color || DEFAULT_TAG_COLOR};`}
                    class="group h-[46px] w-[46px] justify-center rounded-lg cursor-pointer text-[color:var(--tag-color)] data-[active=true]:text-[color:var(--tag-color)] transition hover:text-[color:var(--tag-color)] hover:bg-transparent active:!bg-transparent data-[active=true]:bg-transparent shadow-[0_0_0_1px_var(--sidebar-border)] hover:shadow-[0_0_0_1px_var(--tag-color)] active:!shadow-[0_0_0_1px_var(--tag-color)] data-[active=true]:shadow-[0_0_0_2px_var(--tag-color)] active:!text-[color:var(--tag-color)] transition-shadow"
                    onclick={() => handleTagClick(button.text)}
                  >
                    <span class="inline-flex size-5 shrink-0 items-center justify-center">
                      <Icon
                        path={button.icon || iconPaths.default}
                        size="20"
                        viewBox="0 0 44 44"
                        color="var(--tag-color)"
                        className="opacity-75 transition-opacity group-hover/menu-item:opacity-100 group-data-[active=true]/menu-item:opacity-100"
                      />
                    </span>
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
          <SidebarMenuItem>
            <SidebarMenuButton
              size="lg"
              aria-label="Add new tag"
              tooltipContent="Add"
              style="--tag-color: var(--sidebar-border); --tag-hover: color-mix(in oklch, var(--sidebar-foreground) 40%, var(--sidebar-border)); --tag-active: color-mix(in oklch, var(--sidebar-foreground) 60%, var(--sidebar-border));"
              class="h-[46px] w-[46px] justify-center rounded-lg cursor-pointer text-[color:var(--tag-color)] transition hover:text-[color:var(--tag-hover)] hover:bg-transparent active:!bg-transparent data-[active=true]:bg-transparent shadow-[0_0_0_1px_var(--sidebar-border)] hover:shadow-[0_0_0_1px_var(--tag-hover)] active:!shadow-[0_0_0_1px_var(--tag-hover)] data-[active=true]:shadow-[0_0_0_2px_var(--tag-active)] active:!text-[color:var(--tag-hover)] transition-shadow"
              onclick={handleOpenPopup}
            >
              <span class="inline-flex size-5 shrink-0 items-center justify-center">
                <Plus size="20" />
              </span>
              <span class="sr-only">Add new tag</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
  </SidebarContent>

  <SidebarFooter class="mt-auto w-full border-t border-border/40 pt-4">
    <SidebarMenu class="items-center">
      <SidebarMenuItem>
        <SidebarMenuButton
          size="lg"
          aria-label="Open settings"
          tooltipContent="Settings"
          class="h-[46px] w-[46px] justify-center rounded-lg text-sidebar-foreground/70 transition hover:text-sidebar-foreground"
          onclick={() => showSettingsPopup.set(true)}
        >
          <Settings style="w-5 h-5" className="size-5 opacity-70 transition-opacity group-hover/menu-item:opacity-100" />
          <span class="sr-only">Settings</span>
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  </SidebarFooter>
</SidebarRoot>

