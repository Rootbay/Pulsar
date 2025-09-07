<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link
    href="https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<div class="login-container">
  <section>
    <div class="textContainer">
      <h1>Unlock Pulsar Pass</h1>
      <p>Enter your master password to unlock your vault.</p>
    </div>

    <input type="password" class="master-password-input" placeholder="Master Password"
        bind:value={password}
        on:keydown={(e) => { if (e.key === 'Enter') handleUnlock(); }}
        disabled={isUnlocking} />

    {#if loginError}
      <p class="error">{loginError}</p>
    {/if}

    <div class="buttonContainer">
      <button type="button" class="navButton continue" on:click={handleUnlock} disabled={isUnlocking} >
        {isUnlocking ? 'Unlocking...' : 'Continue'}
      </button>
      <button type="button" class="navButton" on:click={handleChangeDatabase} >
        Change Database
      </button>
    </div>
    <div class="circle-bg"></div>
  </section>
</div>

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { isLocked, isDatabaseLoaded } from '../../stores';

  let password = '';
  let loginError: string | null = null;
  let isUnlocking = false;

  async function handleUnlock() {
    if (isUnlocking || !password.trim()) {
      return;
    }

    isUnlocking = true;
    loginError = null;

    try {
      await invoke('unlock', { password: password });

      isLocked.set(false);

    } catch (error) {
      console.error("Unlock failed:", error);
      loginError = typeof error === 'string' ? error : 'An unknown error occurred.';
    } finally {
      isUnlocking = false;
    }
  }

  async function handleChangeDatabase() {
    await invoke('lock');
    isDatabaseLoaded.set(false);
    isLocked.set(true);
  }
</script>

<style>
  * {
    padding: 0;
    margin: 0;
    box-sizing: border-box;
  }

  :global(body) {
    background-color: var(--main-bg);
    font-family: "Inter", sans-serif;
    color: var(--white);
    min-height: 100vh;
  }

  .login-container {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    min-height: 100vh;
    padding-top: 4rem;
  }

  section {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 80%;
    max-width: 490px;
    gap: 2rem;
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

  .master-password-input {
    width: 100%;
    height: 50px;
    background-color: #101012;
    border: 1.5px solid #393943;
    border-radius: 6px;
    padding: 0 1rem;
    font-size: 16px;
    color: var(--white);
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
    background: linear-gradient(to right, #3a3a47, #1f1f24);
    transition: opacity 0.3s ease;
    border: none;
  }
  .buttonContainer .navButton:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .buttonContainer .navButton:not(:disabled):hover {
    opacity: 1;
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

  @media (max-width: 480px) {
    .textContainer h1 {
      font-size: 20px;
    }
    .textContainer p {
      font-size: 12px;
    }
    .master-password-input {
        height: 45px;
    }
    .buttonContainer .navButton {
      height: 44px;
      font-size: 14px;
    }
  }
</style>