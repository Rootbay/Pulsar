<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { derived, writable, type Readable } from 'svelte/store';
  import { vaultSettings } from '$lib/stores/vault';
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

  interface Vault {
    id: number;
    name: string;
    items: number;
    size: string;
    lastModified: string;
    status: 'unlocked' | 'encrypted';
    encrypted: boolean;
    path: string;
    settings: VaultSettings;
  }

  const DEFAULT_VAULTS: Vault[] = [
    {
      id: 1,
      name: 'Personal Vault',
      items: 127,
      size: '2.3 MB',
      lastModified: '2 hours ago',
      status: 'unlocked',
      encrypted: true,
      path: '/Users/username/Documents/PersonalVault.db',
      settings: { name: 'Personal Vault', totp: true, backups: false, compression: false }
    },
    {
      id: 2,
      name: 'Work Vault',
      items: 89,
      size: '1.8 MB',
      lastModified: '1 day ago',
      status: 'encrypted',
      encrypted: true,
      path: '/Users/username/Documents/WorkVault.db',
      settings: { name: 'Work Vault', totp: true, backups: true, compression: true }
    },
    {
      id: 3,
      name: 'Family Vault',
      items: 34,
      size: '0.9 MB',
      lastModified: '3 days ago',
      status: 'encrypted',
      encrypted: true,
      path: '/Users/username/Documents/FamilyVault.db',
      settings: { name: 'Family Vault', totp: false, backups: true, compression: false }
    }
  ];

  const vaultsStore = writable<Vault[]>([...DEFAULT_VAULTS]);
  const selectedVaultId = writable<number>(DEFAULT_VAULTS[0].id);

  const selectedVault: Readable<Vault | undefined> = derived(
    [vaultsStore, selectedVaultId],
    ([$vaults, id]) => $vaults.find((vault) => vault.id === id) ?? $vaults[0]
  );

  const totalItems = derived(vaultsStore, ($vaults) =>
    $vaults.reduce((sum, vault) => sum + vault.items, 0)
  );

  const encryptedCount = derived(vaultsStore, ($vaults) =>
    $vaults.filter((vault) => vault.encrypted).length
  );

  const WEAK_PASSWORDS = 23;
  const DUPLICATES = 5;

  const unsubscribeSelectedVault = selectedVault.subscribe((vault) => {
    if (vault) {
      vaultSettings.loadSettings({ ...vault.settings, name: vault.name });
    }
  });

  onMount(() => {
    const initialVault = DEFAULT_VAULTS[0];
    vaultSettings.loadSettings({ ...initialVault.settings, name: initialVault.name });
  });

  onDestroy(() => {
    unsubscribeSelectedVault();
  });

  function selectVault(id: number) {
    selectedVaultId.set(id);
  }

  function updateVaultSetting(setting: keyof VaultSettings) {
    vaultSettings.update((current) => ({
      ...current,
      [setting]: !current[setting]
    }));
  }

  function updateVaultName(event: Event) {
    const value = (event.target as HTMLInputElement).value;
    vaultSettings.update((current) => ({
      ...current,
      name: value
    }));
  }

  function removeVault(id: number) {
    const latest = $vaultsStore.filter((vault) => vault.id !== id);
    vaultsStore.set(latest);
    if (!latest.length) {
      return;
    }
    const fallbackId = latest[Math.min(latest.length - 1, 0)].id;
    selectedVaultId.set(id === $selectedVaultId ? fallbackId : $selectedVaultId);
  }

  function runBackup() {
    // TODO: hook into backup flow
  }

  function restoreVault() {
    // TODO: hook into restore flow
  }

  function exportVault() {
    // TODO: hook into export flow
  }
</script>

