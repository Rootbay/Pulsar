<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap" rel="stylesheet" />
</svelte:head>

<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import ImportManagerPopup from '$lib/components/ImportManagerPopup.svelte';
  import { isDatabaseLoaded, isLocked, needsPasswordSetup, totpVerified } from '$lib/stores';
  import { recentDatabases } from '$lib/stores/recentDatabases';
  import { importVaultBackup, notifyVaultRefresh } from '$lib/utils/backup';
  import type { ImportVaultProgressStage } from '$lib/utils/backup';
  import { currentLocale } from '$lib/i18n';
  import { X, Loader2, FolderOpen, PlusCircle, Upload, ArrowLeft } from '@lucide/svelte';

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  $: locale = $currentLocale as 'en' | 'sv';

  let error: string | null = null;
  let importMessage: string | null = null;
  let importProgress: string | null = null;
  let showImportPopup = false;
  let lastAttemptedPath: string | null = null;
  let triedElevated = false;
  let importProgressMessages: Record<ImportVaultProgressStage, string>;

  $: importProgressMessages = {
    decrypting: t(locale, 'Decrypting backup…', 'Dekrypterar backup…'),
    restoring: t(locale, 'Restoring vault contents…', 'Återställer valvets innehåll…')
  };

  $: hasAccessDeniedError = Boolean(error?.toLowerCase().includes('access is denied'));

  $: if (browser && $isDatabaseLoaded) {
    if ($needsPasswordSetup) {
      goto('/setup', { replaceState: true });
    } else if ($isLocked) {
      goto('/login', { replaceState: true });
    } else {
      goto('/', { replaceState: true });
    }
  }

  const loadAndCheckDatabase = async (path: string) => {
    if (!path) {
      return;
    }

    error = null;
    lastAttemptedPath = path;
    totpVerified.set(false);

    try {
      await invoke('switch_database', { dbPath: path });
      const isConfigured = await invoke<boolean>('is_master_password_configured');

      if (isConfigured) {
        $isLocked = true;
        $needsPasswordSetup = false;
      } else {
        $isLocked = false;
        $needsPasswordSetup = true;
      }

      $isDatabaseLoaded = true;
      recentDatabases.addRecentDatabase(path);
    } catch (cause) {
      console.error('Failed to load database:', cause);
      if (typeof cause === 'string') {
        error = cause;
      } else {
        try {
          error = JSON.stringify(cause);
        } catch (_) {
          error = t(locale, 'Failed to load the selected vault.', 'Misslyckades att ladda det valda valvet.');
        }
      }

      const message =
        typeof cause === 'string'
          ? cause
          : (() => {
              try {
                return JSON.stringify(cause);
              } catch (_) {
                return '';
              }
            })();

      if (!triedElevated && message.toLowerCase().includes('access is denied')) {
        triedElevated = true;
        void attemptElevatedCopy();
      }
    }
  };

  const attemptElevatedCopy = async () => {
    if (!lastAttemptedPath) {
      return;
    }

    try {
      const destination = await invoke<string>('elevated_copy', { src: lastAttemptedPath });
      if (typeof destination === 'string') {
        await loadAndCheckDatabase(destination);
      }
    } catch (cause) {
      console.error('Elevated copy failed:', cause);
      if (typeof cause === 'string') {
        error = cause;
      } else {
        try {
          error = JSON.stringify(cause);
        } catch (_) {
          error = t(
            locale,
            'Failed to copy the vault with elevated permissions.',
            'Misslyckades kopiera valvet med förhöjda rättigheter.'
          );
        }
      }
    }
  };

  const forgetRecent = (path: string) => {
    if (!path) {
      return;
    }

    recentDatabases.removeRecentDatabase(path);
  };

  const clearAllRecent = () => {
    recentDatabases.clearRecentDatabases();
  };

  const basename = (path: string) => {
    if (!path) {
      return '';
    }

    const segments = path.split(/[\\/]/);
    return segments.at(-1) || path;
  };

  const selectExistingVault = async () => {
    try {
      const picked = await open({
        title: 'Select a Pulsar Vault',
        filters: [{ name: 'Pulsar Vault', extensions: ['psec'] }],
        multiple: false
      });

      if (typeof picked === 'string') {
        await loadAndCheckDatabase(picked);
      }
    } catch (cause) {
      console.error('Failed to open or switch vault:', cause);
    }
  };

  const createNewVault = async () => {
    try {
      const picked = await save({
        title: 'Create a new Pulsar Vault',
        filters: [{ name: 'Pulsar Vault', extensions: ['psec'] }]
      });

      if (picked) {
        const withExt = picked.endsWith('.psec') ? picked : `${picked}.psec`;
        const sep = withExt.includes('\\') ? '\\' : '/';
        const lastSep = withExt.lastIndexOf(sep);
        const baseDir = lastSep === -1 ? '' : withExt.slice(0, lastSep);
        const baseName = lastSep === -1 ? withExt : withExt.slice(lastSep + 1);
        const stem = baseName.endsWith('.psec') ? baseName.slice(0, -5) : baseName;
        const folder = baseDir ? `${baseDir}${sep}${stem}` : stem;
        const finalPath = `${folder}${sep}${stem}.psec`;

        await loadAndCheckDatabase(finalPath);
      }
    } catch (cause) {
      console.error('Save failed:', cause);
    }
  };

  const handleImportFile = () => {
    showImportPopup = true;
  };

  const handleImportSelected = async (event: CustomEvent<Record<string, unknown>>) => {
    showImportPopup = false;
    importMessage = null;
    importProgress = null;

    const importedPath = typeof event.detail?.importedPath === 'string' ? event.detail.importedPath : null;
    const passphrase = typeof event.detail?.passphrase === 'string' ? event.detail.passphrase.trim() : '';

    if (!importedPath) {
      error = t(locale, 'No backup file was selected.', 'Ingen backupfil valdes.');
      return;
    }

    if (!passphrase) {
      error = t(
        locale,
        'A passphrase is required to unlock the selected backup.',
        'En lösenfras krävs för att låsa upp den valda backupen.'
      );
      return;
    }

    try {
      error = null;
      importProgress = importProgressMessages.decrypting;

      const snapshot = await importVaultBackup(passphrase, {
        sourcePath: importedPath,
        onProgress: (stage) => {
          importProgress = importProgressMessages[stage];
        }
      });
      const itemCount = snapshot.passwordItems.length;
      const tagCount = snapshot.buttons.length;
      importMessage = t(
        locale,
        `Vault restored: ${itemCount} item${itemCount === 1 ? '' : 's'} and ${tagCount} tag${tagCount === 1 ? '' : 's'} imported successfully.`,
        `Valv återställt: ${itemCount} post${itemCount === 1 ? '' : 'er'} och ${tagCount} tagg${tagCount === 1 ? '' : 'ar'} importerades.`
      );
      importProgress = null;
      notifyVaultRefresh('import');
    } catch (cause) {
      console.error('Failed to import backup:', cause);
      if (typeof cause === 'string') {
        error = cause;
      } else if (cause instanceof Error) {
        error = cause.message;
      } else {
        error = t(
          locale,
          'Failed to import the selected backup. Please verify the passphrase and try again.',
          'Misslyckades att importera den valda backupen. Kontrollera lösenfrasen och försök igen.'
        );
      }
      importProgress = null;
    }
  };

  const selectRecentDatabase = async (path: string) => {
    if (!path) {
      return;
    }

    try {
      const exists = await invoke<boolean>('check_file_exists', { path });
      if (exists) {
        await loadAndCheckDatabase(path);
      } else {
        recentDatabases.removeRecentDatabase(path);
        error = t(locale, 'Selected recent file could not be found.', 'Den valda senaste filen kunde inte hittas.');
      }
    } catch (cause) {
      console.error(`Failed to check file existence for ${path}:`, cause);
      error = t(locale, 'An error occurred while checking the file.', 'Ett fel uppstod när filen kontrollerades.');
    }
  };

  function goBack() {
    goto('/', { replaceState: true });
  }
