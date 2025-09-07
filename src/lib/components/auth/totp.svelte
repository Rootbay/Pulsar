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
      <p>Verification step with a TOTP-generated key</p>
    </div>

    <div
      class="totpContainer"
      role="button"
      tabindex="0"
      aria-label="Enter TOTP code"
      on:click={() => hiddenInput.focus()}
      on:keydown={(e) => {
        if (e.key === 'Enter' || e.key === ' ')
          hiddenInput.focus();
      }}
    >
      {#each Array(6) as _, i (i)}
        <div
          class="totpBox"
          aria-label="Digit {i + 1}"
        >
          {code[i] || ""}
        </div>
      {/each}
      <input
        type="text"
        bind:this={hiddenInput}
        bind:value={code}
        maxlength="6"
        inputmode="numeric"
        pattern="[0-9]*"
        autocomplete="one-time-code"
        on:input={sanitize}
      />
    </div>

    <div class="buttonContainer">
      <button
        type="button"
        class="navButton continue"
        on:click={handleContinue}
        disabled={code.length !== 6}
      >
        Continue
      </button>
      <button type="button" class="navButton generate" disabled>
        Generate
      </button>
    </div>

    <div class="circle-bg"></div>
  </section>
</div>

<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  const dispatch = createEventDispatcher();

  let hiddenInput: HTMLInputElement;
  let code = "";
  

  function sanitize() {
    code = code.replace(/\D/g, "").slice(0, 6);
  }

  async function handleContinue() {
    if (code.length === 6) {
      try {
        const isValid = await invoke("verify_totp", { token: code });
        if (isValid) {
          dispatch("loginSuccess");
        } else {
          // no-op
        }
      } catch (_e) {
        // no-op
      }
    }
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

  .totpContainer {
    display: flex;
    justify-content: space-between;
    cursor: text;
    position: relative;
  }

  .totpBox {
    width: 65px;
    height: 80px;
    background-color: #101012;
    border: 1.5px solid #393943;
    border-radius: 6px;
    text-align: center;
    font-size: 32px;
    color: var(--white);
    line-height: 80px;
    transition: box-shadow 0.2s ease, border-color 0.2s ease;
  }

  .totpContainer input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
    width: 100%;
    height: 80px;
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
    .totpBox {
      width: 45px;
      height: 60px;
      font-size: 22px;
      line-height: 60px;
    }
    .buttonContainer .navButton {
      height: 44px;
      font-size: 14px;
    }
  }
</style>
