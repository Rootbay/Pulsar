<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { needsPasswordSetup, isLocked } from '$lib/stores';

  let newMasterPassword = "";
  let confirmMasterPassword = "";
  let loginError: string | null = null;
  let isSetting = false;
  let passwordStrength = "Weak";

  function checkPasswordStrength(password: string) {
    let strength = 0;
    if (password.length > 7) strength++;
    if (password.match(/[a-z]/)) strength++;
    if (password.match(/[A-Z]/)) strength++;
    if (password.match(/[0-9]/)) strength++;
    if (password.match(/[^a-zA-Z0-9]/)) strength++;

    if (strength < 3) {
      passwordStrength = "Weak";
    } else if (strength < 5) {
      passwordStrength = "Medium";
    } else {
      passwordStrength = "Strong";
    }
  }

  $: {
    checkPasswordStrength(newMasterPassword);
  }

  async function handleSetMasterPassword() {
    if (!newMasterPassword || !confirmMasterPassword) {
      loginError = "Please fill in both password fields.";
      return;
    }
    if (newMasterPassword !== confirmMasterPassword) {
      loginError = "Passwords do not match.";
      return;
    }

    loginError = null;
    isSetting = true;
    try {
      await invoke("set_master_password", { password: newMasterPassword });
      
      needsPasswordSetup.set(false);
      isLocked.set(false);

    } catch (error) {
      console.error("Set master password failed:", error);
      loginError = error as string;
    } finally {
      isSetting = false;
    }
  }
</script>

<div class="set-password-container">
  <section>
    <div class="textContainer">
      <h1>Set Master Password</h1>
      <p>This password will encrypt your vault. Don't forget it!</p>
    </div>

        <input
      type="password"
      bind:value={newMasterPassword}
      placeholder="New Master Password"
      class="master-password-input"
      on:keydown={(e) => e.key === 'Enter' && handleSetMasterPassword()}
    />
    <input
      type="password"
      bind:value={confirmMasterPassword}
      placeholder="Confirm Master Password"
      class="master-password-input"
      on:keydown={(e) => e.key === 'Enter' && handleSetMasterPassword()}
    />
    {#if newMasterPassword.length > 0}
      <p class="password-strength">Strength: {passwordStrength}</p>
    {/if}

    <div class="buttonContainer">
      <button
        type="button"
        class="navButton continue"
        on:click={handleSetMasterPassword}
        disabled={newMasterPassword.length === 0 || newMasterPassword !== confirmMasterPassword || isSetting}
      >
        {isSetting ? 'Setting...' : 'Set Password'}
      </button>
    </div>
    {#if loginError}
      <p class="error-message">{loginError}</p>
    {/if}
    <div class="circle-bg"></div>
  </section>
</div>

<style>
  .set-password-container {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    min-height: 100vh;
    padding-top: 4rem;
  }

  .set-password-container section {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 80%;
    max-width: 490px;
    gap: 2rem;
  }

  .set-password-container .textContainer {
    text-align: center;
  }
  .set-password-container .textContainer h1 {
    font-size: 25px;
    margin-bottom: 6px;
  }
  .set-password-container .textContainer p {
    font-size: 14px;
    font-weight: 300;
  }

  .set-password-container .master-password-input {
    width: 100%;
    height: 50px;
    background-color: #101012;
    border: 1.5px solid #393943;
    border-radius: 6px;
    padding: 0 1rem;
    font-size: 16px;
    color: var(--white);
  }

  .set-password-container .buttonContainer {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .set-password-container .buttonContainer .navButton {
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
  .set-password-container .buttonContainer .navButton:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .set-password-container .buttonContainer .navButton:not(:disabled):hover {
    opacity: 1;
  }

  .password-strength {
    text-align: center;
    font-size: 14px;
    margin-top: -1rem;
    color: #aaa;
  }

  .set-password-container .circle-bg {
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
    .set-password-container .textContainer h1 {
      font-size: 20px;
    }
    .set-password-container .textContainer p {
      font-size: 12px;
    }
    .set-password-container .master-password-input {
      height: 45px;
    }
    .set-password-container .buttonContainer .navButton {
      height: 44px;
      font-size: 14px;
    }
  }
</style>