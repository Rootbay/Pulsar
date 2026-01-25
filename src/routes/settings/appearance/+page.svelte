<script lang="ts">
  import { onMount } from 'svelte';
  import { settingsStore } from '$lib/stores';
  import { appearanceSettings } from '$lib/stores/appearance';
  import type { AppearanceSettings } from '$lib/config/settings';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import { cn } from '$lib/utils';
  import { Palette, Contrast, LayoutDashboard, Waves, Monitor } from '@lucide/svelte';
  import { currentLocale, t } from '$lib/i18n';

  type ThemeOption = AppearanceSettings['theme'];
  type DensityOption = AppearanceSettings['pageDensity'];
  type BooleanSettingKey = {
    [K in keyof AppearanceSettings]: AppearanceSettings[K] extends boolean ? K : never;
  }[keyof AppearanceSettings];

  const locale = $derived($currentLocale);

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

  let currentSettings = $state<AppearanceSettings>({} as AppearanceSettings);
  let theme = $derived(currentSettings.theme || 'system');
  let compactMode = $derived(currentSettings.compactMode || false);
  let fontSize = $derived(currentSettings.fontSize || 14);
  let highContrast = $derived(currentSettings.highContrast || false);
  let reducedMotion = $derived(currentSettings.reducedMotion || false);
  let pageDensity = $derived(currentSettings.pageDensity || 'comfortable');

  $effect(() => {
    return appearanceSettings.subscribe((settings) => {
      currentSettings = settings;
    });
  });

  onMount(() => {
    settingsStore.registerModule('appearance', appearanceSettings);
  });

  function applyChanges(partial: Partial<AppearanceSettings>) {
    appearanceSettings.set({ ...currentSettings, ...partial });
  }

  function isThemeOption(value: string): value is ThemeOption {
    return themeOptions.some((option) => option === value);
  }

  function updateTheme(value: string) {
    if (!isThemeOption(value)) {
      return;
    }
    applyChanges({ theme: value });
  }

  function handleFontSizeInput(event: Event) {
    const value = Number((event.target as HTMLInputElement).value);
    applyChanges({ fontSize: value });
  }

  function toggleSetting(setting: BooleanSettingKey) {
    applyChanges({ [setting]: !currentSettings[setting] } as Partial<AppearanceSettings>);
  }

  function selectDensity(value: DensityOption) {
    if (value === pageDensity) return;
    applyChanges({ pageDensity: value });
  }

  function getThemeLabel(value: ThemeOption, locale: 'en' | 'sv') {
    if (value === 'system') return t(locale, 'System');
    if (value === 'light') return t(locale, 'Light');
    if (value === 'dark') return t(locale, 'Dark');
    return 'System';
  }
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <Palette class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>{t(locale, 'Theme & Display')}</CardTitle>
        <CardDescription>
          {t(
            locale,
            'Customise the application look and spacing.',
            'Anpassa appens utseende och mellanrum.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-6 pt-4">
      <div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium">
            {t(locale, 'Theme')}
          </Label>
          {#key locale}
            <Select type="single" value={theme} onValueChange={updateTheme}>
              <SelectTrigger aria-label="Select theme" class="w-full sm:w-56">
                <span data-slot="select-value" class="flex items-center gap-2 truncate text-sm">
                  <Monitor class="text-muted-foreground h-4 w-4" aria-hidden="true" />
                  {getThemeLabel(theme, locale)}
                </span>
              </SelectTrigger>
              <SelectContent>
                {#each themeOptions as option (option)}
                  <SelectItem value={option}>{getThemeLabel(option, locale)}</SelectItem>
                {/each}
              </SelectContent>
            </Select>
          {/key}
        </div>

        <div
          class="border-border/60 bg-muted/20 flex items-center justify-between gap-4 rounded-lg border px-4 py-3"
        >
          <div class="space-y-1">
            <p class="text-foreground text-sm font-semibold">
              {t(locale, 'Compact Mode')}
            </p>
            <p class="text-muted-foreground text-sm">
              {t(locale, 'Reduce spacing and padding.')}
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
        <Label class="text-foreground text-sm font-medium">
          {t(locale, 'Font Size')}
        </Label>
        <div class="flex items-center gap-4">
          <input
            type="range"
            min={FONT_MIN}
            max={FONT_MAX}
            value={fontSize}
            class="bg-secondary accent-primary h-1.5 flex-1 appearance-none rounded-full"
            oninput={handleFontSizeInput}
          />
          <span class="text-muted-foreground w-12 text-right text-sm">{fontSize}px</span>
        </div>
      </div>

      <div class="grid gap-4 md:grid-cols-2">
        {#each toggleOptions as option (option.key)}
          <div
            class="border-border/60 bg-muted/20 flex items-center justify-between gap-4 rounded-lg border px-4 py-3"
          >
            <div class="space-y-1">
              <p class="text-foreground text-sm font-semibold">
                {option.key === 'highContrast'
                  ? t(locale, 'High Contrast')
                  : t(locale, 'Reduced Motion')}
              </p>
              <p class="text-muted-foreground text-sm">
                {option.key === 'highContrast'
                  ? t(
                      locale,
                      'Increase contrast for improved readability.',
                      'Öka kontrasten för bättre läsbarhet.'
                    )
                  : t(
                      locale,
                      'Minimise animations and motion effects.',
                      'Minimera animationer och rörelse.'
                    )}
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

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <LayoutDashboard class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>{t(locale, 'Page Density')}</CardTitle>
        <CardDescription>
          {t(
            locale,
            'Choose how much information appears on each view.',
            'Välj hur mycket information som visas per vy.'
          )}
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
              'border-border/60 bg-muted/20 h-full w-full flex-col items-start gap-3 rounded-xl p-4 text-left transition-colors',
              pageDensity === option.value
                ? 'border-primary/60 bg-primary/10 text-primary shadow-sm'
                : 'hover:border-primary/50 hover:text-primary'
            )}
            aria-pressed={pageDensity === option.value}
            aria-label={`Set page density to ${option.title.toLowerCase()}`}
            onclick={() => selectDensity(option.value)}
          >
            <div>
              <p class="text-foreground text-sm font-semibold">
                {option.value === 'comfortable'
                  ? t(locale, 'Comfortable')
                  : option.value === 'compact'
                    ? t(locale, 'Compact')
                    : t(locale, 'Dense')}
              </p>
            </div>
            <div
              class={cn(
                'bg-background/70 w-full rounded-lg p-3',
                option.spacing,
                '[&>div]:bg-muted-foreground/40 [&>div]:h-1.5 [&>div]:rounded-full'
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
