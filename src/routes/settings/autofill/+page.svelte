<script lang="ts">
  import { settings } from '$lib/stores/appSettings.svelte';
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
  import { i18n, t as translate, type Locale } from '$lib/i18n.svelte';
  import { toast } from '$lib/components/ui/sonner';

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as any, vars);

  let currentSettings = $derived(settings.state.autofill);
  let browserAutofill = $derived(currentSettings.browserAutofill);
  let globalAutotype = $derived(currentSettings.globalAutotype);
  let osUnlock = $derived(currentSettings.osUnlock);
  let perSiteConfirmation = $derived(currentSettings.perSiteConfirmation);

  const toggleSetting = (key: keyof AutofillSettings) => {
    settings.state.autofill[key] = !settings.state.autofill[key];
    settings.save();
  };

  let testResults = $state<Array<{ message: string; status: 'success' | 'failure' }>>([]);

  async function handleSimulateAutoType() {
    toast.info(t('Auto-type simulation started.'), {
      description: t('Please focus a text field. Simulation will start in 2 seconds.')
    });
    
    testResults = [];
    
    try {
      await callBackend('simulate_autotype');
      toast.success(t('Auto-type simulation completed.'));
      testResults = [
        { message: 'Keystroke simulation: Success', status: 'success' }
      ];
    } catch (error) {
      toast.error(t('Auto-type simulation failed.'));
      testResults = [
        { message: 'Keystroke simulation: Failed', status: 'failure' }
      ];
    }
  }

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
      <CardTitle>{t('Browser Auto-fill')}</CardTitle>
      <CardDescription>
        {t('Configure browser auto-fill settings.')}
      </CardDescription>
    </CardHeader>
    <CardContent>
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t('Enable Auto-fill for Browsers')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Automatically fill login forms in web browsers.')}
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
      <CardTitle>{t('Global Auto-type')}</CardTitle>
      <CardDescription>
        {t('Configure global auto-type settings.')}
      </CardDescription>
    </CardHeader>
    <CardContent>
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t('Enable Global Auto-type')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Type passwords automatically using keyboard shortcuts.')}
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
      <CardTitle>{t('Safety Checks')}</CardTitle>
      <CardDescription>
        {t('Configure safety checks for autofill.')}
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t('Require OS-level Unlock for Auto-fill')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Require system authentication before auto-filling.')}
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
            {t('Require Per-site Confirmation')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Ask for confirmation before auto-filling on each site.')}
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
      <CardTitle>{t('Test Auto-type')}</CardTitle>
      <CardDescription>
        {t('Test your auto-type configuration.')}
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div
        class="border-border/60 bg-muted/20 flex flex-col gap-4 rounded-lg border p-4 shadow-sm sm:flex-row sm:items-center sm:justify-between"
      >
        <Button type="button" class="w-full sm:w-auto" onclick={handleSimulateAutoType}>
          <Play class="mr-2 size-4" />
          {t('Simulate Auto-type')}
        </Button>
        <div
          class="border-chart-warning-soft bg-chart-warning-soft text-chart-warning flex items-start gap-3 rounded-md border px-3 py-2 text-sm"
        >
          <TriangleAlert class="mt-0.5 size-4 shrink-0" />
          <span>
            {t('Make sure you have a text field selected before testing.')}
          </span>
        </div>
      </div>

      <div class="border-border/60 bg-muted/10 space-y-4 rounded-lg border p-4">
        <h3 class="text-foreground text-sm font-semibold">
          {t('Test results')}
        </h3>
        <div class="space-y-3">
          {#if testResults.length === 0}
            <p class="text-muted-foreground py-2 text-sm italic">
              {t('No tests run yet.')}
            </p>
          {:else}
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
          {/if}
        </div>
      </div>
    </CardContent>
  </Card>
</div>
