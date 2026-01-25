<script lang="ts">
  import { onDestroy } from 'svelte';
  import { autofillSettings } from '$lib/stores/autofill';
  import type { AutofillSettings } from '$lib/config/settings';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Switch } from '$lib/components/ui/switch';
  import { TriangleAlert, CircleCheck, Play, CircleX } from '@lucide/svelte';
  import { currentLocale, t, type Locale } from '$lib/i18n';

  const locale = $derived($currentLocale);

  let browserAutofill = false;
  let globalAutotype = false;
  let osUnlock = false;
  let perSiteConfirmation = false;

  const unsubscribe = autofillSettings.subscribe((value) => {
    browserAutofill = value.browserAutofill;
    globalAutotype = value.globalAutotype;
    osUnlock = value.osUnlock;
    perSiteConfirmation = value.perSiteConfirmation;
  });

  onDestroy(unsubscribe);

  const toggleSetting = (key: keyof AutofillSettings) => {
    autofillSettings.update((current) => ({
      ...current,
      [key]: !current[key]
    }));
  };

  const testResults = [
    { message: 'Hotkey registration: Success', status: 'success' },
    { message: 'Keystroke simulation: Success', status: 'success' },
    { message: 'Browser focus detection: Failed', status: 'failure' }
  ] satisfies Array<{ message: string; status: 'success' | 'failure' }>;

  function translateTestMessage(message: string, locale: Locale): string {
    if (locale === 'sv') {
      if (message === 'Hotkey registration: Success')
        return 'Registrering av snabbtangent: Lyckades';
      if (message === 'Keystroke simulation: Success')
        return 'Simulering av tangentnedtryckningar: Lyckades';
      if (message === 'Browser focus detection: Failed')
        return 'Upptäckt av webbläsarfokus: Misslyckades';
    }
    return message;
  }
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader>
      <CardTitle>{t(locale, 'Browser Auto-fill')}</CardTitle>
      <CardDescription>
        {t(
          locale,
          'Configure browser auto-fill settings.',
          'Konfigurera inställningar för autofyll i webbläsare.'
        )}
      </CardDescription>
    </CardHeader>
    <CardContent>
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t(locale, 'Enable Auto-fill for Browsers')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Automatically fill login forms in web browsers.',
              'Fyll i inloggningsformulär automatiskt i webbläsare.'
            )}
          </p>
        </div>
        <Switch
          checked={browserAutofill}
          aria-label="Enable Auto-fill for Browsers"
          onclick={() => toggleSetting('browserAutofill')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader>
      <CardTitle>{t(locale, 'Global Auto-type')}</CardTitle>
      <CardDescription>
        {t(
          locale,
          'Configure global auto-type settings.',
          'Konfigurera globala inställningar för autotypning.'
        )}
      </CardDescription>
    </CardHeader>
    <CardContent>
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t(locale, 'Enable Global Auto-type')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Type passwords automatically using keyboard shortcuts.',
              'Skriv lösenord automatiskt med tangentbordsgenvägar.'
            )}
          </p>
        </div>
        <Switch
          checked={globalAutotype}
          aria-label="Enable Global Auto-type"
          onclick={() => toggleSetting('globalAutotype')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader>
      <CardTitle>{t(locale, 'Safety Checks')}</CardTitle>
      <CardDescription>
        {t(
          locale,
          'Configure safety checks for autofill.',
          'Konfigurera säkerhetskontroller för autofyll.'
        )}
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t(locale, 'Require OS-level Unlock for Auto-fill')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Require system authentication before auto-filling.',
              'Kräv systemautentisering innan autofyll används.'
            )}
          </p>
        </div>
        <Switch
          checked={osUnlock}
          aria-label="Require OS-level Unlock for Auto-fill"
          onclick={() => toggleSetting('osUnlock')}
        />
      </div>
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t(locale, 'Require Per-site Confirmation')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Ask for confirmation before auto-filling on each site.',
              'Fråga om bekräftelse innan autofyll på varje webbplats.'
            )}
          </p>
        </div>
        <Switch
          checked={perSiteConfirmation}
          aria-label="Require Per-site Confirmation"
          onclick={() => toggleSetting('perSiteConfirmation')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader>
      <CardTitle>{t(locale, 'Test Auto-type')}</CardTitle>
      <CardDescription>
        {t(
          locale,
          'Test your auto-type configuration.',
          'Testa din konfiguration för autotypning.'
        )}
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div
        class="border-border/60 bg-muted/20 flex flex-col gap-4 rounded-lg border p-4 shadow-sm sm:flex-row sm:items-center sm:justify-between"
      >
        <Button type="button" class="w-full sm:w-auto">
          <Play class="mr-2 size-4" />
          {t(locale, 'Simulate Auto-type')}
        </Button>
        <div
          class="border-chart-warning-soft bg-chart-warning-soft text-chart-warning flex items-start gap-3 rounded-md border px-3 py-2 text-sm"
        >
          <TriangleAlert class="mt-0.5 size-4 shrink-0" />
          <span>
            {t(
              locale,
              'Make sure you have a text field selected before testing.',
              'Se till att ett textfält är markerat innan du testar.'
            )}
          </span>
        </div>
      </div>

      <div class="border-border/60 bg-muted/10 space-y-4 rounded-lg border p-4">
        <h3 class="text-foreground text-sm font-semibold">
          {t(locale, 'Test results')}
        </h3>
        <div class="space-y-3">
          {#each testResults as result (result.message)}
            <div class="text-foreground flex items-center gap-3 text-sm">
              {#if result.status === 'success'}
                <CircleCheck class="text-chart-success size-4" />
              {:else}
                <CircleX class="text-destructive size-4" />
              {/if}
              <span>{translateTestMessage(result.message, locale)}</span>
            </div>
          {/each}
        </div>
      </div>
    </CardContent>
  </Card>
</div>
