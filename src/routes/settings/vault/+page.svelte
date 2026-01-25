<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { derived, get, writable, type Readable } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from '$lib/components/ui/sonner';
  import { vaultSettings } from '$lib/stores/vault';
  import { recentDatabases } from '$lib/stores/recentDatabases';
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
    Database,
    FolderKanban,
    HardDriveDownload,
    ShieldAlert,
    ShieldCheck,
    Trash2
  } from '@lucide/svelte';
  import { exportVaultBackup, importVaultBackup, notifyVaultRefresh } from '$lib/utils/backup';
  import { currentLocale, t, type Locale } from '$lib/i18n';

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

  const vaultsStore = writable<Vault[]>([]);
  const selectedVaultId = writable<string | null>(null);

  const selectedVault: Readable<Vault | undefined> = derived(
    [vaultsStore, selectedVaultId],
    ([$vaults, id]) => $vaults.find((vault) => vault.id === id)
  );

  const totalItems = derived(vaultsStore, ($vaults) =>
    $vaults.reduce((sum, vault) => sum + (vault.itemCount ?? 0), 0)
  );

  const encryptedCount = derived(
    vaultsStore,
    ($vaults) => $vaults.filter((vault) => vault.encrypted).length
  );

  const WEAK_PASSWORDS = 23;
  const DUPLICATES = 5;

  const locale = $derived($currentLocale);

  let loadingVaults = false;
  let busyAction: 'import' | 'create' | 'backup' | 'restore' | 'export' | null = null;

  const relativeTimeFormatter = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });

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
      return t(locale, 'settingsVaultUnknownSize');
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
      return t(locale, 'settingsVaultUnknown');
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

    return t(locale, 'settingsVaultJustNow');
  }

  function formatItemBadge(count: number | undefined, locale: Locale): string {
    if (typeof count === 'number') {
      const label = count === 1 ? t(locale, 'itemSingular') : t(locale, 'itemPlural');
      return `${count} ${label}`;
    }

    return t(locale, 'settingsVaultItemsUnavailable');
  }

  function formatStatusLabel(status: Vault['status'], locale: Locale): string {
    if (status === 'unlocked') return t(locale, 'settingsVaultStatusUnlocked');
    if (status === 'locked') return t(locale, 'settingsVaultStatusLocked');
    if (status === 'available') return t(locale, 'settingsVaultStatusAvailable');
    return status.charAt(0).toUpperCase() + status.slice(1);
  }

  async function refreshVaults({ preserveSelection = true } = {}): Promise<void> {
    loadingVaults = true;

    try {
      const response = await invoke<BackendVault[]>('list_vaults');
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

      vaultsStore.set(mapped);

      if (!mapped.length) {
        selectedVaultId.set(null);
        return;
      }

      const currentSelection = preserveSelection ? get(selectedVaultId) : null;
      const fallback = mapped[0].id;
      const nextSelection =
        currentSelection && mapped.some((vault) => vault.id === currentSelection)
          ? currentSelection
          : fallback;

      selectedVaultId.set(nextSelection);

      const active = mapped.find((vault) => vault.id === nextSelection);
      if (active) {
        vaultSettings.selectVault(active.id, { ...active.settings, name: active.name });
      }
    } catch (cause) {
      console.error('Failed to load vaults:', cause);
      toast.error(t(locale, 'settingsVaultLoadError'));
    } finally {
      loadingVaults = false;
    }
  }

  function selectVault(id: string): void {
    const vault = get(vaultsStore).find((entry) => entry.id === id);
    if (!vault) {
      return;
    }

    selectedVaultId.set(id);
    vaultSettings.selectVault(id, { ...vault.settings, name: vault.name });
  }

  function updateVaultSetting(setting: keyof VaultSettings): void {
    vaultSettings.update((current) => ({
      ...current,
      [setting]: !current[setting]
    }));
  }

  function updateVaultName(event: Event): void {
    const value = (event.target as HTMLInputElement).value;
    vaultSettings.update((current) => ({
      ...current,
      name: value
    }));
  }

  function removeVault(id: string): void {
    const vault = get(vaultsStore).find((entry) => entry.id === id);
    if (!vault) {
      return;
    }

    vaultSettings.clear(id);
    recentDatabases.removeRecentDatabase(vault.path);

    vaultsStore.update((entries) => entries.filter((entry) => entry.id !== id));

    const remaining = get(vaultsStore);
    if (!remaining.length) {
      selectedVaultId.set(null);
      return;
    }

    if (get(selectedVaultId) === id) {
      selectVault(remaining[0].id);
    }

    void refreshVaults();
  }

  async function handleImportVault(): Promise<void> {
    if (busyAction) {
      return;
    }

    busyAction = 'import';

    try {
      const sourcePath = await invoke<string>('pick_open_file');
      const passphrase = window.prompt(t(locale, 'settingsVaultImportPassphrasePrompt'));

      if (!passphrase?.trim()) {
        return;
      }

      await importVaultBackup(passphrase.trim(), { sourcePath });
      toast.success(t(locale, 'settingsVaultImportSuccess'));
      notifyVaultRefresh('import');
      await refreshVaults();
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }

      console.error('Failed to import vault:', cause);
      toast.error(resolveErrorMessage(cause, t(locale, 'settingsVaultImportFailed')));
    } finally {
      busyAction = null;
    }
  }

  async function handleCreateVault(): Promise<void> {
    if (busyAction) {
      return;
    }

    busyAction = 'create';

    try {
      const picked = await invoke<string>('pick_save_file');
      const withExt: string = picked.endsWith('.psec') ? picked : `${picked}.psec`;
      const sep = withExt.includes('\\') ? '\\' : '/';
      const lastSep = withExt.lastIndexOf(sep);
      const baseDir = lastSep === -1 ? '' : withExt.slice(0, lastSep);
      const baseName = lastSep === -1 ? withExt : withExt.slice(lastSep + 1);
      const stem = baseName.endsWith('.psec') ? baseName.slice(0, -5) : baseName;
      const folder = baseDir ? `${baseDir}${sep}${stem}` : stem;
      const finalPath = `${folder}${sep}${stem}.psec`;

      await invoke('switch_database', { dbPath: finalPath });
      await recentDatabases.addRecentDatabase(finalPath);

      toast.success(t(locale, 'settingsVaultCreateSuccess'));
      notifyVaultRefresh('create');
      await refreshVaults({ preserveSelection: false });
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }

      console.error('Failed to create vault:', cause);
      toast.error(resolveErrorMessage(cause, t(locale, 'settingsVaultCreateFailed')));
    } finally {
      busyAction = null;
    }
  }

  async function runBackup(): Promise<void> {
    if (busyAction) {
      return;
    }

    busyAction = 'backup';

    try {
      const passphrase = window.prompt(t(locale, 'settingsVaultBackupPassphrasePrompt'));
      if (!passphrase?.trim()) {
        return;
      }

      const message = await exportVaultBackup(passphrase.trim());
      toast.success(message);
      notifyVaultRefresh('backup');
      await refreshVaults();
    } catch (cause) {
      console.error('Failed to run backup:', cause);
      toast.error(resolveErrorMessage(cause, t(locale, 'settingsVaultBackupFailed')));
    } finally {
      busyAction = null;
    }
  }

  async function restoreVault(): Promise<void> {
    if (busyAction) {
      return;
    }

    busyAction = 'restore';

    try {
      const sourcePath = await invoke<string>('pick_open_file');
      const passphrase = window.prompt(t(locale, 'settingsVaultRestorePassphrasePrompt'));

      if (!passphrase?.trim()) {
        return;
      }

      await importVaultBackup(passphrase.trim(), { sourcePath });
      toast.success(t(locale, 'settingsVaultRestoreSuccess'));
      notifyVaultRefresh('restore');
      await refreshVaults();
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }

      console.error('Failed to restore vault:', cause);
      toast.error(resolveErrorMessage(cause, t(locale, 'settingsVaultRestoreFailed')));
    } finally {
      busyAction = null;
    }
  }

  async function exportVault(): Promise<void> {
    if (busyAction) {
      return;
    }

    busyAction = 'export';

    try {
      const passphrase = window.prompt(t(locale, 'settingsVaultExportPassphrasePrompt'));
      if (!passphrase?.trim()) {
        return;
      }

      const exportPlaintext = window.confirm(t(locale, 'settingsVaultExportConfirmPlaintext'));

      const message = await exportVaultBackup(passphrase.trim(), { plaintext: exportPlaintext });
      toast.success(message);
      notifyVaultRefresh('export');
      await refreshVaults();
    } catch (cause) {
      console.error('Failed to export vault:', cause);
      toast.error(resolveErrorMessage(cause, t(locale, 'settingsVaultExportFailed')));
    } finally {
      busyAction = null;
    }
  }

  let unsubscribeVaultSettings: (() => void) | null = null;

  onMount(() => {
    void refreshVaults({ preserveSelection: false });

    unsubscribeVaultSettings = vaultSettings.subscribe((settings) => {
      const id = get(selectedVaultId);
      if (!id) {
        return;
      }

      vaultsStore.update((vaults) =>
        vaults.map((vault) =>
          vault.id === id
            ? { ...vault, name: settings.name, settings: { ...vault.settings, ...settings } }
            : vault
        )
      );
    });
  });

  onDestroy(() => {
    unsubscribeVaultSettings?.();
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
            {t(locale, 'settingsVaultTitle')}
          </CardTitle>
          <CardDescription>
            {t(locale, 'settingsVaultSubtitle')}
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
          {t(locale, 'settingsVaultImportAction')}
        </Button>
        <Button
          type="button"
          class="gap-2"
          onclick={handleCreateVault}
          disabled={busyAction !== null}
        >
          <Archive class="size-4" aria-hidden="true" />
          {t(locale, 'settingsVaultCreateAction')}
        </Button>
      </div>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="grid gap-4 lg:grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)]">
        <div class="space-y-3">
          {#if loadingVaults}
            <div
              class="border-border/60 bg-muted/20 text-muted-foreground rounded-xl border p-4 text-sm"
            >
              {t(locale, 'settingsVaultLoading')}
            </div>
          {:else if !$vaultsStore.length}
            <div
              class="border-border/60 bg-muted/20 text-muted-foreground rounded-xl border p-4 text-sm"
            >
              {t(locale, 'settingsVaultEmptyState')}
            </div>
          {:else}
            {#each $vaultsStore as vault (vault.id)}
              <button
                type="button"
                class={cn(
                  'border-border/60 bg-background/80 flex w-full flex-col gap-3 rounded-xl border p-4 text-left transition',
                  $selectedVault?.id === vault.id
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
                          {formatStatusLabel(vault.status, locale)}
                        </Badge>
                        <Badge variant="outline">{formatItemBadge(vault.itemCount, locale)}</Badge>
                      </div>
                    </div>
                    <p class="text-muted-foreground text-xs">{vault.path}</p>
                  </div>
                </div>
                <div class="text-muted-foreground flex flex-wrap items-center gap-4 text-xs">
                  <span>{formatBytes(vault.sizeBytes)}</span>
                  <span>
                    {t(locale, 'settingsVaultLastModified')}
                    {formatRelativeTime(vault.modifiedAt)}
                  </span>
                  {#if vault.encrypted}
                    <span class="text-chart-success flex items-center gap-1">
                      <ShieldCheck class="size-3" />
                      {t(locale, 'settingsVaultEncrypted')}
                    </span>
                  {:else}
                    <span class="text-destructive flex items-center gap-1">
                      <ShieldAlert class="size-3" />
                      {t(locale, 'settingsVaultNotEncrypted')}
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
                {t(locale, 'settingsVaultSelectedTitle')}
              </p>
              <p class="text-muted-foreground text-xs">
                {t(locale, 'settingsVaultSelectedSubtitle')}
              </p>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              class="text-muted-foreground hover:text-destructive size-8"
              aria-label={$selectedVault
                ? t(locale, 'settingsVaultRemoveAria', { name: $selectedVault.name })
                : t(locale, 'settingsVaultRemoveAriaFallback')}
              onclick={() => $selectedVault && removeVault($selectedVault.id)}
              disabled={!$selectedVault || busyAction !== null}
            >
              <Trash2 class="size-4" aria-hidden="true" />
            </Button>
          </div>

          {#if $selectedVault}
            <div class="space-y-3">
              <Label for="vault-name" class="text-foreground text-sm font-medium">
                {t(locale, 'settingsVaultDisplayName')}
              </Label>
              <Input
                id="vault-name"
                type="text"
                value={$vaultSettings.name}
                placeholder={t(locale, 'settingsVaultNamePlaceholder')}
                oninput={updateVaultName}
              />
            </div>

            <div class="space-y-3">
              <div
                class="border-border/60 bg-muted/10 flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
              >
                <div>
                  <p class="text-foreground text-sm font-medium">
                    {t(locale, 'settingsVaultTotpTitle')}
                  </p>
                  <p class="text-muted-foreground text-xs">
                    {t(locale, 'settingsVaultTotpDesc')}
                  </p>
                </div>
                <Switch
                  checked={$vaultSettings.totp}
                  aria-label="Toggle per-entry TOTP storage"
                  onclick={() => updateVaultSetting('totp')}
                  disabled={busyAction !== null}
                />
              </div>

              <div
                class="border-border/60 bg-muted/10 flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
              >
                <div>
                  <p class="text-foreground text-sm font-medium">
                    {t(locale, 'settingsVaultAutoBackupTitle')}
                  </p>
                  <p class="text-muted-foreground text-xs">
                    {t(locale, 'settingsVaultAutoBackupDesc')}
                  </p>
                </div>
                <Switch
                  checked={$vaultSettings.backups}
                  aria-label="Toggle automatic backups"
                  onclick={() => updateVaultSetting('backups')}
                  disabled={busyAction !== null}
                />
              </div>

              <div
                class="border-border/60 bg-muted/10 flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
              >
                <div>
                  <p class="text-foreground text-sm font-medium">
                    {t(locale, 'settingsVaultCompressionTitle')}
                  </p>
                  <p class="text-muted-foreground text-xs">
                    {t(locale, 'settingsVaultCompressionDesc')}
                  </p>
                </div>
                <Switch
                  checked={$vaultSettings.compression}
                  aria-label="Toggle compression"
                  onclick={() => updateVaultSetting('compression')}
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
                {t(locale, 'settingsVaultBackupNow')}
              </Button>
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={restoreVault}
                disabled={busyAction !== null}
              >
                <Database class="size-4" aria-hidden="true" />
                {t(locale, 'settingsVaultRestore')}
              </Button>
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={exportVault}
                disabled={busyAction !== null}
              >
                <HardDriveDownload class="size-4" aria-hidden="true" />
                {t(locale, 'settingsVaultExport')}
              </Button>
            </div>
          {:else}
            <p class="text-muted-foreground text-sm">
              {t(locale, 'settingsVaultSelectPrompt')}
            </p>
          {/if}
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex items-start gap-3">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ChartColumn class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'settingsVaultInsightsTitle')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'settingsVaultInsightsDesc')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent>
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div class="border-border/60 bg-background/80 rounded-xl border p-4">
          <p class="text-muted-foreground text-xs">
            {t(locale, 'settingsVaultStatTotalItems')}
          </p>
          <p class="text-foreground text-2xl font-semibold">{$totalItems}</p>
        </div>
        <div class="border-border/60 bg-background/80 rounded-xl border p-4">
          <p class="text-muted-foreground text-xs">
            {t(locale, 'settingsVaultStatWeakPasswords')}
          </p>
          <p class="text-destructive text-2xl font-semibold">{WEAK_PASSWORDS}</p>
        </div>
        <div class="border-border/60 bg-background/80 rounded-xl border p-4">
          <p class="text-muted-foreground text-xs">
            {t(locale, 'settingsVaultStatDuplicateEntries')}
          </p>
          <p class="text-chart-warning text-2xl font-semibold">{DUPLICATES}</p>
        </div>
        <div class="border-border/60 bg-background/80 rounded-xl border p-4">
          <p class="text-muted-foreground text-xs">
            {t(locale, 'settingsVaultStatEncrypted')}
          </p>
          <p class="text-chart-success text-2xl font-semibold">
            {$encryptedCount}/{$vaultsStore.length}
          </p>
        </div>
      </div>
    </CardContent>
  </Card>
</div>




