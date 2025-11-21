<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { derived, get, writable, type Readable } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from 'svelte-sonner';
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
  import {
    exportVaultBackup,
    importVaultBackup,
    notifyVaultRefresh
  } from '$lib/utils/backup';
  import { currentLocale } from '$lib/i18n';

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

  const encryptedCount = derived(vaultsStore, ($vaults) =>
    $vaults.filter((vault) => vault.encrypted).length
  );

  const WEAK_PASSWORDS = 23;
  const DUPLICATES = 5;

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  $: locale = $currentLocale as 'en' | 'sv';

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
      return t(locale, 'Unknown size', 'Okänd storlek');
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
      return t(locale, 'Unknown', 'Okänt');
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

    return t(locale, 'just now', 'nyss');
  }

  function formatItemBadge(count?: number, locale: 'en' | 'sv'): string {
    if (typeof count === 'number') {
      const base = locale === 'sv' ? 'post' : 'item';
      const pluralSuffix = count === 1 ? '' : locale === 'sv' ? 'er' : 's';
      return `${count} ${base}${pluralSuffix}`;
    }

    return locale === 'sv' ? 'Poster ej tillgängliga' : 'Items unavailable';
  }

  function formatStatusLabel(status: Vault['status'], locale: 'en' | 'sv'): string {
    if (locale === 'sv') {
      if (status === 'unlocked') return 'Upplåst';
      if (status === 'locked') return 'Låst';
      if (status === 'available') return 'Tillgängligt';
    }
    // Fallback: capitalise first letter
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
      toast.error(t(locale, 'Unable to load vaults.', 'Det gick inte att läsa in valv.'));
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
      const passphrase = window.prompt(
        t(locale, 'Enter the passphrase for the backup', 'Ange lösenfrasen för säkerhetskopian')
      );

      if (!passphrase?.trim()) {
        return;
      }

      await importVaultBackup(passphrase.trim(), { sourcePath });
      toast.success(
        t(locale, 'Vault imported successfully.', 'Valv importerat.')
      );
      notifyVaultRefresh('import');
      await refreshVaults();
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }

      console.error('Failed to import vault:', cause);
      toast.error(resolveErrorMessage(cause, t(locale, 'Failed to import vault.', 'Misslyckades med att importera valv.')));
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

      toast.success(t(locale, 'Vault created successfully.', 'Valv skapat.'));
      notifyVaultRefresh('create');
      await refreshVaults({ preserveSelection: false });
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }

      console.error('Failed to create vault:', cause);
      toast.error(
        resolveErrorMessage(cause, t(locale, 'Failed to create vault.', 'Misslyckades med att skapa valv.'))
      );
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
      const passphrase = window.prompt(
        t(locale, 'Enter a passphrase to secure the backup', 'Ange en lösenfras för att skydda säkerhetskopian')
      );
      if (!passphrase?.trim()) {
        return;
      }

      const message = await exportVaultBackup(passphrase.trim());
      toast.success(message);
      notifyVaultRefresh('backup');
      await refreshVaults();
    } catch (cause) {
      console.error('Failed to run backup:', cause);
      toast.error(
        resolveErrorMessage(cause, t(locale, 'Failed to run backup.', 'Misslyckades med att skapa säkerhetskopia.'))
      );
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
      const passphrase = window.prompt(
        t(locale, 'Enter the backup passphrase to restore', 'Ange säkerhetskopians lösenfras för att återställa')
      );

      if (!passphrase?.trim()) {
        return;
      }

      await importVaultBackup(passphrase.trim(), { sourcePath });
      toast.success(t(locale, 'Vault restored successfully.', 'Valv återställt.'));
      notifyVaultRefresh('restore');
      await refreshVaults();
    } catch (cause) {
      if (isDialogCancelled(cause)) {
        return;
      }

      console.error('Failed to restore vault:', cause);
      toast.error(
        resolveErrorMessage(cause, t(locale, 'Failed to restore vault.', 'Misslyckades med att återställa valv.'))
      );
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
      const passphrase = window.prompt(
        t(locale, 'Enter a passphrase for the export file', 'Ange en lösenfras för exportfilen')
      );
      if (!passphrase?.trim()) {
        return;
      }

      const exportPlaintext = window.confirm(
        t(
          locale,
          'Export without encryption? Select “OK” to export plaintext data or “Cancel” to keep it encrypted.',
          'Exportera utan kryptering? Välj ”OK” för att exportera okrypterad data eller ”Avbryt” för att behålla krypteringen.'
        )
      );

      const message = await exportVaultBackup(passphrase.trim(), { plaintext: exportPlaintext });
      toast.success(message);
      notifyVaultRefresh('export');
      await refreshVaults();
    } catch (cause) {
      console.error('Failed to export vault:', cause);
      toast.error(resolveErrorMessage(cause, 'Failed to export vault.'));
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
          vault.id === id ? { ...vault, name: settings.name, settings: { ...vault.settings, ...settings } } : vault
        )
      );
    });
  });

  onDestroy(() => {
    unsubscribeVaultSettings?.();
  });
