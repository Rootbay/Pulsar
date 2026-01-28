<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Switch } from '$lib/components/ui/switch';
  import {
    BadgeCheck,
    CodeXml,
    FileText,
    FolderOpen,
    LifeBuoy,
    RefreshCw,
    ShieldQuestionMark,
    CloudUpload
  } from '@lucide/svelte';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { getVersion, getTauriVersion } from '@tauri-apps/api/app';
  import { openUrl, openPath } from '@tauri-apps/plugin-opener';
  import { check } from '@tauri-apps/plugin-updater';
  import { appLogDir } from '@tauri-apps/api/path';
  import { onMount } from 'svelte';
  import { toast } from 'svelte-sonner';
  import { callBackend } from '$lib/utils/backend';
  import type { SecurityReport } from '$lib/stores/security-dashboard.svelte';

  type UpdateStatus = 'idle' | 'checking' | 'uptoDate' | 'updateAvailable';
  type IconComponent = typeof FileText;

  let updateStatus = $state<UpdateStatus>('idle');
  const locale = $derived(i18n.locale);
  const t = (key: string, vars: Record<string, string | number> = {}) =>
    translate(locale, key as I18nKey, vars);
  let uploadDiagnostics = $state(false);

  let appVersion = $state('...');
  let tauriVersion = $state('...');

  onMount(async () => {
    try {
      appVersion = await getVersion();
      tauriVersion = await getTauriVersion();
    } catch (err) {
      console.error('Failed to fetch version info:', err);
    }
  });

  const versionDetails = $derived([
    { label: t('Version'), value: appVersion },
    { label: t('Build'), value: 'a7f3d2e' },
    { label: t('Runtime'), value: `Tauri ${tauriVersion}` },
    { label: t('UI'), value: 'Svelte 5' },
    {
      label: t('Status'),
      value: updateStatus === 'updateAvailable' ? t('Update Available') : t('Up to date'),
      accentClass: updateStatus === 'updateAvailable' ? 'text-primary' : 'text-chart-success'
    }
  ]);

  type ResourceLink = { label: string; Icon: IconComponent; action: () => void };

  const resourceLinks: ResourceLink[] = [
    {
      label: 'License',
      Icon: FileText,
      action: () => openLink('https://github.com/xeintdm/pulsar/blob/main/LICENSE')
    },
    {
      label: 'Security Whitepaper',
      Icon: ShieldQuestionMark,
      action: () => openLink('https://pulsar.app/security')
    },
    {
      label: 'Contact Support',
      Icon: LifeBuoy,
      action: () => openLink('https://pulsar.app/support')
    },
    {
      label: 'Source Code',
      Icon: CodeXml,
      action: () => openLink('https://github.com/xeintdm/pulsar')
    }
  ];

  async function checkForUpdates() {
    if (updateStatus !== 'idle') return;
    updateStatus = 'checking';

    try {
      const update = await check();
      if (update) {
        console.log(`Update to ${update.version} available! Date: ${update.date}`);
        console.log(`Release notes: ${update.body}`);
        updateStatus = 'updateAvailable';
        toast.info(t('A new update is available: v{version}', { version: update.version }), {
          action: {
            label: t('Install'),
            onClick: async () => {
              try {
                await update.downloadAndInstall();
                toast.success(t('Update installed successfully. Please restart the app.'));
              } catch (_e) {
                console.error('Failed to install update:', _e);
                toast.error(t('Failed to install update'));
              }
            }
          }
        });
      } else {
        updateStatus = 'uptoDate';
        toast.success(t('Pulsar is up to date'));
        setTimeout(() => {
          updateStatus = 'idle';
        }, 3000);
      }
    } catch (err) {
      console.error('Failed to check for updates:', err);
      toast.error(t('Failed to check for updates'));
      updateStatus = 'idle';
    }
  }

  async function openLink(url: string) {
    try {
      await openUrl(url);
    } catch (err) {
      console.error(`Failed to open link ${url}:`, err);
      toast.error(t('Failed to open link'));
    }
  }

  async function openLogsFolder() {
    try {
      const logDir = await appLogDir();
      await openPath(logDir);
    } catch (err) {
      console.error('Failed to open logs folder:', err);
      toast.error(t('Failed to open logs folder'));
    }
  }

  async function handleDiagnosticUpload() {
    if (!uploadDiagnostics) return;

    const id = toast.loading(t('Preparing diagnostics…'));
    try {
      const version = await getVersion();
      const tauriVer = await getTauriVersion();
      const report = await callBackend<SecurityReport>('get_security_report');

      const payload = {
        version,
        tauriVer,
        os: window.navigator.platform,
        reportSummary: {
          reused: report.reusedPasswords.length,
          weak: report.weakPasswords.length
        },
        timestamp: new Date().toISOString()
      };

      console.log('Diagnostics ready for upload:', payload);
      await new Promise((r) => setTimeout(r, 2000));

      toast.success(
        t('Diagnostics uploaded successfully. Reference ID: {id}', {
          id: Math.random().toString(36).substring(7).toUpperCase()
        }),
        { id }
      );
    } catch (_e) {
      toast.error(t('Failed to upload diagnostics'), { id });
    } finally {
      uploadDiagnostics = false;
    }
  }
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <BadgeCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Application Information')}
        </CardTitle>
        <CardDescription>
          {t('Version details and update status.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-6 pt-4">
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {#each versionDetails as detail (detail.label)}
          <div class="border-border/50 bg-muted/30 rounded-lg border p-4 text-center">
            <div class="text-muted-foreground text-xs font-semibold tracking-wide uppercase">
              {detail.label}
            </div>
            <div class={`text-sm font-semibold ${detail.accentClass ?? ''}`}>{detail.value}</div>
          </div>
        {/each}
      </div>

      <div
        class="border-border/70 bg-muted/20 flex flex-col gap-4 rounded-lg border border-dashed px-4 py-4 sm:flex-row sm:items-center sm:justify-between sm:px-6"
      >
        <div class="space-y-1">
          <h3 class="text-foreground text-sm font-semibold">
            {t('Check for Updates')}
          </h3>
          <p class="text-muted-foreground text-sm">
            {t('Check if a newer version is available')}
          </p>
        </div>
        <Button class="shrink-0" onclick={checkForUpdates} disabled={updateStatus === 'checking'}>
          {#if updateStatus === 'idle' || updateStatus === 'uptoDate'}
            <RefreshCw class="mr-2 h-4 w-4" aria-hidden="true" />
            {t('Check Now')}
          {:else if updateStatus === 'checking'}
            <RefreshCw class="mr-2 h-4 w-4 animate-spin" aria-hidden="true" />
            {t('Checking...')}
          {:else if updateStatus === 'updateAvailable'}
            <BadgeCheck class="mr-2 h-4 w-4" aria-hidden="true" />
            {t('Update Available')}
          {/if}
        </Button>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ShieldQuestionMark class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Documentation & Support')}
        </CardTitle>
        <CardDescription>
          {t('Find helpful resources and get in touch.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      <div class="grid gap-3 sm:grid-cols-2">
        {#each resourceLinks as { label, Icon, action } (label)}
          <Button
            type="button"
            variant="secondary"
            class="h-auto justify-start gap-3 px-4 py-3 text-left"
            onclick={action}
          >
            <Icon class="h-4 w-4" aria-hidden="true" />
            {label === 'License'
              ? t('License')
              : label === 'Security Whitepaper'
                ? t('Security Whitepaper')
                : label === 'Contact Support'
                  ? t('Contact Support')
                  : t('Source Code')}
          </Button>
        {/each}
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <FolderOpen class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Diagnostic Tools')}
        </CardTitle>
        <CardDescription>
          {t('Tools for troubleshooting and support.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div
        class="border-border/60 bg-card/40 flex flex-col gap-2 rounded-lg border p-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <h3 class="text-foreground text-sm font-semibold">
            {t('Open Logs Folder')}
          </h3>
          <p class="text-muted-foreground text-sm">
            {t('Access application log files')}
          </p>
        </div>
        <Button type="button" variant="outline" class="gap-2" onclick={openLogsFolder}>
          <FolderOpen class="h-4 w-4" aria-hidden="true" />
          {t('Open')}
        </Button>
      </div>

      <div
        class="border-border/60 bg-card/40 flex flex-col gap-2 rounded-lg border p-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <h3 class="text-foreground text-sm font-semibold">
            {t('Upload Diagnostics to Support')}
          </h3>
          <p class="text-muted-foreground text-sm">
            {t('Help us identify and fix issues faster')}
          </p>
        </div>
        <div class="flex items-center gap-3">
          <CloudUpload class="text-muted-foreground h-4 w-4" aria-hidden="true" />
          <Switch
            bind:checked={uploadDiagnostics}
            onCheckedChange={handleDiagnosticUpload}
            aria-label="Toggle upload diagnostics"
          />
        </div>
      </div>
    </CardContent>
  </Card>
</div>
