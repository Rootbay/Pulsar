<script lang="ts">
  import { settings } from '$lib/stores/appSettings.svelte';
  import type { GeneratorSettings, PasswordPreset } from '$lib/config/settings';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Switch } from '$lib/components/ui/switch';
  import { Badge } from '$lib/components/ui/badge';
  import { Progress } from '$lib/components/ui/progress';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { cn } from '$lib/utils';
  import { Copy, Key, RefreshCcw, RotateCcw, Save, Sparkles } from '@lucide/svelte';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { copyText } from '$lib/utils/copyHelper';
  import InputDialog from '$lib/components/ui/InputDialog.svelte';

  import { GeneratorService } from '$lib/utils/generator';

  type GeneratorOptions = GeneratorSettings['options'];
  type GeneratorOptionKey = keyof GeneratorOptions;

  const LENGTH_MIN = 6;
  const LENGTH_MAX = 64;
  const DEFAULT_PASSWORD_LENGTH = 20;
  const MAX_ENTROPY_BITS = 128;
  const ENTROPY_WEAK_THRESHOLD = 60;
  const ENTROPY_GOOD_THRESHOLD = 120;

  const DEFAULT_OPTIONS: GeneratorOptions = {
    uppercase: true,
    lowercase: true,
    digits: true,
    symbols: true,
    ambiguous: false,
    similar: false,
    pronounceable: false,
    mode: 'password',
    wordCount: 4,
    separator: '-'
  };

  type StrengthLevel = 'weak' | 'medium' | 'strong';

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as I18nKey, vars);

  const STRENGTH_META: Record<
    StrengthLevel,
    { label: string; textClass: string; barClass: string }
  > = {
    weak: {
      label: 'Weak',
      textClass: 'text-destructive',
      barClass: '[&_[data-slot=progress-indicator]]:bg-destructive'
    },
    medium: {
      label: 'Good',
      textClass: 'text-chart-warning',
      barClass: '[&_[data-slot=progress-indicator]]:bg-[color:var(--color-chart-5)]'
    },
    strong: {
      label: 'Very strong',
      textClass: 'text-chart-success',
      barClass: '[&_[data-slot=progress-indicator]]:bg-[color:var(--color-chart-4)]'
    }
  };

  type OptionConfig = {
    key: GeneratorOptionKey;
    label: string;
    description: string;
  };

  const CHARACTER_TOGGLES: OptionConfig[] = [
    {
      key: 'uppercase',
      label: 'Include uppercase (A-Z)',
      description: 'Adds capital letters to the character pool.'
    },
    {
      key: 'lowercase',
      label: 'Include lowercase (a-z)',
      description: 'Adds lowercase letters to the character pool.'
    },
    {
      key: 'digits',
      label: 'Include digits (0-9)',
      description: 'Adds numeric characters to the password.'
    },
    {
      key: 'symbols',
      label: 'Include symbols (!@#$)',
      description: 'Adds punctuation and symbol characters.'
    }
  ];

  const ADVANCED_TOGGLES: OptionConfig[] = [
    {
      key: 'ambiguous',
      label: 'Avoid ambiguous characters',
      description: 'Exclude characters like i, l, O, and 0.'
    },
    {
      key: 'similar',
      label: 'Exclude visually similar characters',
      description: 'Avoid characters that look alike in some fonts.'
    },
    {
      key: 'pronounceable',
      label: 'Pronounceable mode',
      description: 'Alternate vowels and consonants for readability.'
    }
  ];

  let presets = $derived(settings.state.passwordPresets);
  let passwordLength = $derived(settings.state.generator.passwordLength);
  let options = $derived(settings.state.generator.options);

  let generatedPassword = $state('');
  let hasCharacterPool = $state(true);
  let copyButtonText = $state('Copy');
  let copyResetTimer: ReturnType<typeof setTimeout> | undefined;

  let selectedPresetName = $state<string | null>(null);
  let showSavePresetDialog = $state(false);

  let strengthEntropy = $state(0);
  let strengthLevel = $state<StrengthLevel>('weak');

  const mode = $derived(settings.state.generator.options.mode || 'password');
  const wordCount = $derived(settings.state.generator.options.wordCount || 4);
  const separator = $derived(settings.state.generator.options.separator || '-');

  async function refreshPassword() {
    if (mode === 'passphrase') {
      await handleGenerate();
      return;
    }
    const poolSize = await GeneratorService.getPoolSize(options);
    hasCharacterPool = poolSize > 0;

    if (hasCharacterPool) {
      await handleGenerate();
    } else {
      generatedPassword = '';
      strengthEntropy = 0;
      strengthLevel = 'weak';
    }
  }

  async function handleGenerate() {
    if (mode === 'passphrase') {
      const pass = await GeneratorService.generatePassphrase(wordCount, separator);
      if (pass) generatedPassword = pass;
    } else {
      const password = GeneratorService.generate(passwordLength, options);
      if (!password) return;
      generatedPassword = password;
    }

    const poolSize = await GeneratorService.getPoolSize(options);
    strengthEntropy = await GeneratorService.calculateEntropy(
      mode === 'passphrase' ? wordCount : passwordLength,
      poolSize,
      mode
    );

    if (strengthEntropy < ENTROPY_WEAK_THRESHOLD) {
      strengthLevel = 'weak';
    } else if (strengthEntropy < ENTROPY_GOOD_THRESHOLD) {
      strengthLevel = 'medium';
    } else {
      strengthLevel = 'strong';
    }
  }

  async function copyPassword() {
    if (!generatedPassword) return;
    try {
      await copyText(generatedPassword, 'Generated Password');
      copyButtonText = 'Copied!';
      if (copyResetTimer) clearTimeout(copyResetTimer);
      copyResetTimer = setTimeout(() => {
        copyButtonText = 'Copy';
      }, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }

  function handleSavePreset(name: string) {
    const charSets: string[] = [];
    if (mode === 'passphrase') {
      charSets.push(`${wordCount} words`);
    } else {
      if (options.uppercase) charSets.push('A-Z');
      if (options.lowercase) charSets.push('a-z');
      if (options.digits) charSets.push('0-9');
      if (options.symbols) charSets.push('!@#$');
    }

    const newPreset: PasswordPreset = {
      name,
      length: mode === 'passphrase' ? wordCount : passwordLength,
      charSet: charSets.join(', '),
      strength: strengthEntropy,
      settings: JSON.parse(JSON.stringify(options))
    };

    settings.state.passwordPresets = [...settings.state.passwordPresets, newPreset];
    settings.save();
  }

  function resetOptions() {
    settings.state.generator.passwordLength = DEFAULT_PASSWORD_LENGTH;
    settings.state.generator.options = { ...DEFAULT_OPTIONS };
    settings.save();
  }

  function updateLength(val: number) {
    settings.state.generator.passwordLength = val;
    settings.save();
  }

  function updateWordCount(val: number) {
    settings.state.generator.options.wordCount = val;
    settings.save();
  }

  function updateSeparator(val: string) {
    settings.state.generator.options.separator = val;
    settings.save();
  }

  function handleModeChange(newMode: string) {
    settings.state.generator.options.mode = newMode as 'password' | 'passphrase';
    settings.save();
  }

  function handlePresetSelect(name: string) {
    const preset = presets.find((p) => p.name === name);
    if (preset) {
      applyPreset(preset);
      selectedPresetName = name;
    }
  }

  function toggleOption(key: GeneratorOptionKey) {
    const val = settings.state.generator.options[key];
    if (typeof val === 'boolean') {
      (settings.state.generator.options as unknown as Record<GeneratorOptionKey, boolean>)[key] =
        !val;
      settings.save();
    }
  }

  function applyPreset(preset: PasswordPreset) {
    if (preset.settings.mode === 'passphrase') {
      // Passphrase presets use wordCount from preset.length, handled by default merge
    } else {
      settings.state.generator.passwordLength = preset.length;
    }
    settings.state.generator.options = { ...DEFAULT_OPTIONS, ...preset.settings };
    settings.save();
  }

  $effect(() => {
    void passwordLength;
    void options;
    refreshPassword();
  });

  $effect(() => {
    return () => {
      if (copyResetTimer) {
        clearTimeout(copyResetTimer);
      }
    };
  });

  const strengthMeta = $derived(STRENGTH_META[strengthLevel]);
  const strengthProgress = $derived(
    Math.min(100, Math.round((strengthEntropy / MAX_ENTROPY_BITS) * 100))
  );
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <Key class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t('Password Generator')}
          </CardTitle>
          <CardDescription>
            {t('Generate strong and secure passwords or passphrases.')}
          </CardDescription>
        </div>
      </div>
      <div class="flex flex-col gap-2 sm:items-end">
        <Select type="single" value={mode} onValueChange={handleModeChange}>
          <SelectTrigger class="w-40">
            <span data-slot="select-value" class="truncate text-sm font-medium">
              {mode === 'password' ? t('Password') : t('Passphrase')}
            </span>
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="password">{t('Password')}</SelectItem>
            <SelectItem value="passphrase">{t('Passphrase')}</SelectItem>
          </SelectContent>
        </Select>
        <Badge variant="secondary" class="w-fit">
          {mode === 'password' ? t('Length') : t('Words')}&nbsp;{mode === 'password'
            ? passwordLength
            : wordCount}
        </Badge>
      </div>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="border-border/60 bg-muted/10 space-y-3 rounded-xl border p-4">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <p class="text-muted-foreground text-sm font-medium">
            {mode === 'password' ? t('Generated password') : t('Generated passphrase')}
          </p>
          <Button
            type="button"
            variant="outline"
            size="sm"
            class="gap-2"
            onclick={handleGenerate}
            disabled={mode === 'password' && !hasCharacterPool}
          >
            <RefreshCcw class="size-4" aria-hidden="true" />
            {t('Generate new')}
          </Button>
        </div>
        <div
          class="border-border/40 bg-background/80 flex flex-col gap-2 rounded-lg border p-4 font-mono text-sm"
        >
          {#if generatedPassword}
            <span class="break-all">{generatedPassword}</span>
          {:else}
            <span class="text-muted-foreground text-sm">
              {t('Select at least one character set to generate a password.')}
            </span>
          {/if}
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="secondary"
            class="gap-2"
            onclick={copyPassword}
            disabled={!generatedPassword}
          >
            <Copy class="size-4" aria-hidden="true" />
            {copyButtonText}
          </Button>
          <Button
            type="button"
            variant="outline"
            class="gap-2"
            onclick={() => (showSavePresetDialog = true)}
          >
            <Save class="size-4" aria-hidden="true" />
            {t('Save as preset')}
          </Button>
          <Button type="button" variant="outline" class="gap-2" onclick={resetOptions}>
            <RotateCcw class="size-4" aria-hidden="true" />
            {t('Reset to defaults')}
          </Button>
        </div>
      </div>

      <div class="border-border/60 bg-muted/10 space-y-3 rounded-xl border p-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="text-foreground text-sm font-semibold">
              {t('Password strength')}
            </p>
            <p class="text-muted-foreground text-xs">
              {t('Entropy')} ≈ {strengthEntropy}
              {t('bits')}
            </p>
          </div>
          <span class={cn('text-sm font-semibold', strengthMeta.textClass)}>
            {strengthLevel === 'weak'
              ? t('Weak')
              : strengthLevel === 'medium'
                ? t('Good')
                : t('Very strong')}
          </span>
        </div>
        <Progress value={strengthProgress} class={cn('bg-muted/40', strengthMeta.barClass)} />
      </div>

      <div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <div class="space-y-4">
          {#if mode === 'password'}
            <div class="space-y-2">
              <p class="text-foreground text-sm font-medium">
                {t('Password length')}
              </p>
              <input
                type="range"
                min={LENGTH_MIN}
                max={LENGTH_MAX}
                value={passwordLength}
                class="bg-muted accent-primary h-2 w-full cursor-pointer appearance-none rounded-full"
                oninput={(event) => updateLength(Number((event.target as HTMLInputElement).value))}
              />
              <div class="text-muted-foreground flex justify-between text-xs">
                <span>{LENGTH_MIN}</span>
                <span>{LENGTH_MAX}</span>
              </div>
            </div>
          {:else}
            <div class="space-y-4">
              <div class="space-y-2">
                <p class="text-foreground text-sm font-medium">
                  {t('Word count')}
                </p>
                <input
                  type="range"
                  min="3"
                  max="12"
                  value={wordCount}
                  class="bg-muted accent-primary h-2 w-full cursor-pointer appearance-none rounded-full"
                  oninput={(event) =>
                    updateWordCount(Number((event.target as HTMLInputElement).value))}
                />
                <div class="text-muted-foreground flex justify-between text-xs">
                  <span>3</span>
                  <span>12</span>
                </div>
              </div>
              <div class="space-y-2">
                <p class="text-foreground text-sm font-medium">
                  {t('Separator')}
                </p>
                <Select type="single" value={separator} onValueChange={updateSeparator}>
                  <SelectTrigger class="w-full">
                    <span data-slot="select-value" class="truncate text-sm">
                      {separator === '-'
                        ? t('Hyphen (-)')
                        : separator === '.'
                          ? t('Dot (.)')
                          : separator === '_'
                            ? t('Underscore (_)')
                            : t('Space ( )')}
                    </span>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="-">{t('Hyphen (-)')}</SelectItem>
                    <SelectItem value=".">{t('Dot (.)')}</SelectItem>
                    <SelectItem value="_">{t('Underscore (_)')}</SelectItem>
                    <SelectItem value=" ">{t('Space ( )')}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          {/if}

          <div class="space-y-2">
            <p class="text-foreground text-sm font-medium">
              {t('Apply saved preset')}
            </p>
            {#if presets.length}
              <Select
                type="single"
                value={selectedPresetName ?? ''}
                onValueChange={handlePresetSelect}
              >
                <SelectTrigger aria-label="Select password preset" class="w-full">
                  <span data-slot="select-value" class="flex items-center gap-2 truncate text-sm">
                    {selectedPresetName ?? t('Choose a preset')}
                  </span>
                </SelectTrigger>
                <SelectContent>
                  {#each presets as preset (preset.name)}
                    <SelectItem value={preset.name}>{preset.name}</SelectItem>
                  {/each}
                </SelectContent>
              </Select>
            {:else}
              <p class="text-muted-foreground text-sm">
                {t('No presets available yet.')}
              </p>
            {/if}
          </div>
        </div>

        {#if mode === 'password'}
          <div class="space-y-4">
            <p class="text-foreground text-sm font-semibold">
              {t('Character options')}
            </p>
            <div class="space-y-3">
              {#each CHARACTER_TOGGLES as option (option.key)}
                <div
                  class="border-border/60 bg-background/60 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
                >
                  <div>
                    <p class="text-foreground text-sm font-medium">
                      {option.key === 'uppercase'
                        ? t('Include uppercase (A-Z)')
                        : option.key === 'lowercase'
                          ? t('Include lowercase (a-z)')
                          : option.key === 'digits'
                            ? t('Include digits (0-9)')
                            : t('Include symbols (!@#$)')}
                    </p>
                    <p class="text-muted-foreground text-xs">
                      {option.key === 'uppercase'
                        ? t('Adds capital letters to the character pool.')
                        : option.key === 'lowercase'
                          ? t('Adds lowercase letters to the character pool.')
                          : option.key === 'digits'
                            ? t('Adds numeric characters to the password.')
                            : t('Adds punctuation and symbol characters.')}
                    </p>
                  </div>
                  <Switch
                    checked={!!options[option.key]}
                    aria-label={option.label}
                    onCheckedChange={() => toggleOption(option.key)}
                  />
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <div class="flex h-full flex-col items-center justify-center p-6 text-center">
            <Sparkles class="text-primary/20 mb-4 h-16 w-16" />
            <p class="text-muted-foreground max-w-60 text-sm">
              {t('Passphrases use random words to create secure but memorable secrets.')}
            </p>
          </div>
        {/if}
      </div>

      {#if mode === 'password'}
        <div class="space-y-4">
          <div class="flex items-center gap-2">
            <Sparkles class="text-muted-foreground size-4" aria-hidden="true" />
            <p class="text-foreground text-sm font-semibold">
              {t('Advanced options')}
            </p>
          </div>
          <div class="grid gap-3 md:grid-cols-2">
            {#each ADVANCED_TOGGLES as option (option.key)}
              <div
                class="border-border/60 bg-background/60 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
              >
                <div>
                  <p class="text-foreground text-sm font-medium">
                    {option.key === 'ambiguous'
                      ? t('Avoid ambiguous characters')
                      : option.key === 'similar'
                        ? t('Exclude visually similar characters')
                        : t('Pronounceable mode')}
                  </p>
                  <p class="text-muted-foreground text-xs">
                    {option.key === 'ambiguous'
                      ? t('Exclude characters like i, l, O, and 0.')
                      : option.key === 'similar'
                        ? t('Avoid characters that look alike in some fonts.')
                        : t('Alternate vowels and consonants for readability.')}
                  </p>
                </div>
                <Switch
                  checked={!!options[option.key]}
                  aria-label={option.label}
                  onCheckedChange={() => toggleOption(option.key)}
                />
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </CardContent>
  </Card>
</div>

<InputDialog
  bind:open={showSavePresetDialog}
  title={t('Save as preset')}
  description={t('Enter a name for this password generator preset:')}
  label={t('Preset Name')}
  placeholder="Work / Personal / Bank"
  onConfirm={handleSavePreset}
/>
