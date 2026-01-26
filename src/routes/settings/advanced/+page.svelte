<script lang="ts">
  import { get } from 'svelte/store';
  import { callBackend } from '$lib/utils/backend';
  import { toast } from '$lib/components/ui/sonner';
  import { advancedSettings } from '$lib/stores/advanced';
  import type { AdvancedSettings } from '$lib/config/settings';
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
  import { cn } from '$lib/utils';
  import { Gauge, TriangleAlert, ShieldCheck, ShieldAlert } from '@lucide/svelte';
  import { currentLocale, t } from '$lib/i18n';

  type KdfPreset = AdvancedSettings['kdfPreset'];

  const locale = $derived($currentLocale);

  const kdfPresets: Array<{ value: KdfPreset }> = [
    { value: 'fast' },
    { value: 'balanced' },
    { value: 'secure' },
    { value: 'paranoid' }
  ];

  const presetConfig: Record<KdfPreset, { time: number; memory: number; parallel: number }> = {
    fast: { time: 1, memory: 16, parallel: 1 },
    balanced: { time: 3, memory: 64, parallel: 4 },
    secure: { time: 6, memory: 256, parallel: 8 },
    paranoid: { time: 12, memory: 1024, parallel: 16 }
  };

  const memoryToggles: Array<{
    key: 'lockMemoryPages' | 'secureMemoryAllocation';
    title: string;
    description: string;
  }> = [
    {
      key: 'lockMemoryPages',
      title: 'Lock Memory Pages',
      description: 'Prevent sensitive pages from being swapped to disk.'
    },
    {
      key: 'secureMemoryAllocation',
      title: 'Secure Memory Allocation',
      description: 'Use hardened allocators for secrets kept in RAM.'
    }
  ];

  const WIPE_CONFIRMATION_TOKEN = 'DELETE VAULT';

  let currentSettings = $state<AdvancedSettings>(get(advancedSettings));

  let kdfPreset = $derived(currentSettings.kdfPreset);
  let timeCost = $derived(currentSettings.timeCost);
  let memoryCost = $derived(currentSettings.memoryCost);
  let parallelism = $derived(currentSettings.parallelism);
  let wipeConfirmationText = $state('');
  let lockMemoryPages = $derived(currentSettings.lockMemoryPages);
  let secureMemoryAllocation = $derived(currentSettings.secureMemoryAllocation);

  $effect(() => {
    return advancedSettings.subscribe((value) => {
      currentSettings = value;
    });
  });

  function applyChanges(partial: Partial<AdvancedSettings>) {
    advancedSettings.set({ ...currentSettings, ...partial });
  }

  function selectPreset(preset: KdfPreset) {
    const config = presetConfig[preset];
    applyChanges({
      kdfPreset: preset,
      timeCost: config.time,
      memoryCost: config.memory,
      parallelism: config.parallel
    });
  }

  function handleSliderInput(field: 'timeCost' | 'memoryCost' | 'parallelism', event: Event) {
    const value = Number((event.target as HTMLInputElement).value);
    applyChanges({ [field]: value } as Partial<AdvancedSettings>);
  }

  function toggleSetting(setting: 'lockMemoryPages' | 'secureMemoryAllocation') {
    applyChanges({ [setting]: !currentSettings[setting] } as Partial<AdvancedSettings>);
  }

  function handleWipeInput(event: Event) {
    const value = (event.target as HTMLInputElement).value;
    wipeConfirmationText = value;
  }

  async function handleWipeVault() {
    if (!canWipeVault) return;

    try {
      await callBackend('wipe_vault_database');
      toast.success(t(locale, 'Vault database wiped successfully.'));
      wipeConfirmationText = '';
    } catch (error) {
      toast.error(`${t(locale, 'Failed to wipe vault')}: ${error}`);
    }
  }

  const canWipeVault = $derived(wipeConfirmationText.trim() === WIPE_CONFIRMATION_TOKEN);
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
          {t(locale, 'KDF Tuning (Argon2id)')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'Adjust key-derivation hardness to balance security with unlock speed.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-6 pt-4">
      <div
        class="border-warning/40 bg-warning/10 text-warning-foreground flex items-start gap-3 rounded-lg border p-3 text-sm"
      >
        <TriangleAlert class="mt-0.5 h-4 w-4" aria-hidden="true" />
        <p>
          {t(locale, 'Increasing these parameters strengthens security but also slows down authentication.')}
        </p>
      </div>

      <div class="space-y-3">
        <Label class="text-foreground text-sm font-medium">
          {t(locale, 'Presets')}
        </Label>
        <div class="flex flex-wrap gap-2">
          {#each kdfPresets as preset (preset.value)}
            <Button
              type="button"
              size="sm"
              variant="outline"
              class={cn(
                'border-border/60 bg-muted/20 rounded-full px-4 py-1.5 text-sm font-medium transition-colors',
                kdfPreset === preset.value
                  ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                  : 'hover:border-primary/50 hover:text-primary'
              )}
              onclick={() => selectPreset(preset.value)}
            >
              {preset.value === 'fast'
                ? t(locale, 'Fast')
                : preset.value === 'balanced'
                  ? t(locale, 'Balanced')
                  : preset.value === 'secure'
                    ? t(locale, 'Secure')
                    : t(locale, 'Paranoid')}
            </Button>
          {/each}
        </div>
      </div>

      <div class="space-y-5">
        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium" for="time-cost">
            {t(locale, 'Time Cost (iterations)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="time-cost"
              type="range"
              min="1"
              max="20"
              value={timeCost}
              class="bg-secondary accent-primary h-1.5 flex-1 appearance-none rounded-full"
              oninput={(event) => handleSliderInput('timeCost', event)}
            />
            <span class="text-muted-foreground w-16 text-right text-sm">{timeCost}</span>
          </div>
        </div>

        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium" for="memory-cost">
            {t(locale, 'Memory Cost (MB)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="memory-cost"
              type="range"
              min="16"
              max="1024"
              step="16"
              value={memoryCost}
              class="bg-secondary accent-primary h-1.5 flex-1 appearance-none rounded-full"
              oninput={(event) => handleSliderInput('memoryCost', event)}
            />
            <span class="text-muted-foreground w-20 text-right text-sm">{memoryCost}&nbsp;MB</span>
          </div>
        </div>

        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium" for="parallelism">
            {t(locale, 'Parallelism (threads)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="parallelism"
              type="range"
              min="1"
              max="16"
              value={parallelism}
              class="bg-secondary accent-primary h-1.5 flex-1 appearance-none rounded-full"
              oninput={(event) => handleSliderInput('parallelism', event)}
            />
            <span class="text-muted-foreground w-20 text-right text-sm">{parallelism}</span>
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
          {t(locale, 'Memory Hardening')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'Apply additional safeguards to keep sensitive data in memory protected.')}
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
              {toggle.key === 'lockMemoryPages'
                ? t(locale, 'Lock Memory Pages')
                : t(locale, 'Secure Memory Allocation')}
            </p>
            <p class="text-muted-foreground text-sm">
              {toggle.key === 'lockMemoryPages'
                ? t(locale, 'Prevent sensitive pages from being swapped to disk.')
                : t(locale, 'Use hardened allocators for secrets kept in RAM.')}
            </p>
          </div>
          <Switch
            checked={toggle.key === 'lockMemoryPages' ? lockMemoryPages : secureMemoryAllocation}
            aria-label={`Toggle ${toggle.title.toLowerCase()}`}
            onclick={() => toggleSetting(toggle.key)}
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
          {t(locale, 'Destructive Actions')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'These operations permanently remove data and cannot be undone.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      <div class="border-destructive/40 bg-destructive/10 space-y-4 rounded-lg border p-4">
        <div>
          <p class="text-destructive text-sm font-semibold">
            {t(locale, 'Wipe Vault Database')}
          </p>
          <p class="text-destructive/80 text-sm">
            {t(locale, 'Enter the confirmation phrase to enable vault wiping.')}
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
          {t(locale, 'Wipe Vault Database')}
        </Button>
      </div>
    </CardContent>
  </Card>
</div>




