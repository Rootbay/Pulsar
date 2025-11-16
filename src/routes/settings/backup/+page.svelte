<script lang="ts">
  import { derived } from 'svelte/store';
  import { backupSettings } from '$lib/stores/backup';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger
  } from '$lib/components/ui/select';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import {
    Alert,
    AlertDescription,
    AlertTitle
  } from '$lib/components/ui/alert';
  import { cn } from '$lib/utils';
  import {
    Archive,
    Box,
    Cloud,
    Database,
    Download,
    HardDrive,
    HardHat,
    ShieldAlert,
    CloudUpload,
    Zap,
    ShieldCheck,
    Shield,
    Loader2
  } from '@lucide/svelte';
  import { fade, slide } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import {
    exportVaultBackup,
    importVaultBackup,
    notifyVaultRefresh
  } from '$lib/utils/backup';
  import type { ImportVaultProgressStage } from '$lib/utils/backup';

  const frequencies = [
    { value: 'daily', label: 'Daily (Default)' },
    { value: 'weekly', label: 'Weekly' },
    { value: 'custom', label: 'Custom interval' }
  ];

  const syncModes = [
    {
      id: 'off',
      title: 'Turned off',
      description: 'Backups stay local to this device.',
      icon: HardDrive
    },
    {
      id: 'manual',
      title: 'Manual sync',
      description: 'Trigger cloud sync on demand.',
      icon: CloudUpload
    },
    {
      id: 'auto',
      title: 'Automatic sync',
      description: 'Keep cloud copy in sync automatically.',
      icon: Cloud
    }
  ];

  const providers = [
    {
      id: 'webdav',
      name: 'WebDAV',
      description: 'Connect to any WebDAV compatible storage.',
      icon: Cloud
    },
    {
      id: 'dropbox',
      name: 'Dropbox',
      description: 'Use Dropbox as your backup destination.',
      icon: Box
    },
    {
      id: 's3',
      name: 'Amazon S3',
      description: 'Sync backups to your S3 bucket.',
      icon: Database
    },
    {
      id: 'custom',
      name: 'Custom',
      description: 'Provide custom credentials for another provider.',
      icon: HardHat
    }
  ];

  const state = derived(backupSettings, ($settings) => ({ ...$settings }));

  let showModal = false;
  let modalTitle = '';
  let modalDescription = '';
  let modalConfirmLabel = 'Continue';
  let modalDanger = false;
  type ModalConfirmHandler = (passphrase: string) => Promise<void> | void;

  let modalOnConfirm: ModalConfirmHandler | null = null;
  let modalPassphrase = '';
  let modalBusy = false;
  let modalError: string | null = null;
  let modalRequiresPassphrase = true;
  let modalStatus: string | null = null;
  let feedback: { type: 'success' | 'error'; message: string } | null = null;

  const importProgressMessages: Record<ImportVaultProgressStage, string> = {
    decrypting: 'Decrypting backup…',
    restoring: 'Restoring vault contents…'
  };

  function openModal({
    title,
    description,
    confirmLabel = 'Continue',
    danger = false,
    requiresPassphrase = true,
    onConfirm
  }: {
    title: string;
    description: string;
    confirmLabel?: string;
    danger?: boolean;
    requiresPassphrase?: boolean;
    onConfirm: ModalConfirmHandler;
  }) {
    modalTitle = title;
    modalDescription = description;
    modalConfirmLabel = confirmLabel;
    modalDanger = danger;
    modalPassphrase = '';
    modalError = null;
    modalBusy = false;
    modalRequiresPassphrase = requiresPassphrase;
    modalOnConfirm = onConfirm;
    modalStatus = null;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    modalOnConfirm = null;
    modalPassphrase = '';
    modalError = null;
    modalBusy = false;
    modalStatus = null;
  }

  async function handleManualBackup() {
    openModal({
      title: 'Create manual backup?',
      description: 'This creates a fresh encrypted backup of your vault using the active settings.',
      confirmLabel: 'Export backup',
      onConfirm: async (passphrase) => {
        const message = await exportVaultBackup(passphrase);
        feedback = { type: 'success', message };
      }
    });
  }

  async function handleExportEncrypted() {
    openModal({
      title: 'Export encrypted data?',
      description: 'Export your vault in encrypted form. Keep the generated file secure.',
      confirmLabel: 'Export encrypted',
      onConfirm: async (passphrase) => {
        const message = await exportVaultBackup(passphrase);
        feedback = { type: 'success', message };
      }
    });
  }

  async function handleExportPlaintext() {
    openModal({
      title: 'Export plaintext data?',
      description:
        'WARNING: This exports all vault contents without encryption. Only proceed on trusted devices.',
      confirmLabel: 'Export plaintext',
      danger: true,
      onConfirm: async (passphrase) => {
        const message = await exportVaultBackup(passphrase, { plaintext: true });
        feedback = { type: 'success', message };
      }
    });
  }

  async function handleImport() {
    openModal({
      title: 'Start import process?',
      description:
        'Select a previous Pulsar backup and provide its passphrase to restore your vault contents.',
      confirmLabel: 'Import backup',
      onConfirm: async (passphrase) => {
        const snapshot = await importVaultBackup(passphrase, {
          onProgress: (stage) => {
            modalStatus = importProgressMessages[stage];
          }
        });
        const totalItems = snapshot.passwordItems.length;
        const tagCount = snapshot.buttons.length;
        const message = `Imported ${totalItems} saved item${totalItems === 1 ? '' : 's'} and ${tagCount} tag${tagCount === 1 ? '' : 's'}.`;
        feedback = { type: 'success', message };
        notifyVaultRefresh('import');
      }
    });
  }

  function toggleSetting(setting: 'automaticBackups' | 'enablePlaintextExport') {
    backupSettings.update((current) => ({
      ...current,
      [setting]: !current[setting]
    }));
  }

  function updateFrequency(value: string) {
    backupSettings.update((current) => ({
      ...current,
      backupFrequency: value
    }));
  }

  function updateRetention(event: Event) {
    const value = Number((event.target as HTMLInputElement).value);
    backupSettings.update((current) => ({
      ...current,
      retentionCount: Number.isNaN(value) ? current.retentionCount : Math.min(Math.max(value, 1), 100)
    }));
  }

  function updateSyncMode(mode: string) {
    backupSettings.update((current) => ({ ...current, syncMode: mode }));
  }

  function openProvider(provider: string) {
    backupSettings.update((current) => ({ ...current, selectedProvider: provider }));
    openModal({
      title: `Configure ${provider}`,
      description: `Provide credentials for your ${provider} connection.`,
      onConfirm: () => {}
    });
  }
