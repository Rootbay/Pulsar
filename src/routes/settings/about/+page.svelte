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
  import { currentLocale } from '$lib/i18n';

  type UpdateStatus = 'idle' | 'checking' | 'uptoDate';
  type IconComponent = typeof FileText;

  let updateStatus: UpdateStatus = 'idle';
  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  $: locale = $currentLocale as 'en' | 'sv';
  let uploadDiagnostics = false;

  const versionDetails = [
    { label: 'Version', value: '2.1.4' },
    { label: 'Build', value: 'a7f3d2e' },
    { label: 'Runtime', value: 'Rust 1.75' },
    { label: 'UI', value: 'Svelte 4.2' },
    { label: 'Status', value: 'Up to date', accentClass: 'text-chart-success' }
  ];

  type ResourceLink = { label: string; Icon: IconComponent; action: () => void };

  const resourceLinks: ResourceLink[] = [
    { label: 'License', Icon: FileText, action: () => openLink('License') },
    {
      label: 'Security Whitepaper',
      Icon: ShieldQuestionMark,
      action: () => openLink('Whitepaper')
    },
    { label: 'Contact Support', Icon: LifeBuoy, action: () => openLink('Support') },
    { label: 'Source Code', Icon: CodeXml, action: () => openLink('Source Code') }
  ];

  function checkForUpdates() {
    if (updateStatus !== 'idle') return;
    updateStatus = 'checking';

    setTimeout(() => {
      updateStatus = 'uptoDate';
      setTimeout(() => {
        updateStatus = 'idle';
      }, 2000);
    }, 1500);
  }

  function openLink(linkName: string) {
    alert(`Opening ${linkName}...`);
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
          {t(locale, 'Application Information', 'Information om applikationen')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Version details and update status.',
            'Versionsinformation och uppdateringsstatus.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-6 pt-4">
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {#each versionDetails as detail}
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
            {t(locale, 'Check for Updates', 'Sök efter uppdateringar')}
          </h3>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Check if a newer version is available',
              'Kontrollera om en nyare version finns tillgänglig'
            )}
          </p>
        </div>
        <Button class="shrink-0" onclick={checkForUpdates} disabled={updateStatus !== 'idle'}>
          {#if updateStatus === 'idle'}
            <RefreshCw class="mr-2 h-4 w-4" aria-hidden="true" />
            {t(locale, 'Check Now', 'Sök nu')}
          {:else if updateStatus === 'checking'}
            <RefreshCw class="mr-2 h-4 w-4 animate-spin" aria-hidden="true" />
            {t(locale, 'Checking...', 'Söker...')}
          {:else}
            <BadgeCheck class="text-chart-success mr-2 h-4 w-4" aria-hidden="true" />
            {t(locale, 'Up to date', 'Uppdaterad')}
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
          {t(locale, 'Documentation & Support', 'Dokumentation & support')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Find helpful resources and get in touch.',
            'Hitta hjälpresurser och kontakta oss.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      <div class="grid gap-3 sm:grid-cols-2">
        {#each resourceLinks as { label, Icon, action }}
          <Button
            type="button"
            variant="secondary"
            class="h-auto justify-start gap-3 px-4 py-3 text-left"
            onclick={action}
          >
            <Icon class="h-4 w-4" aria-hidden="true" />
            {label === 'License'
              ? t(locale, 'License', 'Licens')
              : label === 'Security Whitepaper'
                ? t(locale, 'Security Whitepaper', 'Säkerhetswhitepaper')
                : label === 'Contact Support'
                  ? t(locale, 'Contact Support', 'Kontakta support')
                  : t(locale, 'Source Code', 'Källkod')}
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
          {t(locale, 'Diagnostic Tools', 'Diagnostikverktyg')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Tools for troubleshooting and support.',
            'Verktyg för felsökning och support.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div
        class="border-border/60 bg-card/40 flex flex-col gap-2 rounded-lg border p-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <h3 class="text-foreground text-sm font-semibold">
            {t(locale, 'Open Logs Folder', 'Öppna loggmapp')}
          </h3>
          <p class="text-muted-foreground text-sm">
            {t(locale, 'Access application log files', 'Öppna programmets loggfiler')}
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          class="gap-2"
          onclick={() => openLink('Logs Folder')}
        >
          <FolderOpen class="h-4 w-4" aria-hidden="true" />
          {t(locale, 'Open', 'Öppna')}
        </Button>
      </div>

      <div
        class="border-border/60 bg-card/40 flex flex-col gap-2 rounded-lg border p-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <h3 class="text-foreground text-sm font-semibold">
            {t(locale, 'Upload Diagnostics to Support', 'Ladda upp diagnostik till supporten')}
          </h3>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Help us identify and fix issues faster',
              'Hjälp oss hitta och åtgärda problem snabbare'
            )}
          </p>
        </div>
        <div class="flex items-center gap-3">
          <CloudUpload class="text-muted-foreground h-4 w-4" aria-hidden="true" />
          <Switch bind:checked={uploadDiagnostics} aria-label="Toggle upload diagnostics" />
        </div>
      </div>
    </CardContent>
  </Card>
</div>
