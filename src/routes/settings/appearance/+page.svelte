
<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { settingsStore } from '$lib/stores';
  import { appearanceSettings } from '$lib/stores/appearance';
  import type { AppearanceSettings } from '$lib/config/settings';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import { cn } from '$lib/utils';
  import { Palette, Contrast, LayoutDashboard, Waves, Monitor } from '@lucide/svelte';
  import { currentLocale } from '$lib/i18n';

  type ThemeOption = AppearanceSettings['theme'];
  type DensityOption = AppearanceSettings['pageDensity'];
  type BooleanSettingKey = {
    [K in keyof AppearanceSettings]: AppearanceSettings[K] extends boolean ? K : never;
  }[keyof AppearanceSettings];

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  $: locale = $currentLocale as 'en' | 'sv';

  const themeOptions: ThemeOption[] = ['system', 'light', 'dark'];

  const toggleOptions: Array<{
    key: BooleanSettingKey;
    title: string;
    description: string;
    ariaLabel: string;
    Icon: typeof Contrast;
  }> = [
    {
      key: 'highContrast',
      title: 'High Contrast',
      description: 'Increase contrast for improved readability.',
      ariaLabel: 'Toggle high contrast',
      Icon: Contrast
    },
    {
      key: 'reducedMotion',
      title: 'Reduced Motion',
      description: 'Minimise animations and motion effects.',
      ariaLabel: 'Toggle reduced motion',
      Icon: Waves
    }
  ];

  const densityOptions: Array<{
    value: DensityOption;
    title: string;
    description: string;
    spacing: string;
  }> = [
    {
      value: 'comfortable',
      title: 'Comfortable',
      description: 'Balanced spacing for everyday use.',
      spacing: 'space-y-3'
    },
    {
      value: 'compact',
      title: 'Compact',
      description: 'Denser layout for information-dense views.',
      spacing: 'space-y-2'
    },
    {
      value: 'dense',
      title: 'Dense',
      description: 'Maximum information per view with tight spacing.',
      spacing: 'space-y-1'
    }
  ];

  const FONT_MIN = 12;
  const FONT_MAX = 20;

  let currentSettings: AppearanceSettings;
  let theme: ThemeOption = 'system';
  let compactMode = false;
  let fontSize = 14;
  let highContrast = false;
  let reducedMotion = false;
  let pageDensity: DensityOption = 'comfortable';

  const unsubscribe = appearanceSettings.subscribe((settings) => {
    currentSettings = settings;
    theme = settings.theme;
    compactMode = settings.compactMode;
    fontSize = settings.fontSize;
    highContrast = settings.highContrast;
    reducedMotion = settings.reducedMotion;
    pageDensity = settings.pageDensity;
  });

  onMount(() => {
    settingsStore.registerModule('appearance', appearanceSettings);
  });

  onDestroy(() => {
    unsubscribe();
  });

  function applyChanges(partial: Partial<AppearanceSettings>) {
    const next = { ...currentSettings, ...partial };
    currentSettings = next;
    appearanceSettings.set(next);
  }

  function isThemeOption(value: string): value is ThemeOption {
    return themeOptions.some((option) => option === value);
  }

  function updateTheme(value: string) {
    if (!isThemeOption(value)) {
      return;
    }

    theme = value;
    applyChanges({ theme: value });
  }

  function handleFontSizeInput(event: Event) {
    const value = Number((event.target as HTMLInputElement).value);
    fontSize = value;
    applyChanges({ fontSize: value });
  }

  function toggleSetting(setting: BooleanSettingKey) {
    applyChanges({ [setting]: !currentSettings[setting] } as Partial<AppearanceSettings>);
  }

  function selectDensity(value: DensityOption) {
    if (value === pageDensity) return;
    pageDensity = value;
    applyChanges({ pageDensity: value });
  }

  function getThemeLabel(value: ThemeOption, locale: 'en' | 'sv') {
    if (value === 'system') return t(locale, 'System', 'System');
    if (value === 'light') return t(locale, 'Light', 'Ljust');
    if (value === 'dark') return t(locale, 'Dark', 'Mörkt');
    return 'System';
  }
</script>

