
<script lang="ts">
  import { get } from 'svelte/store';
  import { onDestroy } from 'svelte';
  import { advancedSettings } from '$lib/stores/advanced';
  import type { AdvancedSettings } from '$lib/config/settings';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { cn } from '$lib/utils';
  import { Gauge, TriangleAlert, ShieldCheck, ShieldAlert } from '@lucide/svelte';
  import { currentLocale } from '$lib/i18n';

  type KdfPreset = AdvancedSettings['kdfPreset'];

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  $: locale = $currentLocale as 'en' | 'sv';

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

  let currentSettings: AdvancedSettings = get(advancedSettings);
  let { kdfPreset, timeCost, memoryCost, parallelism, wipeConfirmationText, lockMemoryPages, secureMemoryAllocation } =
    currentSettings;

  const unsubscribe = advancedSettings.subscribe((value) => {
    currentSettings = value;
    ({
      kdfPreset,
      timeCost,
      memoryCost,
      parallelism,
      wipeConfirmationText,
      lockMemoryPages,
      secureMemoryAllocation
    } = value);
  });

  onDestroy(() => {
    unsubscribe();
  });

  function applyChanges(partial: Partial<AdvancedSettings>) {
    currentSettings = { ...currentSettings, ...partial };
    advancedSettings.set(currentSettings);
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
    applyChanges({ wipeConfirmationText: value });
  }

  $: canWipeVault = wipeConfirmationText.trim() === WIPE_CONFIRMATION_TOKEN;
</script>

<div class="flex-1 min-h-0 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Gauge class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'KDF Tuning (Argon2id)', 'KDF-inställning (Argon2id)')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Adjust key-derivation hardness to balance security with unlock speed.',
            'Justera nyckelderiveringens styrka för att balansera säkerhet mot upplåsningshastighet.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-6 pt-4">
      <div class="flex items-start gap-3 rounded-lg border border-warning/40 bg-warning/10 p-3 text-sm text-warning-foreground">
        <TriangleAlert class="mt-0.5 h-4 w-4" aria-hidden="true" />
        <p>
          {t(
            locale,
            'Increasing these parameters strengthens security but also slows down authentication.',
            'Att öka dessa parametrar stärker säkerheten men gör autentisering långsammare.'
          )}
        </p>
      </div>

      <div class="space-y-3">
        <Label class="text-sm font-medium text-foreground">
          {t(locale, 'Presets', 'Förval')}
        </Label>
        <div class="flex flex-wrap gap-2">
          {#each kdfPresets as preset}
            <Button
              type="button"
              size="sm"
              variant="outline"
              class={cn(
                'rounded-full border-border/60 bg-muted/20 px-4 py-1.5 text-sm font-medium transition-colors',
                kdfPreset === preset.value
                  ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                  : 'hover:border-primary/50 hover:text-primary'
              )}
              onclick={() => selectPreset(preset.value)}
            >
              {preset.value === 'fast'
                ? t(locale, 'Fast', 'Snabb')
                : preset.value === 'balanced'
                  ? t(locale, 'Balanced', 'Balanserad')
                  : preset.value === 'secure'
                    ? t(locale, 'Secure', 'Säker')
                    : t(locale, 'Paranoid', 'Paranoid')}
            </Button>
          {/each}
        </div>
      </div>

      <div class="space-y-5">
        <div class="space-y-2">
          <Label class="text-sm font-medium text-foreground" for="time-cost">
            {t(locale, 'Time Cost (iterations)', 'Tidskostnad (iterationer)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="time-cost"
              type="range"
              min="1"
              max="20"
              value={timeCost}
              class="h-1.5 flex-1 appearance-none rounded-full bg-secondary accent-primary"
              oninput={(event) => handleSliderInput('timeCost', event)}
            />
            <span class="w-16 text-right text-sm text-muted-foreground">{timeCost}</span>
          </div>
        </div>

        <div class="space-y-2">
          <Label class="text-sm font-medium text-foreground" for="memory-cost">
            {t(locale, 'Memory Cost (MB)', 'Minneskostnad (MB)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="memory-cost"
              type="range"
              min="16"
              max="1024"
              step="16"
              value={memoryCost}
              class="h-1.5 flex-1 appearance-none rounded-full bg-secondary accent-primary"
              oninput={(event) => handleSliderInput('memoryCost', event)}
            />
            <span class="w-20 text-right text-sm text-muted-foreground">{memoryCost}&nbsp;MB</span>
          </div>
        </div>

        <div class="space-y-2">
          <Label class="text-sm font-medium text-foreground" for="parallelism">
            {t(locale, 'Parallelism (threads)', 'Parallelism (trådar)')}
          </Label>
          <div class="flex items-center gap-4">
            <input
              id="parallelism"
              type="range"
              min="1"
              max="16"
              value={parallelism}
              class="h-1.5 flex-1 appearance-none rounded-full bg-secondary accent-primary"
              oninput={(event) => handleSliderInput('parallelism', event)}
            />
            <span class="w-20 text-right text-sm text-muted-foreground">{parallelism}</span>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Memory Hardening', 'Hårdare minnesskydd')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Apply additional safeguards to keep sensitive data in memory protected.',
            'Lägg till extra skydd för att hålla känslig data i minnet skyddad.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      {#each memoryToggles as toggle}
        <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
          <div>
            <p class="text-sm font-semibold text-foreground">
              {toggle.key === 'lockMemoryPages'
                ? t(locale, 'Lock Memory Pages', 'Lås minnessidor')
                : t(locale, 'Secure Memory Allocation', 'Säker minnesallokering')}
            </p>
            <p class="text-sm text-muted-foreground">
              {toggle.key === 'lockMemoryPages'
                ? t(locale, 'Prevent sensitive pages from being swapped to disk.', 'Förhindra att känsliga sidor växlas ut till disk.')
                : t(locale, 'Use hardened allocators for secrets kept in RAM.', 'Använd härdade allokerare för hemligheter i RAM.')}
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

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ShieldAlert class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Destructive Actions', 'Förstörande åtgärder')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'These operations permanently remove data and cannot be undone.', 'Dessa åtgärder tar bort data permanent och kan inte ångras.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      <div class="space-y-4 rounded-lg border border-destructive/40 bg-destructive/10 p-4">
        <div>
          <p class="text-sm font-semibold text-destructive">
            {t(locale, 'Wipe Vault Database', 'Radera valvdatabas')}
          </p>
          <p class="text-sm text-destructive/80">
            {t(locale, 'Enter the confirmation phrase to enable vault wiping.', 'Skriv in bekräftelsefrasen för att aktivera radering av valvet.')}
          </p>
        </div>
        <Input
          placeholder={
            locale === 'sv'
              ? `Skriv "${WIPE_CONFIRMATION_TOKEN}" för att bekräfta`
              : `Type "${WIPE_CONFIRMATION_TOKEN}" to confirm`
          }
          value={wipeConfirmationText}
          oninput={handleWipeInput}
          class={cn(
            'w-full',
            wipeConfirmationText.length && !canWipeVault
              ? 'border-destructive/60 focus-visible:ring-destructive/20'
              : ''
          )}
        />
        <Button type="button" variant="destructive" class="w-full" disabled={!canWipeVault} onclick={() => {}}>
          {t(locale, 'Wipe Vault Database', 'Radera valvdatabas')}
        </Button>
      </div>
    </CardContent>
  </Card>
</div>
