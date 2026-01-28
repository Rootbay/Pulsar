<script lang="ts">
  import { onMount } from 'svelte';
  import { callBackend } from '$lib/utils/backend';
  import type { ActivityEntry } from '$lib/types/password';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Spinner } from '$lib/components/ui/spinner';
  import {
    Clock,
    Shield,
    Trash2,
    RefreshCw,
    ExternalLink,
    KeyRound,
    FileUp,
    FileDown,
    Plus,
    Pencil,
    ClipboardCopy
  } from '@lucide/svelte';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { appState } from '$lib/stores';
  import { toast } from '$lib/components/ui/sonner';

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as I18nKey, vars);

  let activities = $state<ActivityEntry[]>([]);
  let isLoading = $state(true);

  async function loadActivities() {
    isLoading = true;
    try {
      activities = await callBackend<ActivityEntry[]>('get_activity_log', { limit: 50 });
    } catch (error) {
      console.error('Failed to load activity log:', error);
      toast.error('Failed to load activity log');
    } finally {
      isLoading = false;
    }
  }

  async function clearLog() {
    if (!confirm('Are you sure you want to clear the activity log? This cannot be undone.')) {
      return;
    }
    try {
      await callBackend('clear_activity_log');
      activities = [];
      toast.success('Activity log cleared');
    } catch (_error) {
      toast.error('Failed to clear activity log');
    }
  }

  function getEventIcon(type: string) {
    switch (type) {
      case 'master_password_rotated':
      case 'argon2_params_updated':
        return KeyRound;
      case 'vault_exported':
        return FileDown;
      case 'vault_restored':
        return FileUp;
      case 'item_created':
        return Plus;
      case 'item_updated':
        return Pencil;
      case 'item_deleted':
        return Trash2;
      case 'clipboard_copy':
        return ClipboardCopy;
      default:
        return Shield;
    }
  }

  function getEventColor(type: string) {
    switch (type) {
      case 'master_password_rotated':
      case 'argon2_params_updated':
      case 'vault_restored':
        return 'text-primary';
      case 'item_deleted':
        return 'text-destructive';
      case 'item_created':
        return 'text-emerald-500';
      default:
        return 'text-muted-foreground';
    }
  }

  function formatEventName(type: string) {
    return type
      .split('_')
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  }

  function navigateToItem(id: number | null) {
    if (id === null) return;
    appState.requestedItemId = id;
    appState.showSettingsPopup = false;
  }

  onMount(loadActivities);
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <Clock class="h-5 w-5" aria-hidden="true" />
      </div>
      <div class="flex w-full items-center justify-between">
        <div>
          <CardTitle>{t('Activity Log')}</CardTitle>
          <CardDescription>
            {t('A record of important security events and vault changes.')}
          </CardDescription>
        </div>
        <div class="flex gap-2">
          <Button variant="ghost" size="sm" onclick={loadActivities} disabled={isLoading}>
            <RefreshCw class={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onclick={clearLog}
            disabled={isLoading || activities.length === 0}
          >
            <Trash2 class="text-destructive h-4 w-4" />
          </Button>
        </div>
      </div>
    </CardHeader>
    <CardContent class="pt-6">
      {#if isLoading}
        <div class="flex items-center justify-center py-12">
          <Spinner class="text-primary/40 h-8 w-8" />
        </div>
      {:else if activities.length === 0}
        <div class="flex flex-col items-center justify-center py-12 text-center">
          <Clock class="text-muted-foreground/30 mb-4 h-12 w-12" />
          <p class="text-muted-foreground text-sm">{t('No activity recorded yet.')}</p>
        </div>
      {:else}
        <div class="relative space-y-4">
          <div class="bg-border/40 absolute top-0 bottom-0 left-4.75 w-px"></div>

          {#each activities as activity (activity.id)}
            {@const Icon = getEventIcon(activity.eventType)}
            <div class="relative flex gap-4 pl-10">
              <div
                class="bg-background border-border/60 absolute left-0 flex h-10 w-10 items-center justify-center rounded-full border shadow-sm"
              >
                <Icon class={`h-4 w-4 ${getEventColor(activity.eventType)}`} />
              </div>

              <div class="flex flex-1 flex-col gap-1 pb-6">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-semibold">{formatEventName(activity.eventType)}</h4>
                  <span class="text-muted-foreground text-[10px]">
                    {new Date(activity.createdAt).toLocaleString()}
                  </span>
                </div>

                {#if activity.itemTitle}
                  <div class="flex items-center gap-2">
                    <span class="text-foreground text-sm font-medium">{activity.itemTitle}</span>
                    {#if activity.itemId}
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
                        onclick={() => navigateToItem(activity.itemId)}
                      >
                        <ExternalLink class="h-3 w-3" />
                      </Button>
                    {/if}
                  </div>
                {/if}

                {#if activity.details}
                  <p class="text-muted-foreground text-xs">{activity.details}</p>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </CardContent>
  </Card>
</div>
