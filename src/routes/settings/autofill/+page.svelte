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
</script>

<div class="flex-1 min-h-0 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader>
      <CardTitle>Browser Auto-fill</CardTitle>
      <CardDescription>Configure browser auto-fill settings.</CardDescription>
    </CardHeader>
    <CardContent>
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-sm font-medium text-foreground">Enable Auto-fill for Browsers</p>
          <p class="text-sm text-muted-foreground">Automatically fill login forms in web browsers.</p>
        </div>
        <Switch
          checked={browserAutofill}
          aria-label="Enable Auto-fill for Browsers"
          onclick={() => toggleSetting('browserAutofill')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader>
      <CardTitle>Global Auto-type</CardTitle>
      <CardDescription>Configure global auto-type settings.</CardDescription>
    </CardHeader>
    <CardContent>
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-sm font-medium text-foreground">Enable Global Auto-type</p>
          <p class="text-sm text-muted-foreground">Type passwords automatically using keyboard shortcuts.</p>
        </div>
        <Switch
          checked={globalAutotype}
          aria-label="Enable Global Auto-type"
          onclick={() => toggleSetting('globalAutotype')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader>
      <CardTitle>Safety Checks</CardTitle>
      <CardDescription>Configure safety checks for autofill.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-sm font-medium text-foreground">Require OS-level Unlock for Auto-fill</p>
          <p class="text-sm text-muted-foreground">Require system authentication before auto-filling.</p>
        </div>
        <Switch
          checked={osUnlock}
          aria-label="Require OS-level Unlock for Auto-fill"
          onclick={() => toggleSetting('osUnlock')}
        />
      </div>
      <div class="flex items-center justify-between gap-4">
        <div class="space-y-1">
          <p class="text-sm font-medium text-foreground">Require Per-site Confirmation</p>
          <p class="text-sm text-muted-foreground">Ask for confirmation before auto-filling on each site.</p>
        </div>
        <Switch
          checked={perSiteConfirmation}
          aria-label="Require Per-site Confirmation"
          onclick={() => toggleSetting('perSiteConfirmation')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader>
      <CardTitle>Test Auto-type</CardTitle>
      <CardDescription>Test your auto-type configuration.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex flex-col gap-4 rounded-lg border border-border/60 bg-muted/20 p-4 shadow-sm sm:flex-row sm:items-center sm:justify-between">
        <Button type="button" class="w-full sm:w-auto">
          <Play class="mr-2 size-4" />
          Simulate Auto-type
        </Button>
        <div class="flex items-start gap-3 rounded-md border border-chart-warning-soft bg-chart-warning-soft px-3 py-2 text-sm text-chart-warning">
          <TriangleAlert class="mt-0.5 size-4 shrink-0" />
          <span>Make sure you have a text field selected before testing.</span>
        </div>
      </div>

      <div class="space-y-4 rounded-lg border border-border/60 bg-muted/10 p-4">
        <h3 class="text-sm font-semibold text-foreground">Test results</h3>
        <div class="space-y-3">
          {#each testResults as result (result.message)}
            <div class="flex items-center gap-3 text-sm text-foreground">
              {#if result.status === 'success'}
                <CircleCheck class="size-4 text-chart-success" />
              {:else}
                <CircleX class="size-4 text-destructive" />
              {/if}
              <span>{result.message}</span>
            </div>
          {/each}
        </div>
      </div>
    </CardContent>
  </Card>
</div>