</script>

  <div class="flex-1 min-h-0 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
          <FolderKanban class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t(locale, 'Vault management', 'Valvhantering')}
          </CardTitle>
          <CardDescription>
            {t(locale, 'Select a vault to inspect and adjust its settings.', 'Välj ett valv för att granska och justera dess inställningar.')}
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
          {t(locale, 'Import vault', 'Importera valv')}
        </Button>
        <Button
          type="button"
          class="gap-2"
          onclick={handleCreateVault}
          disabled={busyAction !== null}
        >
          <Archive class="size-4" aria-hidden="true" />
          {t(locale, 'Create vault', 'Skapa valv')}
        </Button>
      </div>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="grid gap-4 lg:grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)]">
        <div class="space-y-3">
          {#if loadingVaults}
            <div class="rounded-xl border border-border/60 bg-muted/20 p-4 text-sm text-muted-foreground">
              {t(locale, 'Loading vaults…', 'Läser in valv…')}
            </div>
          {:else if !$vaultsStore.length}
            <div class="rounded-xl border border-border/60 bg-muted/20 p-4 text-sm text-muted-foreground">
              {t(locale, 'No vaults available yet. Create or import one to get started.', 'Inga valv tillgängliga ännu. Skapa eller importera ett för att komma igång.')}
            </div>
          {:else}
            {#each $vaultsStore as vault (vault.id)}
              <button
                type="button"
                class={cn(
                  'flex w-full flex-col gap-3 rounded-xl border border-border/60 bg-background/80 p-4 text-left transition',
                  $selectedVault?.id === vault.id
                    ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                    : 'hover:border-primary/40 hover:bg-muted/40'
                )}
                onclick={() => selectVault(vault.id)}
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="flex-1 space-y-1">
                    <div class="flex flex-wrap items-center gap-2">
                      <p class="text-sm font-semibold text-foreground">{vault.name}</p>
                      <div class="flex flex-wrap items-center gap-1 text-[10px]">
                        <Badge variant="secondary" class="capitalize">
                          {formatStatusLabel(vault.status, locale)}
                        </Badge>
                        <Badge variant="outline">{formatItemBadge(vault.itemCount, locale)}</Badge>
                      </div>
                    </div>
                    <p class="text-xs text-muted-foreground">{vault.path}</p>
                  </div>
                </div>
                <div class="flex flex-wrap items-center gap-4 text-xs text-muted-foreground">
                  <span>{formatBytes(vault.sizeBytes)}</span>
                  <span>
                    {t(locale, 'Last modified', 'Senast ändrad')} {formatRelativeTime(vault.modifiedAt)}
                  </span>
                  {#if vault.encrypted}
                    <span class="flex items-center gap-1 text-chart-success">
                      <ShieldCheck class="size-3" />
                      {t(locale, 'Encrypted', 'Krypterad')}
                    </span>
                  {:else}
                    <span class="flex items-center gap-1 text-destructive">
                      <ShieldAlert class="size-3" />
                      {t(locale, 'Not encrypted', 'Inte krypterad')}
                    </span>
                  {/if}
                </div>
              </button>
            {/each}
          {/if}
        </div>

        <div class="space-y-4 rounded-xl border border-border/60 bg-background/70 p-4">
          <div class="flex items-start justify-between gap-2">
            <div>
              <p class="text-sm font-semibold text-foreground">
                {t(locale, 'Selected vault', 'Markerat valv')}
              </p>
              <p class="text-xs text-muted-foreground">
                {t(locale, 'Manage metadata and automation for this vault.', 'Hantera metadata och automatisering för detta valv.')}
              </p>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              class="size-8 text-muted-foreground hover:text-destructive"
              aria-label={$selectedVault ? `Remove vault ${$selectedVault.name}` : 'Remove vault'}
              onclick={() => $selectedVault && removeVault($selectedVault.id)}
              disabled={!$selectedVault || busyAction !== null}
            >
              <Trash2 class="size-4" aria-hidden="true" />
            </Button>
          </div>

          {#if $selectedVault}
            <div class="space-y-3">
              <Label for="vault-name" class="text-sm font-medium text-foreground">
                {t(locale, 'Display name', 'Visningsnamn')}
              </Label>
              <Input
                id="vault-name"
                type="text"
                value={$vaultSettings.name}
                placeholder={t(locale, 'Vault name', 'Valvnamn')}
                oninput={updateVaultName}
              />
            </div>

            <div class="space-y-3">
              <div class="flex items-center justify-between gap-3 rounded-lg border border-border/60 bg-muted/10 px-3 py-2">
                <div>
                  <p class="text-sm font-medium text-foreground">
                    {t(locale, 'Store TOTP per entry', 'Spara TOTP per post')}
                  </p>
                  <p class="text-xs text-muted-foreground">
                    {t(locale, 'Allow storing 2FA secrets inside this vault.', 'Tillåt lagring av 2FA-hemligheter i detta valv.')}
                  </p>
                </div>
                <Switch
                  checked={$vaultSettings.totp}
                  aria-label="Toggle per-entry TOTP storage"
                  onclick={() => updateVaultSetting('totp')}
                  disabled={busyAction !== null}
                />
              </div>

              <div class="flex items-center justify-between gap-3 rounded-lg border border-border/60 bg-muted/10 px-3 py-2">
                <div>
                  <p class="text-sm font-medium text-foreground">
                    {t(locale, 'Automatic backups', 'Automatiska säkerhetskopior')}
                  </p>
                  <p class="text-xs text-muted-foreground">
                    {t(locale, 'Schedule periodic backups for this vault.', 'Schemalägg regelbundna säkerhetskopior för detta valv.')}
                  </p>
                </div>
                <Switch
                  checked={$vaultSettings.backups}
                  aria-label="Toggle automatic backups"
                  onclick={() => updateVaultSetting('backups')}
                  disabled={busyAction !== null}
                />
              </div>

              <div class="flex items-center justify-between gap-3 rounded-lg border border-border/60 bg-muted/10 px-3 py-2">
                <div>
                  <p class="text-sm font-medium text-foreground">
                    {t(locale, 'Compression', 'Komprimering')}
                  </p>
                  <p class="text-xs text-muted-foreground">
                    {t(locale, 'Compress vault payloads to save disk space.', 'Komprimera valvets data för att spara diskutrymme.')}
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
                {t(locale, 'Backup now', 'Säkerhetskopiera nu')}
              </Button>
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={restoreVault}
                disabled={busyAction !== null}
              >
                <Database class="size-4" aria-hidden="true" />
                {t(locale, 'Restore', 'Återställ')}
              </Button>
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={exportVault}
                disabled={busyAction !== null}
              >
                <HardDriveDownload class="size-4" aria-hidden="true" />
                {t(locale, 'Export', 'Exportera')}
              </Button>
            </div>
          {:else}
            <p class="text-sm text-muted-foreground">
              {t(locale, 'Select a vault from the list to adjust its settings.', 'Välj ett valv i listan för att ändra dess inställningar.')}
            </p>
          {/if}
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex items-start gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ChartColumn class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Vault insights', 'Valvstatistik')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'Quick statistics across all vaults.', 'Snabb statistik över alla valv.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent>
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div class="rounded-xl border border-border/60 bg-background/80 p-4">
          <p class="text-xs text-muted-foreground">
            {t(locale, 'Total items', 'Totalt antal poster')}
          </p>
          <p class="text-2xl font-semibold text-foreground">{$totalItems}</p>
        </div>
        <div class="rounded-xl border border-border/60 bg-background/80 p-4">
          <p class="text-xs text-muted-foreground">
            {t(locale, 'Weak passwords detected', 'Svaga lösenord upptäckta')}
          </p>
          <p class="text-2xl font-semibold text-destructive">{WEAK_PASSWORDS}</p>
        </div>
        <div class="rounded-xl border border-border/60 bg-background/80 p-4">
          <p class="text-xs text-muted-foreground">
            {t(locale, 'Duplicate entries', 'Dubblerade poster')}
          </p>
          <p class="text-2xl font-semibold text-chart-warning">{DUPLICATES}</p>
        </div>
        <div class="rounded-xl border border-border/60 bg-background/80 p-4">
          <p class="text-xs text-muted-foreground">
            {t(locale, 'Encrypted vaults', 'Krypterade valv')}
          </p>
          <p class="text-2xl font-semibold text-chart-success">
            {$encryptedCount}/{$vaultsStore.length}
          </p>
        </div>
      </div>
    </CardContent>
  </Card>
</div>

