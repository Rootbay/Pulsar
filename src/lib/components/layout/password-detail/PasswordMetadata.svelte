<svelte:options runes />

<script lang="ts">
  import { ArrowUp, ArrowDown } from '@lucide/svelte';
  import type { PasswordItem } from '$lib/types/password';

  interface Props {
    item: PasswordItem;
    expanded: boolean;
  }

  let { item, expanded = $bindable(false) }: Props = $props();

  function toggleExpanded() {
    expanded = !expanded;
  }

  function formatTimestamp(timestamp: string | null | undefined): string {
    if (!timestamp) {
      return 'N/A';
    }
    const date = new Date(timestamp);
    return Number.isNaN(date.valueOf()) ? 'N/A' : date.toLocaleString();
  }
</script>

<div class="border-color:var(--passworddetail-border) mt-5 overflow-hidden rounded-lg border">
  <button
    type="button"
    class="bg-color:var(--passworddetail-elevated) text-color:var(--passworddetail-strong-text) hover:bg-color:var(--passworddetail-hover) focus-visible:ring-color:var(--passworddetail-accent) flex w-full items-center justify-between px-4 py-2.5 text-left font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-0"
    onclick={toggleExpanded}
  >
    <h3 class="text-base font-medium">Metadata</h3>
    {#if expanded}
      <ArrowUp class="h-5 w-5 transition-transform duration-200" />
    {:else}
      <ArrowDown class="h-5 w-5 transition-transform duration-200" />
    {/if}
  </button>
  <div
    class={`bg-color:var(--passworddetail-surface) flex flex-wrap gap-7 px-4 transition-all duration-300 ease-out ${expanded ? 'max-h-42.25 overflow-y-auto py-4' : 'max-h-0 overflow-hidden py-0'}`}
  >
    <div class="flex w-full flex-col gap-1 pb-2">
      <h3 class="text-color:var(--passworddetail-muted-text) text-sm font-medium">Password ID</h3>
      <p class="text-base wrap-break-word">{item.id}</p>
    </div>
    <div class="flex w-full flex-col gap-1 pb-2">
      <h3 class="text-color:var(--passworddetail-muted-text) text-sm font-medium">Created At</h3>
      {#if formatTimestamp(item.created_at) === 'N/A'}
        <p class="text-color:var(--passworddetail-subtle-text) text-base wrap-break-word">N/A</p>
      {:else}
        <p class="text-base wrap-break-word">{formatTimestamp(item.created_at)}</p>
      {/if}
    </div>
    <div class="flex w-full flex-col gap-1 pb-2">
      <h3 class="text-color:var(--passworddetail-muted-text) text-sm font-medium">Last Modified</h3>
      {#if formatTimestamp(item.updated_at) === 'N/A'}
        <p class="text-color:var(--passworddetail-subtle-text) text-base wrap-break-word">N/A</p>
      {:else}
        <p class="text-base wrap-break-word">{formatTimestamp(item.updated_at)}</p>
      {/if}
    </div>
    <div class="flex w-full flex-col gap-1 pb-2">
      <h3 class="text-color:var(--passworddetail-muted-text) text-sm font-medium">
        Key Derivation Function
      </h3>
      <p class="text-base wrap-break-word">Argon2id</p>
    </div>
    <div class="flex w-full flex-col gap-1 pb-2">
      <h3 class="text-color:var(--passworddetail-muted-text) text-sm font-medium">
        Encryption Algorithm
      </h3>
      <p class="text-base wrap-break-word">ChaCha20-Poly1305 (Default) / AES-256-GCM (if AES-NI)</p>
    </div>
    <div class="flex w-full flex-col gap-1 pb-2">
      <h3 class="text-color:var(--passworddetail-muted-text) text-sm font-medium">Nonce Size</h3>
      <p class="text-base wrap-break-word">12-byte random per record</p>
    </div>
    <div class="flex w-full flex-col gap-1">
      <h3 class="text-color:var(--passworddetail-muted-text) text-sm font-medium">Last Accessed</h3>
      <p class="text-color:var(--passworddetail-subtle-text) text-base wrap-break-word">
        N/A (Placeholder)
      </p>
    </div>
  </div>
</div>
