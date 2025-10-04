<svelte:options runes />

<script lang="ts">
	import type { PasswordItem } from '$lib/types/password';
	import type { DisplayField } from '$lib/types/password-fields';
	import { iconPaths } from '$lib/icons';
	import { invoke } from '@tauri-apps/api/core';
	import Input from '../ui/Input.svelte';
	import { Button } from '../ui/button';
	import { Select, SelectContent, SelectItem, SelectTrigger } from '../ui/select';
	import { Skeleton } from '../ui/skeleton';
	import { createEventDispatcher, tick, onDestroy } from 'svelte';
	import UnsavedChangesPopup from '../UnsavedChangesPopup.svelte';
	import PasswordDetailHeader from '../password/PasswordDetailHeader.svelte';
	import PasswordFieldList from './password-detail/PasswordFieldList.svelte';
	import PasswordMetadata from './password-detail/PasswordMetadata.svelte';
	import { selectedTag, filterCategory } from '$lib/stores';
	import { quintOut } from 'svelte/easing';
	import { Plus } from '@lucide/svelte';
	import { buildDisplayFields } from '$lib/utils/passwordFields';

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


	const detailThemeStyle = [
		'--passworddetail-surface: color-mix(in oklch, var(--card) 90%, var(--background) 10%)',
		'--passworddetail-elevated: color-mix(in oklch, var(--card) 80%, var(--background) 20%)',
		'--passworddetail-hover: color-mix(in oklch, var(--passworddetail-elevated) 82%, var(--background) 18%)',
		'--passworddetail-border: var(--border)',
		'--passworddetail-strong-text: var(--foreground)',
		'--passworddetail-muted-text: color-mix(in oklch, var(--foreground) 65%, transparent)',
		'--passworddetail-subtle-text: color-mix(in oklch, var(--foreground) 40%, transparent)',
		'--passworddetail-accent: var(--primary)',
		'--passworddetail-accent-foreground: var(--primary-foreground)',
		'--passworddetail-secondary-surface: color-mix(in oklch, var(--passworddetail-elevated) 85%, var(--background) 15%)',
		'--passworddetail-secondary-hover: color-mix(in oklch, var(--passworddetail-secondary-surface) 80%, var(--passworddetail-elevated) 20%)',
		'--passworddetail-overlay: color-mix(in oklch, var(--passworddetail-surface) 85%, transparent)'
	].join(';') + ';';
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

	let displayFields = $state<DisplayField[]>([]);

	$effect(() => {
		displayFields = buildDisplayFields(selectedPasswordItem, iconPaths);

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

<main class="relative flex h-full flex-col overflow-y-auto bg-[color:var(--passworddetail-surface)] p-5 text-[color:var(--passworddetail-strong-text)]" style={detailThemeStyle}>
    {#key selectedPasswordItem ? selectedPasswordItem.id : 'none'}
	<div class={`flex flex-col gap-6 ${showSkeletonDetail ? 'pointer-events-none opacity-0' : ''}`} in:modernFade aria-hidden={showSkeletonDetail}>
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

		<PasswordFieldList
			isEditing={isEditing}
			displayFields={displayFields}
			bind:editingFields={dndItems}
			displayColor={displayColor}
			bind:showPassword
			showSkeleton={showSkeletonDetail}
			viewSkeletonPlaceholders={showSkeletonDetail ? createPlaceholders(displayFields.length) : []}
			editSkeletonPlaceholders={showSkeletonDetail ? createPlaceholders(dndItems.length) : []}
			on:consider={handleDndConsider}
			on:finalize={handleDndFinalize}
		/>
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

	<PasswordMetadata item={selectedPasswordItem} bind:expanded={showTimestamps} />

	{:else}
		<p class="mt-12 text-center text-sm text-[color:var(--passworddetail-subtle-text)]">Select a password entry to view details.</p>
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





