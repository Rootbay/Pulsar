<script lang="ts">
  import { onDestroy, tick } from 'svelte';
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
    ShieldAlert,
    Timer,
    Trash2
  } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';

  let clipboardIntegration = false;
  let clearAfterDuration = 12;
  let blockHistory = false;
  let onlyUnlocked = false;
  let permissionLevel = 'ask';

  const unsubscribe = clipboardSettings.subscribe((value) => {
    clipboardIntegration = value.clipboardIntegration;
    clearAfterDuration = value.clearAfterDuration;
    blockHistory = value.blockHistory;
    onlyUnlocked = value.onlyUnlocked;
    permissionLevel = value.permissionLevel;
  });

  onDestroy(unsubscribe);

  let showAuditLog = false;

  $: timeoutLabel = clearAfterDuration === 12 ? '12 seconds (default)' : `${clearAfterDuration} seconds`;

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
      const message = error instanceof Error ? error.message : 'Failed to update clipboard timeout.';
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
      const patch: Partial<ClipboardSettings> = { [setting]: nextValue } as Partial<ClipboardSettings>;
      await updateClipboardSettings(patch);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update clipboard setting.';
      toast.error('Clipboard setting update failed.', { description: message });
    }
  };

  const handleRangeChange = async () => {
    try {
      await updateClipboardSettings({ clearAfterDuration });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update clipboard timeout.';
      clearAfterDuration = get(clipboardSettings).clearAfterDuration;
      toast.error('Unable to update clipboard timeout.', { description: message });
    }
  };

  const handleRadioChange = async () => {
    try {
      await updateClipboardSettings({ permissionLevel });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update clipboard permission.';
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

<div class="flex-1 min-h-0 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader>
      <CardTitle>Core Integration</CardTitle>
      <CardDescription>Configure basic clipboard functionality.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex items-start justify-between gap-4">
        <div class="flex items-start gap-3">
          <div class="rounded-lg bg-primary/10 p-2 text-primary">
            <ClipboardCheck class="size-5" />
          </div>
          <div class="space-y-1">
            <p class="text-sm font-medium text-foreground">Clipboard integration</p>
            <p class="text-sm text-muted-foreground">Enable automatic clipboard functionality.</p>
          </div>
        </div>
        <Switch
          checked={clipboardIntegration}
          aria-label="Toggle clipboard integration"
          disabled={!$clipboardIntegrationState.integrationAvailable || $clipboardIntegrationState.applying}
          on:click={() => handleSwitchChange('clipboardIntegration')}
        />
      </div>

      {#if !$clipboardIntegrationState.integrationAvailable}
        <p class="text-sm text-destructive">Clipboard integration is currently unavailable.</p>
      {/if}

      {#if $clipboardIntegrationState.lastError}
        <p class="text-sm text-destructive">{$clipboardIntegrationState.lastError}</p>
      {/if}

      <div class="space-y-4 rounded-lg border border-border/60 bg-muted/10 p-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2 text-sm font-medium text-foreground">
            <Timer class="size-4" />
            <span>Clear clipboard after</span>
          </div>
          <span class="text-sm text-muted-foreground">{timeoutLabel}</span>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          {#each [5, 12, 30, 60] as option}
            <Button
              type="button"
              variant="outline"
              class={cn(
                'h-9 px-3 text-sm',
                clearAfterDuration === option && 'border-primary bg-primary/20 text-primary'
              )}
              disabled={!$clipboardIntegrationState.integrationAvailable || $clipboardIntegrationState.applying}
              on:click={() => handleSetTimeout(option)}
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
            on:change={handleRangeChange}
            disabled={!$clipboardIntegrationState.integrationAvailable || $clipboardIntegrationState.applying}
            class="h-2 w-full cursor-pointer appearance-none rounded-full bg-muted accent-[hsl(var(--primary))]"
          />
          <div class="flex justify-between text-xs text-muted-foreground">
            <span>3s</span>
            <span>30s</span>
            <span>60s</span>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader>
      <CardTitle>Access control</CardTitle>
      <CardDescription>Fine-tune clipboard safety and permissions.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex items-start justify-between gap-4">
        <div class="flex items-start gap-3">
          <div class="rounded-lg bg-secondary/20 p-2 text-secondary-foreground">
            <ClipboardList class="size-5" />
          </div>
          <div class="space-y-1">
            <div class="flex items-center gap-2">
              <p class="text-sm font-medium text-foreground">Block clipboard history</p>
              <Badge
                variant={$clipboardIntegrationState.historyBlockingActive ? 'secondary' : 'outline'}
                class="text-xs"
              >
                {$clipboardIntegrationState.historyBlockingActive ? 'Active' : 'Inactive'}
              </Badge>
            </div>
            <p class="text-sm text-muted-foreground">Prevent system clipboard history from storing entries.</p>
          </div>
        </div>
        <Switch
          checked={blockHistory}
          aria-label="Toggle block clipboard history"
          disabled={
            !$clipboardIntegrationState.integrationAvailable ||
            !$clipboardIntegrationState.historyBlockingSupported ||
            $clipboardIntegrationState.applying
          }
          on:click={() => handleSwitchChange('blockHistory')}
        />
      </div>

      {#if !$clipboardIntegrationState.historyBlockingSupported}
        <p class="pl-11 text-xs text-muted-foreground">
          Clipboard history blocking is not supported on this platform.
        </p>
      {/if}

      <div class="flex items-start justify-between gap-4">
        <div class="flex items-start gap-3">
          <div class="rounded-lg bg-chart-warning-soft p-2 text-chart-warning">
            <ShieldAlert class="size-5" />
          </div>
          <div class="space-y-1">
            <p class="text-sm font-medium text-foreground">Only allow on unlocked session</p>
            <p class="text-sm text-muted-foreground">Disable clipboard export when Pulsar is locked.</p>
          </div>
        </div>
        <Switch
          checked={onlyUnlocked}
          aria-label="Toggle only allow on unlocked session"
          disabled={!$clipboardIntegrationState.integrationAvailable || $clipboardIntegrationState.applying}
          on:click={() => handleSwitchChange('onlyUnlocked')}
        />
      </div>

      <div class="space-y-4 rounded-lg border border-border/60 bg-muted/10 p-4">
        <div class="flex items-start gap-3">
          <div class="rounded-lg bg-primary/10 p-2 text-primary">
            <Shield class="size-5" />
          </div>
          <div class="space-y-1">
            <p class="text-sm font-medium text-foreground">Per-item clipboard permissions</p>
            <p class="text-sm text-muted-foreground">Choose how Pulsar prompts when copying credentials.</p>
          </div>
        </div>

        <div class="space-y-3 pl-11">
          <label class="flex items-center gap-3 text-sm text-foreground">
            <input
              type="radio"
              class="size-4 cursor-pointer accent-[hsl(var(--primary))]"
              name="permission"
              value="ask"
              bind:group={permissionLevel}
              on:change={handleRadioChange}
            />
            <span>Allow once (ask each time)</span>
          </label>
          <label class="flex items-center gap-3 text-sm text-foreground">
            <input
              type="radio"
              class="size-4 cursor-pointer accent-[hsl(var(--primary))]"
              name="permission"
              value="remember"
              bind:group={permissionLevel}
              on:change={handleRadioChange}
            />
            <span>Always allow (remember choice)</span>
          </label>
        </div>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader>
      <CardTitle>Actions & monitoring</CardTitle>
      <CardDescription>Manual controls and local activity tracking.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="flex flex-wrap items-center gap-3">
        <Button
          type="button"
          variant="destructive"
          class="flex items-center gap-2"
          disabled={!$clipboardIntegrationState.integrationAvailable || $clipboardIntegrationState.applying}
          on:click={handleClearClipboard}
        >
          <Trash2 class="size-4" />
          Clear clipboard now
        </Button>
        <Button type="button" variant="outline" class="flex items-center gap-2" on:click={toggleAuditLog}>
          <History class="size-4" />
          {showAuditLog ? 'Hide audit log' : 'View audit log'}
        </Button>
      </div>

      {#if showAuditLog}
        <div id="auditLogSection" class="space-y-4" transition:slide>
          <Separator class="bg-border/60" />
          <div>
            <h4 class="text-sm font-semibold text-foreground">Recent clipboard activity</h4>
            <p class="text-sm text-muted-foreground">Local activity log (not synced)</p>
          </div>
          <div class="max-h-64 space-y-2 overflow-y-auto pr-1">
            {#each auditLogEntries as entry (entry.id)}
              <div
                class="flex items-center justify-between gap-3 rounded-lg border border-border/60 bg-muted/10 px-3 py-2"
                in:fade={{ delay: entry.id * 50 }}
              >
                <div class="flex items-center gap-3">
                  <CircleCheck class="size-4 text-chart-success" />
                  <div class="text-sm text-foreground">
                    <p class="font-medium">{entry.action}</p>
                    <p class="text-xs text-muted-foreground">{entry.time}</p>
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

