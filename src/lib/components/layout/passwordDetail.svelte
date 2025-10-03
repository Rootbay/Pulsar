<svelte:options runes />

<script lang="ts">
	import type { PasswordItem } from '../../../routes/+layout.ts';
	import Icon from '../ui/Icon.svelte';
	import { iconPaths } from '$lib/icons';
	import { invoke } from '@tauri-apps/api/core';
	import Input from '../ui/Input.svelte';
	import { Button } from '../ui/button';
	import { Select, SelectContent, SelectItem, SelectTrigger } from '../ui/select';
	import { Skeleton } from '../ui/skeleton';
	import { createEventDispatcher, tick, onDestroy } from 'svelte';
	import UnsavedChangesPopup from '../UnsavedChangesPopup.svelte';
	import PasswordDetailHeader from '../password/PasswordDetailHeader.svelte';
	import { selectedTag, filterCategory } from '$lib/stores';
	import { flip } from 'svelte/animate';
	import { dndzone } from 'svelte-dnd-action';
	import { quintOut, cubicOut } from 'svelte/easing';
	import { Plus, ArrowUp, ArrowDown, Eye, EyeOff, ArrowDownUp } from '@lucide/svelte';

	function modernFade(node: Element, { duration = 400 } = {}) {
		const reduce =
		typeof window !== 'undefined' &&
		window.matchMedia &&
		window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		const dur = reduce ? 0 : duration;
		return {
		duration: dur,
		easing: quintOut,
		css: (t: number) => {
			const o = t;
			const y = (1 - t) * 8;
			return `opacity:${o}; transform: translateY(${y}px);`;
		}
		};
	}

	interface Props {
		selectedPasswordItem: PasswordItem | null;
		displayColor: string;
		buttons: any[];
	}

	let {
		selectedPasswordItem = $bindable<PasswordItem | null>(),
		displayColor,
		buttons
	}: Props = $props();

	const dispatch = createEventDispatcher<{
		removeEntry: PasswordItem;
		tagsSaved: { id: number; tags: string };
	}>();
	const fieldTypeOptions = [
		{ value: 'text', label: 'Text' },
		{ value: 'password', label: 'Password' },
		{ value: 'file', label: 'File' }
	] as const;
	type FieldType = (typeof fieldTypeOptions)[number]['value'];
	const getFieldTypeLabel = (value: FieldType) =>
		fieldTypeOptions.find((option) => option.value === value)?.label ?? value;

	const MIN_FIELD_SKELETONS = 6;
	const MIN_TAG_SKELETONS = 3;
	const createPlaceholders = (count: number, min = MIN_FIELD_SKELETONS) =>
		Array.from({ length: Math.max(count, min) });
	const extractTags = (tags?: string | null) =>
		tags
			? tags
				.split(',')
				.map((tag) => tag.trim())
				.filter(Boolean)
			: [];
	const createTagPlaceholders = (tags?: string | null) =>
		createPlaceholders(extractTags(tags).length, MIN_TAG_SKELETONS);

	let isEditing = $state(false);
	let hasUnsavedChanges = $state(false);
	let showTimestamps = $state(false);
	let showPassword = $state(false);
	let addingField = $state(false);
	let newFieldType = $state<FieldType>('text');
	let newFieldName = $state('');
	let pendingTagOrder = $state<string | null>(null);
	let dndItems = $state<DisplayField[]>([]);
	let showSkeletonDetail = $state(false);
	let skeletonTimerDetail: any = null;

	function handleTagsSaved(detail: { id: number; tags: string }) {
		dispatch('tagsSaved', detail);
	}

	function handleRemoveRequest(id?: number) {
		if (!selectedPasswordItem) {
			return;
		}
		if (id != null && selectedPasswordItem.id !== id) {
			return;
		}
		dispatch('removeEntry', selectedPasswordItem);
	}
	let lastSkeletonKeyDetail = '';

	$effect(() => {
		const currentSkeletonKeyDetail = `${$selectedTag ?? 'all'}|${$filterCategory}`;
		if (currentSkeletonKeyDetail !== lastSkeletonKeyDetail) {
			lastSkeletonKeyDetail = currentSkeletonKeyDetail;
			(async () => {
				await tick();
				const count = selectedPasswordItem
					? (isEditing ? dndItems.length : displayFields.length)
					: 0;
				if (count > 0) {
					showSkeletonDetail = true;
					clearTimeout(skeletonTimerDetail);
					skeletonTimerDetail = setTimeout(() => {
						showSkeletonDetail = false;
					}, 200);
				}
			})();
		}
	});

	onDestroy(() => {
		clearTimeout(skeletonTimerDetail);
	});

	type DisplayField = {
		id: string;
		name: string;
		value: any;
		type: string;
		icon: string;
	};

	let displayFields = $state<DisplayField[]>([]);

	$effect(() => {
		if (!selectedPasswordItem) {
			displayFields = [];
			return;
		}

		const staticFields: DisplayField[] = [
			{
				id: 'username',
				name: 'Username',
				value: selectedPasswordItem.username,
				type: 'text',
				icon: iconPaths.user
			},
			{
				id: 'password',
				name: 'Password',
				value: selectedPasswordItem.password,
				type: 'password',
				icon: iconPaths.key
			},
			{
				id: 'url',
				name: 'URL',
				value: selectedPasswordItem.url,
				type: 'text',
				icon: iconPaths.link
			},
			{
				id: 'notes',
				name: 'Notes',
				value: selectedPasswordItem.notes,
				type: 'multiline',
				icon: iconPaths.notes
			}
		];

		const customFields: DisplayField[] =
			(selectedPasswordItem.custom_fields ?? []).map((field: { name: string; value: string; field_type: string }) => ({
				id: field.name,
				name: field.name,
				value: field.value,
				type: field.field_type,
				icon: iconPaths.plus
			}));

		let allFields: DisplayField[] = [...staticFields, ...customFields];

		if (selectedPasswordItem.field_order?.length) {
			const orderedFields: DisplayField[] = [];
			const fieldMap = new Map(allFields.map((field) => [field.id, field]));

			for (const fieldId of selectedPasswordItem.field_order) {
				const field = fieldMap.get(fieldId);
				if (field) {
					orderedFields.push(field);
					fieldMap.delete(fieldId);
				}
			}
			orderedFields.push(...fieldMap.values());
			allFields = orderedFields;
		}

		displayFields = allFields;
	});

	function handleDndConsider(e: CustomEvent<{ items: DisplayField[] }>) {
		dndItems = e.detail.items;
	}

	function handleDndFinalize(e: CustomEvent<{ items: DisplayField[] }>) {
		dndItems = e.detail.items;
    dndItems = [...dndItems];
    if (selectedPasswordItem) {
      selectedPasswordItem.field_order = dndItems.map((item) => item.id);
    }
  }

  let originalPasswordItem: PasswordItem | null = null;

  $effect(() => {
    if (isEditing && originalPasswordItem) {
      const currentItemState = JSON.parse(
        JSON.stringify(originalPasswordItem)
      );
      const newCustomFields: {
        name: string;
        value: string;
        field_type: string;
      }[] = [];

      for (const item of dndItems) {
        switch (item.id) {
          case 'username':
            currentItemState.username = item.value;
            break;
          case 'password':
            currentItemState.password = item.value;
            break;
          case 'url':
            currentItemState.url = item.value;
            break;
          case 'notes':
            currentItemState.notes = item.value;
            break;
          default:
            newCustomFields.push({
              name: item.name,
              value: item.value,
              field_type: item.type
            });
            break;
        }
      }
      currentItemState.custom_fields = newCustomFields;
      currentItemState.field_order = dndItems.map((item) => item.id);

      const normalizedCurrent = JSON.parse(JSON.stringify(currentItemState));
      const normalizedOriginal = JSON.parse(
        JSON.stringify(originalPasswordItem)
      );
      if (normalizedCurrent.password === '')
        normalizedCurrent.password = 'N/A';
      if (normalizedOriginal.password == null)
        normalizedOriginal.password = 'N/A';
      if (normalizedCurrent.url === '') normalizedCurrent.url = null;

      hasUnsavedChanges =
        JSON.stringify(normalizedCurrent) !==
        JSON.stringify(normalizedOriginal);

      if (!hasUnsavedChanges) {
        const origTags = (originalPasswordItem?.tags ?? '') as string;
        if (pendingTagOrder !== null && pendingTagOrder !== origTags) {
          hasUnsavedChanges = true;
        }
      }
    } else {
      hasUnsavedChanges = false;
    }
  });

  function enterEditMode() {
    originalPasswordItem = JSON.parse(
      JSON.stringify(selectedPasswordItem)
    );
    dndItems = [...displayFields];
    for (const item of dndItems) {
      if (
        item.id === 'password' &&
        (item.value === 'N/A' || item.value == null)
      ) {
        item.value = '';
      }
    }
    pendingTagOrder = null;
    isEditing = true;
  }

  async function handleSave() {
    if (!selectedPasswordItem) return;
    const item = selectedPasswordItem as PasswordItem;

    const updated = JSON.parse(JSON.stringify(selectedPasswordItem));
    const newCustomFields: {
      name: string;
      value: string;
      field_type: string;
    }[] = [];
    for (const item of dndItems) {
      const val = (item.value ?? '').toString().trim();
      switch (item.id) {
        case 'username':
          updated.username = val.length > 0 ? val : null;
          break;
        case 'password':
          updated.password = val.length > 0 ? val : 'N/A';
          break;
        case 'url': {
          if (val.length === 0) {
            updated.url = null;
          } else {
            updated.url = /^https?:\/\//i.test(val)
              ? val
              : `https://${val}`;
          }
          break;
        }
        case 'notes':
          updated.notes = val.length > 0 ? val : null;
          break;
        default:
          newCustomFields.push({
            name: item.name,
            value: item.value,
            field_type: item.type
          });
          break;
      }
    }
    updated.custom_fields = newCustomFields;
    updated.field_order = dndItems.map((item) => item.id);

    if (JSON.stringify(updated) !== JSON.stringify(originalPasswordItem)) {
      try {
        await invoke('update_password_item', { item: updated });
        selectedPasswordItem = updated;
        originalPasswordItem = JSON.parse(JSON.stringify(updated));
      } catch (error) {
        console.error('Error updating password item:', error);
        alert(`Failed to save changes: ${error}`);
        return;
      }
    }

    if (pendingTagOrder !== null && pendingTagOrder !== (item.tags ?? '')) {
      try {
        await invoke('update_password_item_tags', {
          id: item.id,
          tags: pendingTagOrder
        });
        const updatedItem = { ...item, tags: pendingTagOrder } as any;
        selectedPasswordItem = updatedItem;
			handleTagsSaved({ id: updatedItem.id, tags: pendingTagOrder });
      } catch (error) {
        console.error('Error saving tag order:', error);
        alert(`Failed to save tag order: ${error}`);
        return;
      }
    }

    hasUnsavedChanges = false;
    pendingTagOrder = null;
    await tick();
    setTimeout(() => {
      isEditing = false;
    }, 320);
  }

  function handleReset() {
    if (originalPasswordItem) {
      selectedPasswordItem = JSON.parse(
        JSON.stringify(originalPasswordItem)
      );
    }
    isEditing = false;
    hasUnsavedChanges = false;
    pendingTagOrder = null;
  }

  function formatTimestamp(timestamp: string | null): string {
    if (!timestamp) return 'N/A';
    const date = new Date(timestamp);
    return date.toLocaleString();
  }

  function handleCancelAddField() {
    addingField = false;
    newFieldName = '';
    newFieldType = 'text';
  }

  async function handleConfirmAddField() {
    if (!newFieldName.trim()) {
      alert('Please enter a name for the new field.');
      return;
    }

    if (selectedPasswordItem) {
      try {
        await invoke('add_custom_field', {
          itemId: selectedPasswordItem.id,
          fieldName: newFieldName,
          fieldType: newFieldType
        });
        alert('Custom field added successfully!');
        handleCancelAddField();
      } catch (error) {
        console.error('Error adding custom field:', error);
        alert(`Failed to add custom field: ${error}`);
      }
    }
  }
