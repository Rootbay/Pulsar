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
  import { Trash2 } from '@lucide/svelte';
  import type { PasswordItemOverview } from '$lib/types/password';

  interface Props {
    item: PasswordItemOverview;
    onclose?: () => void;
    onconfirm?: () => void;
  }

  let { item, onclose, onconfirm }: Props = $props();

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
        Delete Password
      </DialogTitle>
      <DialogDescription>
        Are you sure you want to delete this password entry? This action cannot be undone.
      </DialogDescription>
    </DialogHeader>

    <div class="bg-muted/30 border-border/50 mt-2 space-y-3 rounded-lg border p-4">
      <div>
        <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase"
          >Title</span
        >
        <p class="text-sm font-medium">{item.title}</p>
      </div>
      {#if item.username}
        <div>
          <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase"
            >Username</span
          >
          <p class="mt-0.5 text-sm font-mono">{item.username}</p>
        </div>
      {/if}
    </div>

    <DialogFooter class="mt-6">
      <Button variant="outline" onclick={handleClose}>Cancel</Button>
      <Button variant="destructive" onclick={handleConfirm}>Delete Password</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
