<script lang="ts">
  import { generalSettings } from '$lib/stores/general';
  import type { GeneralSettings } from '$lib/config/settings';
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
  import { currentLocale, t } from '$lib/i18n';

  type BooleanSettingKey = {
    [K in keyof GeneralSettings]: GeneralSettings[K] extends boolean ? K : never;
  }[keyof GeneralSettings];

  type SelectSettingKey = Exclude<keyof GeneralSettings, BooleanSettingKey>;

  let currentGeneralSettings = $state<GeneralSettings>({} as GeneralSettings);
  const locale = $derived($currentLocale);

  $effect(() => {
    return generalSettings.subscribe((value) => {
      currentGeneralSettings = value;
    });
  });

  let showKeyboardShortcutsPopup = $state(false);

  const selectOptions: Record<SelectSettingKey, { value: string; label: string }[]> = {
    appLanguage: [
      { value: 'en', label: 'English' },
      { value: 'es', label: 'Español' },
      { value: 'fr', label: 'Français' },
      { value: 'de', label: 'Deutsch' },
      { value: 'pt-BR', label: 'Português (Brasil)' },
      { value: 'zh', label: '简体中文' },
      { value: 'sv', label: 'Svenska' }
    ],
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

  const toggleSettings: Array<{
    key: BooleanSettingKey;
    title: string;
    description: string;
    ariaLabel: string;
  }> = [
    {
      key: 'startOnSystemBoot',
      title: 'Start on System Boot',
      description: 'Launch automatically when your computer starts.',
      ariaLabel: 'Toggle start on system boot'
    },
    {
      key: 'showInSystemTray',
      title: 'Show in System Tray',
      description: 'Keep the app accessible from the system tray.',
      ariaLabel: 'Toggle show in system tray'
    }
  ];

  const authenticationMethods: Array<{
    key: BooleanSettingKey;
    title: string;
    description: string;
    ariaLabel: string;
    icon: typeof Smartphone;
  }> = [
    {
      key: 'totpEnabled',
      title: 'TOTP (Time-based)',
      description: 'Built-in authenticator support.',
      ariaLabel: 'Toggle TOTP (Time-based)',
      icon: Smartphone
    }
  ];

  function updateSetting<K extends keyof GeneralSettings>(setting: K, value: GeneralSettings[K]) {
    generalSettings.set({ ...currentGeneralSettings, [setting]: value });
  }

  function toggleSwitch(setting: BooleanSettingKey) {
    updateSetting(setting, !currentGeneralSettings[setting]);
  }

  function getOptionLabel(setting: SelectSettingKey) {
    const option = selectOptions[setting].find(
      (item) => item.value === currentGeneralSettings[setting]
    );
    return option?.label ?? 'Select an option';
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
          <CardTitle>{t(locale, 'General Settings')}</CardTitle>
          <CardDescription>
            {t(locale, 'Manage default language, startup behaviour, and layout.')}
          </CardDescription>
        </div>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-8 pt-4">
      <div class="grid gap-6 md:grid-cols-2">
        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium">
            {t(locale, 'App Language')}
          </Label>
          {#each [locale] as l (l)}
            <Select
              type="single"
              value={currentGeneralSettings.appLanguage}
              onValueChange={(value) => updateSetting('appLanguage', value)}
            >
              <SelectTrigger aria-label="Select app language" class="w-full">
                <span data-slot="select-value" class="truncate">
                  {getOptionLabel('appLanguage')}
                </span>
              </SelectTrigger>
              <SelectContent>
                {#each selectOptions.appLanguage as option (option.value)}
                  <SelectItem value={option.value}>{option.label}</SelectItem>
                {/each}
              </SelectContent>
            </Select>
          {/each}
        </div>

        <div class="space-y-2">
          <Label class="text-foreground text-sm font-medium">
            {t(locale, 'Default Vault on Startup')}
          </Label>
          <Select
            type="single"
            value={currentGeneralSettings.defaultVaultOnStartup}
            onValueChange={(value) => updateSetting('defaultVaultOnStartup', value)}
          >
            <SelectTrigger aria-label="Select default vault on startup" class="w-full">
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
                {locale === 'sv' && toggleSetting.key === 'startOnSystemBoot'
                  ? 'Starta med systemet'
                  : locale === 'sv' && toggleSetting.key === 'showInSystemTray'
                    ? 'Visa i systemfältet'
                    : toggleSetting.title}
              </p>
              <p class="text-muted-foreground text-sm">
                {locale === 'sv' && toggleSetting.key === 'startOnSystemBoot'
                  ? 'Öppna Pulsar automatiskt när datorn startar.'
                  : locale === 'sv' && toggleSetting.key === 'showInSystemTray'
                    ? 'Håll appen tillgänglig från systemfältet.'
                    : toggleSetting.description}
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
          {t(locale, 'Default View on Open')}
        </Label>
        <Select
          type="single"
          value={currentGeneralSettings.defaultViewOnOpen}
          onValueChange={(value) => updateSetting('defaultViewOnOpen', value)}
        >
          <SelectTrigger aria-label="Select default view on open" class="w-full md:w-80">
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
            {t(locale, 'Two-Factor Authentication')}
          </CardTitle>
          <CardDescription>
            {t(locale, 'Add extra layers of protection to vault access.')}
          </CardDescription>
        </div>
      </div>
      <Switch
        checked={currentGeneralSettings.enable2FAForUnlock}
        aria-label="Toggle enable 2FA for unlock"
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
              {t(locale, 'TOTP (Time-based)')}
            </p>
            <p class="text-muted-foreground text-sm">
              {t(locale, 'Built-in authenticator support.')}
            </p>
          </div>
          <Switch
            checked={currentGeneralSettings[method.key]}
            aria-label={method.ariaLabel}
            onclick={() => toggleSwitch(method.key)}
          />
        </div>
      {/each}
      <p class="text-muted-foreground text-sm">
        Configure per-vault overrides from the vaults settings page to enforce different
        requirements.
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
            {t(locale, 'Keyboard Shortcuts')}
          </CardTitle>
          <CardDescription>
            {t(locale, 'Customize shortcuts for frequently used actions.')}
          </CardDescription>
        </div>
      </div>
      <Button
        variant="outline"
        size="sm"
        class="mt-3 sm:mt-0"
        onclick={() => (showKeyboardShortcutsPopup = true)}
      >
        {t(locale, 'Configure Shortcuts')}
      </Button>
    </CardHeader>
    <CardContent class="pt-4">
      <p class="text-muted-foreground text-sm">
        {t(locale, 'Personalize key combinations to match your workflow and speed up navigation.')}
      </p>
    </CardContent>
  </Card>
</div>

{#if showKeyboardShortcutsPopup}
  <KeyboardShortcutsPopup onclose={() => (showKeyboardShortcutsPopup = false)} />
{/if}
