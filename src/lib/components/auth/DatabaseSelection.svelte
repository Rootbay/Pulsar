<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap" rel="stylesheet" />
</svelte:head>

<div class="container">
  <section>
    <div class="textContainer">
      <h1>Welcome to Pulsar Pass</h1>
      <p>Please select an existing one or create a new vault.</p>
    </div>

    <div class="buttonContainer">
      <button
        type="button"
        class="navButton"
        id="navButton1"
        on:click={selectExistingVault}
      >
        Select existing vault
      </button>
      <button
        type="button"
        class="navButton"
        on:click={createNewVault}
      >
        Create new vault
      </button>
      <button
        type="button"
        class="navButton"
        on:click={handleImportFile}
      >
        Migrate vault
      </button>
    </div>
    <div class="circle-bg"></div>
  </section>

  <div class="recent-databases-section">
    <div class="textContainer">
      <h2>Recent vault</h2>
      <button class="clear-recent" on:click={clearAllRecent} title="Remove all recent entries">
        <Icon path={iconPaths.clear} size="16" color="#fff" />
      </button>
    </div>
    <div class="recent-databases-list">
        {#if $recentDatabases.length > 0}
          {#each $recentDatabases as dbPath (dbPath)}
            <div class="recent-db-item" title={dbPath}>
              <button class="db-name" on:click={() => selectRecentDatabase(dbPath)}>
                {basename(dbPath)}
              </button>
              <button class="forget-btn" on:click|stopPropagation={() => forgetRecent(dbPath)} title="Forget this entry">
                <Icon path={iconPaths.clear} size="16" color="#fff" />
              </button>
            </div>
          {/each}
        {:else}
          <p class="no-recent-databases">No recent databases.</p>
        {/if}
    </div>
  </div>
  {#if error}
    <div class="error-block">
      <p class="error-text">{error}</p>
      {#if error.toLowerCase().includes('access is denied')}
        <div style="display:flex;gap:8px;flex-direction:column;align-items:center;">
          <button class="navButton" on:click={attemptElevatedCopy}>Attempt elevated copy into app folder</button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { recentDatabases } from '$lib/stores/recentDatabases';
  import ImportManagerPopup from '$lib/components/ImportManagerPopup.svelte';
  import { isDatabaseLoaded, needsPasswordSetup, isLocked } from '$lib/stores';
  
  import Icon from '$lib/components/ui/Icon.svelte';
  import { iconPaths } from '$lib/icons';

  
  let error: string | null = null;
  let showImportPopup = false;
  let lastAttemptedPath: string | null = null;
  let triedElevated = false;

  async function loadAndCheckDatabase(path: string) {
    if (!path) return;
    error = null;
    
    try {
      await invoke('switch_database', { dbPath: path });
  lastAttemptedPath = path;
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

    } catch (e) {
      console.error('Failed to load database:', e);
      if (typeof e === 'string') {
        error = e;
      } else {
        try {
          error = JSON.stringify(e);
        } catch (_) {
          error = 'Failed to load the selected vault.';
        }
      }
      lastAttemptedPath = path;
      const errStr = (typeof e === 'string') ? e : (JSON.stringify(e) || '');
      if (!triedElevated && errStr.toLowerCase().includes('access is denied')) {
        triedElevated = true;
        attemptElevatedCopy();
      }
    } finally {
      // no-op
    }
  }

  async function attemptElevatedCopy() {
    if (!lastAttemptedPath) return;
    try {
      const dest = await invoke<string>('elevated_copy', { src: lastAttemptedPath });
      if (typeof dest === 'string') {
        await loadAndCheckDatabase(dest as string);
      }
    } catch (e) {
      console.error('Elevated copy failed:', e);
      if (typeof e === 'string') error = e; else error = JSON.stringify(e);
    }
  }

  

  async function forgetRecent(path: string) {
    if (!path) return;
    recentDatabases.removeRecentDatabase(path);
  }

  async function clearAllRecent() {
    recentDatabases.clearRecentDatabases();
  }

  function basename(path: string) {
    if (!path) return '';
    const parts = path.split(/[\\/\\]/);
    return parts[parts.length - 1] || path;
  }

  async function selectExistingVault() {
    try {
      const picked = await open({
        title: 'Select a Pulsar Vault',
        filters: [{ name: 'Pulsar Vault', extensions: ['pulsar'] }],
        multiple: false,
      });

      if (typeof picked === 'string') {
        await loadAndCheckDatabase(picked);
      }
    } catch (error) {
      console.error("Failed to open or switch vault:", error);
    }
  }

  async function createNewVault() {
    try {
      const picked = await save({
        title: 'Create a new Pulsar Vault',
        filters: [{ name: 'Pulsar Vault', extensions: ['pulsar'] }],
      });
      if (picked) {
        const finalPath = picked.endsWith('.pulsar') ? picked : `${picked}.pulsar`;
        await loadAndCheckDatabase(finalPath);
      }
    } catch (e) {
      console.error('Save failed:', e);
    }
  }

  function handleImportFile() {
    showImportPopup = true;
  }

  async function handleImportSelected(event: CustomEvent<{ importedPath: string }>) {
    showImportPopup = false;
    const importedPath = event.detail.importedPath;
    if (importedPath) {
      await loadAndCheckDatabase(importedPath);
    }
  }

  async function selectRecentDatabase(path: string) {
    if (!path) return;
    try {
      const exists = await invoke<boolean>('check_file_exists', { path });
      if (exists) {
        await loadAndCheckDatabase(path);
      } else {
  recentDatabases.removeRecentDatabase(path);
        error = "Selected recent file could not be found.";
      }
    } catch (e) {
      console.error(`Failed to check file existence for ${path}:`, e);
      error = "An error occurred while checking the file.";
    }
  }
</script>

<ImportManagerPopup bind:show={showImportPopup} on:importSelected={handleImportSelected} />

<style>
  * {
    padding: 0;
    margin: 0;
    box-sizing: border-box;
    font-family: "Inter", sans-serif;
  }

  :global(body) {
    background-color: var(--main-bg);
    font-family: "Inter", sans-serif;
    color: var(--white);
    min-height: 100vh;
  }

  .container {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    min-height: 100vh;
    padding-top: 4rem;
    flex-direction: column;
    align-items: center;
  }

  section {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 80%;
    max-width: 490px;
    gap: 2rem;
    margin-bottom: 2rem;
  }

  .textContainer {
    text-align: center;
  }
  .textContainer h1 {
    font-size: 25px;
    margin-bottom: 6px;
  }
  .textContainer p {
    font-size: 14px;
    font-weight: 300;
  }
  .textContainer h2 {
    font-size: 16px;
    font-weight: 300;
    margin-bottom: 0;
    color: var(--white);
  }

  .recent-databases-section {
    width: 100%;
    max-width: 490px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    margin-bottom: 2rem;
  }

  .recent-databases-section .textContainer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  .buttonContainer {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .buttonContainer .navButton {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 47px;
    text-decoration: none;
    color: var(--white);
    cursor: pointer;
    border-radius: 23px;
    background: transparent;
    border: none;
    font-weight: 300;
    opacity: 0.4;
  }
  .buttonContainer .navButton:hover {
    background: linear-gradient(to right, #3a3a47, #1f1f24);
    opacity: 1;
  }
  .buttonContainer .navButton:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .circle-bg {
    position: absolute;
    opacity: 0.8;
    top: 65%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: -1;
    width: min(90vw, 638px);
    aspect-ratio: 1 / 1;
    border-radius: 50%;
    background: radial-gradient(
      circle,
      rgba(175, 175, 255, 0.08) 0%,
      rgba(175, 175, 255, 0) 70%
    );
  }

  .recent-databases-list {
    display: flex;
    flex-direction: column;
    gap: 5px;
    width: 100%;
    min-height: 110px;
    max-height: 220px;
    overflow-y: auto;
    border-radius: 8px;
    padding: 0 0;
  }

  .clear-recent {
    background: none;
    border: none;
    cursor: pointer;
    transition: opacity 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .clear-recent:hover {
    opacity: 1;
  }

  .no-recent-databases {
    text-align: center;
    width: 100%;
    margin: auto;
  }

  .recent-db-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: var(--white);
    border-radius: 7px;
    transition: background-color 0.1s ease;
    width: 100%;
    position: relative;
    
  }

  .recent-db-item:hover {
    background-color: #1F1F24;
  }

  .db-name {
    background: none;
    border: none;
    color: #ffffff66;
    cursor: pointer;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-grow: 1;
    padding: 10px 10px;
    font-size: 13px;
    font-weight: 300;
    transition: color 0.1s ease;
  }

  .db-name:hover {
    color: var(--white);
  }

  .forget-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    margin-left: 10px;
    margin-right: 15px;
    opacity: 0;
    transition: opacity 0.2s ease;
    display: flex;
    align-items: center;
  }

  .recent-db-item:hover .forget-btn {
    opacity: 1;
  }

  .forget-btn:hover {
    opacity: 1;
  }
  #navButton1 {
    background: linear-gradient(to right, #3a3a47, #1f1f24);
    opacity: 40%;
  }
  #navButton1:hover {
    opacity: 1;
  }

  @media (max-width: 480px) {
    .textContainer h1 {
      font-size: 20px;
    }
    .textContainer p {
      font-size: 12px;
    }
    .buttonContainer .navButton {
      height: 44px;
      font-size: 14px;
    }
    .textContainer h2 {
      font-size: 18px;
    }
  }
</style>