<div class="flex-1 min-h-0 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Palette class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>{t(locale, 'Theme & Display', 'Tema & visning')}</CardTitle>
        <CardDescription>
          {t(locale, 'Customise the application look and spacing.', 'Anpassa appens utseende och mellanrum.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-6 pt-4">
      <div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <div class="space-y-2">
          <Label class="text-sm font-medium text-foreground">
            {t(locale, 'Theme', 'Tema')}
          </Label>
          {#key locale}
            <Select type="single" value={theme} onValueChange={updateTheme}>
              <SelectTrigger aria-label="Select theme" class="w-full sm:w-56">
                <span data-slot="select-value" class="flex items-center gap-2 truncate text-sm">
                  <Monitor class="h-4 w-4 text-muted-foreground" aria-hidden="true" />
                  {getThemeLabel(theme, locale)}
                </span>
              </SelectTrigger>
              <SelectContent>
                {#each themeOptions as option}
                  <SelectItem value={option}>{getThemeLabel(option, locale)}</SelectItem>
                {/each}
              </SelectContent>
            </Select>
          {/key}
        </div>

        <div class="flex items-center justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
          <div class="space-y-1">
            <p class="text-sm font-semibold text-foreground">
              {t(locale, 'Compact Mode', 'Kompakt läge')}
            </p>
            <p class="text-sm text-muted-foreground">
              {t(locale, 'Reduce spacing and padding.', 'Minska mellanrum och padding.')}
            </p>
          </div>
          <Switch
            checked={compactMode}
            aria-label="Toggle compact mode"
            onclick={() => toggleSetting('compactMode')}
          />
        </div>
      </div>

      <div class="space-y-2">
        <Label class="text-sm font-medium text-foreground">
          {t(locale, 'Font Size', 'Textstorlek')}
        </Label>
        <div class="flex items-center gap-4">
          <input
            type="range"
            min={FONT_MIN}
            max={FONT_MAX}
            value={fontSize}
            class="h-1.5 flex-1 appearance-none rounded-full bg-secondary accent-primary"
            oninput={handleFontSizeInput}
          />
          <span class="w-12 text-right text-sm text-muted-foreground">{fontSize}px</span>
        </div>
      </div>

      <div class="grid gap-4 md:grid-cols-2">
        {#each toggleOptions as option (option.key)}
          <div class="flex items-center justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
            <div class="space-y-1">
              <p class="text-sm font-semibold text-foreground">
                {option.key === 'highContrast'
                  ? t(locale, 'High Contrast', 'Hög kontrast')
                  : t(locale, 'Reduced Motion', 'Minskad rörelse')}
              </p>
              <p class="text-sm text-muted-foreground">
                {option.key === 'highContrast'
                  ? t(locale, 'Increase contrast for improved readability.', 'Öka kontrasten för bättre läsbarhet.')
                  : t(locale, 'Minimise animations and motion effects.', 'Minimera animationer och rörelse.')}
              </p>
            </div>
            <Switch
              checked={option.key === 'highContrast' ? highContrast : reducedMotion}
              aria-label={option.ariaLabel}
              onclick={() => toggleSetting(option.key)}
            />
          </div>
        {/each}
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <LayoutDashboard class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>{t(locale, 'Page Density', 'Sidtäthet')}</CardTitle>
        <CardDescription>
          {t(locale, 'Choose how much information appears on each view.', 'Välj hur mycket information som visas per vy.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      <div class="grid gap-3 md:grid-cols-3">
        {#each densityOptions as option (option.value)}
          <Button
            type="button"
            variant="outline"
            class={cn(
              'h-full w-full flex-col items-start gap-3 rounded-xl border-border/60 bg-muted/20 p-4 text-left transition-colors',
              pageDensity === option.value
                ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                : 'hover:border-primary/50 hover:text-primary'
            )}
            aria-pressed={pageDensity === option.value}
            aria-label={`Set page density to ${option.title.toLowerCase()}`}
            onclick={() => selectDensity(option.value)}
          >
            <div>
              <p class="text-sm font-semibold text-foreground">
                {option.value === 'comfortable'
                  ? t(locale, 'Comfortable', 'Bekväm')
                  : option.value === 'compact'
                    ? t(locale, 'Compact', 'Kompakt')
                    : t(locale, 'Dense', 'Tät')}
              </p>
            </div>
            <div
              class={cn(
                'w-full rounded-lg bg-background/70 p-3',
                option.spacing,
                '[&>div]:h-1.5 [&>div]:rounded-full [&>div]:bg-muted-foreground/40'
              )}
            >
              <div class="w-full"></div>
              <div class="w-10/12"></div>
              <div class="w-8/12"></div>
            </div>
          </Button>
        {/each}
      </div>
    </CardContent>
  </Card>
</div>
