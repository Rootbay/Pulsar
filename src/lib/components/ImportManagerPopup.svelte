<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
  } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { cn } from '$lib/utils';
  import { Check } from '@lucide/svelte';

  interface PasswordManager {
    id: string;
    name: string;
  }

  export let show = false;

  const dispatch = createEventDispatcher();

  const passwordManagers: PasswordManager[] = [
    { id: 'lastpass', name: 'LastPass' },
    { id: '1password', name: '1Password' },
    { id: 'keepass', name: 'KeePass' },
    { id: 'bitwarden', name: 'Bitwarden' }
  ];

  let selectedManager: string | null = null;
  let dialogOpen = show;

  $: if (show && !dialogOpen) {
    dialogOpen = true;
  }

  $: if (!show && dialogOpen) {
    dialogOpen = false;
  }

  $: if (!dialogOpen) {
    selectedManager = null;
  }

  function handleSelect(managerId: string) {
    selectedManager = managerId;
  }

  function handleImport() {
    if (!selectedManager) {
      return;
    }

    dispatch('importSelected', { manager: selectedManager });
    closeDialog();
  }

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
</script>

<Dialog open={dialogOpen} onOpenChange={handleOpenChange}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle>Import passwords</DialogTitle>
      <DialogDescription>
        Choose the password manager export you would like to import into Pulsar.
      </DialogDescription>
    </DialogHeader>

    <div class="grid gap-3">
      {#each passwordManagers as manager (manager.id)}
        <Button
          type="button"
          variant="outline"
          class={cn(
            'w-full justify-start gap-3 border-border/60 bg-muted/40 text-left text-sm font-medium hover:bg-muted/60',
            selectedManager === manager.id && 'border-primary bg-primary/10 text-primary'
          )}
          aria-pressed={selectedManager === manager.id}
          onclick={() => handleSelect(manager.id)}
        >
          <span class="flex-1">{manager.name}</span>
          {#if selectedManager === manager.id}
            <Check class="size-4" />
          {/if}
        </Button>
      {/each}
    </div>

    <DialogFooter>
      <Button type="button" variant="outline" onclick={closeDialog}>
        Cancel
      </Button>
      <Button type="button" disabled={!selectedManager} onclick={handleImport}>
        Import
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
