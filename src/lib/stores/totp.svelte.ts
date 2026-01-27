import { browser } from '$app/environment';

class LoginTotpStore {
  #secret = $state<string | null>(null);
  #configured = $state<boolean>(false);

  constructor() {
    if (browser) {
      const storedSecret = localStorage.getItem('pulsar.loginTotpSecret');
      if (storedSecret) {
        try {
          this.#secret = JSON.parse(storedSecret);
        } catch {
          this.#secret = null;
        }
      }

      const storedConfigured = localStorage.getItem('pulsar.loginTotpConfigured');
      if (storedConfigured) {
        try {
          this.#configured = JSON.parse(storedConfigured);
        } catch {
          this.#configured = false;
        }
      }

      $effect.root(() => {
        $effect(() => {
          localStorage.setItem('pulsar.loginTotpSecret', JSON.stringify(this.#secret));
        });
        $effect(() => {
          localStorage.setItem('pulsar.loginTotpConfigured', JSON.stringify(this.#configured));
        });
      });
    }
  }

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
}

export const loginTotpStore = new LoginTotpStore();
