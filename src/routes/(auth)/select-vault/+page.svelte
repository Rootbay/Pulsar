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
  import { Button } from '$lib/components/ui/button';

  import {
    Plus,
    FolderOpen,
    Database,
    Upload,
    ArrowRight,
    Clock,
    FileWarning,
    Shield
  } from '@lucide/svelte';

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

  <div class="mx-auto flex w-full max-w-4xl flex-col items-center justify-start px-6 pt-20 pb-20">
    <div class="mb-12 w-full text-center">
      <div
        class="bg-primary/10 text-primary mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-2xl ring-1 ring-primary/20"
      >
        <Shield class="h-8 w-8" />
      </div>
      <h1 class="text-foreground text-4xl font-bold tracking-tight">
        {t(locale, 'Welcome to Pulsar', 'Välkommen till Pulsar')}
      </h1>
      <p class="text-muted-foreground mt-3 text-lg">
        {t(
          locale,
          'Secure your digital life with professional-grade encryption.',
          'Säkra ditt digitala liv med kryptering av professionell kvalitet.'
        )}
      </p>
    </div>

    <div class="grid w-full gap-6 md:grid-cols-3">
      <button
        type="button"
        class="group border-border/60 bg-card/50 hover:border-primary/50 hover:bg-primary/5 flex cursor-pointer flex-col items-start rounded-2xl border p-6 text-left transition-all"
        onclick={createNewVault}
      >
        <div
          class="bg-primary/10 text-primary mb-4 flex h-12 w-12 items-center justify-center rounded-xl transition-colors group-hover:bg-primary/20"
        >
          <Plus class="h-6 w-6" />
        </div>
        <h3 class="text-foreground text-lg font-semibold">
          {t(locale, 'Create New Vault', 'Skapa nytt valv')}
        </h3>
        <p class="text-muted-foreground mt-2 text-sm leading-relaxed">
          {t(
            locale,
            'Start fresh with a new secure database file.',
            'Börja på nytt med en ny säker databasfil.'
          )}
        </p>
        <div class="text-primary mt-auto flex items-center gap-2 pt-4 text-xs font-medium">
          {t(locale, 'Get Started', 'Kom igång')}
          <ArrowRight class="h-3 w-3 transition-transform group-hover:translate-x-1" />
        </div>
      </button>

      <button
        type="button"
        class="group border-border/60 bg-card/50 hover:border-primary/50 hover:bg-primary/5 flex cursor-pointer flex-col items-start rounded-2xl border p-6 text-left transition-all"
        onclick={selectExistingVault}
      >
        <div
          class="bg-primary/10 text-primary mb-4 flex h-12 w-12 items-center justify-center rounded-xl transition-colors group-hover:bg-primary/20"
        >
          <FolderOpen class="h-6 w-6" />
        </div>
        <h3 class="text-foreground text-lg font-semibold">
          {t(locale, 'Open Existing', 'Öppna befintligt')}
        </h3>
        <p class="text-muted-foreground mt-2 text-sm leading-relaxed">
          {t(
            locale,
            'Browse your files for an existing Pulsar vault.',
            'Bläddra bland dina filer efter ett befintligt Pulsar-valv.'
          )}
        </p>
        <div class="text-primary mt-auto flex items-center gap-2 pt-4 text-xs font-medium">
          {t(locale, 'Select File', 'Välj fil')}
          <ArrowRight class="h-3 w-3 transition-transform group-hover:translate-x-1" />
        </div>
      </button>

      <button
        type="button"
        class="group border-border/60 bg-card/50 hover:border-primary/50 hover:bg-primary/5 flex cursor-pointer flex-col items-start rounded-2xl border p-6 text-left transition-all"
        onclick={handleImportFile}
      >
        <div
          class="bg-primary/10 text-primary mb-4 flex h-12 w-12 items-center justify-center rounded-xl transition-colors group-hover:bg-primary/20"
        >
          <Upload class="h-6 w-6" />
        </div>
        <h3 class="text-foreground text-lg font-semibold">
          {t(locale, 'Migrate / Restore', 'Migrera / Återställ')}
        </h3>
        <p class="text-muted-foreground mt-2 text-sm leading-relaxed">
          {t(
            locale,
            'Import data from a backup or another manager.',
            'Importera data från en backup eller en annan hanterare.'
          )}
        </p>
        <div class="text-primary mt-auto flex items-center gap-2 pt-4 text-xs font-medium">
          {t(locale, 'Start Import', 'Importera')}
          <ArrowRight class="h-3 w-3 transition-transform group-hover:translate-x-1" />
        </div>
      </button>
    </div>

    <div class="mt-12 grid w-full gap-8 md:grid-cols-2">
      <div class="space-y-4">
        <div class="flex items-center gap-2 px-1">
          <Clock class="text-muted-foreground h-4 w-4" />
          <h2 class="text-foreground text-sm font-semibold uppercase tracking-wider">
            {t(locale, 'Recently Opened', 'Senaste öppnade')}
          </h2>
        </div>

        {#if $recentDatabases.length > 0}
          <div class="space-y-2">
            {#each $recentDatabases as dbPath (dbPath)}
              <button
                type="button"
                class="cursor-pointer border-border/40 bg-card/30 hover:border-primary/30 hover:bg-primary/5 group flex w-full items-center justify-between rounded-xl border px-4 py-3 transition-all"
                onclick={() => selectRecentDatabase(dbPath)}
              >
                <div class="flex items-center gap-3">
                  <Database class="text-muted-foreground h-4 w-4" />
                  <div class="text-left">
                    <p class="text-foreground text-sm font-medium transition-colors group-hover:text-primary">
                      {basename(dbPath)}
                    </p>
                    <p class="text-muted-foreground truncate text-[11px] max-w-50" title={dbPath}>
                      {dbPath}
                    </p>
                  </div>
                </div>
                <ArrowRight class="text-muted-foreground h-4 w-4 opacity-0 transition-all group-hover:translate-x-1 group-hover:opacity-100" />
              </button>
            {/each}
          </div>
        {:else}
          <div class="border-border/40 bg-muted/5 flex flex-col items-center justify-center rounded-2xl border border-dashed py-8 text-center">
            <Database class="text-muted-foreground/30 mb-2 h-8 w-8" />
            <p class="text-muted-foreground text-xs italic">
              {t(locale, 'No recent vaults found.', 'Inga senaste valv hittades.')}
            </p>
          </div>
        {/if}
      </div>

      <div class="space-y-4">
        {#if error || importProgress || importMessage}
          <div class="flex items-center gap-2 px-1">
            <FileWarning class="text-muted-foreground h-4 w-4" />
            <h2 class="text-foreground text-sm font-semibold uppercase tracking-wider">
              {t(locale, 'Status', 'Status')}
            </h2>
          </div>

          <div class="space-y-3">
            {#if error}
              <div class="border-destructive/40 bg-destructive/10 rounded-xl border p-4">
                <p class="text-destructive text-sm font-medium leading-relaxed">{error}</p>
                {#if hasAccessDeniedError}
                  <Button
                    variant="outline"
                    size="sm"
                    class="mt-3 w-full border-destructive/20 bg-destructive/5 hover:bg-destructive/10"
                    onclick={attemptElevatedCopy}
                  >
                    {t(locale, 'Try elevated access', 'Försök med förhöjd åtkomst')}
                  </Button>
                {/if}
              </div>
            {/if}

            {#if importProgress}
              <div class="border-primary/40 bg-primary/5 flex items-center gap-3 rounded-xl border p-4">
                <Spinner class="text-primary h-4 w-4" />
                <p class="text-foreground text-sm font-medium">{importProgress}</p>
              </div>
            {/if}

            {#if importMessage}
              <div class="border-emerald-500/40 bg-emerald-500/10 rounded-xl border p-4">
                <p class="text-emerald-600 text-sm font-medium leading-relaxed">{importMessage}</p>
              </div>
            {/if}
          </div>
        {:else}
          <div class="bg-primary/5 flex h-full flex-col justify-center rounded-2xl border border-primary/10 p-6">
            <h4 class="text-primary text-sm font-bold uppercase tracking-widest">{t(locale, 'Pulsar Tip', 'Pulsar Tips')}</h4>
            <p class="text-muted-foreground mt-2 text-sm italic leading-relaxed">
              {t(
                locale,
                'Keep your vault file on an encrypted drive for maximum security. Pulsar encrypts every item, but physical security matters too.',
                'Förvara din valvfil på en krypterad disk för maximal säkerhet. Pulsar krypterar varje post, men fysisk säkerhet är också viktig.'
              )}
            </p>
          </div>
        {/if}
      </div>
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
