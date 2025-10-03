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
  import { X } from '@lucide/svelte';

  let error: string | null = null;
  let showImportPopup = false;
  let lastAttemptedPath: string | null = null;
  let triedElevated = false;

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
          error = 'Failed to load the selected vault.';
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
          error = 'Failed to copy the vault with elevated permissions.';
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

    const segments = path.split(/[\/]/);
    return segments.at(-1) || path;
  };

  const selectExistingVault = async () => {
    try {
      const picked = await open({
        title: 'Select a Pulsar Vault',
        filters: [{ name: 'Pulsar Vault', extensions: ['pulsar'] }],
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
        filters: [{ name: 'Pulsar Vault', extensions: ['pulsar'] }]
      });

      if (picked) {
        const finalPath = picked.endsWith('.pulsar') ? picked : `${picked}.pulsar`;
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
    const importedPath = typeof event.detail?.importedPath === 'string' ? event.detail.importedPath : null;

    if (importedPath) {
      await loadAndCheckDatabase(importedPath);
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
        error = 'Selected recent file could not be found.';
      }
    } catch (cause) {
      console.error(`Failed to check file existence for ${path}:`, cause);
      error = 'An error occurred while checking the file.';
    }
  };
</script>

<div class="container">
  <section>
    <div class="textContainer">
      <h1>Welcome to Pulsar Pass</h1>
      <p>Please select an existing one or create a new vault.</p>
    </div>

    <div class="buttonContainer">
      <button type="button" class="navButton" id="navButton1" onclick={selectExistingVault}>
        Select existing vault
      </button>
      <button type="button" class="navButton" onclick={createNewVault}>
        Create new vault
      </button>
      <button type="button" class="navButton" onclick={handleImportFile}>
        Migrate vault
      </button>
    </div>
    <div class="circle-bg" aria-hidden="true"></div>
  </section>

  <div class="recent-databases-section">
    <div class="textContainer">
      <h2>Recent vault</h2>
      {#if $recentDatabases.length}
        <button
          type="button"
          class="clear-recent"
          onclick={clearAllRecent}
          title="Remove all recent entries"
          aria-label="Remove all recent entries"
        >
          <X class="w-4 h-4" />
        </button>
      {/if}
    </div>
    <div class="recent-databases-list">
      {#if $recentDatabases.length > 0}
        {#each $recentDatabases as dbPath (dbPath)}
          <div class="recent-db-item" title={dbPath}>
            <button type="button" class="db-name" onclick={() => selectRecentDatabase(dbPath)}>
              {basename(dbPath)}
            </button>
            <button
              type="button"
              class="forget-btn"
              onclick={(event) => {
                event.stopPropagation();
                forgetRecent(dbPath);
              }}
              title="Forget this entry"
              aria-label="Forget this entry"
            >
              <X class="w-4 h-4" />
            </button>
          </div>
        {/each}
      {:else}
        <p class="no-recent-databases">No recent databases.</p>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error-block" role="alert">
      <p class="error-text">{error}</p>
      {#if hasAccessDeniedError}
        <div class="error-actions">
          <button type="button" class="navButton" onclick={attemptElevatedCopy}>
            Attempt elevated copy into app folder
          </button>
        </div>
      {/if}
    </div>
  {/if}
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

  section {
    position: relative;
    display: flex;
    flex-direction: column;
    width: min(90%, 490px);
    gap: 2rem;
    margin-bottom: 1rem;
    text-align: center;
  }

  .textContainer h1 {
    font-size: 1.5625rem;
    margin-bottom: 0.375rem;
    font-weight: 600;
  }

  .textContainer p {
    font-size: 0.875rem;
    font-weight: 400;
    color: var(--ds-muted-text);
  }

  .textContainer h2 {
    font-size: 1rem;
    font-weight: 500;
    margin-bottom: 0;
  }

  .recent-databases-section {
    width: min(90%, 490px);
    margin: 0 auto 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .recent-databases-section .textContainer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .buttonContainer {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .buttonContainer .navButton {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 2.9375rem;
    border-radius: 9999px;
    border: 1px solid var(--ds-control-border);
    background: var(--ds-control-bg);
    color: var(--ds-control-text);
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 500;
    transition: background-color 0.2s ease, color 0.2s ease, border-color 0.2s ease;
  }

  .buttonContainer .navButton:hover {
    background: var(--ds-control-hover);
    color: var(--primary-foreground);
    border-color: color-mix(in oklch, var(--primary) 45%, var(--border) 55%);
  }

  .buttonContainer .navButton:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  #navButton1 {
    background: linear-gradient(to right, var(--ds-primary-gradient-start), var(--ds-primary-gradient-end));
    color: var(--primary-foreground);
    border-color: color-mix(in oklch, var(--primary) 45%, var(--border) 55%);
  }

  #navButton1:hover {
    background: linear-gradient(
      to right,
      var(--ds-primary-gradient-hover-start),
      var(--ds-primary-gradient-hover-end)
    );
  }

  .circle-bg {
    position: absolute;
    opacity: 0.7;
    top: 65%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: -1;
    width: min(90vw, 32rem);
    aspect-ratio: 1 / 1;
    border-radius: 50%;
    background: radial-gradient(circle, var(--ds-circle-glow) 0%, transparent 70%);
  }

  .recent-databases-list {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    width: 100%;
    min-height: 6.875rem;
    max-height: 13.75rem;
    overflow-y: auto;
    border-radius: 0.75rem;
    padding: 0.25rem 0;
  }

  .clear-recent {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--ds-muted-text);
    transition: color 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .clear-recent:hover {
    color: var(--destructive);
  }

  .no-recent-databases {
    text-align: center;
    width: 100%;
    margin: 1.5rem auto;
    color: var(--ds-subtle-text);
    font-size: 0.9rem;
  }

  .recent-db-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-radius: 0.75rem;
    transition: background-color 0.15s ease;
    width: 100%;
    padding-right: 0.25rem;
  }

  .recent-db-item:hover {
    background-color: var(--ds-hover-row);
  }

  .db-name {
    background: none;
    border: none;
    color: var(--ds-subtle-text);
    cursor: pointer;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-grow: 1;
    padding: 0.625rem 0.75rem;
    font-size: 0.8125rem;
    font-weight: 500;
    transition: color 0.1s ease;
  }

  .db-name:hover {
    color: var(--foreground);
  }

  .forget-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.375rem 0.625rem;
    margin-left: 0.25rem;
    color: var(--ds-muted-text);
    opacity: 0;
    transition: opacity 0.2s ease, color 0.2s ease;
    display: flex;
    align-items: center;
  }

  .recent-db-item:hover .forget-btn {
    opacity: 1;
  }

  .forget-btn:hover {
    color: var(--destructive);
  }

  .error-block {
    width: min(90%, 490px);
    margin: 0 auto;
    padding: 1.25rem 1.5rem;
    border-radius: 1rem;
    background: var(--ds-error-bg);
    border: 1px solid var(--ds-error-border);
    color: var(--destructive);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    text-align: center;
  }

  .error-text {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .error-actions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    align-items: center;
  }

  .error-actions .navButton {
    width: min(100%, 22rem);
  }

  @media (max-width: 37.5rem) {
    .textContainer h1 {
      font-size: 1.25rem;
    }

    .textContainer p {
      font-size: 0.8rem;
    }

    .textContainer h2 {
      font-size: 0.95rem;
    }

    .buttonContainer .navButton {
      height: 2.75rem;
      font-size: 0.875rem;
    }
  }
</style>