</script>

<main class="passwordDetail" class:editing={isEditing}>
    {#key selectedPasswordItem ? selectedPasswordItem.id : 'none'}
    <div class="detail-anim detail-content" in:modernFade aria-hidden={showSkeletonDetail}>
	{#if selectedPasswordItem}
		<PasswordDetailHeader
			bind:selectedPasswordItem
			bind:isEditing={isEditing}
			displayColor={displayColor}
			{buttons}

			onEnterEditMode={enterEditMode}
			onHandleReset={handleReset}
			onSave={handleSave}
			onRemoveEntry={handleRemoveRequest}
			onTagsReorderedPending={(detail) => { pendingTagOrder = detail?.tags ?? null; }}
		/>

		<div class="detail-group" aria-busy={showSkeletonDetail}>
			{#if !isEditing}
				{#if showSkeletonDetail}
					{#each createPlaceholders(displayFields.length) as _}
						<div class="flex items-center gap-4 py-2" aria-hidden="true">
							<Skeleton class="h-5 w-5 rounded-md" />
							<div class="flex flex-1 flex-col gap-2">
								<Skeleton class="h-4 w-40" />
								<Skeleton class="h-3 w-32 opacity-70" />
							</div>
						</div>
					{/each}
				{:else}
					{#each displayFields as field (field.id)}
						<Input
							title={field.name}
							inputValue={
								field.id === 'password'
									? (field.value && field.value.length ? field.value : 'N/A')
									: (field.id === 'username' || field.id === 'url' || field.id === 'notes'
										? field.value || 'N/A'
										: field.value)
							}
							selectedIconPath={field.icon}
							selectedIconName={field.id === 'username'
								? 'user'
								: field.id === 'password'
									? 'key'
									: field.id === 'url'
										? 'link'
										: field.id === 'notes'
											? 'notes'
											: field.id}
							readOnly={true}
							selectedColor={displayColor}
							isMultiline={field.type === 'multiline'}
							type={field.id === 'password'
								? ((field.value && field.value.length && field.value !== 'N/A')
										? (showPassword ? 'text' : 'password')
										: 'text')
								: (field.type === 'password' ? 'password' : 'text')}
							isExpandable={true}
						>
							<div slot="rightIcon">
								{#if field.id === 'password' && field.value && field.value.length && field.value !== 'N/A'}
									<Button
										type="button"
										variant="ghost"
										size="icon"
										class="h-6 w-6 p-0 text-muted-foreground hover:text-foreground"
										aria-pressed={showPassword}
										aria-label={showPassword ? 'Hide password' : 'Show password'}
										onclick={() => (showPassword = !showPassword)}
									>
										{#if showPassword}
											<Eye class="h-5 w-5" />
										{:else}
											<EyeOff class="h-5 w-5" />
										{/if}
									</Button>
								{/if}
							</div>
						</Input>
					{/each}
				{/if}
			{:else}
				{#if showSkeletonDetail}
					{#each createPlaceholders(dndItems.length) as _}
						<div class="flex items-center gap-4 py-2" aria-hidden="true">
							<Skeleton class="h-5 w-5 rounded-md" />
							<div class="flex flex-1 flex-col gap-2">
								<Skeleton class="h-4 w-48" />
								<Skeleton class="h-3 w-36 opacity-70" />
							</div>
						</div>
					{/each}
				{:else}
				<div
					class="detail-group"
					use:dndzone={{ items: dndItems, flipDurationMs: 300, dropFromOthersDisabled: true }}
					onconsider={handleDndConsider}
					onfinalize={handleDndFinalize}
				>
					{#each dndItems as field (field.id)}
							<div animate:flip={{ duration: 300, easing: cubicOut }} class="dnd-item">
								<Input
									title={field.name}
									bind:inputValue={field.value}
									readOnly={!isEditing}
									selectedColor={displayColor}
									selectedIconPath={field.icon}
									selectedIconName={field.id}
									isMultiline={field.type === 'multiline'}
									type={field.id === 'url'
										? 'url'
										: field.id === 'password'
											? showPassword
												? 'text'
												: 'password'
											: field.type === 'password'
												? 'password'
												: 'text'}
								>
									<div
										slot="rightIcon"
										class="flex items-center gap-2"
									>
										{#if field.id === 'password'}
											<Button
												type="button"
												variant="ghost"
												size="icon"
												class="h-6 w-6 p-0 text-muted-foreground hover:text-foreground"
												aria-pressed={showPassword}
												aria-label={showPassword ? 'Hide password' : 'Show password'}
												onclick={() => (showPassword = !showPassword)}
											>
													{#if showPassword}
														<Eye class="h-5 w-5" />
													{:else}
														<EyeOff class="h-5 w-5" />
													{/if}
											</Button>
										{/if}
										{#if isEditing}
											<div class="drag-handle" data-dnd-handle>
												<ArrowDownUp class="h-6 w-6" />
											</div>
										{/if}
									</div>
								</Input>
							</div>
						{/each}
				</div>
				{/if}
			{/if}
		</div>

		{#if isEditing}
			{#if !addingField}
				<Button
					type="button"
					variant="outline"
					class="mt-3 flex h-14 w-full items-center justify-center rounded-lg border border-border/60 bg-muted/20 text-muted-foreground transition-colors hover:border-primary/50 hover:text-primary"
					onclick={() => (addingField = true)}
				>
					<Plus class="h-5 w-5" />
				</Button>
			{:else}
				<div class="mt-3 flex items-center gap-3">
					<Button
						type="button"
						variant="ghost"
						size="icon"
						class="h-9 w-9 text-muted-foreground"
						disabled
						aria-hidden="true"
					>
						<Plus class="h-5 w-5" />
					</Button>
					<Input
						title="New Field"
						placeholder="New Field"
						bind:inputValue={newFieldName}
						readOnly={false}
						selectedColor={displayColor}
						showTitle={false}
						selectedIconPath={iconPaths.plus}
						selectedIconName="plus"
					/>
					<Select
						type="single"
						value={newFieldType}
						onValueChange={(value) => (newFieldType = value as FieldType)}
					>
						<SelectTrigger class="w-40">
							<span data-slot="select-value" class="flex items-center gap-2 truncate text-sm">
								{getFieldTypeLabel(newFieldType)}
							</span>
						</SelectTrigger>
						<SelectContent>
							{#each fieldTypeOptions as option}
								<SelectItem value={option.value}>{option.label}</SelectItem>
							{/each}
						</SelectContent>
					</Select>
				</div>
				<div class="mt-3 flex justify-end gap-2">
					<Button type="button" variant="ghost" onclick={handleCancelAddField}>
						Cancel
					</Button>
					<Button type="button" onclick={handleConfirmAddField}>
						Confirm
					</Button>
				</div>
			{/if}
		{/if}

		<div class="collapsible-section">
			<button
				type="button"
				class="collapsible-header"
				class:expanded={showTimestamps}
				onclick={() => (showTimestamps = !showTimestamps)}
			>
				<h3>Metadata</h3>
				{#if showTimestamps}
					<ArrowUp class="h-5 w-5" />
				{:else}
					<ArrowDown class="h-5 w-5" />
				{/if}
			</button>
			<div class="detail-group timestamps collapsible-content" class:collapsed={!showTimestamps}>
				<div class="detail-section">
					<h3>Password ID</h3>
					<p>{selectedPasswordItem.id}</p>
				</div>

					<div class="detail-section">
						<h3>Created At</h3>
						<p class:placeholder-value={formatTimestamp(selectedPasswordItem.created_at) === 'N/A'}>{formatTimestamp(selectedPasswordItem.created_at)}</p>
					</div>

					<div class="detail-section">
						<h3>Last Modified</h3>
						<p class:placeholder-value={formatTimestamp(selectedPasswordItem.updated_at) === 'N/A'}>{formatTimestamp(selectedPasswordItem.updated_at)}</p>
					</div>

				<div class="detail-section">
					<h3>Key Derivation Function</h3>
					<p>Argon2id</p>
				</div>

				<div class="detail-section">
					<h3>Encryption Algorithm</h3>
					<p>ChaCha20-Poly1305 (Default) / AES-256-GCM (if AES-NI)</p>
				</div>

				<div class="detail-section">
					<h3>Nonce Size</h3>
					<p>12-byte random per record</p>
				</div>

					<div class="detail-section">
						<h3>Last Accessed</h3>
						<p class="placeholder-value">N/A (Placeholder)</p>
					</div>
			</div>
		</div>
	{:else}
		<p class="no-selection">Select a password entry to view details.</p>
    {/if}
    </div>
    {/key}

	{#if showSkeletonDetail}
		<div
			class="pointer-events-none absolute inset-0 flex flex-col gap-6 bg-[color:var(--passworddetail-overlay)] p-5"
			aria-hidden="true"
		>
			<div class="flex flex-col gap-4">
				<div class="flex items-start justify-between gap-4">
					<div class="flex items-center gap-3">
						<Skeleton class="h-6 w-6 rounded-md" />
						<div class="flex flex-col gap-2">
							<Skeleton class="h-4 w-44" />
							<Skeleton class="h-3 w-32 opacity-70" />
						</div>
					</div>
					<div class="flex items-center gap-3">
						<Skeleton class="h-6 w-20 rounded-full" />
						<Skeleton class="h-6 w-10 rounded-full" />
					</div>
				</div>
				<div class="flex flex-wrap gap-3">
					{#each createTagPlaceholders(selectedPasswordItem?.tags) as _}
						<Skeleton class="h-6 w-16 rounded-full" />
					{/each}
				</div>
			</div>
			<div class="flex flex-col gap-4">
				{#each createPlaceholders(isEditing ? dndItems.length : displayFields.length) as _}
					<div class="flex items-center gap-4">
						<Skeleton class="h-5 w-5 rounded-md" />
						<div class="flex flex-1 flex-col gap-2">
							<Skeleton class="h-4 w-52" />
							<Skeleton class="h-3 w-40 opacity-70" />
						</div>
					</div>
				{/each}
			</div>
			<div class="flex flex-col gap-3">
				<Skeleton class="h-4 w-36" />
				<Skeleton class="h-3 w-3/4" />
				<Skeleton class="h-3 w-2/3 opacity-70" />
			</div>
		</div>
	{/if}
	{#if hasUnsavedChanges}
		<UnsavedChangesPopup on:save={handleSave} on:reset={handleReset} />
	{/if}
</main>

<style>
	.detail-group[aria-busy="true"] {
		pointer-events: none;
	}

	.detail-anim {
		will-change: opacity, transform;
	}

	.passwordDetail {
		--passworddetail-surface: color-mix(in oklch, var(--card) 90%, var(--background) 10%);
		--passworddetail-elevated: color-mix(in oklch, var(--card) 80%, var(--background) 20%);
		--passworddetail-hover: color-mix(in oklch, var(--passworddetail-elevated) 82%, var(--background) 18%);
		--passworddetail-border: var(--border);
		--passworddetail-strong-text: var(--foreground);
		--passworddetail-muted-text: color-mix(in oklch, var(--foreground) 65%, transparent);
		--passworddetail-subtle-text: color-mix(in oklch, var(--foreground) 40%, transparent);
		--passworddetail-accent: var(--primary);
		--passworddetail-accent-foreground: var(--primary-foreground);
		--passworddetail-secondary-surface: color-mix(in oklch, var(--passworddetail-elevated) 85%, var(--background) 15%);
		--passworddetail-secondary-hover: color-mix(in oklch, var(--passworddetail-secondary-surface) 80%, var(--passworddetail-elevated) 20%);
		--passworddetail-overlay: color-mix(in oklch, var(--passworddetail-surface) 85%, transparent);
		padding: 20px;
		background-color: var(--passworddetail-surface);
		color: var(--passworddetail-strong-text);
		display: flex;
		flex-direction: column;
		height: 100%;
		box-sizing: border-box;
		overflow-y: auto;
		position: relative;
	}

	.detail-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
		outline: none !important;
	}

	.dnd-item {
		will-change: transform;
		touch-action: none;
	}

	.detail-group.timestamps {
		flex-direction: row;
		flex-wrap: wrap;
		gap: 30px;
	}

	.detail-section {
		margin-bottom: 10px;
	}

	.detail-section h3 {
		margin-top: 0;
		margin-bottom: 5px;
		color: var(--passworddetail-muted-text);
		font-size: 14px;
		display: flex;
		align-items: center;
		gap: 5px;
	}

	.detail-section p {
		margin: 0;
		font-size: 16px;
		word-wrap: break-word;
	}

	.no-selection {
		text-align: center;
		color: var(--passworddetail-subtle-text);
		margin-top: 50px;
	}

	.collapsible-section {
		margin-top: 20px;
		border: 1px solid var(--passworddetail-border);
		border-radius: 8px;
	}

	.collapsible-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 10px 15px;
		background-color: var(--passworddetail-elevated);
		cursor: pointer;
		font-weight: 500;
		color: var(--passworddetail-strong-text);
		border: none;
		width: 100%;
	}

	.collapsible-header:hover {
		background-color: var(--passworddetail-hover);
	}

	.collapsible-header h3 {
		margin: 0;
		font-size: 16px;
	}

	.collapsible-header :global(svg) {
		transition: transform 0.2s ease-in-out;
	}

	.collapsible-header.expanded :global(svg) {
		transform: rotate(180deg);
	}

	.collapsible-content {
		padding: 15px;
		background-color: var(--passworddetail-surface);
		overflow-y: auto;
		max-height: 169px;
		transition: max-height 0.3s ease-out;
	}

	.collapsible-content.collapsed {
		max-height: 0;
		padding-top: 0;
		padding-bottom: 0;
		overflow-y: hidden;
	}

	.drag-handle {
		cursor: grab;
		margin-left: 10px;
	}

	.detail-content[aria-hidden="true"] {
		visibility: hidden;
		pointer-events: none;
	}
</style>
