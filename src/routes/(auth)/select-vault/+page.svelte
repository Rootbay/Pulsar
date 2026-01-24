<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { callBackend } from '$lib/utils/backend';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import ImportManagerPopup from '$lib/components/ImportManagerPopup.svelte';
  import { isDatabaseLoaded, isLocked, needsPasswordSetup, totpVerified } from '$lib/stores';
  import { recentDatabases } from '$lib/stores/recentDatabases';
  import { importVaultBackup, notifyVaultRefresh } from '$lib/utils/backup';
  import type { ImportVaultProgressStage } from '$lib/utils/backup';
  import { currentLocale } from '$lib/i18n';
  import { Spinner } from '$lib/components/ui/spinner/index.js';

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);

  let locale = $derived($currentLocale as 'en' | 'sv');
  let error = $state<string | null>(null);
  let importMessage = $state<string | null>(null);
  let importProgress = $state<string | null>(null);
  let showImportPopup = $state(false);
  let lastAttemptedPath = $state<string | null>(null);
  let triedElevated = $state(false);

  const importProgressMessages = $derived<Record<ImportVaultProgressStage, string>>({
    decrypting: t(locale, 'Decrypting backup…', 'Dekrypterar backup…'),

    restoring: t(locale, 'Restoring vault contents…', 'Återställer valvets innehåll…')
  });

  const hasAccessDeniedError = $derived(Boolean(error?.toLowerCase().includes('access is denied')));

  $effect(() => {
    if (browser && $isDatabaseLoaded) {
      if ($needsPasswordSetup) {
        goto('/setup', { replaceState: true });
      } else if ($isLocked) {
        goto('/login', { replaceState: true });
      } else {
        goto('/', { replaceState: true });
      }
    }
  });

  const loadAndCheckDatabase = async (path: string) => {
    if (!path) {
      return;
    }

    error = null;

    lastAttemptedPath = path;

    totpVerified.set(false);

    try {
      await callBackend('switch_database', { dbPath: path });

      const isConfigured = await callBackend<boolean>('is_master_password_configured');

      if (isConfigured) {
        $isLocked = true;

        $needsPasswordSetup = false;
      } else {
        $isLocked = false;

        $needsPasswordSetup = true;
      }

      $isDatabaseLoaded = true;

      recentDatabases.addRecentDatabase(path);
    } catch (cause: any) {
      console.error('Failed to load database:', cause);

      error =
        cause.message ||
        t(locale, 'Failed to load the selected vault.', 'Misslyckades att ladda det valda valvet.');

      const message = cause.message || cause.toString() || '';

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
      const destination = await callBackend<string>('elevated_copy', { src: lastAttemptedPath });

      if (typeof destination === 'string') {
        await loadAndCheckDatabase(destination);
      }
    } catch (cause: any) {
      console.error('Elevated copy failed:', cause);

      error =
        cause.message ||
        t(
          locale,

          'Failed to copy the vault with elevated permissions.',

          'Misslyckades kopiera valvet med förhöjda rättigheter.'
        );
    }
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

  const handleImportSelected = async (detail: any) => {
    showImportPopup = false;

    importMessage = null;

    importProgress = null;

    const importedPath = typeof detail?.importedPath === 'string' ? detail.importedPath : null;

    const passphrase = typeof detail?.passphrase === 'string' ? detail.passphrase.trim() : '';

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
    } catch (cause: any) {
      console.error('Failed to import backup:', cause);

      error =
        cause.message ||
        t(
          locale,

          'Failed to import the selected backup. Please verify the passphrase and try again.',

          'Misslyckades att importera den valda backupen. Kontrollera lösenfrasen och försök igen.'
        );

      importProgress = null;
    }
  };

  const selectRecentDatabase = async (path: string) => {
    if (!path) {
      return;
    }

    try {
      const exists = await callBackend<boolean>('check_file_exists', { path });

      if (exists) {
        await loadAndCheckDatabase(path);
      } else {
        recentDatabases.removeRecentDatabase(path);

        error = t(
          locale,
          'Selected recent file could not be found.',
          'Den valda senaste filen kunde inte hittas.'
        );
      }
    } catch (cause) {
      console.error(`Failed to check file existence for ${path}:`, cause);

      error = t(
        locale,
        'An error occurred while checking the file.',
        'Ett fel uppstod när filen kontrollerades.'
      );
    }
  };
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link
    href="https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<div class="bg-background relative min-h-screen">
  <div class="pointer-events-none absolute inset-0 -z-10 overflow-hidden">
    <div class="bg-primary/10 absolute top-[12%] left-[8%] h-88 w-88 rounded-full blur-3xl"></div>
    <div
      class="bg-muted/40 absolute right-[10%] bottom-[10%] h-72 w-[18rem] rounded-full blur-2xl"
    ></div>
  </div>

  <div
    class="mx-auto flex min-h-[65vh] w-full max-w-122.5 translate-x-4.75 flex-col items-center justify-start px-6 pt-15"
  >
    <div class="w-full text-center">
      <h1 class="text-foreground text-3xl font-semibold">
        {t(locale, 'Welcome to Pulsar Pass', 'Välkommen till Pulsar Pass')}
      </h1>
      <p class="text-muted-foreground mt-2 text-sm">
        {t(
          locale,
          'Please select an existing vault or create a new one',
          'Välj ett befintligt valv eller skapa ett nytt'
        )}
      </p>
    </div>

    <div class="mt-8 w-full space-y-2.25">
      <button
        type="button"
        class="mx-auto h-11.75 w-full max-w-122.5 rounded-full bg-white/10 px-5 text-sm text-white transition-colors hover:bg-white/15 focus:outline-none"
        onclick={selectExistingVault}
      >
        {t(locale, 'Select existing vault', 'Välj befintligt valv')}
      </button>

      <button
        type="button"
        class="mx-auto h-11.75 w-full max-w-122.5 rounded-full bg-transparent px-5 text-sm text-white/70 transition-colors hover:bg-white/10 hover:text-white focus:outline-none"
        onclick={createNewVault}
      >
        {t(locale, 'Create new vault', 'Skapa nytt valv')}
      </button>

      <button
        type="button"
        class="mx-auto h-11.75 w-full max-w-122.5 rounded-full bg-transparent px-5 text-sm text-white/70 transition-colors hover:bg-white/10 hover:text-white focus:outline-none"
        onclick={handleImportFile}
      >
        {t(locale, 'Migrate vault', 'Migrera valv')}
      </button>
    </div>

    {#if error}
      <div
        class="border-destructive/40 bg-destructive/10 mt-6 w-full rounded-xl border p-3 text-left"
      >
        <p class="text-destructive text-sm font-medium">{error}</p>
        {#if hasAccessDeniedError}
          <div class="mt-3">
            <button
              type="button"
              class="border-border/60 bg-muted/20 text-foreground hover:bg-muted/30 h-9 rounded-full border px-4 text-sm"
              onclick={attemptElevatedCopy}
            >
              {t(
                locale,
                'Attempt elevated copy into app folder',
                'Försök kopiera med förhöjda rättigheter till appmappen'
              )}
            </button>
          </div>
        {/if}
      </div>
    {/if}

    {#if importProgress}
      <div
        class="border-border/60 bg-muted/20 mt-3 flex w-full items-center gap-3 rounded-xl border p-3"
      >
        <Spinner class="text-muted-foreground h-4 w-4" aria-hidden="true" />
        <p class="text-foreground text-sm">{importProgress}</p>
      </div>
    {/if}

    {#if importMessage}
      <div class="border-primary/40 bg-primary/10 mt-3 w-full rounded-xl border p-3">
        <p class="text-foreground text-sm font-medium">{importMessage}</p>
      </div>
    {/if}

    <div class="mt-10 w-full text-left">
      <h2 class="text-foreground text-sm font-medium">
        {t(locale, 'Recent vault', 'Senaste valv')}
      </h2>
      {#if $recentDatabases.length > 0}
        <ul class="mx-auto mt-3 w-full max-w-122.5 space-y-2">
          {#each $recentDatabases as dbPath (dbPath)}
            <li>
              <button
                type="button"
                class="text-muted-foreground hover:text-foreground w-full rounded-lg bg-white/5 px-3 py-2 text-left text-[13px] transition hover:bg-white/10"
                title={dbPath}
                onclick={() => selectRecentDatabase(dbPath)}
              >
                {basename(dbPath)}
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="text-muted-foreground mt-2 text-xs">
          {t(locale, 'No recent databases.', 'Inga senaste databaser.')}
        </p>
      {/if}
    </div>
  </div>
</div>

<ImportManagerPopup bind:show={showImportPopup} onimportSelected={handleImportSelected} />

<style>
  * {
    box-sizing: border-box;
  }

  :global(body) {
    font-family: 'Inter', sans-serif;
  }
</style>
