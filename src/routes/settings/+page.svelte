<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { appSettings } from '$lib/stores/appSettings';
  import { settingsStore } from '$lib/stores';
  import { setTheme, theme as themeStore } from '$lib/stores/theme';
  import { Button } from '$lib/components/ui/button';
  import { Separator } from '$lib/components/ui/separator';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import * as Card from '$lib/components/ui/card';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import { X, Palette, ShieldCheck, Clipboard as ClipboardIcon, Info } from '@lucide/svelte';

  const dispatch = createEventDispatcher();

  function closeSettings() {
    dispatch('close');
  }

  // Helpers to update settings
  function update<K extends keyof import('$lib/stores/appSettings').AllSettings>(key: K, patch: Partial<import('$lib/stores/appSettings').AllSettings[K]>) {
    appSettings.update((s) => ({ ...s, [key]: { ...(s as any)[key], ...patch } }));
  }

  $: currentTheme = $themeStore;
</script>

<div class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" role="dialog" aria-label="Settings" tabindex="0"
  on:click={(e) => { if (e.target === e.currentTarget) closeSettings(); }}
  on:keydown={(e) => { if (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') closeSettings(); }}>
  <div class="mx-auto my-10 w-[min(980px,95vw)] rounded-2xl border border-border/60 bg-card shadow-xl">
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4">
      <div class="flex items-center gap-3">
        <span class="inline-flex size-6 items-center justify-center rounded-md bg-primary/10 text-primary">
          <Palette class="size-4" />
        </span>
        <h2 class="text-base font-semibold">Settings</h2>
      </div>
      <div class="flex items-center gap-2">
        <Button variant="ghost" class="h-8 px-2 text-muted-foreground hover:text-foreground" onclick={() => settingsStore.resetAll()}>Reset</Button>
        <Button variant="default" class="h-8 px-3" onclick={() => settingsStore.saveAll()}>Save</Button>
        <Button variant="ghost" size="icon" class="h-8 w-8" onclick={closeSettings} aria-label="Close">
          <X class="size-4" />
        </Button>
      </div>
    </div>
    <Separator />

    <!-- Content -->
    <ScrollArea class="max-h-[70vh]">
      <div class="grid gap-5 px-5 py-5 md:grid-cols-2">
        <!-- Appearance -->
        <Card.Root class="border border-border/60">
          <Card.Header class="flex flex-row items-center gap-3">
            <span class="inline-flex size-8 items-center justify-center rounded-md bg-primary/10 text-primary">
              <Palette class="size-4" />
            </span>
            <div>
              <Card.Title class="text-sm">Appearance</Card.Title>
              <Card.Description class="text-xs">Theme, density and accessibility</Card.Description>
            </div>
          </Card.Header>
          <Card.Content class="space-y-4">
            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Theme</p>
                <p class="text-xs text-muted-foreground">Light, Dark or System</p>
              </div>
              <Select type="single" value={currentTheme} onValueChange={(value) => setTheme(value as 'system' | 'light' | 'dark')}>
                <SelectTrigger aria-label="Select theme" class="w-36">
                  <span data-slot="select-value" class="text-sm">{$themeStore}</span>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="system">System</SelectItem>
                  <SelectItem value="light">Light</SelectItem>
                  <SelectItem value="dark">Dark</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Compact mode</p>
                <p class="text-xs text-muted-foreground">Denser layout and spacing</p>
              </div>
              <Switch checked={$appSettings.appearance.compactMode} onclick={() => update('appearance', { compactMode: !$appSettings.appearance.compactMode })} />
            </div>

            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">High contrast</p>
                <p class="text-xs text-muted-foreground">Improved contrast for readability</p>
              </div>
              <Switch checked={$appSettings.appearance.highContrast} onclick={() => update('appearance', { highContrast: !$appSettings.appearance.highContrast })} />
            </div>

            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Reduced motion</p>
                <p class="text-xs text-muted-foreground">Minimize animations</p>
              </div>
              <Switch checked={$appSettings.appearance.reducedMotion} onclick={() => update('appearance', { reducedMotion: !$appSettings.appearance.reducedMotion })} />
            </div>
          </Card.Content>
        </Card.Root>

        <!-- Security -->
        <Card.Root class="border border-border/60">
          <Card.Header class="flex flex-row items-center gap-3">
            <span class="inline-flex size-8 items-center justify-center rounded-md bg-primary/10 text-primary">
              <ShieldCheck class="size-4" />
            </span>
            <div>
              <Card.Title class="text-sm">Security</Card.Title>
              <Card.Description class="text-xs">Lock & clipboard behavior</Card.Description>
            </div>
          </Card.Header>
          <Card.Content class="space-y-4">
            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Lock on suspend</p>
                <p class="text-xs text-muted-foreground">Auto-lock when the system sleeps</p>
              </div>
              <Switch checked={$appSettings.security.lockOnSuspend} onclick={() => update('security', { lockOnSuspend: !$appSettings.security.lockOnSuspend })} />
            </div>
            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Lock on minimize</p>
                <p class="text-xs text-muted-foreground">Lock the vault when the app is minimized</p>
              </div>
              <Switch checked={$appSettings.security.lockOnMinimize} onclick={() => update('security', { lockOnMinimize: !$appSettings.security.lockOnMinimize })} />
            </div>
            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Clipboard clear time</p>
                <p class="text-xs text-muted-foreground">How long to keep copied secrets</p>
              </div>
              <Select type="single" value={String($appSettings.security.clipboardClearTime)} onValueChange={(value) => update('security', { clipboardClearTime: Number(value) })}>
                <SelectTrigger aria-label="Select clipboard clear time" class="w-24">
                  <span data-slot="select-value" class="text-sm">{ $appSettings.security.clipboardClearTime }s</span>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="0">Off</SelectItem>
                  <SelectItem value="5">5s</SelectItem>
                  <SelectItem value="10">10s</SelectItem>
                  <SelectItem value="30">30s</SelectItem>
                  <SelectItem value="60">60s</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Clear clipboard on copy</p>
                <p class="text-xs text-muted-foreground">Automatically clear sensitive content</p>
              </div>
              <Switch checked={$appSettings.security.clearClipboardOnCopy} onclick={() => update('security', { clearClipboardOnCopy: !$appSettings.security.clearClipboardOnCopy })} />
            </div>
          </Card.Content>
        </Card.Root>

        <!-- Clipboard -->
        <Card.Root class="border border-border/60">
          <Card.Header class="flex flex-row items-center gap-3">
            <span class="inline-flex size-8 items-center justify-center rounded-md bg-primary/10 text-primary">
              <ClipboardIcon class="size-4" />
            </span>
            <div>
              <Card.Title class="text-sm">Clipboard</Card.Title>
              <Card.Description class="text-xs">Integration & privacy</Card.Description>
            </div>
          </Card.Header>
          <Card.Content class="space-y-4">
            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Enable clipboard integration</p>
                <p class="text-xs text-muted-foreground">Allow copy/paste from the app</p>
              </div>
              <Switch checked={$appSettings.clipboard.clipboardIntegration} onclick={() => update('clipboard', { clipboardIntegration: !$appSettings.clipboard.clipboardIntegration })} />
            </div>
            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-sm font-medium">Block clipboard history</p>
                <p class="text-xs text-muted-foreground">Prevent OS clipboard history saving</p>
              </div>
              <Switch checked={$appSettings.clipboard.blockHistory} onclick={() => update('clipboard', { blockHistory: !$appSettings.clipboard.blockHistory })} />
            </div>
          </Card.Content>
        </Card.Root>

        <!-- About -->
        <Card.Root class="border border-border/60">
          <Card.Header class="flex flex-row items-center gap-3">
            <span class="inline-flex size-8 items-center justify-center rounded-md bg-primary/10 text-primary">
              <Info class="size-4" />
            </span>
            <div>
              <Card.Title class="text-sm">About</Card.Title>
              <Card.Description class="text-xs">Version & diagnostics</Card.Description>
            </div>
          </Card.Header>
          <Card.Content class="space-y-4 text-sm text-muted-foreground">
            <p>Pulsar — Secure password vault powered by Tauri + Svelte.</p>
            <div class="flex gap-2">
              <Button variant="outline" class="h-8 px-3">Export settings</Button>
              <Button variant="outline" class="h-8 px-3">Import settings</Button>
            </div>
          </Card.Content>
        </Card.Root>
      </div>
    </ScrollArea>
  </div>
</div>

<style>
  /* Rely on shadcn styles */
</style>
