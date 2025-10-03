<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
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
  import { Badge } from '$lib/components/ui/badge';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Check, X } from '@lucide/svelte';
  import Icon from './ui/Icon.svelte';
  import { cn } from '$lib/utils';

  interface TagOption {
    id: number;
    text: string;
    icon: string;
    color: string;
  }

  const FIXED_PASSWORD = 'N/A';

  const dispatch = createEventDispatcher();

  let title = '';
  let tags: string[] = [];
  let availableTags: TagOption[] = [];
  let dialogOpen = true;

  $: canSave = title.trim().length > 0;

  const toggleTag = (tag: string) => {
    tags = tags.includes(tag) ? tags.filter((t) => t !== tag) : [...tags, tag];
  };

  const removeTag = (tag: string) => {
    tags = tags.filter((t) => t !== tag);
  };

  const clearTags = () => {
    if (tags.length) {
      tags = [];
    }
  };

  onMount(async () => {
    try {
      availableTags = await invoke<TagOption[]>('get_buttons');
    } catch (error) {
      console.error('Error fetching tags:', error);
    }
  });

  function handleOpenChange(open: boolean) {
    dialogOpen = open;
    if (!dialogOpen) {
      dispatch('close');
    }
  }

  function closeDialog() {
    dialogOpen = false;
    dispatch('close');
  }

  async function savePassword() {
    const trimmedTitle = title.trim();
    if (!trimmedTitle) {
      return;
    }

    try {
      await invoke('save_password_item', {
        item: {
          id: 0,
          title: trimmedTitle,
          username: null,
          url: null,
          notes: null,
          password: FIXED_PASSWORD,
          description: null,
          img: null,
          tags: tags.length ? tags.join(',') : null,
          created_at: '',
          updated_at: '',
          color: null,
          totp_secret: null,
          custom_fields: [],
          field_order: null
        }
      });

      dispatch('passwordSaved');
      closeDialog();
    } catch (error) {
      console.error('Error saving password:', error);
    }
  }
</script>

<Dialog open={dialogOpen} onOpenChange={handleOpenChange}>
  <DialogContent class="sm:max-w-xl">
    <DialogHeader>
      <DialogTitle>New item</DialogTitle>
      <DialogDescription>
        Store a placeholder entry now and fill in more details whenever you are ready.
      </DialogDescription>
    </DialogHeader>

    <form
      id="create-password-form"
      class="grid gap-6"
      onsubmit={savePassword}
    >
      <div class="grid gap-2">
        <Label for="title">Title</Label>
        <Input
          id="title"
          placeholder="Enter title"
          bind:value={title}
          required
          autofocus
        />
      </div>

      <div class="grid gap-3">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium text-foreground">Tags</span>
          {#if tags.length}
            <Button
              type="button"
              variant="ghost"
              size="sm"
              class="h-8 px-2 text-xs"
              onclick={clearTags}
            >
              Clear selection
            </Button>
          {/if}
        </div>

        <div class="flex flex-wrap items-center gap-2">
          {#if !tags.length}
            <p class="text-sm text-muted-foreground">No tags selected yet.</p>
          {/if}

          {#each tags as tag (tag)}
            <Badge
              variant="secondary"
              class="flex items-center gap-1 rounded-full px-3 py-1 text-xs"
            >
              <span>{tag}</span>
              <button
                type="button"
                class="rounded-full p-[2px] text-xs text-muted-foreground transition hover:bg-foreground/10 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
                onclick={() => removeTag(tag)}
                aria-label={`Remove ${tag}`}
              >
                <X class="size-3" />
              </button>
            </Badge>
          {/each}
        </div>

        <ScrollArea class="max-h-48 rounded-md border border-border/60 bg-muted/40 p-2">
          <div class="grid gap-2">
            {#if !availableTags.length}
              <p class="text-sm text-muted-foreground">No tags available yet.</p>
            {:else}
              {#each availableTags as tagOption (tagOption.id)}
                <Button
                  type="button"
                  variant={tags.includes(tagOption.text) ? 'secondary' : 'ghost'}
                  class={cn(
                    'w-full justify-start gap-2',
                    tags.includes(tagOption.text) ? 'bg-secondary text-secondary-foreground' : 'hover:bg-muted/80'
                  )}
                  onclick={() => toggleTag(tagOption.text)}
                >
                  <span
                    class="size-2.5 rounded-full"
                    style={`background: ${tagOption.color || '#94a3b8'};`}
                    aria-hidden="true"
                  ></span>
                  <Icon
                    path={tagOption.icon}
                    size="16"
                    color="currentColor"
                    className="shrink-0"
                  />
                  <span class="text-sm">{tagOption.text}</span>
                  {#if tags.includes(tagOption.text)}
                    <Check class="ms-auto size-4 text-muted-foreground" />
                  {/if}
                </Button>
              {/each}
            {/if}
          </div>
        </ScrollArea>
      </div>
    </form>

    <DialogFooter>
      <Button type="button" variant="outline" onclick={closeDialog}>
        Cancel
      </Button>
      <Button type="submit" form="create-password-form" disabled={!canSave}>
        Save
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

