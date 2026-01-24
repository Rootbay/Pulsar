<script lang="ts">
  import { onMount } from 'svelte';
  import { callBackend } from '$lib/utils/backend';
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
  import { Check, X, Key, CreditCard, User, StickyNote } from '@lucide/svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import { cn } from '$lib/utils';
  import { ChartColumnStacked } from '@lucide/svelte';

  interface TagOption {
    id: number;
    text: string;
    icon: string;
    color: string;
  }

  interface Props {
    onclose?: () => void;
    onpasswordSaved?: () => void;
    ontagCreated?: () => void;
  }

  let { onclose, onpasswordSaved, ontagCreated }: Props = $props();

  const PLACEHOLDER_PASSWORD = '';

  const categories = [
    { id: 'login', label: 'Login', icon: Key },
    { id: 'card', label: 'Credit Card', icon: CreditCard },
    { id: 'identity', label: 'Identity', icon: User },
    { id: 'note', label: 'Secure Note', icon: StickyNote }
  ] as const;

  type CategoryId = (typeof categories)[number]['id'];

  let title = $state('');
  let selectedCategory = $state<CategoryId>('login');
  let tags = $state<string[]>([]);
  let availableTags = $state<TagOption[]>([]);
  let dialogOpen = $state(true);

  const canSave = $derived(title.trim().length > 0);

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
      availableTags = await callBackend<TagOption[]>('get_buttons');
    } catch (error) {
      console.error('Error fetching tags:', error);
    }
  });

  function handleOpenChange(open: boolean) {
    dialogOpen = open;
    if (!dialogOpen) {
      onclose?.();
    }
  }

  function closeDialog() {
    dialogOpen = false;
    onclose?.();
  }

  async function savePassword(e: SubmitEvent) {
    e.preventDefault();
    const trimmedTitle = title.trim();
    if (!trimmedTitle) {
      return;
    }

    try {
      await callBackend('save_password_item', {
        item: {
          id: 0,
          category: selectedCategory,
          title: trimmedTitle,
          username: null,
          url: null,
          notes: null,
          password: PLACEHOLDER_PASSWORD,
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

      onpasswordSaved?.();
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

    <form id="create-password-form" class="grid gap-6" onsubmit={savePassword}>
      <div class="grid gap-3">
        <Label>Category</Label>
        <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
          {#each categories as category (category.id)}
            <Button
              type="button"
              variant={selectedCategory === category.id ? 'secondary' : 'outline'}
              class={cn(
                'h-auto flex-col items-center justify-center gap-2 py-3 transition-all',
                selectedCategory === category.id
                  ? 'border-primary/50 bg-primary/10 ring-primary/30 ring-1'
                  : 'hover:border-primary/30 hover:bg-primary/5'
              )}
              onclick={() => (selectedCategory = category.id)}
            >
              <div
                class="bg-background flex size-8 items-center justify-center rounded-lg shadow-xs"
              >
                <ChartColumnStacked class="size-4.5 text-current" />
              </div>
              <span class="text-[10px] font-bold tracking-wider uppercase">{category.label}</span>
            </Button>
          {/each}
        </div>
      </div>

      <div class="grid gap-2">
        <Label for="title">Title</Label>
        <Input id="title" placeholder="Enter title" bind:value={title} required autofocus />
      </div>

      <div class="grid gap-3">
        <div class="flex items-center justify-between">
          <span class="text-foreground text-sm font-medium">Tags</span>
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
            <p class="text-muted-foreground text-sm">No tags selected yet.</p>
          {/if}

          {#each tags as tag (tag)}
            <Badge
              variant="secondary"
              class="flex items-center gap-1 rounded-full px-3 py-1 text-xs"
            >
              <span>{tag}</span>
              <button
                type="button"
                class="text-muted-foreground hover:bg-foreground/10 hover:text-foreground focus-visible:ring-ring/50 rounded-full p-0.5 text-xs transition focus-visible:ring-2 focus-visible:outline-none"
                onclick={() => removeTag(tag)}
                aria-label={`Remove ${tag}`}
              >
                <X class="size-3" />
              </button>
            </Badge>
          {/each}
        </div>

        <ScrollArea class="border-border/60 bg-muted/40 max-h-48 rounded-md border p-2">
          <div class="grid gap-2">
            {#if !availableTags.length}
              <p class="text-muted-foreground text-sm">No tags available yet.</p>
            {:else}
              {#each availableTags as tagOption (tagOption.id)}
                <Button
                  type="button"
                  variant={tags.includes(tagOption.text) ? 'secondary' : 'ghost'}
                  class={cn(
                    'w-full justify-start gap-2',
                    tags.includes(tagOption.text)
                      ? 'bg-secondary text-secondary-foreground'
                      : 'hover:bg-muted/80'
                  )}
                  onclick={() => toggleTag(tagOption.text)}
                >
                  <span
                    class="size-2.5 rounded-full"
                    style={`background: ${tagOption.color || '#94a3b8'};`}
                    aria-hidden="true"
                  ></span>
                  <Icon path={tagOption.icon} size="16" color="currentColor" className="shrink-0" />
                  <span class="text-sm">{tagOption.text}</span>
                  {#if tags.includes(tagOption.text)}
                    <Check class="text-muted-foreground ms-auto size-4" />
                  {/if}
                </Button>
              {/each}
            {/if}
          </div>
        </ScrollArea>
      </div>
    </form>

    <DialogFooter>
      <Button type="button" variant="outline" onclick={closeDialog}>Cancel</Button>
      <Button type="submit" form="create-password-form" disabled={!canSave}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
