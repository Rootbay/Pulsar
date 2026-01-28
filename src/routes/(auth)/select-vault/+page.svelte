<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { callBackend } from '$lib/utils/backend';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import ImportManagerPopup from '$lib/components/ImportManagerPopup.svelte';
  import { appState } from '$lib/stores';
  import { settings } from '$lib/stores/appSettings.svelte';
  import { addRecentDatabase, removeRecentDatabase } from '$lib/stores/recentDatabases.svelte';
  import { importVaultBackup, notifyVaultRefresh } from '$lib/utils/backup';
  import type { ImportVaultProgressStage } from '$lib/utils/backup';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { parseError } from '$lib/utils/error';
  import { Spinner } from '$lib/components/ui/spinner/index.js';
  import { Button } from '$lib/components/ui/button';

  import {
    Plus,
    FolderOpen,
    Database,
    Upload,
    ArrowRight,
    Clock,
    MailWarning
  } from '@lucide/svelte';

  let locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as I18nKey, vars);

  let error = $state<string | null>(null);
  let importMessage = $state<string | null>(null);
  let importProgress = $state<string | null>(null);
  let showImportPopup = $state(false);
  let lastAttemptedPath = $state<string | null>(null);
  let triedElevated = $state(false);
  let isLoading = $state(false);
  let recentDbPaths = $derived(settings.state.recentDatabases || []);

  const importProgressMessages = $derived<Record<ImportVaultProgressStage, string>>({
    decrypting: t('decryptingBackup'),
    restoring: t('restoringVault')
  });

  const hasAccessDeniedError = $derived(Boolean(error?.toLowerCase().includes('access is denied')));

  $effect(() => {
    if (browser && appState.isDatabaseLoaded) {
      if (appState.needsPasswordSetup) {
        goto('/setup', { replaceState: true });
      } else if (appState.isLocked) {
        goto('/login', { replaceState: true });
      } else {
        goto('/', { replaceState: true });
      }
    }
  });

  const loadAndCheckDatabase = async (path: string) => {
    if (!path) {
      console.warn('loadAndCheckDatabase called with empty path');
      return;
    }

    if (isLoading) return;

    error = null;
    lastAttemptedPath = path;
    appState.totpVerified = false;
    isLoading = true;

    try {
      await callBackend('switch_database', { dbPath: path });

      const isConfigured = await callBackend<boolean>('is_master_password_configured');

      if (isConfigured) {
        appState.isLocked = true;
        appState.needsPasswordSetup = false;
      } else {
        appState.isLocked = false;
        appState.needsPasswordSetup = true;
      }

      appState.isDatabaseLoaded = true;
      await addRecentDatabase(path);
    } catch (cause: unknown) {
      console.error('Failed to load database:', cause);
      error = parseError(cause) || t('failedToLoadVault');

      const message = parseError(cause);
      if (!triedElevated && message.toLowerCase().includes('access is denied')) {
        triedElevated = true;
        void attemptElevatedCopy();
      }
    } finally {
      isLoading = false;
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
    } catch (cause: unknown) {
      console.error('Elevated copy failed:', cause);
      error = parseError(cause) || t('elevatedCopyFailed');
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
        title: t('selectVaultDialogTitle'),
        filters: [{ name: t('vaultFileFilterName'), extensions: ['psec'] }],
        multiple: false
      });

      if (picked) {
        const path = typeof picked === 'string' ? picked : (picked as Record<string, unknown>).path;
        if (typeof path === 'string') {
          await loadAndCheckDatabase(path);
        } else {
          console.warn('Dialog returned unexpected type:', typeof picked, picked);
        }
      } else {
        console.log('File selection cancelled');
      }
    } catch (cause) {
      console.error('Failed to open or switch vault:', cause);
    }
  };

  const createNewVault = async () => {
    try {
      const picked = await save({
        title: t('createVaultDialogTitle'),
        filters: [{ name: t('vaultFileFilterName'), extensions: ['psec'] }]
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

  const handleImportSelected = async (detail: { importedPath: string; passphrase: string }) => {
    showImportPopup = false;
    importMessage = null;
    importProgress = null;

    const importedPath = typeof detail?.importedPath === 'string' ? detail.importedPath : null;
    const passphrase = typeof detail?.passphrase === 'string' ? detail.passphrase.trim() : '';

    if (!importedPath) {
      error = t('noBackupFileSelected');
      return;
    }

    if (!passphrase) {
      error = t('passphraseRequired');
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

      const itemLabel = itemCount === 1 ? t('itemSingular') : t('itemPlural');
      const tagLabel = tagCount === 1 ? t('tagSingular') : t('tagPlural');
      importMessage = `${t('vaultRestoredPrefix')} ${itemCount} ${itemLabel} ${t(
        'and'
      )} ${tagCount} ${tagLabel} ${t('vaultRestoredSuffix')}`;

      importProgress = null;
      notifyVaultRefresh('import');
    } catch (cause: unknown) {
      console.error('Failed to import backup:', cause);
      error = ((cause as Record<string, unknown>).message as string) || t('importFailed');
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
        removeRecentDatabase(path);
        error = t('selectedRecentNotFound');
      }
    } catch (cause) {
      console.error(`Failed to check file existence for ${path}:`, cause);
      error = t('fileCheckError');
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
  <div class="mx-auto flex w-full max-w-4xl flex-col items-center justify-start px-6 pt-20 pb-20">
    <div class="mb-12 w-full text-center">
      <img src="/logo.png" alt="Pulsar Logo" class="mx-auto mb-6 h-20 w-20" />
      <h1 class="text-foreground text-4xl font-bold tracking-tight">
        {t('welcomeTitle')}
      </h1>
      <p class="text-muted-foreground mt-3 text-lg">
        {t('welcomeSubtitle')}
      </p>
    </div>

    <div class="grid w-full gap-6 md:grid-cols-3">
      <button
        type="button"
        class="group border-border/60 bg-card/50 hover:border-primary/50 hover:bg-primary/5 flex cursor-pointer flex-col items-start rounded-2xl border p-6 text-left transition-all disabled:pointer-events-none disabled:opacity-50"
        disabled={isLoading}
        onclick={createNewVault}
      >
        <div
          class="bg-primary/10 text-primary group-hover:bg-primary/20 mb-4 flex h-12 w-12 items-center justify-center rounded-xl transition-colors"
        >
          <Plus class="h-6 w-6" />
        </div>
        <h3 class="text-foreground text-lg font-semibold">
          {t('createNewVaultTitle')}
        </h3>
        <p class="text-muted-foreground mt-2 text-sm leading-relaxed">
          {t('createNewVaultDesc')}
        </p>
        <div class="text-primary mt-auto flex items-center gap-2 pt-4 text-xs font-medium">
          {t('createNewVaultCta')}
          <ArrowRight class="h-3 w-3 transition-transform group-hover:translate-x-1" />
        </div>
      </button>

      <button
        type="button"
        class="group border-border/60 bg-card/50 hover:border-primary/50 hover:bg-primary/5 flex cursor-pointer flex-col items-start rounded-2xl border p-6 text-left transition-all disabled:pointer-events-none disabled:opacity-50"
        disabled={isLoading}
        onclick={selectExistingVault}
      >
        <div
          class="bg-primary/10 text-primary group-hover:bg-primary/20 mb-4 flex h-12 w-12 items-center justify-center rounded-xl transition-colors"
        >
          <FolderOpen class="h-6 w-6" />
        </div>
        <h3 class="text-foreground text-lg font-semibold">
          {t('openExistingTitle')}
        </h3>
        <p class="text-muted-foreground mt-2 text-sm leading-relaxed">
          {t('openExistingDesc')}
        </p>
        <div class="text-primary mt-auto flex items-center gap-2 pt-4 text-xs font-medium">
          {t('openExistingCta')}
          <ArrowRight class="h-3 w-3 transition-transform group-hover:translate-x-1" />
        </div>
      </button>

      <button
        type="button"
        class="group border-border/60 bg-card/50 hover:border-primary/50 hover:bg-primary/5 flex cursor-pointer flex-col items-start rounded-2xl border p-6 text-left transition-all disabled:pointer-events-none disabled:opacity-50"
        disabled={isLoading}
        onclick={handleImportFile}
      >
        <div
          class="bg-primary/10 text-primary group-hover:bg-primary/20 mb-4 flex h-12 w-12 items-center justify-center rounded-xl transition-colors"
        >
          <Upload class="h-6 w-6" />
        </div>
        <h3 class="text-foreground text-lg font-semibold">
          {t('migrateTitle')}
        </h3>
        <p class="text-muted-foreground mt-2 text-sm leading-relaxed">
          {t('migrateDesc')}
        </p>
        <div class="text-primary mt-auto flex items-center gap-2 pt-4 text-xs font-medium">
          {t('migrateCta')}
          <ArrowRight class="h-3 w-3 transition-transform group-hover:translate-x-1" />
        </div>
      </button>
    </div>

    <div class="mt-12 grid w-full gap-8 md:grid-cols-2">
      <div class="space-y-4">
        <div class="flex items-center gap-2 px-1">
          <Clock class="text-muted-foreground h-4 w-4" />
          <h2 class="text-foreground text-sm font-semibold tracking-wider uppercase">
            {t('recentlyOpenedTitle')}
          </h2>
        </div>

        {#if recentDbPaths.length > 0}
          <div class="space-y-2">
            {#each recentDbPaths as dbPath (dbPath)}
              <button
                type="button"
                class="border-border/40 bg-card/30 hover:border-primary/30 hover:bg-primary/5 group flex w-full cursor-pointer items-center justify-between rounded-xl border px-4 py-3 transition-all disabled:pointer-events-none disabled:opacity-50"
                disabled={isLoading}
                onclick={() => selectRecentDatabase(dbPath)}
              >
                <div class="flex items-center gap-3">
                  <Database class="text-muted-foreground h-4 w-4" />
                  <div class="text-left">
                    <p
                      class="text-foreground group-hover:text-primary text-sm font-medium transition-colors"
                    >
                      {basename(dbPath)}
                    </p>
                    <p class="text-muted-foreground max-w-50 truncate text-[11px]" title={dbPath}>
                      {dbPath}
                    </p>
                  </div>
                </div>
                <ArrowRight
                  class="text-muted-foreground h-4 w-4 opacity-0 transition-all group-hover:translate-x-1 group-hover:opacity-100"
                />
              </button>
            {/each}
          </div>
        {:else}
          <div
            class="border-border/40 bg-muted/5 flex flex-col items-center justify-center rounded-2xl border border-dashed py-8 text-center"
          >
            <Database class="text-muted-foreground/30 mb-2 h-8 w-8" />
            <p class="text-muted-foreground text-xs italic">
              {t('noRecentVaults')}
            </p>
          </div>
        {/if}
      </div>

      <div class="space-y-4">
        {#if error || importProgress || importMessage || isLoading}
          <div class="flex items-center gap-2 px-1">
            <MailWarning class="text-muted-foreground h-4 w-4" />
            <h2 class="text-foreground text-sm font-semibold tracking-wider uppercase">
              {t('statusTitle')}
            </h2>
          </div>

          <div class="space-y-3">
            {#if isLoading}
              <div
                class="border-primary/40 bg-primary/5 flex items-center gap-3 rounded-xl border p-4"
              >
                <Spinner class="text-primary h-4 w-4" />
                <p class="text-foreground text-sm font-medium">
                  {t('settingsVaultLoading')}
                </p>
              </div>
            {/if}

            {#if error}
              <div class="border-destructive/40 bg-destructive/10 rounded-xl border p-4">
                <p class="text-destructive text-sm leading-relaxed font-medium">{error}</p>
                {#if hasAccessDeniedError}
                  <Button
                    variant="outline"
                    size="sm"
                    class="border-destructive/20 bg-destructive/5 hover:bg-destructive/10 mt-3 w-full"
                    onclick={attemptElevatedCopy}
                  >
                    {t('tryElevatedAccess')}
                  </Button>
                {/if}
              </div>
            {/if}

            {#if importProgress}
              <div
                class="border-primary/40 bg-primary/5 flex items-center gap-3 rounded-xl border p-4"
              >
                <Spinner class="text-primary h-4 w-4" />
                <p class="text-foreground text-sm font-medium">{importProgress}</p>
              </div>
            {/if}

            {#if importMessage}
              <div class="rounded-xl border border-emerald-500/40 bg-emerald-500/10 p-4">
                <p class="text-sm leading-relaxed font-medium text-emerald-600">{importMessage}</p>
              </div>
            {/if}
          </div>
        {:else}
          <div
            class="bg-primary/5 border-primary/10 flex h-full flex-col justify-center rounded-2xl border p-6"
          >
            <h4 class="text-primary text-sm font-bold tracking-widest uppercase">
              {t('pulsarTipTitle')}
            </h4>
            <p class="text-muted-foreground mt-2 text-sm leading-relaxed italic">
              {t('pulsarTipBody')}
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

  :global(html) {
    overflow-y: scroll !important;
  }

  :global(body) {
    font-family: 'Inter', sans-serif;
  }
</style>
