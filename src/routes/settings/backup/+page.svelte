<script lang="ts">
  import { settings } from '$lib/stores/appSettings.svelte';
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
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { Spinner } from '$lib/components/ui/spinner/index.js';
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
    Shield
  } from '@lucide/svelte';
  import { fade, slide } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import { exportVaultBackup, importVaultBackup, notifyVaultRefresh } from '$lib/utils/backup';
  import type { ImportVaultProgressStage } from '$lib/utils/backup';
  import { i18n, t as translate, type I18nKey, type Locale } from '$lib/i18n.svelte';

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as I18nKey, vars);
  const isDev = import.meta.env.DEV;

  let backupSettings = $derived(settings.state.backup);

  const frequencies = [
    { value: 'daily', label: 'Daily (Default)' },
    { value: 'weekly', label: 'Weekly' },
    { value: 'custom', label: 'Custom interval' }
  ];

  function getFrequencyLabel(value: string, _locale: Locale): string {
    if (value === 'daily') return t('Daily (Default)');
    if (value === 'weekly') return t('Weekly');
    if (value === 'custom') return t('Custom interval');
    return value;
  }

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

  let showModal = $state(false);
  let modalTitle = $state('');
  let modalDescription = $state('');
  let modalConfirmLabel = $state('Continue');
  let modalDanger = $state(false);
  type ModalConfirmHandler = (passphrase: string) => Promise<void> | void;

  let modalOnConfirm = $state<ModalConfirmHandler | null>(null);
  let modalPassphrase = $state('');
  let modalMasterPassword = $state('');
  let modalBusy = $state(false);
  let modalError = $state<string | null>(null);
  let modalRequiresPassphrase = $state(true);
  let modalRequiresMasterPassword = $state(false);
  let modalStatus = $state<string | null>(null);
  let feedback = $state<{ type: 'success' | 'error'; message: string } | null>(null);

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
    requiresMasterPassword = false,
    onConfirm
  }: {
    title: string;
    description: string;
    confirmLabel?: string;
    danger?: boolean;
    requiresPassphrase?: boolean;
    requiresMasterPassword?: boolean;
    onConfirm: ModalConfirmHandler;
  }) {
    modalTitle = title;
    modalDescription = description;
    modalConfirmLabel = confirmLabel;
    modalDanger = danger;
    modalPassphrase = '';
    modalMasterPassword = '';
    modalError = null;
    modalBusy = false;
    modalRequiresPassphrase = requiresPassphrase;
    modalRequiresMasterPassword = requiresMasterPassword;
    modalOnConfirm = onConfirm;
    modalStatus = null;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    modalOnConfirm = null;
    modalPassphrase = '';
    modalMasterPassword = '';
    modalError = null;
    modalBusy = false;
    modalStatus = null;
  }

  async function handleManualBackup() {
    openModal({
      title: t('Create manual backup?'),
      description: t(
        'This creates a fresh encrypted backup of your vault using the active settings.'
      ),
      confirmLabel: t('Export backup'),
      requiresMasterPassword: true,
      onConfirm: async (passphrase) => {
        const message = await exportVaultBackup(passphrase, {
          masterPassword: modalMasterPassword
        });
        feedback = { type: 'success', message };
      }
    });
  }

  async function handleExportEncrypted() {
    openModal({
      title: t('Export encrypted data?'),
      description: t('Export your vault in encrypted form. Keep the generated file secure.'),
      confirmLabel: t('Export encrypted'),
      requiresMasterPassword: true,
      onConfirm: async (passphrase) => {
        const message = await exportVaultBackup(passphrase, {
          masterPassword: modalMasterPassword
        });
        feedback = { type: 'success', message };
      }
    });
  }

  async function handleExportPlaintext() {
    openModal({
      title: t('Export plaintext data?'),
      description: t(
        'WARNING: This exports all vault contents without encryption. Only proceed on trusted devices.'
      ),
      confirmLabel: t('Export plaintext'),
      danger: true,
      requiresMasterPassword: true,
      onConfirm: async (passphrase) => {
        const message = await exportVaultBackup(passphrase, {
          plaintext: true,
          masterPassword: modalMasterPassword
        });
        feedback = { type: 'success', message };
      }
    });
  }

  async function handleImport() {
    openModal({
      title: t('Start import process?'),
      description: t(
        'Select a previous Pulsar backup and provide its passphrase to restore your vault contents.'
      ),
      confirmLabel: t('Import backup'),
      requiresMasterPassword: true,
      onConfirm: async (passphrase) => {
        const snapshot = await importVaultBackup(passphrase, {
          masterPassword: modalMasterPassword,
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
    settings.state.backup[setting] = !settings.state.backup[setting];
    settings.save();
  }

  function updateFrequency(value: string) {
    settings.state.backup.backupFrequency = value;
    settings.save();
  }

  function updateRetention(event: Event) {
    const value = Number((event.target as HTMLInputElement).value);
    const retention = Number.isNaN(value)
      ? backupSettings.retentionCount
      : Math.min(Math.max(value, 1), 100);
    settings.state.backup.retentionCount = retention;
    settings.save();
  }

  function updateSyncMode(mode: string) {
    settings.state.backup.syncMode = mode;
    settings.save();
  }

  function openProvider(provider: string) {
    settings.state.backup.selectedProvider = provider;
    settings.save();

    openModal({
      title: t('Configure {provider}', { provider }),
      description: t('Provide credentials for your {provider} connection.', { provider }),
      requiresMasterPassword: false,
      onConfirm: () => {}
    });
  }
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  {#if feedback}
    <Alert
      variant={feedback.type === 'error' ? 'destructive' : 'default'}
      class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur"
    >
      <AlertTitle>
        {feedback.type === 'error' ? t('Something went wrong') : t('Action completed')}
      </AlertTitle>
      <AlertDescription>{feedback.message}</AlertDescription>
    </Alert>
  {/if}

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <Archive class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>{t('Backups')}</CardTitle>
          <CardDescription>
            {t('Manage automated and on-demand backups for your vault.')}
          </CardDescription>
        </div>
      </div>
      <Badge variant="secondary" class="w-fit">
        {t('Retaining')}
        {backupSettings.retentionCount}
        {t('copies')}
      </Badge>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="space-y-4">
        <div
          class="border-border/60 bg-muted/10 flex items-center justify-between gap-3 rounded-xl border px-4 py-3"
        >
          <div>
            <p class="text-foreground text-sm font-medium">
              {t('Automatic backups')}
            </p>
            <p class="text-muted-foreground text-xs">
              {t('Create backups at regular intervals based on your chosen schedule.')}
            </p>
          </div>
          <Switch
            checked={backupSettings.automaticBackups}
            aria-label="Toggle automatic backups"
            onclick={() => toggleSetting('automaticBackups')}
          />
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div class="space-y-2">
            <Label class="text-foreground text-sm font-medium">
              {t('Backup frequency')}
            </Label>
            {#key locale}
              <Select
                type="single"
                value={backupSettings.backupFrequency}
                onValueChange={updateFrequency}
              >
                <SelectTrigger aria-label="Select backup frequency" class="w-full">
                  <span data-slot="select-value" class="truncate text-sm">
                    {getFrequencyLabel(backupSettings.backupFrequency, locale) ??
                      t('Select frequency')}
                  </span>
                </SelectTrigger>
                <SelectContent>
                  {#each frequencies as option (option.value)}
                    <SelectItem value={option.value}
                      >{getFrequencyLabel(option.value, locale)}</SelectItem
                    >
                  {/each}
                </SelectContent>
              </Select>
            {/key}
          </div>

          <div class="space-y-2">
            <Label for="retention-count" class="text-foreground text-sm font-medium">
              {t('Retention count')}
            </Label>
            <Input
              id="retention-count"
              type="number"
              min="1"
              max="100"
              value={backupSettings.retentionCount}
              oninput={updateRetention}
            />
            <p class="text-muted-foreground text-xs">
              {t('Number of backup versions to keep on disk.')}
            </p>
          </div>
        </div>
      </div>

      <div class="flex flex-wrap gap-2">
        <Button type="button" class="gap-2" onclick={handleManualBackup}>
          <Download class="size-4" aria-hidden="true" />
          {t('Create manual backup')}
        </Button>
        <Button type="button" variant="outline" class="gap-2" onclick={handleImport}>
          <CloudUpload class="size-4" aria-hidden="true" />
          {t('Import data')}
        </Button>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex items-start gap-3">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <Archive class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>{t('Export options')}</CardTitle>
        <CardDescription>
          {t('Generate encrypted or plaintext exports of your vault.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="grid gap-4 md:grid-cols-2">
        <div class="border-border/60 bg-muted/10 space-y-2 rounded-xl border p-4">
          <p class="text-foreground text-sm font-semibold">
            {t('Encrypted export')}
          </p>
          <p class="text-muted-foreground text-xs">
            {t('Secured with your export passphrase.')}
          </p>
          <Button type="button" class="gap-2" onclick={handleExportEncrypted}>
            <ShieldCheck class="size-4" aria-hidden="true" />
            {t('Export encrypted')}
          </Button>
        </div>

        {#if isDev}
          <div class="border-border/60 bg-muted/10 space-y-2 rounded-xl border p-4">
            <div class="flex items-center gap-2">
              <ShieldAlert class="text-destructive size-4" aria-hidden="true" />
              <p class="text-foreground text-sm font-semibold">
                {t('Plaintext export')}
              </p>
            </div>
            <p class="text-muted-foreground text-xs">
              {t('Only use on trusted devices. Sensitive data remains unprotected.')}
            </p>
            <div class="flex items-center justify-between gap-2">
              <Switch
                checked={backupSettings.enablePlaintextExport}
                aria-label="Allow plaintext exports"
                onclick={() => toggleSetting('enablePlaintextExport')}
              />
              <Button
                type="button"
                variant="outline"
                class="text-destructive gap-2"
                onclick={handleExportPlaintext}
                disabled={!backupSettings.enablePlaintextExport}
              >
                <Shield class="size-4" aria-hidden="true" />
                {t('Export plaintext')}
              </Button>
            </div>
          </div>
        {/if}
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex items-start gap-3">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <Cloud class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>{t('Sync')}</CardTitle>
        <CardDescription>
          {t('Configure cloud providers to replicate backups across devices.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="grid gap-3 md:grid-cols-3">
        {#each syncModes as mode (mode.id)}
          <button
            type="button"
            class={cn(
              'border-border/60 bg-background/70 flex h-full flex-col items-start gap-3 rounded-xl border p-4 text-left transition',
              backupSettings.syncMode === mode.id
                ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                : 'hover:border-primary/40 hover:bg-muted/40'
            )}
            aria-pressed={backupSettings.syncMode === mode.id}
            onclick={() => updateSyncMode(mode.id)}
          >
            <mode.icon class="size-5" aria-hidden="true" />
            <div>
              <p class="text-foreground text-sm font-semibold">
                {mode.id === 'off'
                  ? t('Turned off')
                  : mode.id === 'manual'
                    ? t('Manual sync')
                    : t('Automatic sync')}
              </p>
              <p class="text-muted-foreground text-xs">
                {mode.id === 'off'
                  ? t('Backups stay local to this device.')
                  : mode.id === 'manual'
                    ? t('Trigger cloud sync on demand.')
                    : t('Keep cloud copy in sync automatically.')}
              </p>
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
              'border-border/60 bg-background/70 flex h-full flex-col items-start gap-3 rounded-xl border p-4 text-left transition',
              backupSettings.selectedProvider === provider.id
                ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                : 'hover:border-primary/40 hover:bg-muted/40'
            )}
            aria-pressed={backupSettings.selectedProvider === provider.id}
            onclick={() => openProvider(provider.id)}
          >
            <provider.icon class="size-5" aria-hidden="true" />
            <div>
              <p class="text-foreground text-sm font-semibold">{provider.name}</p>
              <p class="text-muted-foreground text-xs">
                {provider.id === 'webdav'
                  ? t('backupProviderWebdavDesc')
                  : provider.id === 'dropbox'
                    ? t('backupProviderDropboxDesc')
                    : provider.id === 's3'
                      ? t('backupProviderS3Desc')
                      : t('backupProviderCustomDesc')}
              </p>
            </div>
          </button>
        {/each}
      </div>
    </CardContent>
  </Card>
</div>

{#if showModal}
  <div
    class="bg-background/80 fixed inset-0 z-50 flex items-center justify-center backdrop-blur"
    transition:fade
  >
    <div
      class="border-border/60 bg-card w-full max-w-lg rounded-xl border p-6 shadow-xl"
      transition:slide={{ axis: 'y', duration: 200, easing: quintOut }}
    >
      <div class="flex items-center gap-3">
        <Zap
          class={cn('size-5', modalDanger ? 'text-destructive' : 'text-primary')}
          aria-hidden="true"
        />
        <div>
          <h3 class="text-foreground text-lg font-semibold">{modalTitle}</h3>
          <p class="text-muted-foreground text-xs">{modalDescription}</p>
        </div>
      </div>

      {#if modalRequiresPassphrase}
        <div class="mt-6 space-y-2">
          <Label for="backup-passphrase" class="text-foreground text-sm font-medium">
            {t('Backup passphrase')}
          </Label>
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
          <p class="text-muted-foreground text-xs">
            {t(
              'This passphrase encrypts or decrypts your vault backup. Use the same passphrase you will remember later.'
            )}
          </p>
        </div>
      {/if}

      {#if modalRequiresMasterPassword}
        <div class="mt-4 space-y-2">
          <Label for="master-password" class="text-foreground text-sm font-medium">
            {t('loginMasterPasswordLabel')}
          </Label>
          <Input
            id="master-password"
            type="password"
            autocomplete="current-password"
            value={modalMasterPassword}
            oninput={(event) => {
              modalMasterPassword = (event.target as HTMLInputElement).value;
              modalError = null;
            }}
          />
          <p class="text-muted-foreground text-xs">
            {t('Confirm with your master password to proceed.')}
          </p>
        </div>
      {/if}

      {#if modalError}
        <p class="text-destructive mt-4 text-sm">{modalError}</p>
      {/if}

      {#if modalStatus}
        <p class="text-muted-foreground mt-4 flex items-center gap-2 text-sm">
          <Spinner class="size-4" aria-hidden="true" />
          <span>{modalStatus}</span>
        </p>
      {/if}

      <div class="mt-6 flex justify-end gap-2">
        <Button type="button" variant="ghost" onclick={closeModal}>
          {t('Cancel')}
        </Button>
        <Button
          type="button"
          variant={modalDanger ? 'destructive' : 'default'}
          class="gap-2"
          disabled={modalBusy ||
            (modalRequiresPassphrase && modalPassphrase.trim().length === 0) ||
            (modalRequiresMasterPassword && modalMasterPassword.trim().length === 0)}
          onclick={async () => {
            if (!modalOnConfirm) {
              closeModal();
              return;
            }

            const trimmed = modalPassphrase.trim();
            const masterTrimmed = modalMasterPassword.trim();
            if (modalRequiresPassphrase && trimmed.length === 0) {
              modalError = 'A passphrase is required.';
              return;
            }
            if (modalRequiresMasterPassword && masterTrimmed.length === 0) {
              modalError = t('loginMasterPasswordPlaceholder');
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
            <Spinner class="size-4" aria-hidden="true" />
          {/if}
          {modalConfirmLabel}
        </Button>
      </div>
    </div>
  </div>
{/if}
