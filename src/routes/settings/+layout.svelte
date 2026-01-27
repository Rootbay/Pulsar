<script lang="ts">
  import { page } from '$app/stores';
  import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import { settings } from '$lib/stores/appSettings.svelte';
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
    Globe,
    ArrowLeft
  } from '@lucide/svelte';
  import { i18n, t as translate } from '$lib/i18n.svelte';
  import type { I18nKey } from '$lib/i18n.svelte';
  import { goto } from '$app/navigation';

  let { children } = $props();

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as any, vars);

  type NavItem = { href: string; labelKey: I18nKey; Icon: typeof Settings };
  const navItems: NavItem[] = [
    { href: '/settings/general', labelKey: 'General', Icon: Settings },
    { href: '/settings/appearance', labelKey: 'Appearance', Icon: Palette },
    { href: '/settings/security', labelKey: 'Security', Icon: ShieldCheck },
    { href: '/settings/clipboard', labelKey: 'Clipboard', Icon: Clipboard },
    { href: '/settings/autofill', labelKey: 'Autofill', Icon: Globe },
    { href: '/settings/generator', labelKey: 'Generator', Icon: WandSparkles },
    { href: '/settings/vault', labelKey: 'Vault', Icon: Database },
    { href: '/settings/backup', labelKey: 'Backup', Icon: Database },
    { href: '/settings/advanced', labelKey: 'Advanced', Icon: SlidersHorizontal },
    { href: '/settings/about', labelKey: 'About', Icon: Info }
  ];

  let currentPath = $derived($page.url.pathname.replace(/\/$/, ''));
</script>

<div class="bg-background flex h-screen w-full overflow-hidden">
  <aside
    class="border-border/60 bg-card/50 hidden h-screen w-64 shrink-0 overflow-y-auto border-r p-4 md:block"
  >
    <nav class="space-y-1">
      {#each navItems as { href, labelKey, Icon } (href)}
        <a
          {href}
          class={`flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition ${
            currentPath === href.replace(/\/$/, '')
              ? 'bg-muted text-foreground'
              : 'text-muted-foreground hover:bg-muted/60'
          }`}
        >
          <Icon class="size-4" />
          <span>{t(labelKey)}</span>
        </a>
      {/each}
    </nav>

    <div class="border-border/60 mt-6 border-t pt-4">
      <p class="text-muted-foreground px-3 text-xs font-medium">
        {t('More')}
      </p>
      <a
        href="/settings/security"
        class="text-muted-foreground hover:bg-muted/60 mt-2 flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm"
      >
        <ShieldCheck class="size-4" />
        <span>{t('Sessions')}</span>
      </a>
    </div>
  </aside>

  <div class="flex h-screen min-w-0 flex-1 flex-col overflow-hidden">
    <header
      class="border-border/60 bg-background/80 supports-backdrop-filter:bg-background/70 sticky top-0 z-10 w-full border-b backdrop-blur"
    >
      <div class="mx-auto flex h-14 max-w-400 items-center gap-3 px-4">
        <Button
          variant="ghost"
          size="icon"
          class="text-muted-foreground hover:bg-muted hidden md:inline-flex"
          onclick={() => goto('/')}
          aria-label={t('back')}
        >
          <ArrowLeft class="size-4" />
        </Button>

        <button
          class="text-muted-foreground hover:bg-muted inline-flex items-center justify-center rounded-md p-2 md:hidden"
          aria-label={t('Open menu')}
        >
          <Menu class="size-4" />
        </button>

        <div class="relative ml-1 hidden w-95 items-center md:flex">
          <Search class="text-muted-foreground pointer-events-none absolute left-2 size-4" />
          <Input class="w-full pl-8 text-sm" placeholder={t('Search settings')} />
        </div>

        <div class="ml-auto flex items-center gap-2">
          <button
            class="text-muted-foreground hover:bg-muted rounded-md p-2"
            aria-label={t('Notifications')}
          >
            <Bell class="size-4" />
          </button>
          
          <Avatar class="h-8 w-8">
            <AvatarImage src="/logo.png" alt="Avatar" />
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