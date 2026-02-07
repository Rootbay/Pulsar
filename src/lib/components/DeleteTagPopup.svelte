<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
  } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import type { TagButton } from '$lib/stores/tags.svelte';
  import { Trash2 } from '@lucide/svelte';

  interface Props {
    tag: TagButton;
    onclose?: () => void;
    onconfirm?: () => void;
  }

  let { tag, onclose, onconfirm }: Props = $props();

  let dialogOpen = $state(true);

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

  function handleConfirm() {
    onconfirm?.();
    handleClose();
  }
</script>

<Dialog open={dialogOpen} onOpenChange={handleOpenChange}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2 text-destructive">
        <Trash2 class="size-5" />
        Delete Tag
      </DialogTitle>
      <DialogDescription>
        Are you sure you want to delete this tag? This action cannot be undone.
      </DialogDescription>
    </DialogHeader>

    <div class="bg-muted/30 border-border/50 mt-2 space-y-3 rounded-lg border p-4">
      <div>
        <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase"
          >Tag Name</span
        >
        <p class="text-sm font-medium">{tag.text}</p>
      </div>
      <div>
        <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase"
          >Usage</span
        >
        <p class="mt-0.5 text-sm">
          {#if tag.count && tag.count > 0}
            Used by <span class="font-bold text-foreground">{tag.count}</span> password{tag.count === 1 ? '' : 's'}.
          {:else}
            Not currently used.
          {/if}
        </p>
      </div>
      {#if tag.count && tag.count > 0}
        <div class="rounded-md bg-destructive/10 p-2 text-xs text-destructive">
            Deleting this tag will remove it from all associated passwords.
        </div>
      {/if}
    </div>

    <DialogFooter class="mt-6">
      <Button variant="outline" onclick={handleClose}>Cancel</Button>
      <Button variant="destructive" onclick={handleConfirm}>Delete Tag</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
