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
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { cn } from '$lib/utils';
  import { Check } from '@lucide/svelte';
  import { open } from '@tauri-apps/plugin-dialog';

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
  let selectedFilePath: string | null = null;
  let selectingFile = false;
  let fileError: string | null = null;
  let passphrase = '';

  $: if (show && !dialogOpen) {
    dialogOpen = true;
  }

  $: if (!show && dialogOpen) {
    dialogOpen = false;
  }

  $: if (!dialogOpen) {
    selectedManager = null;
    selectedFilePath = null;
    passphrase = '';
    fileError = null;
  }

  function handleSelect(managerId: string) {
    selectedManager = managerId;
    fileError = null;
  }

  async function pickFile() {
    try {
      selectingFile = true;
      const result = await open({
        multiple: false,
        filters: [
          { name: 'Pulsar Backup', extensions: ['pulsar', 'json'] },
          { name: 'All files', extensions: ['*'] }
        ]
      });

      if (typeof result === 'string') {
        selectedFilePath = result;
        fileError = null;
      }
    } catch (error) {
      console.error('Failed to pick import file:', error);
      fileError = 'Failed to open the file picker. Please try again.';
    } finally {
      selectingFile = false;
    }
  }

  async function handleImport() {
    if (!selectedManager) {
      return;
    }

    if (!selectedFilePath) {
      await pickFile();
      if (!selectedFilePath) {
        return;
      }
    }

    if (!passphrase.trim()) {
      fileError = 'Enter the passphrase that protects your backup.';
      return;
    }

    dispatch('importSelected', {
      manager: selectedManager,
      importedPath: selectedFilePath,
      passphrase: passphrase.trim()
    });
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

    <div class="mt-4 space-y-4">
      <div class="space-y-2">
        <Label for="import-file" class="text-sm font-medium text-foreground">Backup file</Label>
        <div class="flex items-center gap-2">
          <Input
            id="import-file"
            type="text"
            placeholder="Select a Pulsar backup file"
            readonly
            value={selectedFilePath ?? ''}
          />
          <Button type="button" variant="outline" onclick={pickFile} disabled={selectingFile}>
            {#if selectingFile}
              Selecting...
            {:else if selectedFilePath}
              Change
            {:else}
              Choose file
            {/if}
          </Button>
        </div>
        <p class="text-xs text-muted-foreground">Accepted formats: .pulsar or .json backup files.</p>
      </div>

      <div class="space-y-2">
        <Label for="import-passphrase" class="text-sm font-medium text-foreground">
          Backup passphrase
        </Label>
        <Input
          id="import-passphrase"
          type="password"
          autocomplete="current-password"
          value={passphrase}
          oninput={(event) => {
            passphrase = (event.target as HTMLInputElement).value;
            fileError = null;
          }}
        />
      </div>

      {#if fileError}
        <p class="text-sm text-destructive">{fileError}</p>
      {/if}
    </div>

    <DialogFooter>
      <Button type="button" variant="outline" onclick={closeDialog}>
        Cancel
      </Button>
      <Button
        type="button"
        disabled={
          !selectedManager || !selectedFilePath || !passphrase.trim() || selectingFile
        }
        onclick={handleImport}
      >
        Import
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
