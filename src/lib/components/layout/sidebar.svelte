<script lang="ts">
  import { iconPaths } from '$lib/icons';
  import { appState } from '$lib/stores';
  import { goto } from '$app/navigation';
  import Icon from '$lib/components/ui/Icon.svelte';
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

  interface ButtonOption {
    id: number;
    text: string;
    icon: string;
    color: string;
  }

  const DEFAULT_TAG_COLOR = 'var(--sidebar-border)';

  interface Props {
    buttons?: ButtonOption[];
    onopenPopup?: (detail: { mode: 'create' | 'edit'; tag?: ButtonOption }) => void;
    ontagDeleteRequested?: (button: ButtonOption) => void;
  }

  let { buttons = [], onopenPopup, ontagDeleteRequested }: Props = $props();

  function handleOpenPopup() {
    onopenPopup?.({ mode: 'create' });
  }

  function handleShowAll() {
    appState.selectedTag = null;
    appState.filterCategory = 'all';
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
</script>

<SidebarRoot collapsible="none" class="bg-sidebar backdrop-blur">
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
              isActive={appState.selectedTag === null && appState.filterCategory === 'all'}
              style="--tag-color: var(--sidebar-border); --tag-hover: color-mix(in oklch, var(--sidebar-foreground) 40%, var(--sidebar-border)); --tag-active: color-mix(in oklch, var(--sidebar-foreground) 60%, var(--sidebar-border));"
              class="hover:text-color:var(--tag-hover) data-[active=true]:text-color:var(--tag-active) active:text-color:var(--tag-hover)! h-11.5 w-11.5 cursor-pointer justify-center rounded-lg text-(--tag-color) shadow-[0_0_0_1px_var(--sidebar-border)] transition hover:bg-transparent hover:shadow-[0_0_0_1px_var(--tag-hover)] active:bg-transparent! active:shadow-[0_0_0_1px_var(--tag-hover)]! data-[active=true]:bg-transparent data-[active=true]:shadow-[0_0_0_2px_var(--tag-active)]"
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
                    isActive={appState.selectedTag === button.text}
                    style={`--tag-color: ${button.color || DEFAULT_TAG_COLOR};`}
                    class="group text-color:var(--tag-color) data-[active=true]:text-color:var(--tag-color) hover:text-color:var(--tag-color) active:!text-color:var(--tag-color) h-11.5 w-11.5 cursor-pointer justify-center rounded-lg shadow-[0_0_0_1px_var(--sidebar-border)] transition hover:bg-transparent hover:shadow-[0_0_0_1px_var(--tag-color)] active:bg-transparent! active:shadow-[0_0_0_1px_var(--tag-color)]! data-[active=true]:bg-transparent data-[active=true]:shadow-[0_0_0_2px_var(--tag-color)]"
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
                    class="text-destructive focus:text-destructive data-highlighted:bg-destructive/10"
                    onSelect={() => handleRemove(button)}>Remove Tag</ContextMenuItem
                  >
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
              class="text-color:var(--tag-color) hover:text-color:var(--tag-hover) active:text-color:var(--tag-hover)! h-11.5 w-11.5 cursor-pointer justify-center rounded-lg shadow-[0_0_0_1px_var(--sidebar-border)] transition hover:bg-transparent hover:shadow-[0_0_0_1px_var(--tag-hover)] active:bg-transparent! active:shadow-[0_0_0_1px_var(--tag-hover)]! data-[active=true]:bg-transparent data-[active=true]:shadow-[0_0_0_2px_var(--tag-active)]"
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

  <SidebarFooter class="border-border/40 mt-auto w-full border-t pt-4">
    <SidebarMenu class="items-center">
      <SidebarMenuItem>
        <SidebarMenuButton
          size="lg"
          aria-label="Open settings"
          tooltipContent="Settings"
          class="text-sidebar-foreground/70 hover:text-sidebar-foreground h-11.5 w-11.5 justify-center rounded-lg transition"
          onclick={() => goto('/settings/general')}
        >
          <Settings
            style="w-5 h-5"
            className="size-5 opacity-70 transition-opacity group-hover/menu-item:opacity-100"
          />
          <span class="sr-only">Settings</span>
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  </SidebarFooter>
</SidebarRoot>
