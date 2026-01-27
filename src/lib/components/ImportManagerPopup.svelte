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
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { cn } from '$lib/utils';
  import { Check } from '@lucide/svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { currentLocale, t as translate, type I18nKey } from '$lib/i18n.svelte';

  interface PasswordManager {
    id: string;
    name: string;
  }

  interface Props {
    show?: boolean;
    onimportSelected?: (detail: {
      manager: string;
      importedPath: string;
      passphrase: string;
    }) => void;
    onclose?: () => void;
  }

  let { show = $bindable(false), onimportSelected, onclose }: Props = $props();

  const locale = $derived($currentLocale);
  const t = (key: I18nKey, vars: Record<string, string | number> = {}) =>
    translate(locale, key, vars);

  const passwordManagers: PasswordManager[] = [
    { id: 'lastpass', name: 'LastPass' },
    { id: '1password', name: '1Password' },
    { id: 'keepass', name: 'KeePass' },
    { id: 'bitwarden', name: 'Bitwarden' }
  ];

  let selectedManager = $state<string | null>(null);
  let dialogOpen = $state(show);
  let selectedFilePath = $state<string | null>(null);
  let selectingFile = $state(false);
  let fileError = $state<string | null>(null);
  let passphrase = $state('');

  $effect(() => {
    if (show && !dialogOpen) {
      dialogOpen = true;
    }
  });

  $effect(() => {
    if (!show && dialogOpen) {
      dialogOpen = false;
    }
  });

  $effect(() => {
    if (!dialogOpen) {
      selectedManager = null;
      selectedFilePath = null;
      passphrase = '';
      fileError = null;
    }
  });

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
          { name: 'Pulsar Backup', extensions: ['psec', 'json'] },
          { name: 'All files', extensions: ['*'] }
        ]
      });

      if (typeof result === 'string') {
        selectedFilePath = result;
        fileError = null;
      }
    } catch (error) {
      console.error('Failed to pick import file:', error);
      fileError = t('Failed to open the file picker. Please try again.');
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
      fileError = t('Enter the passphrase that protects your backup.');
      return;
    }

    onimportSelected?.({
      manager: selectedManager,
      importedPath: selectedFilePath,
      passphrase: passphrase.trim()
    });
    closeDialog();
  }

  function handleOpenChange(open: boolean) {
    dialogOpen = open;
    if (!dialogOpen) {
      show = false;
      onclose?.();
    }
  }

  function closeDialog() {
    dialogOpen = false;
    show = false;
    onclose?.();
  }
</script>

<Dialog open={dialogOpen} onOpenChange={handleOpenChange}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle>{t('Import passwords')}</DialogTitle>
      <DialogDescription>
        {t('Choose the password manager export you would like to import into Pulsar.')}
      </DialogDescription>
    </DialogHeader>

    <div class="grid gap-3">
      {#each passwordManagers as manager (manager.id)}
        <Button
          type="button"
          variant="outline"
          class={cn(
            'border-border/60 bg-muted/40 hover:bg-muted/60 w-full justify-start gap-3 text-left text-sm font-medium',
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
        <Label for="import-file" class="text-foreground text-sm font-medium">
          {t('Backup file')}
        </Label>
        <div class="flex items-center gap-2">
          <Input
            id="import-file"
            type="text"
            placeholder={t('Select a Pulsar backup file')}
            readonly
            value={selectedFilePath ?? ''}
          />
          <Button type="button" variant="outline" onclick={pickFile} disabled={selectingFile}>
            {#if selectingFile}
              {t('Selecting...')}
            {:else if selectedFilePath}
              {t('Change')}
            {:else}
              {t('Choose file')}
            {/if}
          </Button>
        </div>
        <p class="text-muted-foreground text-xs">
          {t('Accepted formats: .psec or .json backup files.')}
        </p>
      </div>

      <div class="space-y-2">
        <Label for="import-passphrase" class="text-foreground text-sm font-medium">
          {t('Backup passphrase')}
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
        <p class="text-destructive text-sm">{fileError}</p>
      {/if}
    </div>

    <DialogFooter>
      <Button type="button" variant="outline" onclick={closeDialog}>
        {t('Cancel')}
      </Button>
      <Button
        type="button"
        disabled={!selectedManager || !selectedFilePath || !passphrase.trim() || selectingFile}
        onclick={handleImport}
      >
        {t('Import')}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