</script>

<div class="relative min-h-screen bg-background">
  <button
    type="button"
    class="absolute left-4 top-4 z-10 flex items-center gap-1 rounded-md px-2 py-1 text-sm text-muted-foreground transition hover:text-foreground"
    onclick={goBack}
  >
    <ArrowLeft class="h-4 w-4" />
    {t(locale, 'Back', 'Tillbaka')}
  </button>
  <div class="pointer-events-none absolute inset-0 -z-10 overflow-hidden">
    <div class="absolute left-[8%] top-[12%] h-[22rem] w-[22rem] rounded-full bg-primary/10 blur-3xl"></div>
    <div class="absolute right-[10%] bottom-[10%] h-[18rem] w-[18rem] rounded-full bg-muted/40 blur-2xl"></div>
  </div>

  <div class="mx-auto flex min-h-[65vh] w-full max-w-[490px] flex-col items-center justify-start pt-[60px] px-6 translate-x-[19px]">
    <div class="w-full text-center">
      <h1 class="text-3xl font-semibold text-foreground">
        {t(locale, 'Welcome to Pulsar Pass', 'Välkommen till Pulsar Pass')}
      </h1>
      <p class="mt-2 text-sm text-muted-foreground">
        {t(locale, 'Please select an existing vault or create a new one', 'Välj ett befintligt valv eller skapa ett nytt')}
      </p>
    </div>

    <div class="mt-8 w-full space-y-[9px]">
      <button
        type="button"
        class="h-[47px] w-full max-w-[490px] rounded-full bg-white/10 px-5 text-sm text-white transition-colors hover:bg-white/15 focus:outline-none mx-auto"
        onclick={selectExistingVault}
      >
        {t(locale, 'Select existing vault', 'Välj befintligt valv')}
      </button>

      <button
        type="button"
        class="h-[47px] w-full max-w-[490px] rounded-full bg-transparent px-5 text-sm text-white/70 transition-colors hover:text-white hover:bg-white/10 focus:outline-none mx-auto"
        onclick={createNewVault}
      >
        {t(locale, 'Create new vault', 'Skapa nytt valv')}
      </button>

      <button
        type="button"
        class="h-[47px] w-full max-w-[490px] rounded-full bg-transparent px-5 text-sm text-white/70 transition-colors hover:text-white hover:bg-white/10 focus:outline-none mx-auto"
        onclick={handleImportFile}
      >
        {t(locale, 'Migrate vault', 'Migrera valv')}
      </button>
    </div>

    {#if error}
      <div class="mt-6 w-full rounded-xl border border-destructive/40 bg-destructive/10 p-3 text-left">
        <p class="text-sm font-medium text-destructive">{error}</p>
        {#if hasAccessDeniedError}
          <div class="mt-3">
            <button
              type="button"
              class="h-9 rounded-full border border-border/60 bg-muted/20 px-4 text-sm text-foreground hover:bg-muted/30"
              onclick={attemptElevatedCopy}
            >
              {t(locale, 'Attempt elevated copy into app folder', 'Försök kopiera med förhöjda rättigheter till appmappen')}
            </button>
          </div>
        {/if}
      </div>
    {/if}

    {#if importProgress}
      <div class="mt-3 flex w-full items-center gap-3 rounded-xl border border-border/60 bg-muted/20 p-3">
        <Loader2 class="h-4 w-4 animate-spin text-muted-foreground" aria-hidden="true" />
        <p class="text-sm text-foreground">{importProgress}</p>
      </div>
    {/if}

    {#if importMessage}
      <div class="mt-3 w-full rounded-xl border border-primary/40 bg-primary/10 p-3">
        <p class="text-sm font-medium text-foreground">{importMessage}</p>
      </div>
    {/if}

    <div class="mt-10 w-full text-left">
      <h2 class="text-sm font-medium text-foreground">
        {t(locale, 'Recent vault', 'Senaste valv')}
      </h2>
      {#if $recentDatabases.length > 0}
        <ul class="mt-3 space-y-2 w-full max-w-[490px] mx-auto">
          {#each $recentDatabases as dbPath (dbPath)}
            <li>
              <button
                type="button"
                class="w-full rounded-lg bg-white/5 px-3 py-2 text-left text-[13px] text-muted-foreground transition hover:bg-white/10 hover:text-foreground"
                title={dbPath}
                onclick={() => selectRecentDatabase(dbPath)}
              >
                {basename(dbPath)}
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="mt-2 text-xs text-muted-foreground">
          {t(locale, 'No recent databases.', 'Inga senaste databaser.')}
        </p>
      {/if}
    </div>
  </div>
</div>

<ImportManagerPopup bind:show={showImportPopup} on:importSelected={handleImportSelected} />

<style>
  * {
    box-sizing: border-box;
  }

  :global(body) {
    font-family: 'Inter', sans-serif;
  }

  .container {
    --ds-surface: color-mix(in oklch, var(--background) 92%, var(--muted) 8%);
    --ds-surface-strong: color-mix(in oklch, var(--background) 80%, var(--muted) 20%);
    --ds-control-bg: color-mix(in oklch, var(--muted) 85%, transparent 15%);
    --ds-control-border: color-mix(in oklch, var(--border) 70%, transparent 30%);
    --ds-control-hover: color-mix(in oklch, var(--primary) 22%, var(--muted) 78%);
    --ds-control-text: var(--foreground);
    --ds-primary-gradient-start: color-mix(in oklch, var(--primary) 32%, var(--background) 68%);
    --ds-primary-gradient-end: color-mix(in oklch, var(--primary) 18%, var(--background) 82%);
    --ds-primary-gradient-hover-start: color-mix(in oklch, var(--primary) 46%, var(--background) 54%);
    --ds-primary-gradient-hover-end: color-mix(in oklch, var(--primary) 30%, var(--background) 70%);
    --ds-muted-text: color-mix(in oklch, var(--foreground) 65%, transparent);
    --ds-subtle-text: color-mix(in oklch, var(--foreground) 42%, transparent);
    --ds-hover-row: color-mix(in oklch, var(--muted) 60%, transparent 40%);
    --ds-error-bg: color-mix(in oklch, var(--destructive) 12%, transparent);
    --ds-error-border: color-mix(in oklch, var(--destructive) 38%, var(--border) 62%);
    --ds-circle-glow: color-mix(in oklch, var(--primary) 22%, transparent);
    display: flex;
    flex-direction: column;
    align-items: center;
    min-height: 100vh;
    padding: 4rem 1.5rem 2rem;
    gap: 2rem;
    background: var(--background);
    color: var(--foreground);
  }

  /* legacy styles kept for layout correctness */
</style>