<div class="flex-1 space-y-6 overflow-y-auto px-6 py-8">
  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
      <div class="flex items-center gap-3">
        <div class="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
          <FolderKanban class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>Vault management</CardTitle>
          <CardDescription>Select a vault to inspect and adjust its settings.</CardDescription>
        </div>
      </div>
      <div class="flex gap-2">
        <Button type="button" variant="outline" class="gap-2">
          <HardDriveDownload class="size-4" aria-hidden="true" />
          Import vault
        </Button>
        <Button type="button" class="gap-2">
          <Archive class="size-4" aria-hidden="true" />
          Create vault
        </Button>
      </div>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="grid gap-4 lg:grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)]">
        <div class="space-y-3">
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
                <div>
                  <p class="text-sm font-semibold text-foreground">{vault.name}</p>
                  <p class="text-xs text-muted-foreground">{vault.path}</p>
                </div>
                <div class="flex flex-col items-end gap-1">
                  <Badge variant="secondary" class="w-fit capitalize">{vault.status}</Badge>
                  <Badge variant="outline" class="w-fit">{vault.items} items</Badge>
                </div>
              </div>
              <div class="flex flex-wrap items-center gap-4 text-xs text-muted-foreground">
                <span>{vault.size}</span>
                <span>Last modified {vault.lastModified}</span>
                {#if vault.encrypted}
                  <span class="flex items-center gap-1 text-chart-success">
                    <ShieldCheck class="size-3" />
                    Encrypted
                  </span>
                {:else}
                  <span class="flex items-center gap-1 text-destructive">
                    <ShieldAlert class="size-3" />
                    Not encrypted
                  </span>
                {/if}
              </div>
            </button>
          {/each}
        </div>

        <div class="space-y-4 rounded-xl border border-border/60 bg-background/70 p-4">
          <div class="flex items-start justify-between gap-2">
            <div>
              <p class="text-sm font-semibold text-foreground">Selected vault</p>
              <p class="text-xs text-muted-foreground">Manage metadata and automation for this vault.</p>
            </div>
            {#if $selectedVault}
              <Button
                type="button"
                variant="ghost"
                size="icon"
                class="size-8 text-muted-foreground hover:text-destructive"
                aria-label={`Remove vault ${$selectedVault.name}`}
                onclick={() => removeVault($selectedVault.id)}
              >
                <Trash2 class="size-4" aria-hidden="true" />
              </Button>
            {/if}
          </div>

          <div class="space-y-3">
            <Label for="vault-name" class="text-sm font-medium text-foreground">Display name</Label>
            <Input
              id="vault-name"
              type="text"
              value={$vaultSettings.name}
              placeholder="Vault name"
              oninput={updateVaultName}
            />
          </div>

          <div class="space-y-3">
            <div class="flex items-center justify-between gap-3 rounded-lg border border-border/60 bg-muted/10 px-3 py-2">
              <div>
                <p class="text-sm font-medium text-foreground">Store TOTP per entry</p>
                <p class="text-xs text-muted-foreground">Allow storing 2FA secrets inside this vault.</p>
              </div>
              <Switch
                checked={$vaultSettings.totp}
                aria-label="Toggle per-entry TOTP storage"
                onclick={() => updateVaultSetting('totp')}
              />
            </div>

            <div class="flex items-center justify-between gap-3 rounded-lg border border-border/60 bg-muted/10 px-3 py-2">
              <div>
                <p class="text-sm font-medium text-foreground">Automatic backups</p>
                <p class="text-xs text-muted-foreground">Schedule periodic backups for this vault.</p>
              </div>
              <Switch
                checked={$vaultSettings.backups}
                aria-label="Toggle automatic backups"
                onclick={() => updateVaultSetting('backups')}
              />
            </div>

            <div class="flex items-center justify-between gap-3 rounded-lg border border-border/60 bg-muted/10 px-3 py-2">
              <div>
                <p class="text-sm font-medium text-foreground">Compression</p>
                <p class="text-xs text-muted-foreground">Compress vault payloads to save disk space.</p>
              </div>
              <Switch
                checked={$vaultSettings.compression}
                aria-label="Toggle compression"
                onclick={() => updateVaultSetting('compression')}
              />
            </div>
          </div>

          <div class="flex flex-wrap gap-2">
            <Button type="button" variant="outline" class="gap-2" onclick={runBackup}>
              <Archive class="size-4" aria-hidden="true" />
              Backup now
            </Button>
            <Button type="button" variant="outline" class="gap-2" onclick={restoreVault}>
              <Database class="size-4" aria-hidden="true" />
              Restore
            </Button>
            <Button type="button" variant="outline" class="gap-2" onclick={exportVault}>
              <HardDriveDownload class="size-4" aria-hidden="true" />
              Export
            </Button>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex items-start gap-3">
      <div class="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ChartColumn class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Vault insights</CardTitle>
        <CardDescription>Quick statistics across all vaults.</CardDescription>
      </div>
    </CardHeader>
    <CardContent>
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div class="rounded-xl border border-border/60 bg-background/80 p-4">
          <p class="text-xs text-muted-foreground">Total items</p>
          <p class="text-2xl font-semibold text-foreground">{$totalItems}</p>
        </div>
        <div class="rounded-xl border border-border/60 bg-background/80 p-4">
          <p class="text-xs text-muted-foreground">Weak passwords detected</p>
          <p class="text-2xl font-semibold text-destructive">{WEAK_PASSWORDS}</p>
        </div>
        <div class="rounded-xl border border-border/60 bg-background/80 p-4">
          <p class="text-xs text-muted-foreground">Duplicate entries</p>
          <p class="text-2xl font-semibold text-chart-warning">{DUPLICATES}</p>
        </div>
        <div class="rounded-xl border border-border/60 bg-background/80 p-4">
          <p class="text-xs text-muted-foreground">Encrypted vaults</p>
          <p class="text-2xl font-semibold text-chart-success">
            {$encryptedCount}/{$vaultsStore.length}
          </p>
        </div>
      </div>
    </CardContent>
  </Card>
</div>

