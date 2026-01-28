<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { toast } from '$lib/components/ui/sonner';
  import { vaultStore } from '$lib/stores/vault.svelte';
  import { addRecentDatabase, removeRecentDatabase } from '$lib/stores/recentDatabases.svelte';
  import type { VaultSettings } from '$lib/config/settings';
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
  import { Badge } from '$lib/components/ui/badge';
  import { cn } from '$lib/utils';
  import {
    Archive,
    ChartColumn,
    Copy,
    Database,
    FolderKanban,
    HardDriveDownload,
    RefreshCw,
    ShieldAlert,
    ShieldCheck,
    Trash2
  } from '@lucide/svelte';
  import { exportVaultBackup, importVaultBackup, notifyVaultRefresh } from '$lib/utils/backup';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { callBackend } from '$lib/utils/backend';
  import PassphraseDialog from '$lib/components/ui/PassphraseDialog.svelte';

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as I18nKey, vars);

  interface BackendVault {
    id: string;
    path: string;
    name: string;
    status: 'unlocked' | 'locked' | 'available';
    encrypted: boolean;
    size_bytes?: number | null;
    modified_at?: number | null;
    item_count?: number | null;
    settings: VaultSettings;
  }

  interface Vault {
    id: string;
    path: string;
    name: string;
    status: 'unlocked' | 'locked' | 'available';
    encrypted: boolean;
    sizeBytes?: number;
    modifiedAt?: number;
    itemCount?: number;
    settings: VaultSettings;
  }

  let vaults = $state<Vault[]>([]);
  let selectedVaultId = $state<string | null>(null);

  let activeVaultSettings = $derived(vaultStore.settings);

  const selectedVault = $derived(vaults.find((vault) => vault.id === selectedVaultId));

  const totalItems = $derived(vaults.reduce((sum, vault) => sum + (vault.itemCount ?? 0), 0));

  const encryptedCount = $derived(vaults.filter((vault) => vault.encrypted).length);

  let weakPasswordsCount = $state(0);
  let duplicatePasswordsCount = $state(0);

  let loadingVaults = $state(false);
  let busyAction = $state<'import' | 'create' | 'backup' | 'restore' | 'export' | null>(null);

  let passphraseDialogOpen = $state(false);
  let passphraseDialogTitle = $state('');
  let passphraseDialogDescription = $state('');
  let passphraseDialogConfirmLabel = $state('');
  let passphraseDialogAction = $state<((passphrase: string) => Promise<void>) | null>(null);

  const relativeTimeFormatter = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });

  function openPassphraseDialog(
    title: string,
    description: string,
    confirmLabel: string,
    action: (passphrase: string) => Promise<void>
  ) {
    passphraseDialogTitle = title;
    passphraseDialogDescription = description;
    passphraseDialogConfirmLabel = confirmLabel;
    passphraseDialogAction = action;
    passphraseDialogOpen = true;
  }

  async function handleImportVault(): Promise<void> {
    if (busyAction) {
      return;
    }

    try {
      const sourcePath = await callBackend<string>('pick_open_file');

      openPassphraseDialog(
        t('settingsVaultImportAction'),
        t('settingsVaultImportPassphrasePrompt'),
        t('settingsVaultImportAction'),
        async (passphrase) => {
          busyAction = 'import';
          try {
            await importVaultBackup(passphrase, { sourcePath });
            toast.success(t('settingsVaultImportSuccess'));
            notifyVaultRefresh('import');
            await refreshVaults();
          } catch (cause) {
            console.error('Failed to import vault:', cause);
            toast.error(resolveErrorMessage(cause, t('settingsVaultImportFailed')));
          } finally {
            busyAction = null;
          }
        }
      );
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }
      console.error('Failed to pick file:', cause);
    }
  }

  async function handleCreateVault(): Promise<void> {
    if (busyAction) {
      return;
    }

    busyAction = 'create';

    try {
      const picked = await callBackend<string>('pick_save_file');
      const withExt: string = picked.endsWith('.psec') ? picked : `${picked}.psec`;
      const sep = withExt.includes('\\') ? '\\' : '/';
      const lastSep = withExt.lastIndexOf(sep);
      const baseDir = lastSep === -1 ? '' : withExt.slice(0, lastSep);
      const baseName = lastSep === -1 ? withExt : withExt.slice(lastSep + 1);
      const stem = baseName.endsWith('.psec') ? baseName.slice(0, -5) : baseName;
      const folder = baseDir ? `${baseDir}${sep}${stem}` : stem;
      const finalPath = `${folder}${sep}${stem}.psec`;

      await callBackend('switch_database', { dbPath: finalPath });
      await addRecentDatabase(finalPath);

      toast.success(t('settingsVaultCreateSuccess'));
      notifyVaultRefresh('create');
      await refreshVaults({ preserveSelection: false });
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }

      console.error('Failed to create vault:', cause);
      toast.error(resolveErrorMessage(cause, t('settingsVaultCreateFailed')));
    } finally {
      busyAction = null;
    }
  }

  async function runBackup(): Promise<void> {
    if (busyAction) {
      return;
    }

    openPassphraseDialog(
      t('settingsVaultBackupNow'),
      t('settingsVaultBackupPassphrasePrompt'),
      t('settingsVaultBackupNow'),
      async (passphrase) => {
        busyAction = 'backup';
        try {
          const message = await exportVaultBackup(passphrase);
          toast.success(message);
          notifyVaultRefresh('backup');
          await refreshVaults();
        } catch (cause) {
          console.error('Failed to run backup:', cause);
          toast.error(resolveErrorMessage(cause, t('settingsVaultBackupFailed')));
        } finally {
          busyAction = null;
        }
      }
    );
  }

  async function restoreVault(): Promise<void> {
    if (busyAction) {
      return;
    }

    try {
      const sourcePath = await callBackend<string>('pick_open_file');

      openPassphraseDialog(
        t('settingsVaultRestore'),
        t('settingsVaultRestorePassphrasePrompt'),
        t('settingsVaultRestore'),
        async (passphrase) => {
          busyAction = 'restore';
          try {
            await importVaultBackup(passphrase, { sourcePath });
            toast.success(t('settingsVaultRestoreSuccess'));
            notifyVaultRefresh('restore');
            await refreshVaults();
          } catch (cause) {
            console.error('Failed to restore vault:', cause);
            toast.error(resolveErrorMessage(cause, t('settingsVaultRestoreFailed')));
          } finally {
            busyAction = null;
          }
        }
      );
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }
      console.error('Failed to pick file:', cause);
    }
  }

  async function exportVault(): Promise<void> {
    if (busyAction) {
      return;
    }

    openPassphraseDialog(
      t('settingsVaultExport'),
      t('settingsVaultExportPassphrasePrompt'),
      t('settingsVaultExport'),
      async (passphrase) => {
        const exportPlaintext = window.confirm(t('settingsVaultExportConfirmPlaintext'));
        busyAction = 'export';
        try {
          const message = await exportVaultBackup(passphrase, { plaintext: exportPlaintext });
          toast.success(message);
          notifyVaultRefresh('export');
          await refreshVaults();
        } catch (cause) {
          console.error('Failed to export vault:', cause);
          toast.error(resolveErrorMessage(cause, t('settingsVaultExportFailed')));
        } finally {
          busyAction = null;
        }
      }
    );
  }

  function isDialogCancelled(error: unknown): boolean {
    return typeof error === 'string' && error.toLowerCase().includes('cancelled');
  }

  function resolveErrorMessage(error: unknown, fallback: string): string {
    if (typeof error === 'string') return error;
    if (error instanceof Error) return error.message;
    try {
      return JSON.stringify(error);
    } catch (cause) {
      console.error('Failed to serialise error:', cause);
      return fallback;
    }
  }

  function formatBytes(bytes?: number): string {
    if (bytes === undefined) {
      return t('settingsVaultUnknownSize');
    }

    if (bytes === 0) {
      return '0 B';
    }

    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let value = bytes;
    let unitIndex = 0;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }

    return `${value.toFixed(value < 10 && unitIndex > 0 ? 1 : 0)} ${units[unitIndex]}`;
  }

  function formatRelativeTime(timestamp?: number): string {
    if (!timestamp) {
      return t('settingsVaultUnknown');
    }

    const difference = timestamp - Date.now();
    const divisions: Array<[Intl.RelativeTimeFormatUnit, number]> = [
      ['year', 1000 * 60 * 60 * 24 * 365],
      ['month', 1000 * 60 * 60 * 24 * 30],
      ['week', 1000 * 60 * 60 * 24 * 7],
      ['day', 1000 * 60 * 60 * 24],
      ['hour', 1000 * 60 * 60],
      ['minute', 1000 * 60],
      ['second', 1000]
    ];

    for (const [unit, amount] of divisions) {
      const delta = difference / amount;
      if (Math.abs(delta) >= 1 || unit === 'second') {
        return relativeTimeFormatter.format(Math.round(delta), unit);
      }
    }

    return t('settingsVaultJustNow');
  }

  function formatItemBadge(count: number | undefined): string {
    if (typeof count === 'number') {
      const label = count === 1 ? t('itemSingular') : t('itemPlural');
      return `${count} ${label}`;
    }

    return t('settingsVaultItemsUnavailable');
  }

  function formatStatusLabel(status: Vault['status']): string {
    if (status === 'unlocked') return t('settingsVaultStatusUnlocked');
    if (status === 'locked') return t('settingsVaultStatusLocked');
    if (status === 'available') return t('settingsVaultStatusAvailable');
    return t('settingsVaultUnknown');
  }

  async function fetchSecurityReport() {
    try {
      const report = await callBackend<{
        reusedPasswords: Array<{ count: number }>;
        weakPasswordsCount: number;
      }>('get_security_report');
      weakPasswordsCount = report.weakPasswordsCount;
      duplicatePasswordsCount = report.reusedPasswords.reduce(
        (sum, group) => sum + (group.count - 1),
        0
      );
    } catch (e) {
      console.error('Failed to fetch security report:', e);
      toast.error(t('settingsVaultLoadError'));
    }
  }

  async function refreshVaults({ preserveSelection = true } = {}): Promise<void> {
    loadingVaults = true;

    try {
      const response = await callBackend<BackendVault[]>('list_vaults');
      const mapped = response.map((vault) => ({
        id: vault.id,
        path: vault.path,
        name: vault.name,
        status: vault.status,
        encrypted: vault.encrypted,
        sizeBytes: vault.size_bytes ?? undefined,
        modifiedAt: vault.modified_at ?? undefined,
        itemCount: vault.item_count ?? undefined,
        settings: vault.settings
      }));

      vaults = mapped;
      await fetchSecurityReport();

      if (!mapped.length) {
        selectedVaultId = null;
        return;
      }

      const currentSelection = preserveSelection ? selectedVaultId : null;
      const fallback = mapped[0].id;
      const nextSelection =
        currentSelection && mapped.some((vault) => vault.id === currentSelection)
          ? currentSelection
          : fallback;

      selectedVaultId = nextSelection;

      const active = mapped.find((vault) => vault.id === nextSelection);
      if (active) {
        vaultStore.selectVault(active.id, { ...active.settings, name: active.name });
      }
    } catch (cause) {
      console.error('Failed to load vaults:', cause);
      toast.error(t('settingsVaultLoadError'));
    } finally {
      loadingVaults = false;
    }
  }

  function selectVault(id: string): void {
    const vault = vaults.find((entry) => entry.id === id);
    if (!vault) {
      return;
    }

    selectedVaultId = id;
    vaultStore.selectVault(id, { ...vault.settings, name: vault.name });
  }

  function updateVaultSetting(setting: keyof VaultSettings, value: boolean): void {
    vaultStore.updateSettings((current) => ({
      ...current,
      [setting]: value
    }));
  }

  function updateVaultName(event: Event): void {
    const value = (event.target as HTMLInputElement).value;
    vaultStore.updateSettings((current) => ({
      ...current,
      name: value
    }));
  }

  function removeVault(id: string): void {
    const vault = vaults.find((entry) => entry.id === id);
    if (!vault) {
      return;
    }

    if (!window.confirm(t('settingsVaultRemoveConfirm', { name: vault.name }))) {
      return;
    }

    vaultStore.clearVault(id);
    removeRecentDatabase(vault.path);

    vaults = vaults.filter((entry) => entry.id !== id);

    if (!vaults.length) {
      selectedVaultId = null;
      return;
    }

    if (selectedVaultId === id) {
      selectVault(vaults[0].id);
    }

    void refreshVaults();
  }

  onMount(() => {
    void refreshVaults({ preserveSelection: false });
  });

  $effect(() => {
    const id = selectedVaultId;
    const currentSettings = activeVaultSettings;
    if (id && currentSettings) {
      const currentVaults = untrack(() => vaults);
      const index = currentVaults.findIndex((v) => v.id === id);

      if (index !== -1) {
        const v = currentVaults[index];
        const hasChanges =
          v.name !== currentSettings.name ||
          v.settings.totp !== currentSettings.totp ||
          v.settings.backups !== currentSettings.backups ||
          v.settings.compression !== currentSettings.compression;

        if (hasChanges) {
          const nextVaults = [...currentVaults];
          nextVaults[index] = {
            ...v,
            name: currentSettings.name,
            settings: { ...v.settings, ...currentSettings }
          };
          vaults = nextVaults;
        }
      }
    }
  });
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <FolderKanban class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t('settingsVaultTitle')}
          </CardTitle>
          <CardDescription>
            {t('settingsVaultSubtitle')}
          </CardDescription>
        </div>
      </div>
      <div class="flex gap-2">
        <Button
          type="button"
          variant="outline"
          class="gap-2"
          onclick={handleImportVault}
          disabled={busyAction !== null}
        >
          <HardDriveDownload class="size-4" aria-hidden="true" />
          {t('settingsVaultImportAction')}
        </Button>
        <Button
          type="button"
          class="gap-2"
          onclick={handleCreateVault}
          disabled={busyAction !== null}
        >
          <Archive class="size-4" aria-hidden="true" />
          {t('settingsVaultCreateAction')}
        </Button>
        <div class="border-border/60 ml-2 border-l pl-2">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="text-muted-foreground hover:text-primary size-9 h-10 w-10"
            onclick={() => refreshVaults()}
            disabled={loadingVaults || busyAction !== null}
            aria-label={t('totpRefresh')}
          >
            <RefreshCw class={cn('size-4', loadingVaults && 'animate-spin')} aria-hidden="true" />
          </Button>
        </div>
      </div>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="grid gap-4 lg:grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)]">
        <div class="space-y-3">
          {#if loadingVaults}
            <div
              class="border-border/60 bg-muted/20 text-muted-foreground rounded-xl border p-4 text-sm"
            >
              {t('settingsVaultLoading')}
            </div>
          {:else if !vaults.length}
            <div
              class="border-border/60 bg-muted/20 text-muted-foreground rounded-xl border p-4 text-sm"
            >
              {t('settingsVaultEmptyState')}
            </div>
          {:else}
            {#each vaults as vault (vault.id)}
              <button
                type="button"
                class={cn(
                  'border-border/60 bg-background/80 flex w-full flex-col gap-3 rounded-xl border p-4 text-left transition',
                  selectedVault?.id === vault.id
                    ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                    : 'hover:border-primary/40 hover:bg-muted/40'
                )}
                onclick={() => selectVault(vault.id)}
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="flex-1 space-y-1">
                    <div class="flex flex-wrap items-center gap-2">
                      <p class="text-foreground text-sm font-semibold">{vault.name}</p>
                      <div class="flex flex-wrap items-center gap-1 text-[10px]">
                        <Badge variant="secondary" class="capitalize">
                          {formatStatusLabel(vault.status)}
                        </Badge>
                        <Badge variant="outline">{formatItemBadge(vault.itemCount)}</Badge>
                      </div>
                    </div>
                    <div class="flex items-center gap-1 overflow-hidden">
                      <p class="text-muted-foreground truncate text-xs" title={vault.path}>
                        {vault.path}
                      </p>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        class="text-muted-foreground hover:text-primary size-6 shrink-0"
                        onclick={(e) => {
                          e.stopPropagation();
                          navigator.clipboard.writeText(vault.path);
                          toast.success(t('totpCopySuccess'));
                        }}
                      >
                        <Copy class="size-3" aria-hidden="true" />
                      </Button>
                    </div>
                  </div>
                </div>
                <div class="text-muted-foreground flex flex-wrap items-center gap-4 text-xs">
                  <span>{formatBytes(vault.sizeBytes)}</span>
                  <span>
                    {t('settingsVaultLastModified')}
                    {formatRelativeTime(vault.modifiedAt)}
                  </span>
                  {#if vault.encrypted}
                    <span class="text-chart-success flex items-center gap-1">
                      <ShieldCheck class="size-3" />
                      {t('settingsVaultEncrypted')}
                    </span>
                  {:else}
                    <span class="text-destructive flex items-center gap-1">
                      <ShieldAlert class="size-3" />
                      {t('settingsVaultNotEncrypted')}
                    </span>
                  {/if}
                </div>
              </button>
            {/each}
          {/if}
        </div>

        <div class="border-border/60 bg-background/70 space-y-4 rounded-xl border p-4">
          <div class="flex items-start justify-between gap-2">
            <div>
              <p class="text-foreground text-sm font-semibold">
                {t('settingsVaultSelectedTitle')}
              </p>
              <p class="text-muted-foreground text-xs">
                {t('settingsVaultSelectedSubtitle')}
              </p>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              class="text-muted-foreground hover:text-destructive size-8"
              aria-label={selectedVault
                ? t('settingsVaultRemoveAria', { name: selectedVault.name })
                : t('settingsVaultRemoveAriaFallback')}
              onclick={() => selectedVault && removeVault(selectedVault.id)}
              disabled={!selectedVault || busyAction !== null}
            >
              <Trash2 class="size-4" aria-hidden="true" />
            </Button>
          </div>

          {#if selectedVault}
            <div class="space-y-3">
              <Label for="vault-name" class="text-foreground text-sm font-medium">
                {t('settingsVaultDisplayName')}
              </Label>
              <Input
                id="vault-name"
                type="text"
                value={activeVaultSettings.name}
                placeholder={t('settingsVaultNamePlaceholder')}
                oninput={updateVaultName}
              />
            </div>

            <div class="space-y-3">
              <div
                class="border-border/60 bg-muted/10 flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
              >
                <div>
                  <p class="text-foreground text-sm font-medium">
                    {t('settingsVaultTotpTitle')}
                  </p>
                  <p class="text-muted-foreground text-xs">
                    {t('settingsVaultTotpDesc')}
                  </p>
                </div>
                <Switch
                  checked={activeVaultSettings.totp}
                  aria-label="Toggle per-entry TOTP storage"
                  onCheckedChange={(checked) => updateVaultSetting('totp', checked)}
                  disabled={busyAction !== null}
                />
              </div>

              <div
                class="border-border/60 bg-muted/10 flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
              >
                <div>
                  <p class="text-foreground text-sm font-medium">
                    {t('settingsVaultAutoBackupTitle')}
                  </p>
                  <p class="text-muted-foreground text-xs">
                    {t('settingsVaultAutoBackupDesc')}
                  </p>
                </div>
                <Switch
                  checked={activeVaultSettings.backups}
                  aria-label="Toggle automatic backups"
                  onCheckedChange={(checked) => updateVaultSetting('backups', checked)}
                  disabled={busyAction !== null}
                />
              </div>

              <div
                class="border-border/60 bg-muted/10 flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
              >
                <div>
                  <p class="text-foreground text-sm font-medium">
                    {t('settingsVaultCompressionTitle')}
                  </p>
                  <p class="text-muted-foreground text-xs">
                    {t('settingsVaultCompressionDesc')}
                  </p>
                </div>
                <Switch
                  checked={activeVaultSettings.compression}
                  aria-label="Toggle compression"
                  onCheckedChange={(checked) => updateVaultSetting('compression', checked)}
                  disabled={busyAction !== null}
                />
              </div>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={runBackup}
                disabled={busyAction !== null}
              >
                <Archive class="size-4" aria-hidden="true" />
                {t('settingsVaultBackupNow')}
              </Button>
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={restoreVault}
                disabled={busyAction !== null}
              >
                <Database class="size-4" aria-hidden="true" />
                {t('settingsVaultRestore')}
              </Button>
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={exportVault}
                disabled={busyAction !== null}
              >
                <HardDriveDownload class="size-4" aria-hidden="true" />
                {t('settingsVaultExport')}
              </Button>
            </div>
          {:else}
            <p class="text-muted-foreground text-sm">
              {t('settingsVaultSelectPrompt')}
            </p>
          {/if}
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex flex-row items-start justify-between gap-3">
      <div class="flex items-start gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <ChartColumn class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t('settingsVaultInsightsTitle')}
          </CardTitle>
          <CardDescription>
            {t('settingsVaultInsightsDesc')}
          </CardDescription>
        </div>
      </div>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        class="text-muted-foreground hover:text-primary size-8"
        onclick={() => fetchSecurityReport()}
        disabled={loadingVaults}
        aria-label={t('totpRefresh')}
      >
        <RefreshCw class="size-4" aria-hidden="true" />
      </Button>
    </CardHeader>
    <CardContent>
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div class="border-border/60 bg-background/80 rounded-xl border p-4">
          <p class="text-muted-foreground text-xs">
            {t('settingsVaultStatTotalItems')}
          </p>
          <p class="text-foreground text-2xl font-semibold">{totalItems}</p>
        </div>
        <div class="border-border/60 bg-background/80 rounded-xl border p-4">
          <p class="text-muted-foreground text-xs">
            {t('settingsVaultStatWeakPasswords')}
          </p>
          <p class="text-destructive text-2xl font-semibold">{weakPasswordsCount}</p>
        </div>
        <div class="border-border/60 bg-background/80 rounded-xl border p-4">
          <p class="text-muted-foreground text-xs">
            {t('settingsVaultStatDuplicateEntries')}
          </p>
          <p class="text-chart-warning text-2xl font-semibold">{duplicatePasswordsCount}</p>
        </div>
        <div class="border-border/60 bg-background/80 rounded-xl border p-4">
          <p class="text-muted-foreground text-xs">
            {t('settingsVaultStatEncrypted')}
          </p>
          <p class="text-chart-success text-2xl font-semibold">
            {encryptedCount}/{vaults.length}
          </p>
        </div>
      </div>
    </CardContent>
  </Card>
</div>

<PassphraseDialog
  bind:open={passphraseDialogOpen}
  title={passphraseDialogTitle}
  description={passphraseDialogDescription}
  confirmLabel={passphraseDialogConfirmLabel}
  onConfirm={async (passphrase) => {
    if (passphraseDialogAction) {
      await passphraseDialogAction(passphrase);
    }
  }}
  busy={busyAction !== null}
/>
