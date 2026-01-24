<script lang="ts">
  import { tick } from 'svelte';
  import { get } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
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
  import { Separator } from '$lib/components/ui/separator';
  import { clipboardSettings } from '$lib/stores/clipboard';
  import { cn } from '$lib/utils';
  import type { ClipboardSettings } from '$lib/config/settings';
  import { currentLocale } from '$lib/i18n';
  import {
    clearClipboardNow,
    clipboardIntegrationState,
    updateClipboardSettings
  } from '$lib/utils/clipboardService';
  import {
    CircleCheck,
    ClipboardCheck,
    ClipboardList,
    History,
    Shield,
    Timer,
    Trash2
  } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  let locale = $derived($currentLocale as 'en' | 'sv');

  let currentSettings = $state<ClipboardSettings>({} as ClipboardSettings);
  let clearAfterDuration = $state(12);
  let permissionLevel = $state<string>('ask');
  let clipboardIntegration = $state(false);
  let blockHistory = $state(false);
  let onlyUnlocked = $state(false);
  let showAuditLog = $state(false);

  $effect(() => {
    return clipboardSettings.subscribe((settings) => {
      currentSettings = settings;
      clearAfterDuration = settings.clearAfterDuration;
      permissionLevel = settings.permissionLevel;
      clipboardIntegration = settings.clipboardIntegration;
      blockHistory = settings.blockHistory;
      onlyUnlocked = settings.onlyUnlocked;
    });
  });

  const timeoutLabel = $derived(
    currentSettings.clearAfterDuration === 12
      ? '12 seconds (default)'
      : `${currentSettings.clearAfterDuration} seconds`
  );

  const auditLogEntries = [
    { id: 1, action: 'Password for gmail.com', time: '2 minutes ago', status: 'copied' },
    { id: 2, action: 'Username: john.doe@email.com', time: '15 minutes ago', status: 'copied' },
    { id: 3, action: 'Password for github.com', time: '1 hour ago', status: 'copied' },
    { id: 4, action: 'API Key: sk-proj-...', time: '3 hours ago', status: 'copied' }
  ] satisfies Array<{ id: number; action: string; time: string; status: string }>;

  const handleSetTimeout = async (seconds: number) => {
    clearAfterDuration = seconds;
    try {
      await updateClipboardSettings({ clearAfterDuration: seconds });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Failed to update clipboard timeout.';
      clearAfterDuration = get(clipboardSettings).clearAfterDuration;
      toast.error('Unable to update clipboard timeout.', { description: message });
    }
  };

  const handleSwitchChange = async (
    setting: 'clipboardIntegration' | 'blockHistory' | 'onlyUnlocked'
  ) => {
    const current = get(clipboardSettings);
    const nextValue = !current[setting];

    try {
      const patch: Partial<ClipboardSettings> = {
        [setting]: nextValue
      } as Partial<ClipboardSettings>;
      await updateClipboardSettings(patch);
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Failed to update clipboard setting.';
      toast.error('Clipboard setting update failed.', { description: message });
    }
  };

  const handleRangeChange = async () => {
    try {
      await updateClipboardSettings({ clearAfterDuration });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Failed to update clipboard timeout.';
      clearAfterDuration = get(clipboardSettings).clearAfterDuration;
      toast.error('Unable to update clipboard timeout.', { description: message });
    }
  };

  const handleRadioChange = async () => {
    try {
      await updateClipboardSettings({ permissionLevel });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Failed to update clipboard permission.';
      permissionLevel = get(clipboardSettings).permissionLevel;
      toast.error('Unable to update clipboard permission.', { description: message });
    }
  };

  const handleClearClipboard = async () => {
    try {
      await clearClipboardNow();
      toast.success('Clipboard cleared.');
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to clear clipboard.';
      toast.error('Unable to clear clipboard.', { description: message });
    }
  };

  const toggleAuditLog = async () => {
    showAuditLog = !showAuditLog;

    if (showAuditLog) {
      await tick();
      document
        .getElementById('auditLogSection')
        ?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ClipboardCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>{t(locale, 'Core Integration', 'Grundläggande integration')}</CardTitle>
        <CardDescription>
          {t(
            locale,
            'Configure basic clipboard functionality.',
            'Ställ in grundläggande urklippsfunktioner.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex items-start justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t(locale, 'Clipboard integration', 'Urklippsintegration')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Enable automatic clipboard functionality.',
              'Aktivera automatisk hantering av urklipp.'
            )}
          </p>
        </div>
        <Switch
          checked={clipboardIntegration}
          aria-label="Toggle clipboard integration"
          disabled={!$clipboardIntegrationState.integrationAvailable ||
            $clipboardIntegrationState.applying}
          onclick={() => handleSwitchChange('clipboardIntegration')}
        />
      </div>

      {#if !$clipboardIntegrationState.integrationAvailable}
        <p class="text-destructive text-sm">
          {t(
            locale,
            'Clipboard integration is currently unavailable.',
            'Urklippsintegration är för närvarande inte tillgänglig.'
          )}
        </p>
      {/if}

      {#if $clipboardIntegrationState.lastError}
        <p class="text-destructive text-sm">{$clipboardIntegrationState.lastError}</p>
      {/if}

      <div class="border-border/60 bg-muted/10 space-y-4 rounded-lg border p-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="text-foreground flex items-center gap-2 text-sm font-medium">
            <Timer class="size-4" />
            <span>Clear clipboard after</span>
          </div>
          <span class="text-muted-foreground text-sm">{timeoutLabel}</span>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          {#each [5, 12, 30, 60] as option (option)}
            <Button
              type="button"
              variant="outline"
              class={cn(
                'h-9 px-3 text-sm',
                clearAfterDuration === option && 'border-primary bg-primary/20 text-primary'
              )}
              disabled={!$clipboardIntegrationState.integrationAvailable ||
                $clipboardIntegrationState.applying}
              onclick={() => handleSetTimeout(option)}
            >
              {option}s
            </Button>
          {/each}
        </div>

        <div class="space-y-2">
          <input
            type="range"
            min="3"
            max="60"
            step="1"
            bind:value={clearAfterDuration}
            onchange={handleRangeChange}
            disabled={!$clipboardIntegrationState.integrationAvailable ||
              $clipboardIntegrationState.applying}
            class="bg-muted h-2 w-full cursor-pointer appearance-none rounded-full accent-[hsl(var(--primary))]"
          />
          <div class="text-muted-foreground flex justify-between text-xs">
            <span>3s</span>
            <span>30s</span>
            <span>60s</span>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <Shield class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>{t(locale, 'Access control', 'Åtkomstkontroll')}</CardTitle>
        <CardDescription>
          {t(
            locale,
            'Fine-tune clipboard safety and permissions.',
            'Finjustera säkerhet och behörigheter för urklipp.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex items-start justify-between gap-4">
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <p class="text-foreground text-sm font-medium">
              {t(locale, 'Block clipboard history', 'Blockera urklippshistorik')}
            </p>
            <Badge
              variant={$clipboardIntegrationState.historyBlockingActive ? 'secondary' : 'outline'}
              class="text-xs"
            >
              {$clipboardIntegrationState.historyBlockingActive ? 'Active' : 'Inactive'}
            </Badge>
          </div>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Prevent system clipboard history from storing entries.',
              'Förhindra att systemets urklippshistorik sparar poster.'
            )}
          </p>
        </div>
        <Switch
          checked={blockHistory}
          aria-label="Toggle block clipboard history"
          disabled={!$clipboardIntegrationState.integrationAvailable ||
            !$clipboardIntegrationState.historyBlockingSupported ||
            $clipboardIntegrationState.applying}
          onclick={() => handleSwitchChange('blockHistory')}
        />
      </div>

      {#if !$clipboardIntegrationState.historyBlockingSupported}
        <p class="text-muted-foreground pl-11 text-xs">
          {t(
            locale,
            'Clipboard history blocking is not supported on this platform.',
            'Blockering av urklippshistorik stöds inte på denna plattform.'
          )}
        </p>
      {/if}

      <div class="flex items-start justify-between gap-4">
        <div class="space-y-1">
          <p class="text-foreground text-sm font-medium">
            {t(locale, 'Only allow on unlocked session', 'Tillåt endast när Pulsar är upplåst')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Disable clipboard export when Pulsar is locked.',
              'Inaktivera urklippsexport när Pulsar är låst.'
            )}
          </p>
        </div>
        <Switch
          checked={onlyUnlocked}
          aria-label="Toggle only allow on unlocked session"
          disabled={!$clipboardIntegrationState.integrationAvailable ||
            $clipboardIntegrationState.applying}
          onclick={() => handleSwitchChange('onlyUnlocked')}
        />
      </div>

      <div class="border-border/60 bg-muted/10 space-y-4 rounded-lg border p-4">
        <div class="flex items-start gap-3">
          <div
            class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
          >
            <ClipboardList class="size-5" aria-hidden="true" />
          </div>
          <div class="space-y-1">
            <p class="text-foreground text-sm font-medium">
              {t(locale, 'Per-item clipboard permissions', 'Behörigheter per post för urklipp')}
            </p>
            <p class="text-muted-foreground text-sm">
              {t(
                locale,
                'Choose how Pulsar prompts when copying credentials.',
                'Välj hur Pulsar frågar när uppgifter kopieras.'
              )}
            </p>
          </div>
        </div>

        <div class="space-y-3 pl-11">
          <div class="flex flex-wrap gap-2">
            <Button
              type="button"
              size="sm"
              variant={permissionLevel === 'ask' ? 'default' : 'outline'}
              class="h-8 px-3 text-xs"
              aria-pressed={permissionLevel === 'ask'}
              onclick={async () => {
                permissionLevel = 'ask';
                await handleRadioChange();
              }}
            >
              {t(locale, 'Allow once (ask each time)', 'Tillåt en gång (fråga varje gång)')}
            </Button>
            <Button
              type="button"
              size="sm"
              variant={permissionLevel === 'remember' ? 'default' : 'outline'}
              class="h-8 px-3 text-xs"
              aria-pressed={permissionLevel === 'remember'}
              onclick={async () => {
                permissionLevel = 'remember';
                await handleRadioChange();
              }}
            >
              {t(locale, 'Always allow (remember choice)', 'Tillåt alltid (kom ihåg valet)')}
            </Button>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader>
      <CardTitle>
        {t(locale, 'Actions & monitoring', 'Åtgärder & övervakning')}
      </CardTitle>
      <CardDescription>
        {t(
          locale,
          'Manual controls and local activity tracking.',
          'Manuella kontroller och lokal aktivitetslogg.'
        )}
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex flex-wrap items-center gap-3">
        <Button
          type="button"
          variant="destructive"
          class="flex items-center gap-2"
          disabled={!$clipboardIntegrationState.integrationAvailable ||
            $clipboardIntegrationState.applying}
          onclick={handleClearClipboard}
        >
          <Trash2 class="size-4" />
          {t(locale, 'Clear clipboard now', 'Töm urklipp nu')}
        </Button>
        <Button
          type="button"
          variant="outline"
          class="flex items-center gap-2"
          onclick={toggleAuditLog}
        >
          <History class="size-4" />
          {showAuditLog
            ? t(locale, 'Hide audit log', 'Dölj aktivitetslogg')
            : t(locale, 'View audit log', 'Visa aktivitetslogg')}
        </Button>
      </div>

      {#if showAuditLog}
        <div id="auditLogSection" class="space-y-4" transition:slide>
          <Separator class="bg-border/60" />
          <div>
            <h4 class="text-foreground text-sm font-semibold">
              {t(locale, 'Recent clipboard activity', 'Senaste urklippsaktivitet')}
            </h4>
            <p class="text-muted-foreground text-sm">
              {t(locale, 'Local activity log (not synced)', 'Lokal aktivitetslogg (inte synkad)')}
            </p>
          </div>
          <div class="max-h-64 space-y-2 overflow-y-auto pr-1">
            {#each auditLogEntries as entry (entry.id)}
              <div
                class="border-border/60 bg-muted/10 flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
                in:fade={{ delay: entry.id * 50 }}
              >
                <div class="flex items-center gap-3">
                  <CircleCheck class="text-chart-success size-4" />
                  <div class="text-foreground text-sm">
                    <p class="font-medium">{entry.action}</p>
                    <p class="text-muted-foreground text-xs">{entry.time}</p>
                  </div>
                </div>
                <Badge variant="secondary" class="text-xs capitalize">{entry.status}</Badge>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </CardContent>
  </Card>
</div>
