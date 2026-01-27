<script lang="ts">
  import { onMount } from 'svelte';
  import { callBackend } from '$lib/utils/backend';
  import { tagStore } from '$lib/stores/tags.svelte';
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
  import {
    Check,
    X,
    Key,
    CreditCard,
    User,
    StickyNote,
    WandSparkles,
    Eye,
    EyeOff
  } from '@lucide/svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import { cn } from '$lib/utils';
  import { GeneratorService } from '$lib/utils/generator';
  import PasswordStrength from './password/PasswordStrength.svelte';

  interface Props {
    onclose?: () => void;
    onpasswordSaved?: (id: number) => void;
  }

  let { onclose, onpasswordSaved }: Props = $props();

  const categories = [
    { id: 'login', label: 'Login', icon: Key },
    { id: 'card', label: 'Credit Card', icon: CreditCard },
    { id: 'identity', label: 'Identity', icon: User },
    { id: 'note', label: 'Secure Note', icon: StickyNote }
  ] as const;

  type CategoryId = (typeof categories)[number]['id'];

  let title = $state('');
  let username = $state('');
  let password = $state('');
  let showPassword = $state(false);
  let selectedCategory = $state<CategoryId>('login');
  let tags = $state<string[]>([]);
  const availableTags = $derived(tagStore.tags);
  let dialogOpen = $state(true);

  const canSave = $derived(title.trim().length > 0);

  function generatePassword() {
    password = GeneratorService.generate(20);
    showPassword = true;
  }

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

  onMount(() => {
    void tagStore.refresh();
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
      const newId = await callBackend<number>('save_password_item', {
        item: {
          id: 0,
          category: selectedCategory,
          title: trimmedTitle,
          username: username.trim() || null,
          url: null,
          notes: null,
          password: password,
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

      onpasswordSaved?.(newId);
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
      <DialogDescription>Add a new credential to your secure vault.</DialogDescription>
    </DialogHeader>

    <ScrollArea class="max-h-[70vh] pr-4">
      <form id="create-password-form" class="grid gap-6 pb-4" onsubmit={savePassword}>
        <div class="grid gap-3">
          <Label>Category</Label>
          <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
            {#each categories as category (category.id)}
              <button
                type="button"
                class={cn(
                  'flex flex-col items-center justify-center gap-2 rounded-lg border py-3 transition-all',
                  selectedCategory === category.id
                    ? 'border-primary/50 bg-primary/10 ring-primary/30 ring-1'
                    : 'border-border/60 hover:border-primary/30 hover:bg-primary/5'
                )}
                onclick={() => (selectedCategory = category.id)}
              >
                <div
                  class="bg-background flex size-8 items-center justify-center rounded-lg shadow-xs"
                >
                  <category.icon class="size-4.5 text-current" />
                </div>
                <span class="text-[10px] font-bold tracking-wider uppercase">{category.label}</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="grid gap-2">
          <Label for="title">Title</Label>
          <Input
            id="title"
            placeholder="e.g. GitHub, Netflix"
            bind:value={title}
            required
            autofocus
          />
        </div>

        <div class="grid gap-2">
          <Label for="username">Username / Email</Label>
          <Input id="username" placeholder="Enter username" bind:value={username} />
        </div>

        <div class="grid gap-2">
          <div class="flex items-center justify-between">
            <Label for="password">Password</Label>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              class="h-7 px-2 text-xs"
              onclick={generatePassword}
            >
              <WandSparkles class="mr-1.5 size-3" />
              Generate
            </Button>
          </div>
          <div class="relative">
            <Input
              id="password"
              type={showPassword ? 'text' : 'password'}
              placeholder="Enter password"
              bind:value={password}
              class="pr-10"
            />
            <button
              type="button"
              class="text-muted-foreground hover:text-foreground absolute top-1/2 right-3 -translate-y-1/2"
              onclick={() => (showPassword = !showPassword)}
            >
              {#if showPassword}
                <EyeOff class="size-4" />
              {:else}
                <Eye class="size-4" />
              {/if}
            </button>
          </div>
          {#if password}
            <PasswordStrength {password} showDetails={true} className="mt-1" />
          {/if}
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

          <ScrollArea class="border-border/60 bg-muted/40 max-h-40 rounded-md border p-2">
            <div class="grid gap-2">
              {#if !availableTags.length}
                <p class="text-muted-foreground py-4 text-center text-sm">No tags available yet.</p>
              {:else}
                {#each availableTags as tagOption (tagOption.id)}
                  <Button
                    type="button"
                    variant={tags.includes(tagOption.text) ? 'secondary' : 'ghost'}
                    class={cn(
                      'h-9 w-full justify-start gap-2',
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
                    <Icon
                      path={tagOption.icon}
                      size="14"
                      color="currentColor"
                      className="shrink-0"
                    />
                    <span class="text-sm">{tagOption.text}</span>
                    {#if tags.includes(tagOption.text)}
                      <Check class="text-muted-foreground ms-auto size-3.5" />
                    {/if}
                  </Button>
                {/each}
              {/if}
            </div>
          </ScrollArea>
        </div>
      </form>
    </ScrollArea>

    <DialogFooter>
      <Button type="button" variant="outline" onclick={closeDialog}>Cancel</Button>
      <Button type="submit" form="create-password-form" disabled={!canSave}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
