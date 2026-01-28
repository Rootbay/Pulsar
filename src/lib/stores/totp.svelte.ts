import { browser } from '$app/environment';
import { appState } from './appState.svelte';

class LoginTotpStore {
  #secret = $state<string | null>(null);
  #configured = $state<boolean>(false);

  get secret() {
    return this.#secret;
  }

  set secret(value: string | null) {
    this.#secret = value;
  }

  get configured() {
    return this.#configured;
  }

  set configured(value: boolean) {
    this.#configured = value;
  }

  setSecret(value: string | null) {
    this.#secret = value;
  }

  setConfigured(value: boolean) {
    this.#configured = value;
  }

  reset() {
    this.#secret = null;
    this.#configured = false;
  }
}

export const loginTotpStore = new LoginTotpStore();
