<script lang="ts">
  import { keybinds } from '../stores/keybinds';
  import type { Keybind } from '../config/keybinds';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
  } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Input } from '$lib/components/ui/input';
  import { cn } from '$lib/utils';
  import { X } from '@lucide/svelte';

  interface Props {
    onclose?: () => void;
  }

  let { onclose }: Props = $props();

  let dialogOpen = $state(true);
  let duplicateKeybinds = $state<Record<string, boolean>>({});
  let shakeInput = $state<Record<string, boolean>>({});

  function close() {
    dialogOpen = false;
    onclose?.();
  }

  function validateKeybinds() {
    const currentKeybinds = $keybinds;

    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const keyMap = new Map<string, string[]>();
    let hasDuplicates = false;
    const newDuplicateState: Record<string, boolean> = {};

    currentKeybinds.forEach((kb: Keybind) => {
      const key = kb.key ?? '';
      if (!key) return;
      const bucket = keyMap.get(key);
      if (bucket) {
        bucket.push(kb.name);
      } else {
        keyMap.set(key, [kb.name]);
      }
    });

    keyMap.forEach((names) => {
      if (names.length > 1) {
        hasDuplicates = true;
        names.forEach((name) => {
          newDuplicateState[name] = true;
        });
      }
    });

    duplicateKeybinds = newDuplicateState;
    return !hasDuplicates;
  }

  function handleKeydown(event: KeyboardEvent, keybindName: string) {
    event.preventDefault();

    const keys: string[] = [];
    if (event.ctrlKey) keys.push('Ctrl');
    if (event.shiftKey) keys.push('Shift');
    if (event.altKey) keys.push('Alt');
    if (event.metaKey) keys.push('Meta');

    if (!['Control', 'Shift', 'Alt', 'Meta'].includes(event.key)) {
      keys.push(event.key.length === 1 ? event.key.toUpperCase() : event.key);
    }

    const newKey = keys.join('+');
    keybinds.updateKeybind(keybindName, newKey);
    validateKeybinds();
  }

  function handleReset() {
    keybinds.resetKeybinds();
    validateKeybinds();
  }

  function handleSave() {
    if (validateKeybinds()) {
      close();
      return;
    }

    const shakeState: Record<string, boolean> = {};
    for (const [name, hasDuplicate] of Object.entries(duplicateKeybinds)) {
      if (hasDuplicate) {
        shakeState[name] = true;
      }
    }
    shakeInput = shakeState;

    setTimeout(() => {
      shakeInput = {};
    }, 500);
  }

  function handleOpenChange(open: boolean) {
    if (!open) {
      close();
    } else {
      dialogOpen = true;
    }
  }

  validateKeybinds();
</script>

<Dialog open={dialogOpen} onOpenChange={handleOpenChange}>
  <DialogContent class="sm:max-w-3xl">
    <button
      type="button"
      class="text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:ring-ring absolute top-4 right-4 inline-flex size-8 items-center justify-center rounded-md transition focus-visible:ring-2 focus-visible:outline-none"
      onclick={close}
      aria-label="Close keyboard shortcuts popup"
    >
      <X class="size-4" />
    </button>

    <DialogHeader class="space-y-2 text-left">
      <DialogTitle>Keyboard Shortcuts</DialogTitle>
      <DialogDescription>
        Customize how you navigate Pulsar with key combinations that fit your workflow.
      </DialogDescription>
    </DialogHeader>

    <ScrollArea class="max-h-[60vh] pr-2">
      <div class="flex flex-col gap-4 pr-2">
        {#each $keybinds as keybind (keybind.name)}
          <div
            class="border-border/40 bg-muted/30 flex flex-col gap-4 rounded-lg border p-4 sm:flex-row sm:items-center sm:justify-between"
          >
            <div class="flex flex-col gap-1">
              <span class="text-foreground text-sm font-medium">{keybind.name}</span>
              <span class="text-muted-foreground text-xs">{keybind.description}</span>
            </div>
            <Input
              bind:value={keybind.key}
              class={cn(
                'focus-visible:ring-ring/50 h-10 w-44 shrink-0 font-mono text-sm shadow-none focus-visible:ring-2',
                duplicateKeybinds[keybind.name] &&
                  'border-destructive ring-destructive/40 focus-visible:ring-destructive/50 ring-1',
                shakeInput[keybind.name] && 'animate-shake'
              )}
              readonly
              aria-invalid={duplicateKeybinds[keybind.name] ?? false}
              onkeydown={(event) => handleKeydown(event, keybind.name)}
            />
          </div>
        {/each}
      </div>
    </ScrollArea>

    <DialogFooter class="flex-col sm:flex-row sm:items-center sm:justify-between">
      <Button type="button" variant="ghost" class="justify-start" onclick={handleReset}>
        Reset to defaults
      </Button>
      <div class="flex items-center gap-2">
        <Button type="button" variant="outline" onclick={close}>Cancel</Button>
        <Button type="button" onclick={handleSave}>Save changes</Button>
      </div>
    </DialogFooter>
  </DialogContent>
</Dialog>

<style>
  :global(.animate-shake) {
    animation: shake 0.5s cubic-bezier(0.36, 0.07, 0.19, 0.97) both;
    transform: translate3d(0, 0, 0);
    backface-visibility: hidden;
    perspective: 1000px;
  }

  @keyframes shake {
    10%,
    90% {
      transform: translate3d(-1px, 0, 0);
    }

    20%,
    80% {
      transform: translate3d(2px, 0, 0);
    }

    30%,
    50%,
    70% {
      transform: translate3d(-4px, 0, 0);
    }

    40%,
    60% {
      transform: translate3d(4px, 0, 0);
    }
  }
</style>
