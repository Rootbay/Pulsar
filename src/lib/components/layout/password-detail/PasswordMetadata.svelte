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

<div class="mt-5 overflow-hidden rounded-lg border border-[color:var(--passworddetail-border)]">
	<button
		type="button"
		class="flex w-full items-center justify-between bg-[color:var(--passworddetail-elevated)] px-4 py-2.5 text-left font-medium text-[color:var(--passworddetail-strong-text)] transition-colors hover:bg-[color:var(--passworddetail-hover)] focus:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--passworddetail-accent)] focus-visible:ring-offset-0"
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
		class={`flex flex-wrap gap-7 bg-[color:var(--passworddetail-surface)] px-4 transition-all duration-300 ease-out ${expanded ? 'max-h-[169px] py-4 overflow-y-auto' : 'max-h-0 py-0 overflow-hidden'}`}
	>
		<div class="flex w-full flex-col gap-1 pb-2">
			<h3 class="text-sm font-medium text-[color:var(--passworddetail-muted-text)]">Password ID</h3>
			<p class="text-base break-words">{item.id}</p>
		</div>
		<div class="flex w-full flex-col gap-1 pb-2">
			<h3 class="text-sm font-medium text-[color:var(--passworddetail-muted-text)]">Created At</h3>
			{#if formatTimestamp(item.created_at) === 'N/A'}
				<p class="text-base break-words text-[color:var(--passworddetail-subtle-text)]">N/A</p>
			{:else}
				<p class="text-base break-words">{formatTimestamp(item.created_at)}</p>
			{/if}
		</div>
		<div class="flex w-full flex-col gap-1 pb-2">
			<h3 class="text-sm font-medium text-[color:var(--passworddetail-muted-text)]">Last Modified</h3>
			{#if formatTimestamp(item.updated_at) === 'N/A'}
				<p class="text-base break-words text-[color:var(--passworddetail-subtle-text)]">N/A</p>
			{:else}
				<p class="text-base break-words">{formatTimestamp(item.updated_at)}</p>
			{/if}
		</div>
		<div class="flex w-full flex-col gap-1 pb-2">
			<h3 class="text-sm font-medium text-[color:var(--passworddetail-muted-text)]">Key Derivation Function</h3>
			<p class="text-base break-words">Argon2id</p>
		</div>
		<div class="flex w-full flex-col gap-1 pb-2">
			<h3 class="text-sm font-medium text-[color:var(--passworddetail-muted-text)]">Encryption Algorithm</h3>
			<p class="text-base break-words">ChaCha20-Poly1305 (Default) / AES-256-GCM (if AES-NI)</p>
		</div>
		<div class="flex w-full flex-col gap-1 pb-2">
			<h3 class="text-sm font-medium text-[color:var(--passworddetail-muted-text)]">Nonce Size</h3>
			<p class="text-base break-words">12-byte random per record</p>
		</div>
		<div class="flex w-full flex-col gap-1">
			<h3 class="text-sm font-medium text-[color:var(--passworddetail-muted-text)]">Last Accessed</h3>
			<p class="text-base break-words text-[color:var(--passworddetail-subtle-text)]">N/A (Placeholder)</p>
		</div>
	</div>
</div>
