<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Switch } from '$lib/components/ui/switch';
  import { BadgeCheck, CodeXml, FileText, FolderOpen, LifeBuoy, RefreshCw, ShieldQuestionMark, CloudUpload } from '@lucide/svelte';

  type UpdateStatus = 'idle' | 'checking' | 'uptoDate';
  type IconComponent = typeof FileText;

  let updateStatus: UpdateStatus = 'idle';
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
    { label: 'Security Whitepaper', Icon: ShieldQuestionMark, action: () => openLink('Whitepaper') },
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

<div class="flex flex-1 flex-col gap-6 overflow-y-auto px-8 py-8">
  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <BadgeCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Application Information</CardTitle>
        <CardDescription>Version details and update status.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-6 pt-4">
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {#each versionDetails as detail}
          <div class="rounded-lg border border-border/50 bg-muted/30 p-4 text-center">
            <div class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{detail.label}</div>
            <div class={`text-sm font-semibold ${detail.accentClass ?? ''}`}>{detail.value}</div>
          </div>
        {/each}
      </div>

      <div class="flex flex-col gap-4 rounded-lg border border-dashed border-border/70 bg-muted/20 px-4 py-4 sm:flex-row sm:items-center sm:justify-between sm:px-6">
        <div class="space-y-1">
          <h3 class="text-sm font-semibold text-foreground">Check for Updates</h3>
          <p class="text-sm text-muted-foreground">Check if a newer version is available</p>
        </div>
        <Button class="shrink-0" onclick={checkForUpdates} disabled={updateStatus !== 'idle'}>
          {#if updateStatus === 'idle'}
            <RefreshCw class="mr-2 h-4 w-4" aria-hidden="true" />
            Check Now
          {:else if updateStatus === 'checking'}
            <RefreshCw class="mr-2 h-4 w-4 animate-spin" aria-hidden="true" />
            Checking...
          {:else}
            <BadgeCheck class="mr-2 h-4 w-4 text-chart-success" aria-hidden="true" />
            Up to date
          {/if}
        </Button>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ShieldQuestionMark class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Documentation & Support</CardTitle>
        <CardDescription>Find helpful resources and get in touch.</CardDescription>
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
            {label}
          </Button>
        {/each}
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <FolderOpen class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Diagnostic Tools</CardTitle>
        <CardDescription>Tools for troubleshooting and support.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div class="flex flex-col gap-2 rounded-lg border border-border/60 bg-card/40 p-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 class="text-sm font-semibold text-foreground">Open Logs Folder</h3>
          <p class="text-sm text-muted-foreground">Access application log files</p>
        </div>
        <Button type="button" variant="outline" class="gap-2" onclick={() => openLink('Logs Folder')}>
          <FolderOpen class="h-4 w-4" aria-hidden="true" />
          Open
        </Button>
      </div>

      <div class="flex flex-col gap-2 rounded-lg border border-border/60 bg-card/40 p-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 class="text-sm font-semibold text-foreground">Upload Diagnostics to Support</h3>
          <p class="text-sm text-muted-foreground">Help us identify and fix issues faster</p>
        </div>
        <div class="flex items-center gap-3">
          <CloudUpload class="h-4 w-4 text-muted-foreground" aria-hidden="true" />
          <Switch bind:checked={uploadDiagnostics} aria-label="Toggle upload diagnostics" />
        </div>
      </div>
    </CardContent>
  </Card>
</div>


