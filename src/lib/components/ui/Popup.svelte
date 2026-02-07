<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
  } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { cn } from '$lib/utils';
  import Icon from '$lib/components/ui/Icon.svelte';
  import { iconPaths } from '$lib/icons';
  import type { TagInput } from '$lib/stores/tags.svelte';

  interface Props {
    mode?: 'create' | 'edit';
    tag?: TagInput | null;
    onclose?: () => void;
    onsave?: (detail: { mode: 'create' | 'edit'; tag: TagInput }) => void | Promise<void>;
  }

  let { mode = 'create', tag = null, onclose, onsave }: Props = $props();

  let dialogOpen = $state(true);
  let inputValue = $state('');
  let selectedColor = $state('#F29292');
  
  const colors = [
    '#F29292',
    '#F7D775',
    '#91C799',
    '#92B3F2',
    '#EB8DD6',
    '#CD5A6F',
    '#E4A367',
    '#E6E6E6',
    '#9EE2E6'
  ];

  interface IconType {
    name: string;
    path: string;
  }

  const icons: IconType[] = [
    { name: 'home', path: iconPaths.home },
    { name: 'globe', path: iconPaths.globe },
    { name: 'messenger', path: iconPaths.messenger },
    { name: 'wallet', path: iconPaths.wallet },
    { name: 'lock', path: iconPaths.lock },
    { name: 'folder', path: iconPaths.folder },
    { name: 'paper', path: iconPaths.paper },
    { name: 'card', path: iconPaths.card },
    { name: 'key', path: iconPaths.key }
  ];

  let selectedIcon = $state<IconType>(icons[0]);

  onMount(() => {
    if (mode === 'edit' && tag) {
      inputValue = tag.text;
      selectedColor = tag.color;
      selectedIcon = icons.find((i) => i.path === tag.icon) || icons[0];
    }
  });

  function handleOpenChange(open: boolean) {
    dialogOpen = open;
    if (!open) {
      onclose?.();
    }
  }

  function handleClose() {
    dialogOpen = false;
    onclose?.();
  }

  async function handleSave() {
    if (!inputValue.trim() || !selectedIcon || !selectedColor) return;

    const buttonData: TagInput = {
      id: tag?.id,
      text: inputValue.trim(),
      icon: selectedIcon.path,
      color: selectedColor
    };

    if (mode === 'edit' && !buttonData.id) {
      return;
    }

    try {
      await onsave?.({ mode, tag: buttonData });
      handleClose();
    } catch (error) {
      console.error('Failed to save tag:', error);
    }
  }
</script>

<Dialog open={dialogOpen} onOpenChange={handleOpenChange}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle>{mode === 'create' ? 'Create New Tag' : 'Edit Tag'}</DialogTitle>
      <DialogDescription>
        {mode === 'create' ? 'Add a new tag to organize your vault items.' : 'Update the details of your tag.'}
      </DialogDescription>
    </DialogHeader>

    <div class="grid gap-6 py-4">
      <div class="grid gap-2">
        <Label for="name">Name</Label>
        <Input
          id="name"
          placeholder="e.g. Social, Finance, Work"
          bind:value={inputValue}
          onkeydown={(e) => e.key === 'Enter' && handleSave()}
          autofocus
        />
      </div>

      <div class="grid gap-2">
        <Label>Color</Label>
        <div class="flex flex-wrap gap-3">
          {#each colors as color}
            <button
              type="button"
              class={cn(
                "h-8 w-8 rounded-full border transition-all hover:scale-110 focus:outline-none focus:ring-1 focus:ring-ring focus:ring-offset-2",
                selectedColor === color ? "border-foreground/20 scale-110 ring-1 ring-ring ring-offset-2" : "border-transparent"
              )}
              style="background-color: {color}"
              onclick={() => selectedColor = color}
              aria-label="Select color {color}"
            ></button>
          {/each}
        </div>
      </div>

      <div class="grid gap-2">
        <Label>Icon</Label>
        <div class="grid grid-cols-5 gap-2">
          {#each icons as icon}
            <button
              type="button"
              class={cn(
                "flex h-9 w-full items-center justify-center rounded-md border transition-all hover:bg-muted",
                selectedIcon.name === icon.name 
                  ? "border-current bg-current/10" 
                  : "border-transparent bg-transparent text-muted-foreground opacity-70 hover:opacity-100"
              )}
              style={selectedIcon.name === icon.name ? `color: ${selectedColor};` : ''}
              onclick={() => selectedIcon = icon}
              title={icon.name}
              aria-label="Select icon {icon.name}"
            >
              <Icon 
                path={icon.path} 
                size="18" 
                viewBox="0 0 48 48" 
                color={selectedIcon.name === icon.name ? 'currentColor' : selectedColor} 
              />
            </button>
          {/each}
        </div>
      </div>
    </div>

    <DialogFooter>
      <Button variant="outline" onclick={handleClose}>Cancel</Button>
      <Button onclick={handleSave} disabled={!inputValue.trim()}>
        {mode === 'create' ? 'Create Tag' : 'Save Changes'}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
