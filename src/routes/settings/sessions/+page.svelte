<script lang="ts">
  import { onMount } from 'svelte';
  import { callBackend } from '$lib/utils/backend';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Spinner } from '$lib/components/ui/spinner/index.js';
  import { MonitorSmartphone, ShieldCheck, Smartphone, FingerprintPattern, Trash2 } from '@lucide/svelte';
  import { i18n, t as translate } from '$lib/i18n.svelte';
  import { cn } from '$lib/utils';
  import { toast } from '$lib/components/ui/sonner';

  interface DeviceRecord {
    id: string;
    name: string;
    kind: string;
    lastSeen: string | null;
    isCurrent: boolean;
  }

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as any, vars);

  let devices = $state<DeviceRecord[]>([]);
  let devicesLoading = $state(false);
  let deviceActionPending = $state<Record<string, boolean>>({});
  let isRevokingDevices = $state(false);

  function getDeviceIcon(kind: string): typeof ShieldCheck {
    switch (kind) {
      case 'biometric':
        return FingerprintPattern;
      case 'device-key':
      case 'key':
        return Smartphone;
      default:
        return ShieldCheck;
    }
  }

  function getDeviceTypeLabel(kind: string): string {
    if (!kind) return t('Unknown');
    return kind
      .split(/[-_ ]+/)
      .filter(Boolean)
      .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
      .join(' ');
  }

  async function loadDevices() {
    devicesLoading = true;
    try {
      const result = await callBackend<DeviceRecord[]>('list_devices');
      devices = result.map((device) => ({
        ...device,
        kind: device.kind ?? 'unknown',
        lastSeen: device.lastSeen ?? null
      }));
      deviceActionPending = {};
    } catch (error) {
      toast.error(t('Failed to load devices'));
    } finally {
      devicesLoading = false;
    }
  }

  async function removeDevice(device: DeviceRecord) {
    deviceActionPending = { ...deviceActionPending, [device.id]: true };
    try {
      await callBackend('remove_device', { deviceId: device.id });
      devices = devices.filter((entry) => entry.id !== device.id);
      const updated = { ...deviceActionPending };
      delete updated[device.id];
      deviceActionPending = updated;
      toast.success(t('Removed {name}', { name: device.name }));
    } catch (error) {
      deviceActionPending = { ...deviceActionPending, [device.id]: false };
      toast.error(t('Failed to remove device'));
    }
  }

  async function revokeAllDevices() {
    if (!confirm(t('Are you sure you want to revoke all other devices?'))) return;
    isRevokingDevices = true;
    try {
      await callBackend('revoke_all_devices');
      devices = devices.filter(d => d.isCurrent);
      deviceActionPending = {};
      toast.success(t('All other devices revoked'));
    } catch (error) {
      toast.error(t('Failed to revoke devices'));
    } finally {
      isRevokingDevices = false;
    }
  }

  function handlePairDevice() {
    toast.info(t('Device pairing is not yet available. Stay tuned!'));
  }

  onMount(() => {
    loadDevices();
  });
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <MonitorSmartphone class="h-5 w-5" aria-hidden="true" />
      </div>
      <div class="flex w-full flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <CardTitle>
            {t('Paired Devices & Sessions')}
          </CardTitle>
          <CardDescription>
            {t('Review devices authorised for biometric or key-based unlock.')}
          </CardDescription>
        </div>
        <Button variant="outline" size="sm" onclick={handlePairDevice} disabled={devicesLoading}>
          {t('Pair New Device')}
        </Button>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      {#if devicesLoading}
        <div class="text-muted-foreground flex items-center gap-2 text-sm">
          <Spinner class="h-4 w-4" aria-hidden="true" />
          <span>{t('Loading devices…')}</span>
        </div>
      {:else if devices.length === 0}
        <p class="text-muted-foreground text-sm">
          {t('No devices have been paired yet.')}
        </p>
      {:else}
        {#each devices as device (device.id)}
          {@const DeviceIcon = getDeviceIcon(device.kind)}
          <div
            class={cn(
              'border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-4 sm:items-center',
              device.isCurrent ? 'border-primary/60 bg-primary/10' : ''
            )}
          >
            <div class="flex items-start gap-3">
              <div
                class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
              >
                <DeviceIcon class="h-5 w-5" aria-hidden="true" />
              </div>
              <div>
                <p class="text-foreground text-sm font-semibold">{device.name}</p>
                <p class="text-muted-foreground text-xs">
                  {device.lastSeen ?? (locale === 'sv' ? 'Ingen senaste aktivitet' : 'No recent activity')}
                  {device.isCurrent ? (locale === 'sv' ? ' • Aktuell enhet' : ' • Current device') : ''}
                </p>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <Badge variant="secondary">{getDeviceTypeLabel(device.kind)}</Badge>
              {#if !device.isCurrent}
                <Button
                  variant="ghost"
                  size="icon"
                  onclick={() => removeDevice(device)}
                  disabled={!!deviceActionPending[device.id]}
                  aria-label={t('Remove {name}', { name: device.name })}
                >
                  {#if deviceActionPending[device.id]}
                    <Spinner class="h-4 w-4" aria-hidden="true" />
                  {:else}
                    <Trash2 class="h-4 w-4" aria-hidden="true" />
                  {/if}
                </Button>
              {/if}
            </div>
          </div>
        {/each}

        {#if devices.some(d => !d.isCurrent)}
          <div class="flex justify-end">
            <Button
              variant="destructive"
              size="sm"
              onclick={revokeAllDevices}
              disabled={isRevokingDevices}
            >
              {#if isRevokingDevices}
                <Spinner class="mr-2 h-4 w-4" aria-hidden="true" />
              {/if}
              {t('Revoke All Other Devices')}
            </Button>
          </div>
        {/if}
      {/if}
    </CardContent>
  </Card>
</div>
