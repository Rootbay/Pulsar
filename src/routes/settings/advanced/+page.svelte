<script lang="ts">
  import { settings } from '$lib/stores/appSettings.svelte';
  import { callBackend } from '$lib/utils/backend';
  import { toast } from '$lib/components/ui/sonner';
  import type { AdvancedSettings } from '$lib/config/settings';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { Input } from '$lib/components/ui/input';
  import { ShieldAlert, Gauge, TriangleAlert, ShieldCheck } from '@lucide/svelte';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { cn } from '$lib/utils';

  let currentSettings = $derived(settings.state.advanced);
  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as I18nKey, vars);

  type KdfPreset = AdvancedSettings['kdfPreset'];

  const kdfPresets: { value: KdfPreset; label: string }[] = [
    { value: 'fast', label: 'Fast' },
    { value: 'balanced', label: 'Balanced' },
    { value: 'secure', label: 'Secure' },
    { value: 'paranoid', label: 'Paranoid' }
  ];

  const WIPE_CONFIRMATION_TOKEN = 'WIPE';
  let wipeConfirmationText = $state('');
  let canWipeVault = $derived(wipeConfirmationText === WIPE_CONFIRMATION_TOKEN);

  function updateSetting<K extends keyof AdvancedSettings>(key: K, value: AdvancedSettings[K]) {
    settings.state.advanced[key] = value;
    settings.save();
  }

  function handlePresetChange(preset: KdfPreset) {
    let updates: Partial<AdvancedSettings> = { kdfPreset: preset };

    switch (preset) {
      case 'fast':
        updates = { ...updates, timeCost: 2, memoryCost: 32, parallelism: 2 };
        break;
      case 'balanced':
        updates = { ...updates, timeCost: 3, memoryCost: 64, parallelism: 4 };
        break;
      case 'secure':
        updates = { ...updates, timeCost: 4, memoryCost: 128, parallelism: 4 };
        break;
      case 'paranoid':
        updates = { ...updates, timeCost: 8, memoryCost: 1024, parallelism: 8 };
        break;
    }

    settings.state.advanced = { ...settings.state.advanced, ...updates };
    settings.save();
  }

  function handleWipeInput(e: Event & { currentTarget: HTMLInputElement }) {
    wipeConfirmationText = e.currentTarget.value;
  }

  async function handleWipeVault() {
    if (!canWipeVault) return;
    if (confirm(t('Are you absolutely sure you want to wipe the vault? This cannot be undone.'))) {
      try {
        await callBackend('wipe_vault_database');
        toast.success(t('Vault wiped successfully.'));
        wipeConfirmationText = '';
      } catch (error) {
        console.error('Failed to wipe vault:', error);
        toast.error(t('Failed to wipe vault.'));
      }
    }
  }

  const memoryToggles = [
    { key: 'lockMemoryPages', title: 'Lock Memory Pages' },
    { key: 'secureMemoryAllocation', title: 'Secure Memory Allocation' }
  ];
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <Gauge class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('KDF Tuning (Argon2id)')}
        </CardTitle>
        <CardDescription>
          {t('Adjust key-derivation hardness to balance security with unlock speed.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-6 pt-4">
      <div
        class="border-warning/40 bg-warning/10 text-warning-foreground flex items-start gap-3 rounded-lg border p-3 text-sm"
      >
        <TriangleAlert class="mt-0.5 h-4 w-4" aria-hidden="true" />
        <p>
          {t(
            'Increasing these parameters strengthens security but also slows down authentication.'
          )}
        </p>
      </div>

      <div class="space-y-3">
        <Label class="text-foreground text-sm font-medium">
          {t('Presets')}
        </Label>
        <div class="flex flex-wrap gap-2">
          {#each kdfPresets as preset (preset.value)}
            <Button
              type="button"
              size="sm"
              variant="outline"
              class={cn(
                'border-border/60 bg-muted/20 rounded-full px-4 py-1.5 text-sm font-medium transition-colors',
                currentSettings.kdfPreset === preset.value
                  ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                  : 'hover:border-primary/50 hover:text-primary'
              )}
              onclick={() => handlePresetChange(preset.value)}
            >
              {t(preset.label)}
            </Button>
          {/each}
        </div>
      </div>

      <div class="space-y-5">
        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium" for="time-cost">
            {t('Time Cost (iterations)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="time-cost"
              type="range"
              min="1"
              max="20"
              value={currentSettings.timeCost}
              class="bg-secondary accent-primary h-1.5 flex-1 appearance-none rounded-full"
              oninput={(e) => updateSetting('timeCost', parseInt(e.currentTarget.value))}
            />
            <span class="text-muted-foreground w-16 text-right text-sm"
              >{currentSettings.timeCost}</span
            >
          </div>
        </div>

        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium" for="memory-cost">
            {t('Memory Cost (MB)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="memory-cost"
              type="range"
              min="16"
              max="1024"
              step="16"
              value={currentSettings.memoryCost}
              class="bg-secondary accent-primary h-1.5 flex-1 appearance-none rounded-full"
              oninput={(e) => updateSetting('memoryCost', parseInt(e.currentTarget.value))}
            />
            <span class="text-muted-foreground w-20 text-right text-sm"
              >{currentSettings.memoryCost}&nbsp;MB</span
            >
          </div>
        </div>

        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium" for="parallelism">
            {t('Parallelism (threads)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="parallelism"
              type="range"
              min="1"
              max="16"
              value={currentSettings.parallelism}
              class="bg-secondary accent-primary h-1.5 flex-1 appearance-none rounded-full"
              oninput={(e) => updateSetting('parallelism', parseInt(e.currentTarget.value))}
            />
            <span class="text-muted-foreground w-20 text-right text-sm"
              >{currentSettings.parallelism}</span
            >
          </div>
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Memory Hardening')}
        </CardTitle>
        <CardDescription>
          {t('Apply additional safeguards to keep sensitive data in memory protected.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      {#each memoryToggles as toggle (toggle.key)}
        <div
          class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
        >
          <div>
            <p class="text-foreground text-sm font-semibold">
              {t(toggle.title)}
            </p>
            <p class="text-muted-foreground text-sm">
              {toggle.key === 'lockMemoryPages'
                ? t('Prevent sensitive pages from being swapped to disk.')
                : t('Use hardened allocators for secrets kept in RAM.')}
            </p>
          </div>
          <Switch
            checked={currentSettings[toggle.key as 'lockMemoryPages' | 'secureMemoryAllocation']}
            aria-label={`Toggle ${toggle.title.toLowerCase()}`}
            onCheckedChange={(v) => updateSetting(toggle.key as keyof AdvancedSettings, v)}
          />
        </div>
      {/each}
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ShieldAlert class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Destructive Actions')}
        </CardTitle>
        <CardDescription>
          {t('These operations permanently remove data and cannot be undone.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      <div class="border-destructive/40 bg-destructive/10 space-y-4 rounded-lg border p-4">
        <div>
          <p class="text-destructive text-sm font-semibold">
            {t('Wipe Vault Database')}
          </p>
          <p class="text-destructive/80 text-sm">
            {t('Enter the confirmation phrase to enable vault wiping.')}
          </p>
        </div>
        <Input
          placeholder={locale === 'sv'
            ? `Skriv "${WIPE_CONFIRMATION_TOKEN}" för att bekräfta`
            : `Type "${WIPE_CONFIRMATION_TOKEN}" to confirm`}
          value={wipeConfirmationText}
          oninput={handleWipeInput}
          class={cn(
            'w-full',
            wipeConfirmationText.length && !canWipeVault
              ? 'border-destructive/60 focus-visible:ring-destructive/20'
              : ''
          )}
        />
        <Button
          type="button"
          variant="destructive"
          class="w-full"
          disabled={!canWipeVault}
          onclick={handleWipeVault}
        >
          {t('Wipe Vault Database')}
        </Button>
      </div>
    </CardContent>
  </Card>
</div>
