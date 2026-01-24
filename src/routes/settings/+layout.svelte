<script lang="ts">
  import { page } from '$app/stores';
  import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import { settingsStore } from '$lib/stores';
  import {
    Bell,
    Menu,
    Search,
    Settings,
    ShieldCheck,
    Palette,
    Clipboard,
    Info,
    Database,
    SlidersHorizontal,
    WandSparkles,
    Globe
  } from '@lucide/svelte';
  import { currentLocale } from '$lib/i18n';

  let { children } = $props();

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  let locale = $derived($currentLocale as 'en' | 'sv');

  type NavItem = { href: string; label: (locale: 'en' | 'sv') => string; Icon: typeof Settings };
  const navItems: NavItem[] = [
    { href: '/settings/general', label: (l) => t(l, 'General', 'Allmänt'), Icon: Settings },
    { href: '/settings/appearance', label: (l) => t(l, 'Appearance', 'Utseende'), Icon: Palette },
    { href: '/settings/security', label: (l) => t(l, 'Security', 'Säkerhet'), Icon: ShieldCheck },
    { href: '/settings/clipboard', label: (l) => t(l, 'Clipboard', 'Urklipp'), Icon: Clipboard },
    { href: '/settings/autofill', label: (l) => t(l, 'Autofill', 'Autofyll'), Icon: Globe },
    { href: '/settings/generator', label: (l) => t(l, 'Generator', 'Generator'), Icon: WandSparkles },
    { href: '/settings/vault', label: (l) => t(l, 'Vault', 'Valv'), Icon: Database },
    { href: '/settings/backup', label: (l) => t(l, 'Backup', 'Säkerhetskopior'), Icon: Database },
    { href: '/settings/advanced', label: (l) => t(l, 'Advanced', 'Avancerat'), Icon: SlidersHorizontal },
    { href: '/settings/about', label: (l) => t(l, 'About', 'Om Pulsar'), Icon: Info }
  ];

  let currentPath = $derived($page.url.pathname.replace(/\/$/, ''));
</script>

<div class="bg-background flex h-screen w-full overflow-hidden">
  <aside
    class="border-border/60 bg-card/50 hidden h-screen w-64 shrink-0 overflow-y-auto border-r p-4 md:block"
  >
    <nav class="space-y-1">
      {#each navItems as { href, label, Icon }}
        <a
          {href}
          class={`flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition ${
            currentPath === href.replace(/\/$/, '')
              ? 'bg-muted text-foreground'
              : 'text-muted-foreground hover:bg-muted/60'
          }`}
        >
          <Icon class="size-4" />
          <span>{label(locale)}</span>
        </a>
      {/each}
    </nav>

    <div class="border-border/60 mt-6 border-t pt-4">
      <p class="text-muted-foreground px-3 text-xs font-medium">
        {locale === 'sv' ? 'Mer' : 'More'}
      </p>
      <a
        href="/settings/security"
        class="text-muted-foreground hover:bg-muted/60 mt-2 flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm"
      >
        <ShieldCheck class="size-4" />
        <span>{locale === 'sv' ? 'Sessioner' : 'Sessions'}</span>
      </a>
    </div>
  </aside>

  <div class="flex h-screen min-w-0 flex-1 flex-col overflow-hidden">
    <header
      class="border-border/60 bg-background/80 supports-backdrop-filter:bg-background/70 sticky top-0 z-10 w-full border-b backdrop-blur"
    >
      <div class="mx-auto flex h-14 max-w-400 items-center gap-3 px-4">
        <button
          class="text-muted-foreground hover:bg-muted inline-flex items-center justify-center rounded-md p-2 md:hidden"
          aria-label="Open menu"
        >
          <Menu class="size-4" />
        </button>

        <div class="relative ml-1 hidden w-95 items-center md:flex">
          <Search class="text-muted-foreground pointer-events-none absolute left-2 size-4" />
          <Input class="w-full pl-8 text-sm" placeholder="Search settings" />
        </div>

        <div class="ml-auto flex items-center gap-2">
          <button
            class="text-muted-foreground hover:bg-muted rounded-md p-2"
            aria-label="Notifications"
          >
            <Bell class="size-4" />
          </button>
          <Button
            variant="ghost"
            class="h-8 px-2 disabled:opacity-50"
            onclick={() => settingsStore.resetAll()}
            disabled={!$settingsStore}>Reset</Button
          >
          <Button
            class="h-8 px-3 disabled:opacity-50"
            onclick={() => settingsStore.saveAll()}
            disabled={!$settingsStore}>Save</Button
          >
          <Avatar class="h-8 w-8">
            <AvatarImage src="/svelte.svg" alt="Avatar" />
            <AvatarFallback>PV</AvatarFallback>
          </Avatar>
        </div>
      </div>
    </header>

    <main class="mx-auto w-full max-w-225 flex-1 overflow-y-auto px-4 py-6">
      {@render children()}
    </main>
  </div>
</div>
