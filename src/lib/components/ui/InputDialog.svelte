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
  import { i18n, t as translate } from '$lib/i18n.svelte';

  let {
    open = $bindable(false),
    title,
    description,
    label,
    placeholder = '',
    confirmLabel = 'Save',
    onConfirm,
    defaultValue = ''
  }: {
    open: boolean;
    title: string;
    description: string;
    label: string;
    placeholder?: string;
    confirmLabel?: string;
    onConfirm: (value: string) => void;
    defaultValue?: string;
  } = $props();

  let value = $state(defaultValue);

  $effect(() => {
    if (open) {
      value = defaultValue;
    }
  });

  function handleConfirm() {
    if (!value.trim()) return;
    onConfirm(value.trim());
    open = false;
  }

  function handleCancel() {
    open = false;
  }

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as any, vars);
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
        <Label for="input-value">{label}</Label>
        <Input
          id="input-value"
          placeholder={placeholder}
          bind:value={value}
          onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handleConfirm()}
        />
      </div>
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={handleCancel}>
        {t('Cancel')}
      </Button>
      <Button onclick={handleConfirm} disabled={!value.trim()}>
        {confirmLabel}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
