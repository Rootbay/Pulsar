<script lang="ts">
  import { generalSettings } from '$lib/stores/general';
  import type { AppLanguage, GeneralSettings } from '$lib/config/settings';
  import KeyboardShortcutsPopup from '$lib/components/KeyboardShortcutsPopup.svelte';
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
  import { Smartphone, Key, Lock, Settings } from '@lucide/svelte';
  import { currentLocale, t as translate, type I18nKey } from '$lib/i18n';

  type BooleanSettingKey = {
    [K in keyof GeneralSettings]: GeneralSettings[K] extends boolean ? K : never;
  }[keyof GeneralSettings];

  type SelectSettingKey = Exclude<keyof GeneralSettings, BooleanSettingKey>;
  type NonLanguageSelectKey = Exclude<SelectSettingKey, 'appLanguage'>;

  let currentGeneralSettings = $state<GeneralSettings>({} as GeneralSettings);
  const locale = $derived($currentLocale);
  const t = (key: I18nKey, vars: Record<string, string | number> = {}) =>
    translate(locale, key, vars);

  $effect(() => {
    return generalSettings.subscribe((value) => {
      currentGeneralSettings = value;
    });
  });

  let showKeyboardShortcutsPopup = $state(false);
  let languageSearch = $state('');

  type LanguageOption = { value: AppLanguage; label: string; search: string };

  const selectOptions: Record<NonLanguageSelectKey, { value: string; label: string }[]> = {
    defaultVaultOnStartup: [
      { value: '8 characters', label: '8 characters' },
      { value: '12 characters', label: '12 characters' },
      { value: '16 characters', label: '16 characters' },
      { value: '20 characters', label: '20 characters' },
      { value: '32 characters', label: '32 characters' }
    ],
    defaultViewOnOpen: [
      { value: '8 characters', label: '8 characters' },
      { value: '12 characters', label: '12 characters' },
      { value: '16 characters', label: '16 characters' },
      { value: '20 characters', label: '20 characters' },
      { value: '32 characters', label: '32 characters' }
    ]
  };

  const toggleSettings = $derived<
    Array<{
      key: BooleanSettingKey;
      title: string;
      description: string;
      ariaLabel: string;
    }>
  >([
    {
      key: 'startOnSystemBoot',
      title: t('settingsGeneralStartOnBootTitle'),
      description: t('settingsGeneralStartOnBootDesc'),
      ariaLabel: t('settingsGeneralStartOnBootAria')
    },
    {
      key: 'showInSystemTray',
      title: t('settingsGeneralShowInTrayTitle'),
      description: t('settingsGeneralShowInTrayDesc'),
      ariaLabel: t('settingsGeneralShowInTrayAria')
    }
  ]);

  const authenticationMethods: Array<{
    key: BooleanSettingKey;
    icon: typeof Smartphone;
  }> = [
    {
      key: 'totpEnabled',
      icon: Smartphone
    }
  ];

  function updateSetting<K extends keyof GeneralSettings>(setting: K, value: GeneralSettings[K]) {
    generalSettings.set({ ...currentGeneralSettings, [setting]: value });
  }

  function toggleSwitch(setting: BooleanSettingKey) {
    updateSetting(setting, !currentGeneralSettings[setting]);
  }

  function getLanguageOptions(): LanguageOption[] {
    return [
      {
        value: 'system',
        label: t('settingsGeneralLanguageSystem'),
        search: 'system auto os default'
      },
      { value: 'en', label: 'English', search: 'english en' },
      { value: 'es', label: 'Español', search: 'spanish espanol es' },
      { value: 'fr', label: 'Français', search: 'french francais fr' },
      { value: 'de', label: 'Deutsch', search: 'german deutsch de' },
      { value: 'pt-BR', label: 'Português (Brasil)', search: 'portuguese brasil br pt' },
      { value: 'zh', label: '简体中文', search: 'chinese simplified zh' },
      { value: 'sv', label: 'Svenska', search: 'swedish svenska sv' },
      { value: 'ru', label: 'Русский', search: 'russian russkiy ru' },
      { value: 'ja', label: '日本語', search: 'japanese nihongo ja' },
      { value: 'hi', label: 'हिन्दी', search: 'hindi hi' },
      { value: 'ko', label: '한국어', search: 'korean ko' },
      { value: 'ar', label: 'العربية', search: 'arabic ar' },
      { value: 'it', label: 'Italiano', search: 'italian it' },
      { value: 'tr', label: 'Türkçe', search: 'turkish turkce tr' },
      { value: 'nl', label: 'Nederlands', search: 'dutch nederlands nl' },
      { value: 'pl', label: 'Polski', search: 'polish polski pl' },
      { value: 'id', label: 'Bahasa Indonesia', search: 'indonesian bahasa indonesia id' },
      { value: 'th', label: 'ไทย', search: 'thai th' },
      { value: 'vi', label: 'Tiếng Việt', search: 'vietnamese tieng viet vi' },
      { value: 'el', label: 'Ελληνικά', search: 'greek ellinika el' }
    ];
  }

  function getFilteredLanguageOptions(): LanguageOption[] {
    const options = getLanguageOptions();
    const query = languageSearch.trim().toLowerCase();
    if (!query) return options;
    return options.filter((option) => {
      const haystack = `${option.label} ${option.value} ${option.search}`.toLowerCase();
      return haystack.includes(query);
    });
  }

  function getOptionLabel(setting: SelectSettingKey) {
    if (setting === 'appLanguage') {
      const option = getLanguageOptions().find(
        (item) => item.value === currentGeneralSettings.appLanguage
      );
      return option?.label ?? t('settingsGeneralLanguageSystem');
    }
    const option = selectOptions[setting].find(
      (item) => item.value === currentGeneralSettings[setting]
    );
    return option?.label ?? t('settingsGeneralSelectOptionFallback');
  }
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <Settings size={20} color="currentColor" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>{t('settingsGeneralTitle')}</CardTitle>
          <CardDescription>
            {t('settingsGeneralDescription')}
          </CardDescription>
        </div>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-8 pt-4">
      <div class="grid gap-6 md:grid-cols-2">
        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium">
            {t('settingsGeneralLanguageLabel')}
          </Label>
          {#each [locale] as l (l)}
            <Select
              type="single"
              value={currentGeneralSettings.appLanguage}
              onValueChange={(value) => {
                const next = getLanguageOptions().find((option) => option.value === value)?.value;
                if (!next) return;
                updateSetting('appLanguage', next);
                languageSearch = '';
              }}
            >
              <SelectTrigger aria-label={t('settingsGeneralLanguageAria')} class="w-full">
                <span data-slot="select-value" class="truncate">
                  {getOptionLabel('appLanguage')}
                </span>
              </SelectTrigger>
              <SelectContent>
                <div class="px-2 pt-2">
                  <input
                    class="border-border/60 bg-muted/30 focus:border-primary focus:ring-primary/30 text-foreground placeholder:text-muted-foreground w-full rounded-md border px-3 py-2 text-sm outline-none focus:ring-2"
                    type="text"
                    placeholder={t('settingsGeneralLanguageSearchPlaceholder')}
                    aria-label={t('settingsGeneralLanguageSearchPlaceholder')}
                    bind:value={languageSearch}
                  />
                </div>
                <div class="max-h-64 overflow-y-auto pb-1">
                  {#if getFilteredLanguageOptions().length === 0}
                    <div class="text-muted-foreground px-3 py-2 text-sm">
                      {t('settingsGeneralLanguageNoResults')}
                    </div>
                  {:else}
                    {#each getFilteredLanguageOptions() as option (option.value)}
                      <SelectItem value={option.value}>{option.label}</SelectItem>
                    {/each}
                  {/if}
                </div>
              </SelectContent>
            </Select>
          {/each}
        </div>

        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium">
            {t('settingsGeneralDefaultVaultLabel')}
          </Label>
          <Select
            type="single"
            value={currentGeneralSettings.defaultVaultOnStartup}
            onValueChange={(value) => updateSetting('defaultVaultOnStartup', value)}
          >
            <SelectTrigger aria-label={t('settingsGeneralDefaultVaultAria')} class="w-full">
              <span data-slot="select-value" class="truncate">
                {getOptionLabel('defaultVaultOnStartup')}
              </span>
            </SelectTrigger>
            <SelectContent>
              {#each selectOptions.defaultVaultOnStartup as option (option.value)}
                <SelectItem value={option.value}>{option.label}</SelectItem>
              {/each}
            </SelectContent>
          </Select>
        </div>
      </div>

      <div class="space-y-5">
        {#each toggleSettings as toggleSetting (toggleSetting.key)}
          <div
            class="border-border/60 bg-muted/20 flex items-center justify-between gap-4 rounded-lg border px-4 py-3"
          >
            <div class="space-y-1">
              <p class="text-foreground text-sm font-medium">
                {toggleSetting.title}
              </p>
              <p class="text-muted-foreground text-sm">
                {toggleSetting.description}
              </p>
            </div>
            <Switch
              checked={currentGeneralSettings[toggleSetting.key]}
              aria-label={toggleSetting.ariaLabel}
              onclick={() => toggleSwitch(toggleSetting.key)}
            />
          </div>
        {/each}
      </div>

      <div class="space-y-2">
        <Label class="text-foreground text-sm font-medium">
          {t('settingsGeneralDefaultViewLabel')}
        </Label>
        <Select
          type="single"
          value={currentGeneralSettings.defaultViewOnOpen}
          onValueChange={(value) => updateSetting('defaultViewOnOpen', value)}
        >
          <SelectTrigger aria-label={t('settingsGeneralDefaultViewAria')} class="w-full md:w-80">
            <span data-slot="select-value" class="truncate">
              {getOptionLabel('defaultViewOnOpen')}
            </span>
          </SelectTrigger>
          <SelectContent>
            {#each selectOptions.defaultViewOnOpen as option (option.value)}
              <SelectItem value={option.value}>{option.label}</SelectItem>
            {/each}
          </SelectContent>
        </Select>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader
      class="border-border/40 flex flex-col gap-3 border-b pb-4 sm:flex-row sm:items-start sm:justify-between"
    >
      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <Lock size={20} color="currentColor" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t('settingsGeneralTwoFactorTitle')}
          </CardTitle>
          <CardDescription>
            {t('settingsGeneralTwoFactorDesc')}
          </CardDescription>
        </div>
      </div>
      <Switch
        checked={currentGeneralSettings.enable2FAForUnlock}
        aria-label={t('settingsGeneralTwoFactorAria')}
        onclick={() => toggleSwitch('enable2FAForUnlock')}
      />
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      {#each authenticationMethods as method (method.key)}
        <div
          class="border-border/60 bg-muted/20 flex items-center justify-between gap-4 rounded-lg border px-4 py-3"
        >
          <div class="space-y-1">
            <p class="text-foreground text-sm font-medium">
              {t('settingsGeneralTotpTitle')}
            </p>
            <p class="text-muted-foreground text-sm">
              {t('settingsGeneralTotpDesc')}
            </p>
          </div>
          <Switch
            checked={currentGeneralSettings[method.key]}
            aria-label={t('settingsGeneralTotpAria')}
            onclick={() => toggleSwitch(method.key)}
          />
        </div>
      {/each}
      <p class="text-muted-foreground text-sm">
        {t('settingsGeneralTwoFactorNote')}
      </p>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader
      class="border-border/40 flex flex-col gap-3 border-b pb-4 sm:flex-row sm:items-start sm:justify-between"
    >
      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <Key size={20} aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t('settingsGeneralKeyboardTitle')}
          </CardTitle>
          <CardDescription>
            {t('settingsGeneralKeyboardDesc')}
          </CardDescription>
        </div>
      </div>
      <Button
        variant="outline"
        size="sm"
        class="mt-3 sm:mt-0"
        onclick={() => (showKeyboardShortcutsPopup = true)}
      >
        {t('settingsGeneralKeyboardCta')}
      </Button>
    </CardHeader>
    <CardContent class="pt-4">
      <p class="text-muted-foreground text-sm">
        {t('settingsGeneralKeyboardBody')}
      </p>
    </CardContent>
  </Card>
</div>

{#if showKeyboardShortcutsPopup}
  <KeyboardShortcutsPopup onclose={() => (showKeyboardShortcutsPopup = false)} />
{/if}
