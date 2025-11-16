<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { generatorSettings } from '$lib/stores/generator';
  import { passwordPresets } from '$lib/stores/passwordPresets';
  import { siteRules } from '$lib/stores/siteRules';
  import type { GeneratorSettings, PasswordPreset, SiteRule } from '$lib/config/settings';
  import EditModal from '$lib/components/ui/EditModal.svelte';
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
  import {
    Copy,
    FileText,
    Key,
    ListChecks,
    Pencil,
    RefreshCcw,
    RotateCcw,
    Save,
    Sparkles,
    Trash2
  } from '@lucide/svelte';

  type GeneratorOptions = GeneratorSettings['options'];
  type GeneratorOptionKey = keyof GeneratorOptions;

  const SYMBOL_CHARSET = '!@#$%^&*()_+-=[]{}|;:,.<>?';
  const AMBIGUOUS_CHARS = new Set(['i', 'I', '1', 'L', 'o', 'O', '0']);
  const SIMILAR_CHARS = new Set(
    'oO0l1IvVwWsScCpPkKxXzZbBdDgGqQeEfFtTuUjJmMnrRhHaAyY'.split('')
  );

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
    pronounceable: false
  };

  type StrengthLevel = 'weak' | 'medium' | 'strong';

  const STRENGTH_META: Record<StrengthLevel, { label: string; textClass: string; barClass: string }> = {
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

  let presets: PasswordPreset[] = [];
  let rules: SiteRule[] = [];

  let passwordLength = DEFAULT_PASSWORD_LENGTH;
  let options: GeneratorOptions = { ...DEFAULT_OPTIONS };
  let generatedPassword = '';
  let hasCharacterPool = true;
  let copyButtonText = 'Copy';
  let copyResetTimer: ReturnType<typeof setTimeout> | undefined;

  let showEditModal = false;
  let itemToEdit: PasswordPreset | SiteRule | null = null;
  let editModalType: 'preset' | 'rule' | null = null;
  let selectedPresetName: string | null = null;

  let strengthEntropy = 0;
  let strengthLevel: StrengthLevel = 'weak';

  const unsubscribeGenerator = generatorSettings.subscribe((settings) => {
    passwordLength = settings.passwordLength;
    options = { ...settings.options };
    refreshPassword();
  });

  const unsubscribePresets = passwordPresets.subscribe((value) => {
    presets = value;
    if (selectedPresetName && !value.some((preset) => preset.name === selectedPresetName)) {
      selectedPresetName = null;
    }
  });

  const unsubscribeRules = siteRules.subscribe((value) => {
    rules = value;
  });

  onMount(() => {
    generatorSettings.update((current) => ({
      ...current,
      options: { ...DEFAULT_OPTIONS, ...current.options }
    }));
  });

  onDestroy(() => {
    unsubscribeGenerator();
    unsubscribePresets();
    unsubscribeRules();
    if (copyResetTimer) {
      clearTimeout(copyResetTimer);
    }
  });

  function buildBaseCharset(opts: GeneratorOptions): string {
    let charset = '';
    if (opts.uppercase) charset += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    if (opts.lowercase) charset += 'abcdefghijklmnopqrstuvwxyz';
    if (opts.digits) charset += '0123456789';
    if (opts.symbols) charset += SYMBOL_CHARSET;
    return charset;
  }

  function getEffectiveCharsetArray(baseCharset: string, opts: GeneratorOptions): string[] {
    let effective = baseCharset.split('');

    if (opts.ambiguous) {
      effective = effective.filter((char) => !AMBIGUOUS_CHARS.has(char));
    }

    if (opts.similar) {
      effective = effective.filter((char) => !SIMILAR_CHARS.has(char));
    }

    return effective;
  }

  function generatePronounceablePassword(length: number, charset: string[]): string {
    const vowels = new Set('aeiouAEIOU');
    const consonants = new Set('bcdfghjklmnpqrstvwxyzBCDFGHJKLMNPQRSTVWXYZ');

    const availableVowels = charset.filter((char) => vowels.has(char));
    const availableConsonants = charset.filter((char) => consonants.has(char));

    const password: string[] = [];
    let useVowel = Math.random() < 0.5;

    for (let index = 0; index < length; index += 1) {
      let char: string | undefined;
      if (useVowel && availableVowels.length > 0) {
        char = availableVowels[Math.floor(Math.random() * availableVowels.length)];
      } else if (!useVowel && availableConsonants.length > 0) {
        char = availableConsonants[Math.floor(Math.random() * availableConsonants.length)];
      }

      if (!char) {
        char = charset[Math.floor(Math.random() * charset.length)];
      }

      password.push(char);
      useVowel = !useVowel;
    }

    return password.join('');
  }

  function generateRandomPassword(length: number, charset: string[]): string {
    const password: string[] = [];
    for (let index = 0; index < length; index += 1) {
      password.push(charset[Math.floor(Math.random() * charset.length)]);
    }
    return password.join('');
  }

  function refreshPassword() {
    const baseCharset = buildBaseCharset(options);
    const effectiveCharset = getEffectiveCharsetArray(baseCharset, options);

    hasCharacterPool = effectiveCharset.length > 0;

    if (!hasCharacterPool) {
      generatedPassword = '';
      strengthEntropy = 0;
      strengthLevel = 'weak';
      return;
    }

    generatedPassword = options.pronounceable
      ? generatePronounceablePassword(passwordLength, effectiveCharset)
      : generateRandomPassword(passwordLength, effectiveCharset);

    strengthEntropy = calculateEntropy(passwordLength, options);
    strengthLevel = classifyStrength(strengthEntropy);
  }

  function calculateEntropy(length: number, opts: GeneratorOptions): number {
    let pool = 0;
    if (opts.uppercase) pool += 26;
    if (opts.lowercase) pool += 26;
    if (opts.digits) pool += 10;
    if (opts.symbols) pool += SYMBOL_CHARSET.length;

    if (pool === 0) {
      return 0;
    }

    return Math.round(length * Math.log2(pool));
  }

  function classifyStrength(entropy: number): StrengthLevel {
    if (entropy < ENTROPY_WEAK_THRESHOLD) {
      return 'weak';
    }
    if (entropy < ENTROPY_GOOD_THRESHOLD) {
      return 'medium';
    }
    return 'strong';
  }

  function updateGeneratorSettings(partial: Partial<GeneratorSettings>) {
    generatorSettings.update((current) => ({
      ...current,
      ...partial,
      options: partial.options ? { ...partial.options } : current.options
    }));
  }

  function toggleOption(key: GeneratorOptionKey) {
    const nextOptions = { ...options, [key]: !options[key] };
    options = nextOptions;
    updateGeneratorSettings({ options: nextOptions });
  }

  function updateLength(value: number) {
    passwordLength = value;
    updateGeneratorSettings({ passwordLength: value });
  }

  function handleGenerate() {
    refreshPassword();
  }

  async function copyPassword() {
    if (!generatedPassword) {
      return;
    }

    try {
      await navigator.clipboard.writeText(generatedPassword);
      copyButtonText = 'Copied!';
      if (copyResetTimer) {
        clearTimeout(copyResetTimer);
      }
      copyResetTimer = setTimeout(() => {
        copyButtonText = 'Copy';
      }, 2000);
    } catch (error) {
      console.error('Failed to copy password:', error);
    }
  }

  function applyPreset(preset: PasswordPreset) {
    selectedPresetName = preset.name;
    passwordLength = preset.length;
    const nextOptions: GeneratorOptions = {
      uppercase: preset.settings.uppercase,
      lowercase: preset.settings.lowercase,
      digits: preset.settings.digits,
      symbols: preset.settings.symbols,
      ambiguous: preset.settings.ambiguous,
      similar: preset.settings.similar,
      pronounceable: preset.settings.pronounceable
    };
    options = nextOptions;
    updateGeneratorSettings({ passwordLength: preset.length, options: nextOptions });
  }

  function handlePresetSelect(name: string) {
    const preset = presets.find((item) => item.name === name);
    if (!preset) {
      return;
    }
    applyPreset(preset);
  }

  function saveCurrentSettingsAsPreset() {
    const presetName = prompt('Enter a name for this preset:');
    if (!presetName) {
      return;
    }

    const newPreset: PasswordPreset = {
      name: presetName,
      length: passwordLength,
      charSet: 'Custom',
      strength: strengthEntropy,
      settings: { ...options }
    };

    passwordPresets.addPreset(newPreset);
    selectedPresetName = presetName;
  }

  function resetOptions() {
    selectedPresetName = null;
    passwordLength = DEFAULT_PASSWORD_LENGTH;
    options = { ...DEFAULT_OPTIONS };
    updateGeneratorSettings({
      passwordLength: DEFAULT_PASSWORD_LENGTH,
      options: { ...DEFAULT_OPTIONS }
    });
  }

  function removePreset(name: string) {
    if (confirm(`Are you sure you want to delete preset "${name}"?`)) {
      passwordPresets.deletePreset(name);
    }
  }

  function removeRule(url: string) {
    if (confirm(`Are you sure you want to delete rule for "${url}"?`)) {
      siteRules.deleteRule(url);
    }
  }

  function handleEditPreset(preset: PasswordPreset) {
    itemToEdit = preset;
    editModalType = 'preset';
    showEditModal = true;
  }

  function handleEditRule(rule: SiteRule) {
    itemToEdit = rule;
    editModalType = 'rule';
    showEditModal = true;
  }

  function handleSaveEdit(event: CustomEvent<PasswordPreset | SiteRule>) {
    const updatedItem = event.detail;

    if (editModalType === 'preset') {
      const preset = updatedItem as PasswordPreset;
      passwordPresets.updatePreset(preset.name, preset);
      selectedPresetName = preset.name;
    } else if (editModalType === 'rule') {
      const rule = updatedItem as SiteRule;
      siteRules.updateRule(rule.url, rule);
    }

    closeModal();
  }

  function closeModal() {
    showEditModal = false;
    itemToEdit = null;
    editModalType = null;
  }

  $: strengthMeta = STRENGTH_META[strengthLevel];
  $: strengthProgress = Math.min(100, Math.round((strengthEntropy / MAX_ENTROPY_BITS) * 100));
</script>

  <div class="flex-1 min-h-0 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
      <div class="flex items-center gap-3">
        <div class="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
          <Key class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>Password Generator</CardTitle>
          <CardDescription>Generate strong and secure passwords on demand.</CardDescription>
        </div>
      </div>
      <Badge variant="secondary" class="mt-2 w-fit sm:mt-0">
        Length&nbsp;{passwordLength}
      </Badge>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="space-y-3 rounded-xl border border-border/60 bg-muted/10 p-4">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <p class="text-sm font-medium text-muted-foreground">Generated password</p>
          <Button
            type="button"
            variant="outline"
            size="sm"
            class="gap-2"
            onclick={handleGenerate}
            disabled={!hasCharacterPool}
          >
            <RefreshCcw class="size-4" aria-hidden="true" />
            Generate new
          </Button>
        </div>
        <div class="flex flex-col gap-2 rounded-lg border border-border/40 bg-background/80 p-4 font-mono text-base">
          {#if generatedPassword}
            <span class="break-all">{generatedPassword}</span>
          {:else}
            <span class="text-sm text-muted-foreground">Select at least one character set to generate a password.</span>
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
            onclick={saveCurrentSettingsAsPreset}
          >
            <Save class="size-4" aria-hidden="true" />
            Save as preset
          </Button>
          <Button
            type="button"
            variant="outline"
            class="gap-2"
            onclick={resetOptions}
          >
            <RotateCcw class="size-4" aria-hidden="true" />
            Reset to defaults
          </Button>
        </div>
      </div>

      <div class="space-y-3 rounded-xl border border-border/60 bg-muted/10 p-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="text-sm font-semibold text-foreground">Password strength</p>
            <p class="text-xs text-muted-foreground">Entropy ≈ {strengthEntropy} bits</p>
          </div>
          <span class={cn('text-sm font-semibold', strengthMeta.textClass)}>{strengthMeta.label}</span>
        </div>
        <Progress value={strengthProgress} class={cn('bg-muted/40', strengthMeta.barClass)} />
      </div>

      <div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <div class="space-y-4">
          <div class="space-y-2">
            <p class="text-sm font-medium text-foreground">Password length</p>
            <input
              type="range"
              min={LENGTH_MIN}
              max={LENGTH_MAX}
              value={passwordLength}
              class="h-2 w-full cursor-pointer appearance-none rounded-full bg-muted accent-primary"
              oninput={(event) => updateLength(Number((event.target as HTMLInputElement).value))}
            />
            <div class="flex justify-between text-xs text-muted-foreground">
              <span>{LENGTH_MIN}</span>
              <span>{LENGTH_MAX}</span>
            </div>
          </div>

          <div class="space-y-2">
            <p class="text-sm font-medium text-foreground">Apply saved preset</p>
            {#if presets.length}
              <Select type="single" value={selectedPresetName ?? ''} onValueChange={handlePresetSelect}>
                <SelectTrigger aria-label="Select password preset" class="w-full">
                  <span data-slot="select-value" class="flex items-center gap-2 truncate text-sm">
                    {selectedPresetName ?? 'Choose a preset'}
                  </span>
                </SelectTrigger>
                <SelectContent>
                  {#each presets as preset}
                    <SelectItem value={preset.name}>{preset.name}</SelectItem>
                  {/each}
                </SelectContent>
              </Select>
            {:else}
              <p class="text-sm text-muted-foreground">No presets available yet.</p>
            {/if}
          </div>
        </div>

        <div class="space-y-4">
          <p class="text-sm font-semibold text-foreground">Character options</p>
          <div class="space-y-3">
            {#each CHARACTER_TOGGLES as option (option.key)}
              <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-background/60 px-4 py-3">
                <div>
                  <p class="text-sm font-medium text-foreground">{option.label}</p>
                  <p class="text-xs text-muted-foreground">{option.description}</p>
                </div>
                <Switch
                  checked={options[option.key]}
                  aria-label={option.label}
                  onclick={() => toggleOption(option.key)}
                />
              </div>
            {/each}
          </div>
        </div>
      </div>

      <div class="space-y-4">
        <div class="flex items-center gap-2">
          <Sparkles class="size-4 text-muted-foreground" aria-hidden="true" />
          <p class="text-sm font-semibold text-foreground">Advanced options</p>
        </div>
        <div class="grid gap-3 md:grid-cols-2">
          {#each ADVANCED_TOGGLES as option (option.key)}
            <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-background/60 px-4 py-3">
              <div>
                <p class="text-sm font-medium text-foreground">{option.label}</p>
                <p class="text-xs text-muted-foreground">{option.description}</p>
              </div>
              <Switch
                checked={options[option.key]}
                aria-label={option.label}
                onclick={() => toggleOption(option.key)}
              />
            </div>
          {/each}
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex items-start gap-3">
      <div class="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ListChecks class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Saved presets</CardTitle>
        <CardDescription>Manage and reuse your favourite password configurations.</CardDescription>
      </div>
    </CardHeader>
    <CardContent>
      {#if presets.length}
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          {#each presets as preset (preset.name)}
            <div class="flex flex-col gap-3 rounded-xl border border-border/60 bg-background/70 p-4">
              <div class="flex items-start justify-between gap-2">
                <div>
                  <p class="text-sm font-semibold text-foreground">{preset.name}</p>
                  <p class="text-xs text-muted-foreground">Length {preset.length} · {preset.charSet}</p>
                </div>
                <div class="flex items-center gap-1">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="size-8 text-muted-foreground hover:text-foreground"
                    aria-label={`Edit preset ${preset.name}`}
                    onclick={() => handleEditPreset(preset)}
                  >
                    <Pencil class="size-4" aria-hidden="true" />
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="size-8 text-muted-foreground hover:text-destructive"
                    aria-label={`Delete preset ${preset.name}`}
                    onclick={() => removePreset(preset.name)}
                  >
                    <Trash2 class="size-4" aria-hidden="true" />
                  </Button>
                </div>
              </div>

              <Progress
                value={Math.min(100, Math.round((preset.strength / MAX_ENTROPY_BITS) * 100))}
                class="bg-muted/40 [&_[data-slot=progress-indicator]]:bg-primary"
              />

              <Button type="button" variant="secondary" class="mt-1 w-full" onclick={() => applyPreset(preset)}>
                Use preset
              </Button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm text-muted-foreground">No saved presets yet. Configure the generator and save your first preset.</p>
      {/if}
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex items-start gap-3">
      <div class="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <FileText class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Site rule templates</CardTitle>
        <CardDescription>Maintain site-specific password requirements.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-3">
      {#if rules.length}
        <div class="space-y-3">
          {#each rules as rule (rule.url)}
            <div class="flex flex-col gap-3 rounded-xl border border-border/60 bg-background/70 p-4">
              <div class="flex flex-wrap items-start justify-between gap-2">
                <div>
                  <p class="text-sm font-semibold text-foreground">{rule.url}</p>
                  <p class="text-xs text-muted-foreground">{rule.desc}</p>
                </div>
                <div class="flex items-center gap-1">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="size-8 text-muted-foreground hover:text-foreground"
                    aria-label={`Edit rule for ${rule.url}`}
                    onclick={() => handleEditRule(rule)}
                  >
                    <Pencil class="size-4" aria-hidden="true" />
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="size-8 text-muted-foreground hover:text-destructive"
                    aria-label={`Delete rule for ${rule.url}`}
                    onclick={() => removeRule(rule.url)}
                  >
                    <Trash2 class="size-4" aria-hidden="true" />
                  </Button>
                </div>
              </div>
              <div class="flex flex-wrap gap-2 text-xs text-muted-foreground">
                <Badge variant="secondary">Length {rule.length}</Badge>
                <Badge variant="outline">{rule.type}</Badge>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm text-muted-foreground">No site rule templates configured yet.</p>
      {/if}
    </CardContent>
  </Card>
</div>

{#if showEditModal && itemToEdit && editModalType}
  <EditModal
    show={showEditModal}
    item={itemToEdit}
    type={editModalType}
    on:close={closeModal}
    on:save={handleSaveEdit}
  />
{/if}

