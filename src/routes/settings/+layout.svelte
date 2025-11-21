<script lang="ts">
  import { page } from '$app/stores';
  import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import { settingsStore } from '$lib/stores';
  import { Bell, Menu, Search, Settings, ShieldCheck, Palette, Clipboard, Info, Database, Sliders, Wand2, Globe } from '@lucide/svelte';

  type NavItem = { href: string; label: string; Icon: typeof Settings };
  const navItems: NavItem[] = [
    { href: '/settings/general', label: 'General', Icon: Settings },
    { href: '/settings/appearance', label: 'Appearance', Icon: Palette },
    { href: '/settings/security', label: 'Security', Icon: ShieldCheck },
    { href: '/settings/clipboard', label: 'Clipboard', Icon: Clipboard },
    { href: '/settings/autofill', label: 'Autofill', Icon: Globe },
    { href: '/settings/generator', label: 'Generator', Icon: Wand2 },
    { href: '/settings/vault', label: 'Vault', Icon: Database },
    { href: '/settings/backup', label: 'Backup', Icon: Database },
    { href: '/settings/advanced', label: 'Advanced', Icon: Sliders },
    { href: '/settings/about', label: 'About', Icon: Info }
  ];

  function isActive(href: string): boolean {
    const current = $page.url.pathname.replace(/\/$/, '');
    const target = href.replace(/\/$/, '');
    return current === target;
  }
</script>

<div class="flex h-screen w-full overflow-hidden bg-background">
  <!-- Left sidebar -->
  <aside class="hidden h-screen w-64 shrink-0 overflow-y-auto border-r border-border/60 bg-card/50 p-4 md:block">
    <nav class="space-y-1">
      {#each navItems as { href, label, Icon }}
        <a href={href} class={`flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition ${isActive(href) ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-muted/60'}`}>
          <Icon class="size-4" />
          <span>{label}</span>
        </a>
      {/each}
    </nav>

    <div class="mt-6 border-t border-border/60 pt-4">
      <p class="px-3 text-xs font-medium text-muted-foreground">More</p>
      <a href="/settings/security" class="mt-2 flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-muted/60">
        <ShieldCheck class="size-4" />
        <span>Sessions</span>
      </a>
    </div>
  </aside>

  <!-- Main area -->
  <div class="flex h-screen min-w-0 flex-1 flex-col overflow-hidden">
    <!-- Top header bar -->
    <header class="sticky top-0 z-10 w-full border-b border-border/60 bg-background/80 backdrop-blur supports-[backdrop-filter]:bg-background/70">
      <div class="mx-auto flex h-14 max-w-[1600px] items-center gap-3 px-4">
        <button class="inline-flex items-center justify-center rounded-md p-2 text-muted-foreground hover:bg-muted md:hidden" aria-label="Open menu">
          <Menu class="size-4" />
        </button>

        <div class="relative ml-1 hidden w-[380px] items-center md:flex">
          <Search class="pointer-events-none absolute left-2 size-4 text-muted-foreground" />
          <Input class="w-full pl-8 text-sm" placeholder="Search settings" />
        </div>

        <div class="ml-auto flex items-center gap-2">
          <button class="rounded-md p-2 text-muted-foreground hover:bg-muted" aria-label="Notifications">
            <Bell class="size-4" />
          </button>
          <Button variant="ghost" class="h-8 px-2 disabled:opacity-50" on:click={() => settingsStore.resetAll()} disabled={!$settingsStore}>Reset</Button>
          <Button class="h-8 px-3 disabled:opacity-50" on:click={() => settingsStore.saveAll()} disabled={!$settingsStore}>Save</Button>
          <Avatar class="h-8 w-8">
            <AvatarImage src="/svelte.svg" alt="Avatar" />
            <AvatarFallback>PV</AvatarFallback>
          </Avatar>
        </div>
      </div>
    </header>

    <!-- Routed content -->
    <main class="mx-auto w-full max-w-[900px] flex-1 overflow-y-auto px-4 py-6">
      <slot />
    </main>
  </div>
</div>
