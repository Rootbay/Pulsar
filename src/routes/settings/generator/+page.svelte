<script lang="ts">
  import { onMount } from 'svelte';
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
  import { currentLocale } from '$lib/i18n';
  import { copyText } from '$lib/utils/copyHelper';

  type GeneratorOptions = GeneratorSettings['options'];
  type GeneratorOptionKey = keyof GeneratorOptions;

  const SYMBOL_CHARSET = '!@#$%^&*()_+-=[]{}|;:,.<>?';
  const AMBIGUOUS_CHARS = new Set(['i', 'I', '1', 'L', 'o', 'O', '0']);
  const SIMILAR_CHARS = new Set('oO0l1IvVwWsScCpPkKxXzZbBdDgGqQeEfFtTuUjJmMnrRhHaAyY'.split(''));

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

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  let locale = $derived($currentLocale as 'en' | 'sv');

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

  let presets = $state<PasswordPreset[]>([]);
  let rules = $state<SiteRule[]>([]);

  let passwordLength = $state(DEFAULT_PASSWORD_LENGTH);
  let options = $state<GeneratorOptions>({ ...DEFAULT_OPTIONS });
  let generatedPassword = $state('');
  let hasCharacterPool = $state(true);
  let copyButtonText = $state('Copy');
  let copyResetTimer: ReturnType<typeof setTimeout> | undefined;

  let showEditModal = $state(false);
  let itemToEdit = $state<PasswordPreset | SiteRule | null>(null);
  let editModalType = $state<'preset' | 'rule' | null>(null);
  let selectedPresetName = $state<string | null>(null);

  let strengthEntropy = $state(0);
  let strengthLevel = $state<StrengthLevel>('weak');

  function calculateEntropy(length: number, poolSize: number): number {
    if (poolSize <= 0) return 0;
    return Math.floor(length * Math.log2(poolSize));
  }

  function getPoolSize(): number {
    let size = 0;
    if (options.uppercase) size += 26;
    if (options.lowercase) size += 26;
    if (options.digits) size += 10;
    if (options.symbols) size += SYMBOL_CHARSET.length;

    if (options.ambiguous) {
      size = Math.max(0, size - 7);
    }
    return size;
  }

  function refreshPassword() {
    const poolSize = getPoolSize();
    hasCharacterPool = poolSize > 0;

    if (hasCharacterPool) {
      handleGenerate();
    } else {
      generatedPassword = '';
      strengthEntropy = 0;
      strengthLevel = 'weak';
    }
  }

  function handleGenerate() {
    const poolSize = getPoolSize();
    if (poolSize <= 0) return;

    let charset = '';
    if (options.uppercase) charset += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    if (options.lowercase) charset += 'abcdefghijklmnopqrstuvwxyz';
    if (options.digits) charset += '0123456789';
    if (options.symbols) charset += SYMBOL_CHARSET;

    if (options.ambiguous) {
      charset = charset
        .split('')
        .filter((c) => !AMBIGUOUS_CHARS.has(c))
        .join('');
    }

    if (options.similar) {
      charset = charset
        .split('')
        .filter((c) => !SIMILAR_CHARS.has(c))
        .join('');
    }

    if (charset.length === 0) return;

    let password = '';
    const array = new Uint32Array(passwordLength);
    crypto.getRandomValues(array);

    if (options.pronounceable) {
      const vowels = 'aeiou';
      const consonants = charset
        .split('')
        .filter((c) => !vowels.includes(c.toLowerCase()))
        .join('');
      const actualVowels = charset
        .split('')
        .filter((c) => vowels.includes(c.toLowerCase()))
        .join('');

      if (actualVowels.length > 0 && consonants.length > 0) {
        for (let i = 0; i < passwordLength; i++) {
          const source = i % 2 === 0 ? consonants : actualVowels;
          password += source[array[i] % source.length];
        }
      } else {
        for (let i = 0; i < passwordLength; i++) {
          password += charset[array[i] % charset.length];
        }
      }
    } else {
      for (let i = 0; i < passwordLength; i++) {
        password += charset[array[i] % charset.length];
      }
    }

    generatedPassword = password;
    strengthEntropy = calculateEntropy(passwordLength, charset.length);

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
      await copyText(generatedPassword);
      copyButtonText = 'Copied!';
      if (copyResetTimer) clearTimeout(copyResetTimer);
      copyResetTimer = setTimeout(() => {
        copyButtonText = 'Copy';
      }, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }

  function saveCurrentSettingsAsPreset() {
    const name = prompt('Enter a name for this preset:');
    if (!name) return;

    const charSets: string[] = [];
    if (options.uppercase) charSets.push('A-Z');
    if (options.lowercase) charSets.push('a-z');
    if (options.digits) charSets.push('0-9');
    if (options.symbols) charSets.push('!@#$');

    passwordPresets.addPreset({
      name,
      length: passwordLength,
      charSet: charSets.join(', '),
      strength: strengthEntropy,
      settings: JSON.parse(JSON.stringify(options))
    });
  }

  function resetOptions() {
    generatorSettings.update((current) => ({
      ...current,
      passwordLength: DEFAULT_PASSWORD_LENGTH,
      options: { ...DEFAULT_OPTIONS }
    }));
  }

  function updateLength(val: number) {
    generatorSettings.update((current) => ({
      ...current,
      passwordLength: val
    }));
  }

  function handlePresetSelect(name: string) {
    const preset = presets.find((p) => p.name === name);
    if (preset) {
      applyPreset(preset);
      selectedPresetName = name;
    }
  }

  function toggleOption(key: GeneratorOptionKey) {
    generatorSettings.update((current) => ({
      ...current,
      options: {
        ...current.options,
        [key]: !current.options[key]
      }
    }));
  }

  function applyPreset(preset: PasswordPreset) {
    generatorSettings.update((current) => ({
      ...current,
      passwordLength: preset.length,
      options: { ...preset.settings }
    }));
  }

  function handleEditPreset(preset: PasswordPreset) {
    itemToEdit = JSON.parse(JSON.stringify(preset));
    editModalType = 'preset';
    showEditModal = true;
  }

  function removePreset(name: string) {
    if (confirm(`Are you sure you want to delete preset "${name}"?`)) {
      passwordPresets.deletePreset(name);
    }
  }

  function handleEditRule(rule: SiteRule) {
    itemToEdit = JSON.parse(JSON.stringify(rule));
    editModalType = 'rule';
    showEditModal = true;
  }

  function removeRule(url: string) {
    if (confirm(`Are you sure you want to delete rule for "${url}"?`)) {
      siteRules.deleteRule(url);
    }
  }

  function closeModal() {
    showEditModal = false;
    itemToEdit = null;
    editModalType = null;
  }

  function handleSaveEdit(updatedItem: any) {
    if (editModalType === 'preset' && itemToEdit) {
      passwordPresets.updatePreset((itemToEdit as PasswordPreset).name, updatedItem);
    } else if (editModalType === 'rule' && itemToEdit) {
      siteRules.updateRule((itemToEdit as SiteRule).url, updatedItem);
    }
    closeModal();
  }

  $effect(() => {
    return generatorSettings.subscribe((settings) => {
      passwordLength = settings.passwordLength;
      options = { ...settings.options };
      refreshPassword();
    });
  });

  $effect(() => {
    return passwordPresets.subscribe((value) => {
      presets = value;
      if (selectedPresetName && !value.some((preset) => preset.name === selectedPresetName)) {
        selectedPresetName = null;
      }
    });
  });

  $effect(() => {
    return siteRules.subscribe((value) => {
      rules = value;
    });
  });

  onMount(() => {
    generatorSettings.update((current) => ({
      ...current,
      options: { ...DEFAULT_OPTIONS, ...current.options }
    }));
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
    <CardHeader class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <Key class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t(locale, 'Password Generator', 'Lösenordsgenerator')}
          </CardTitle>
          <CardDescription>
            {t(
              locale,
              'Generate strong and secure passwords on demand.',
              'Generera starka och säkra lösenord vid behov.'
            )}
          </CardDescription>
        </div>
      </div>
      <Badge variant="secondary" class="mt-2 w-fit sm:mt-0">
        {t(locale, 'Length', 'Längd')}&nbsp;{passwordLength}
      </Badge>
    </CardHeader>

    <CardContent class="space-y-6">
      <div class="border-border/60 bg-muted/10 space-y-3 rounded-xl border p-4">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <p class="text-muted-foreground text-sm font-medium">
            {t(locale, 'Generated password', 'Genererat lösenord')}
          </p>
          <Button
            type="button"
            variant="outline"
            size="sm"
            class="gap-2"
            onclick={handleGenerate}
            disabled={!hasCharacterPool}
          >
            <RefreshCcw class="size-4" aria-hidden="true" />
            {t(locale, 'Generate new', 'Generera nytt')}
          </Button>
        </div>
        <div
          class="border-border/40 bg-background/80 flex flex-col gap-2 rounded-lg border p-4 font-mono text-sm"
        >
          {#if generatedPassword}
            <span class="break-all">{generatedPassword}</span>
          {:else}
            <span class="text-muted-foreground text-sm">
              {t(
                locale,
                'Select at least one character set to generate a password.',
                'Välj minst en teckenuppsättning för att generera ett lösenord.'
              )}
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
            onclick={saveCurrentSettingsAsPreset}
          >
            <Save class="size-4" aria-hidden="true" />
            {t(locale, 'Save as preset', 'Spara som förval')}
          </Button>
          <Button type="button" variant="outline" class="gap-2" onclick={resetOptions}>
            <RotateCcw class="size-4" aria-hidden="true" />
            {t(locale, 'Reset to defaults', 'Återställ till standard')}
          </Button>
        </div>
      </div>

      <div class="border-border/60 bg-muted/10 space-y-3 rounded-xl border p-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="text-foreground text-sm font-semibold">
              {t(locale, 'Password strength', 'Lösenordsstyrka')}
            </p>
            <p class="text-muted-foreground text-xs">
              {t(locale, 'Entropy', 'Entropi')} ≈ {strengthEntropy}
              {t(locale, 'bits', 'bitar')}
            </p>
          </div>
          <span class={cn('text-sm font-semibold', strengthMeta.textClass)}>
            {strengthLevel === 'weak'
              ? t(locale, 'Weak', 'Svagt')
              : strengthLevel === 'medium'
                ? t(locale, 'Good', 'Bra')
                : t(locale, 'Very strong', 'Mycket starkt')}
          </span>
        </div>
        <Progress value={strengthProgress} class={cn('bg-muted/40', strengthMeta.barClass)} />
      </div>

      <div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <div class="space-y-4">
          <div class="space-y-2">
            <p class="text-foreground text-sm font-medium">
              {t(locale, 'Password length', 'Lösenordslängd')}
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

          <div class="space-y-2">
            <p class="text-foreground text-sm font-medium">
              {t(locale, 'Apply saved preset', 'Använd sparat förval')}
            </p>
            {#if presets.length}
              <Select
                type="single"
                value={selectedPresetName ?? ''}
                onValueChange={handlePresetSelect}
              >
                <SelectTrigger aria-label="Select password preset" class="w-full">
                  <span data-slot="select-value" class="flex items-center gap-2 truncate text-sm">
                    {selectedPresetName ?? t(locale, 'Choose a preset', 'Välj ett förval')}
                  </span>
                </SelectTrigger>
                <SelectContent>
                  {#each presets as preset}
                    <SelectItem value={preset.name}>{preset.name}</SelectItem>
                  {/each}
                </SelectContent>
              </Select>
            {:else}
              <p class="text-muted-foreground text-sm">
                {t(locale, 'No presets available yet.', 'Inga förval sparade ännu.')}
              </p>
            {/if}
          </div>
        </div>

        <div class="space-y-4">
          <p class="text-foreground text-sm font-semibold">
            {t(locale, 'Character options', 'Teckenväljare')}
          </p>
          <div class="space-y-3">
            {#each CHARACTER_TOGGLES as option (option.key)}
              <div
                class="border-border/60 bg-background/60 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
              >
                <div>
                  <p class="text-foreground text-sm font-medium">
                    {option.key === 'uppercase'
                      ? t(locale, 'Include uppercase (A-Z)', 'Inkludera versaler (A–Z)')
                      : option.key === 'lowercase'
                        ? t(locale, 'Include lowercase (a-z)', 'Inkludera gemener (a–z)')
                        : option.key === 'digits'
                          ? t(locale, 'Include digits (0-9)', 'Inkludera siffror (0–9)')
                          : t(locale, 'Include symbols (!@#$)', 'Inkludera symboler (!@#$)')}
                  </p>
                  <p class="text-muted-foreground text-xs">
                    {option.key === 'uppercase'
                      ? t(
                          locale,
                          'Adds capital letters to the character pool.',
                          'Lägger till versaler i teckenmängden.'
                        )
                      : option.key === 'lowercase'
                        ? t(
                            locale,
                            'Adds lowercase letters to the character pool.',
                            'Lägger till gemener i teckenmängden.'
                          )
                        : option.key === 'digits'
                          ? t(
                              locale,
                              'Adds numeric characters to the password.',
                              'Lägger till siffror i lösenordet.'
                            )
                          : t(
                              locale,
                              'Adds punctuation and symbol characters.',
                              'Lägger till skiljetecken och symboler.'
                            )}
                  </p>
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
          <Sparkles class="text-muted-foreground size-4" aria-hidden="true" />
          <p class="text-foreground text-sm font-semibold">
            {t(locale, 'Advanced options', 'Avancerade alternativ')}
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
                    ? t(locale, 'Avoid ambiguous characters', 'Undvik tvetydiga tecken')
                    : option.key === 'similar'
                      ? t(
                          locale,
                          'Exclude visually similar characters',
                          'Exkludera visuellt liknande tecken'
                        )
                      : t(locale, 'Pronounceable mode', 'Läsläge (uttalbart)')}
                </p>
                <p class="text-muted-foreground text-xs">
                  {option.key === 'ambiguous'
                    ? t(
                        locale,
                        'Exclude characters like i, l, O, and 0.',
                        'Exkludera tecken som i, l, O och 0.'
                      )
                    : option.key === 'similar'
                      ? t(
                          locale,
                          'Avoid characters that look alike in some fonts.',
                          'Undvik tecken som ser lika ut i vissa typsnitt.'
                        )
                      : t(
                          locale,
                          'Alternate vowels and consonants for readability.',
                          'Växla vokaler och konsonanter för bättre läsbarhet.'
                        )}
                </p>
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

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex items-start gap-3">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ListChecks class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Saved presets', 'Sparade förval')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Manage and reuse your favourite password configurations.',
            'Hantera och återanvänd dina favoritkonfigurationer för lösenord.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent>
      {#if presets.length}
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          {#each presets as preset (preset.name)}
            <div
              class="border-border/60 bg-background/70 flex flex-col gap-3 rounded-xl border p-4"
            >
              <div class="flex items-start justify-between gap-2">
                <div>
                  <p class="text-foreground text-sm font-semibold">{preset.name}</p>
                  <p class="text-muted-foreground text-xs">
                    {t(locale, 'Length', 'Längd')}
                    {preset.length} · {preset.charSet}
                  </p>
                </div>
                <div class="flex items-center gap-1">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-foreground size-8"
                    aria-label={`Edit preset ${preset.name}`}
                    onclick={() => handleEditPreset(preset)}
                  >
                    <Pencil class="size-4" aria-hidden="true" />
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-destructive size-8"
                    aria-label={`Delete preset ${preset.name}`}
                    onclick={() => removePreset(preset.name)}
                  >
                    <Trash2 class="size-4" aria-hidden="true" />
                  </Button>
                </div>
              </div>

              <Progress
                value={Math.min(100, Math.round((preset.strength / MAX_ENTROPY_BITS) * 100))}
                class="bg-muted/40 **:data-[slot=progress-indicator]:bg-primary"
              />

              <Button
                type="button"
                variant="secondary"
                class="mt-1 w-full"
                onclick={() => applyPreset(preset)}
              >
                {t(locale, 'Use preset', 'Använd förval')}
              </Button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-muted-foreground text-sm">
          {t(
            locale,
            'No saved presets yet. Configure the generator and save your first preset.',
            'Inga sparade förval ännu. Konfigurera generatorn och spara ditt första förval.'
          )}
        </p>
      {/if}
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex items-start gap-3">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <FileText class="size-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Site rule templates', 'Mall för webbplatsregler')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Maintain site-specific password requirements.',
            'Hantera webbplatsspecifika lösenordskrav.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-3">
      {#if rules.length}
        <div class="space-y-3">
          {#each rules as rule (rule.url)}
            <div
              class="border-border/60 bg-background/70 flex flex-col gap-3 rounded-xl border p-4"
            >
              <div class="flex flex-wrap items-start justify-between gap-2">
                <div>
                  <p class="text-foreground text-sm font-semibold">{rule.url}</p>
                  <p class="text-muted-foreground text-xs">{rule.desc}</p>
                </div>
                <div class="flex items-center gap-1">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-foreground size-8"
                    aria-label={`Edit rule for ${rule.url}`}
                    onclick={() => handleEditRule(rule)}
                  >
                    <Pencil class="size-4" aria-hidden="true" />
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-destructive size-8"
                    aria-label={`Delete rule for ${rule.url}`}
                    onclick={() => removeRule(rule.url)}
                  >
                    <Trash2 class="size-4" aria-hidden="true" />
                  </Button>
                </div>
              </div>
              <div class="text-muted-foreground flex flex-wrap gap-2 text-xs">
                <Badge variant="secondary">
                  {t(locale, 'Length', 'Längd')}
                  {rule.length}
                </Badge>
                <Badge variant="outline">{rule.type}</Badge>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-muted-foreground text-sm">
          {t(
            locale,
            'No site rule templates configured yet.',
            'Inga mallar för webbplatsregler är konfigurerade ännu.'
          )}
        </p>
      {/if}
    </CardContent>
  </Card>
</div>

{#if showEditModal && itemToEdit && editModalType}
  <EditModal
    show={showEditModal}
    item={itemToEdit}
    type={editModalType}
    onclose={closeModal}
    onsave={handleSaveEdit}
  />
{/if}
