<script lang="ts">
  import { Button } from './button';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
  } from './dialog';
  import { Input } from './input';
  import { Label } from './label';
  import { Spinner } from './spinner';
  import { i18n, t as translate } from '$lib/i18n.svelte';

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as any, vars);

  let {
    open = $bindable(false),
    title,
    description,
    confirmLabel = 'Continue',
    onConfirm,
    busy = false
  }: {
    open: boolean;
    title: string;
    description: string;
    confirmLabel?: string;
    onConfirm: (passphrase: string) => Promise<void> | void;
    busy?: boolean;
  } = $props();

  let passphrase = $state('');

  async function handleConfirm() {
    if (!passphrase.trim()) return;
    await onConfirm(passphrase.trim());
    passphrase = '';
    open = false;
  }

  function handleCancel() {
    passphrase = '';
    open = false;
  }
</script>

<Dialog bind:open>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle>{title}</DialogTitle>
      <DialogDescription>
        {description}
      </DialogDescription>
    </DialogHeader>
    <div class="space-y-4 py-2">
      <div class="space-y-2">
        <Label for="passphrase">Passphrase</Label>
        <Input
          id="passphrase"
          type="password"
          placeholder="Enter passphrase"
          bind:value={passphrase}
          onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handleConfirm()}
          disabled={busy}
        />
      </div>
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={handleCancel} disabled={busy}>
        Cancel
      </Button>
      <Button onclick={handleConfirm} disabled={busy || !passphrase.trim()}>
        {#if busy}
          <Spinner class="mr-2 h-4 w-4" />
        {/if}
        {confirmLabel}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