</script>

<div class="flex-1 min-h-0 space-y-6 px-6 py-8">
  {#if feedback}
    <Alert
      variant={feedback.type === 'error' ? 'destructive' : 'default'}
      class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70"
    >
      <AlertTitle>{feedback.type === 'error' ? 'Something went wrong' : 'Action completed'}</AlertTitle>
      <AlertDescription>{feedback.message}</AlertDescription>
    </Alert>
  {/if}

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
      <div class="flex items-center gap-3">
        <div class="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
          <Archive class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>Backups</CardTitle>
          <CardDescription>Manage automated and on-demand backups for your vault.</CardDescription>
        </div>
      </div>
      <Badge variant="secondary" class="w-fit">
        Retaining {$state.retentionCount} copies
      </Badge>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="space-y-4">
        <div class="flex items-center justify-between gap-3 rounded-xl border border-border/60 bg-muted/10 px-4 py-3">
          <div>
            <p class="text-sm font-medium text-foreground">Automatic backups</p>
            <p class="text-xs text-muted-foreground">
              Create backups at regular intervals based on your chosen schedule.
            </p>
          </div>
          <Switch
            checked={$state.automaticBackups}
            aria-label="Toggle automatic backups"
            onclick={() => toggleSetting('automaticBackups')}
          />
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div class="space-y-2">
            <Label class="text-sm font-medium text-foreground">Backup frequency</Label>
            <Select type="single" value={$state.backupFrequency} onValueChange={updateFrequency}>
              <SelectTrigger aria-label="Select backup frequency" class="w-full">
                <span data-slot="select-value" class="truncate text-sm">
                  {frequencies.find((item) => item.value === $state.backupFrequency)?.label ?? 'Select frequency'}
                </span>
              </SelectTrigger>
              <SelectContent>
                {#each frequencies as option}
                  <SelectItem value={option.value}>{option.label}</SelectItem>
                {/each}
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-2">
            <Label for="retention-count" class="text-sm font-medium text-foreground">Retention count</Label>
            <Input
              id="retention-count"
              type="number"
              min="1"
              max="100"
              value={$state.retentionCount}
              oninput={updateRetention}
            />
            <p class="text-xs text-muted-foreground">Number of backup versions to keep on disk.</p>
          </div>
        </div>
      </div>

      <div class="flex flex-wrap gap-2">
        <Button type="button" class="gap-2" onclick={handleManualBackup}>
          <Download class="size-4" aria-hidden="true" />
          Create manual backup
        </Button>
        <Button type="button" variant="outline" class="gap-2" onclick={handleImport}>
          <CloudUpload class="size-4" aria-hidden="true" />
          Import data
        </Button>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex items-start gap-3">
      <div class="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Archive class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Export options</CardTitle>
        <CardDescription>Generate encrypted or plaintext exports of your vault.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="grid gap-4 md:grid-cols-2">
        <div class="space-y-2 rounded-xl border border-border/60 bg-muted/10 p-4">
          <p class="text-sm font-semibold text-foreground">Encrypted export</p>
          <p class="text-xs text-muted-foreground">Secured with your export passphrase.</p>
          <Button type="button" class="gap-2" onclick={handleExportEncrypted}>
            <ShieldCheck class="size-4" aria-hidden="true" />
            Export encrypted
          </Button>
        </div>

        <div class="space-y-2 rounded-xl border border-border/60 bg-muted/10 p-4">
          <div class="flex items-center gap-2">
            <ShieldAlert class="size-4 text-destructive" aria-hidden="true" />
            <p class="text-sm font-semibold text-foreground">Plaintext export</p>
          </div>
          <p class="text-xs text-muted-foreground">
            Only use on trusted devices. Sensitive data remains unprotected.
          </p>
          <div class="flex items-center justify-between gap-2">
            <Switch
              checked={$state.enablePlaintextExport}
              aria-label="Allow plaintext exports"
              onclick={() => toggleSetting('enablePlaintextExport')}
            />
            <Button
              type="button"
              variant="outline"
              class="gap-2 text-destructive"
              onclick={handleExportPlaintext}
              disabled={!$state.enablePlaintextExport}
            >
              <Shield class="size-4" aria-hidden="true" />
              Export plaintext
            </Button>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex items-start gap-3">
      <div class="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Cloud class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Sync</CardTitle>
        <CardDescription>Configure cloud providers to replicate backups across devices.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="grid gap-3 md:grid-cols-3">
        {#each syncModes as mode (mode.id)}
          <button
            type="button"
            class={cn(
              'flex h-full flex-col items-start gap-3 rounded-xl border border-border/60 bg-background/70 p-4 text-left transition',
              $state.syncMode === mode.id
                ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                : 'hover:border-primary/40 hover:bg-muted/40'
            )}
            aria-pressed={$state.syncMode === mode.id}
            onclick={() => updateSyncMode(mode.id)}
          >
            <mode.icon class="size-5" aria-hidden="true" />
            <div>
              <p class="text-sm font-semibold text-foreground">{mode.title}</p>
              <p class="text-xs text-muted-foreground">{mode.description}</p>
            </div>
          </button>
        {/each}
      </div>

      <Separator class="bg-border/60" />

      <div class="grid gap-4 md:grid-cols-2">
        {#each providers as provider (provider.id)}
          <button
            type="button"
            class={cn(
              'flex h-full flex-col items-start gap-3 rounded-xl border border-border/60 bg-background/70 p-4 text-left transition',
              $state.selectedProvider === provider.id
                ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                : 'hover:border-primary/40 hover:bg-muted/40'
            )}
            aria-pressed={$state.selectedProvider === provider.id}
            onclick={() => openProvider(provider.id)}
          >
            <provider.icon class="size-5" aria-hidden="true" />
            <div>
              <p class="text-sm font-semibold text-foreground">{provider.name}</p>
              <p class="text-xs text-muted-foreground">{provider.description}</p>
            </div>
          </button>
        {/each}
      </div>
    </CardContent>
  </Card>
</div>

{#if showModal}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur"
    transition:fade
  >
    <div
      class="w-full max-w-lg rounded-xl border border-border/60 bg-card p-6 shadow-xl"
      transition:slide={{ axis: 'y', duration: 200, easing: quintOut }}
    >
      <div class="flex items-center gap-3">
        <Zap class={cn('size-5', modalDanger ? 'text-destructive' : 'text-primary')} aria-hidden="true" />
        <div>
          <h3 class="text-lg font-semibold text-foreground">{modalTitle}</h3>
          <p class="text-xs text-muted-foreground">{modalDescription}</p>
        </div>
      </div>

      {#if modalRequiresPassphrase}
        <div class="mt-6 space-y-2">
          <Label for="backup-passphrase" class="text-sm font-medium text-foreground">Backup passphrase</Label>
          <Input
            id="backup-passphrase"
            type="password"
            autocomplete="new-password"
            value={modalPassphrase}
            oninput={(event) => {
              modalPassphrase = (event.target as HTMLInputElement).value;
              modalError = null;
            }}
          />
          <p class="text-xs text-muted-foreground">
            This passphrase encrypts or decrypts your vault backup. Use the same passphrase you will remember later.
          </p>
        </div>
      {/if}

      {#if modalError}
        <p class="mt-4 text-sm text-destructive">{modalError}</p>
      {/if}

      {#if modalStatus}
        <p class="mt-4 flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 class="size-4 animate-spin" aria-hidden="true" />
          <span>{modalStatus}</span>
        </p>
      {/if}

      <div class="mt-6 flex justify-end gap-2">
        <Button type="button" variant="ghost" onclick={closeModal}>Cancel</Button>
        <Button
          type="button"
          variant={modalDanger ? 'destructive' : 'default'}
          class="gap-2"
          disabled={
            modalBusy || (modalRequiresPassphrase && modalPassphrase.trim().length === 0)
          }
          onclick={async () => {
            if (!modalOnConfirm) {
              closeModal();
              return;
            }

            const trimmed = modalPassphrase.trim();
            if (modalRequiresPassphrase && trimmed.length === 0) {
              modalError = 'A passphrase is required.';
              return;
            }

            modalBusy = true;
            modalError = null;

            try {
              await modalOnConfirm(trimmed);
              closeModal();
            } catch (error) {
              if (typeof error === 'string') {
                modalError = error;
              } else if (error instanceof Error) {
                modalError = error.message;
              } else {
                modalError = 'An unexpected error occurred while processing the request.';
              }
              feedback = { type: 'error', message: modalError };
              modalStatus = null;
            } finally {
              modalBusy = false;
            }
          }}
        >
          {#if modalBusy}
            <Loader2 class="size-4 animate-spin" aria-hidden="true" />
          {/if}
          {modalConfirmLabel}
        </Button>
      </div>
    </div>
  </div>
{/if}
