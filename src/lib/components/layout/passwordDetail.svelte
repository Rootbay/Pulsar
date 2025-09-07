<script lang="ts">
	import type { PasswordItem } from '../../../routes/+layout.ts';
	import Icon from '../ui/Icon.svelte';
	import { iconPaths } from '$lib/icons';
	import { invoke } from '@tauri-apps/api/core';
	
	import Input from '../ui/Input.svelte';
	import { createEventDispatcher, tick, onDestroy } from 'svelte';
	
	
	import UnsavedChangesPopup from '../UnsavedChangesPopup.svelte';
	import PasswordDetailHeader from '../password/PasswordDetailHeader.svelte';
	import { selectedTag, filterCategory } from '$lib/stores';

	// import { SvelteMap } from 'svelte/collections';
    import { flip } from 'svelte/animate';
    import { dndzone } from 'svelte-dnd-action';
    import { quintOut } from 'svelte/easing';

    // Clean, slower fade-in with gentle lift
    function modernFade(node: Element, { duration = 400 } = {}) {
        const reduce = typeof window !== 'undefined' && window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        const dur = reduce ? 0 : duration;
        return {
            duration: dur,
            easing: quintOut,
            css: (t: number) => {
                const o = t; // opacity 0->1
                const y = (1 - t) * 8; // translateY 8px->0
                return `opacity:${o}; transform: translateY(${y}px);`;
            }
        };
    }

	const dispatch = createEventDispatcher();

	export let selectedPasswordItem: PasswordItem | null;
	export let displayColor: string;
	export let buttons: any[];

	
	let isEditing = false;
	let hasUnsavedChanges = false;
	let showTimestamps = false;
	let showPassword = false;
	let addingField = false;
	let newFieldType = 'text';
	let newFieldName = '';
    let pendingTagOrder: string | null = null;

	let dndItems: any[] = [];

	// Skeleton loading when switching via sidebar (tags/filter)
	let showSkeletonDetail = false;
	let skeletonTimerDetail: any = null;
	let lastSkeletonKeyDetail = '';
	$: currentSkeletonKeyDetail = `${$selectedTag ?? 'all'}|${$filterCategory}`;
	$: if (currentSkeletonKeyDetail !== lastSkeletonKeyDetail) {
		lastSkeletonKeyDetail = currentSkeletonKeyDetail;
		(async () => {
			await tick(); // ensure fields computed for new context
			const count = selectedPasswordItem ? (isEditing ? dndItems.length : displayFields.length) : 0;
			if (count > 0) {
				showSkeletonDetail = true;
				clearTimeout(skeletonTimerDetail);
				skeletonTimerDetail = setTimeout(() => {
					showSkeletonDetail = false;
				}, 200); // match list skeleton duration
			}
		})();
	}

	onDestroy(() => {
		clearTimeout(skeletonTimerDetail);
	});

	$: displayFields = (() => {
		if (!selectedPasswordItem) return [];

		const staticFields = [
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
			{ id: 'url', name: 'URL', value: selectedPasswordItem.url, type: 'text', icon: iconPaths.link },
			{
				id: 'notes',
				name: 'Notes',
				value: selectedPasswordItem.notes,
				type: 'multiline',
				icon: iconPaths.notes
			}
		];

		const customFields = (selectedPasswordItem.custom_fields || []).map((f) => ({
			...f,
			id: f.name,
			name: f.name,
			value: f.value,
			type: f.field_type,
			icon: iconPaths.plus
		}));

		let allFields = [...staticFields, ...customFields];

		if (selectedPasswordItem.field_order && selectedPasswordItem.field_order.length > 0) {
			const orderedFields: any[] = [];
			const fieldMap = new Map(allFields.map((field) => [field.id, field]));

			for (const fieldId of selectedPasswordItem.field_order) {
				if (fieldMap.has(fieldId)) {
					orderedFields.push(fieldMap.get(fieldId));
					fieldMap.delete(fieldId);
				}
			}
			for (const remainingField of fieldMap.values()) {
				orderedFields.push(remainingField);
			}
			return orderedFields;
		} else {
			return allFields;
		}
	})();

	

	function handleDndConsider(e: CustomEvent) {
		dndItems = e.detail.items;
	}

	function handleDndFinalize(e: CustomEvent) {
		dndItems = e.detail.items;
		dndItems = [...dndItems];
		if (selectedPasswordItem) {
			selectedPasswordItem.field_order = dndItems.map((item) => item.id);
		}
	}

	

	

	let originalPasswordItem: PasswordItem | null = null;

	$: {
		if (isEditing && originalPasswordItem) {
		const currentItemState = JSON.parse(JSON.stringify(originalPasswordItem));
			const newCustomFields: { name: string; value: string; field_type: string }[] = [];

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
						newCustomFields.push({ name: item.name, value: item.value, field_type: item.type });
						break;
			}
		}
		currentItemState.custom_fields = newCustomFields;
		currentItemState.field_order = dndItems.map((item) => item.id);

		// Normalize transient edit-state differences so clicking "Modify" alone
		// does not trigger the Unsaved Changes popup.
		const normalizedCurrent = JSON.parse(JSON.stringify(currentItemState));
		const normalizedOriginal = JSON.parse(JSON.stringify(originalPasswordItem));
		// Treat empty password during edit as placeholder 'N/A'
		if (normalizedCurrent.password === '') normalizedCurrent.password = 'N/A';
		if (normalizedOriginal.password == null) normalizedOriginal.password = 'N/A';
		// Treat empty URL as null (same as save normalization)
		if (normalizedCurrent.url === '') normalizedCurrent.url = null;

		hasUnsavedChanges = JSON.stringify(normalizedCurrent) !== JSON.stringify(normalizedOriginal);
	// Also consider pending tag changes
	if (!hasUnsavedChanges) {
		const origTags = (originalPasswordItem?.tags ?? '') as string;
		if (pendingTagOrder !== null && pendingTagOrder !== origTags) {
			hasUnsavedChanges = true;
		}
	}
	} else {
		hasUnsavedChanges = false;
	}
	}

function enterEditMode() {
	originalPasswordItem = JSON.parse(JSON.stringify(selectedPasswordItem));
	dndItems = [...displayFields];
	// If password is placeholder "N/A", clear it for editing
	for (const item of dndItems) {
		if (item.id === 'password' && (item.value === 'N/A' || item.value == null)) {
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
    const newCustomFields: { name: string; value: string; field_type: string }[] = [];
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
                    // Ensure URL passes backend validation
                    updated.url = (/^https?:\/\//i.test(val) ? val : `https://${val}`);
                }
                break;
            }
            case 'notes':
                updated.notes = val.length > 0 ? val : null;
                break;
            default:
                newCustomFields.push({ name: item.name, value: item.value, field_type: item.type });
                break;
        }
    }
    updated.custom_fields = newCustomFields;
    updated.field_order = dndItems.map((item) => item.id);

    if (JSON.stringify(updated) !== JSON.stringify(originalPasswordItem)) {
        try {
            await invoke('update_password_item', { item: updated });
            // sync local state with saved version and baseline
            selectedPasswordItem = updated;
            originalPasswordItem = JSON.parse(JSON.stringify(updated));
        } catch (error) {
            console.error('Error updating password item:', error);
            alert(`Failed to save changes: ${error}`);
            return; // stay in edit mode to correct
        }
    } else {
        // no field changes, continue to check tag reorder below
    }

    // Apply pending tag reorder only on Save
    // Note: allow empty string ('') to clear all tags
    if (pendingTagOrder !== null && pendingTagOrder !== (item.tags ?? '')) {
        try {
            await invoke('update_password_item_tags', { id: item.id, tags: pendingTagOrder });
            const updatedItem = { ...item, tags: pendingTagOrder } as any;
            selectedPasswordItem = updatedItem;
            dispatch('tagsSaved', { id: updatedItem.id, tags: pendingTagOrder });
        } catch (error) {
            console.error('Error saving tag order:', error);
            alert(`Failed to save tag order: ${error}`);
            return; // stay in edit mode if tag save fails
        }
    }

    // Exit edit mode after a short delay so color transitions
    // in the edit view can play before the DOM branch swaps.
    hasUnsavedChanges = false;
    pendingTagOrder = null;
    await tick();
    setTimeout(() => { isEditing = false; }, 320);
}

	function handleReset() {
		if (originalPasswordItem) {
			selectedPasswordItem = JSON.parse(JSON.stringify(originalPasswordItem));
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
			on:enterEditMode={enterEditMode}
			on:handleReset={handleReset}
			on:save={handleSave}
			on:tagsReorderedPending={(e) => { pendingTagOrder = e.detail?.tags ?? null; }}
			on:tagsSaved={(e) => dispatch('tagsSaved', e.detail)}
			on:removeEntry={(event) => dispatch('removeEntry', event.detail)}
		/>

		<div class="detail-group" aria-busy={showSkeletonDetail}>
			{#if !isEditing}
				{#if showSkeletonDetail}
					{#each displayFields as _field, i}
						<div class="detail-skel-row" aria-hidden="true">
							<div class="detail-skel-avatar" aria-hidden="true"></div>
							<div class="detail-skel-texts">
								<div class="detail-skel-line title" aria-hidden="true"></div>
								<div class="detail-skel-line desc" aria-hidden="true"></div>
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
									<button
										type="button"
										class="password-toggle"
										on:click={() => (showPassword = !showPassword)}
									>
										<Icon
											path={showPassword ? iconPaths.eye : iconPaths.eyeOff}
											size="24"
											color="currentColor"
										/>
									</button>
								{/if}
						</div>
						</Input>
					{/each}
				{/if}
			{:else}
				{#if showSkeletonDetail}
					{#each dndItems as _item, i}
						<div class="detail-skel-row" aria-hidden="true">
							<div class="detail-skel-avatar" aria-hidden="true"></div>
							<div class="detail-skel-texts">
								<div class="detail-skel-line title" aria-hidden="true"></div>
								<div class="detail-skel-line desc" aria-hidden="true"></div>
							</div>
						</div>
					{/each}
				{:else}
				<div
					class="detail-group"
					use:dndzone={{ items: dndItems, flipDurationMs: 300, dropFromOthersDisabled: true }}
					on:consider={handleDndConsider}
					on:finalize={handleDndFinalize}
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
										style="display: flex; align-items: center; gap: 10px;"
									>
										{#if field.id === 'password'}
											<button
												type="button"
												class="password-toggle"
												on:click={() => (showPassword = !showPassword)}
											>
												<Icon
													path={showPassword ? iconPaths.eye : iconPaths.eyeOff}
													size="24"
													color="currentColor"
												/>
											</button>
										{/if}
										{#if isEditing}
											<div class="drag-handle" data-dnd-handle>
												<Icon path={iconPaths.reorder} size="24" color="currentColor" />
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
				<button class="add-field-button" on:click={() => (addingField = true)}>
					<Icon path={iconPaths.plus} color="#fff" size="20" />
				</button>
			{:else}
				<div class="new-field-container">
					<button class="icon-button">
						<Icon path={iconPaths.plus} size="24" color="currentColor" />
					</button>
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
					<select class="field-type-dropdown" bind:value={newFieldType}>
						<option value="text">Text</option>
						<option value="password">Password</option>
						<option value="file">File</option>
					</select>
				</div>
				<div class="new-field-actions">
					<button class="cancel-button" on:click={handleCancelAddField}>Cancel</button>
					<button class="confirm-button" on:click={handleConfirmAddField}>Confirm</button>
				</div>
			{/if}
		{/if}

		<div class="collapsible-section">
			<button
				type="button"
				class="collapsible-header"
				class:expanded={showTimestamps}
				on:click={() => (showTimestamps = !showTimestamps)}
			>
				<h3>Metadata</h3>
				<Icon
					path={showTimestamps ? iconPaths.arrowUp : iconPaths.arrowDown}
					size="20"
					color="currentColor"
				/>
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
        <!-- Full overlay skeleton to cover entire detail page -->
        <div class="detail-skeleton-overlay" aria-hidden="true">
            
            <div class="sk-header-block">
                <div class="sk-top">
                    <div class="sk-header">
                        <div class="sk-avatar"></div>
                        <div class="sk-texts">
                            <div class="sk-line title"></div>
                            <div class="sk-line subtitle"></div>
                        </div>
                    </div>
                <div class="sk-actions">
                    <div class="sk-edit-btn" aria-hidden="true"></div>
                    <div class="sk-more-btn" aria-hidden="true"></div>
                </div>
                </div>
                <div class="sk-tags">
                    {#each (selectedPasswordItem?.tags ? selectedPasswordItem.tags.split(',').map(t => t.trim()).filter(Boolean) : []) as _tag}
                        <div class="sk-chip" aria-hidden="true"></div>
                    {/each}
                </div>
            </div>
            <div class="sk-fields">
                {#each Array((isEditing ? dndItems.length : displayFields.length) || 6) as _, i}
                    <div class="sk-field-row">
                        <div class="sk-icon"></div>
                        <div class="sk-lines">
                            <div class="sk-line title"></div>
                            <div class="sk-line desc"></div>
                        </div>
                    </div>
                {/each}
            </div>
            <div class="sk-sections">
                <div class="sk-section-title"></div>
                <div class="sk-line wide"></div>
                <div class="sk-line wide faint"></div>
            </div>
        </div>
    {/if}
	{#if hasUnsavedChanges}
		<UnsavedChangesPopup on:save={handleSave} on:reset={handleReset} />
	{/if}
</main>

<style>
	/* Detail skeleton styles */
	@keyframes detail-skeleton-shimmer {
		0% { background-position: -160px 0; }
		100% { background-position: 160px 0; }
	}

	.detail-group[aria-busy="true"] {
		pointer-events: none;
	}

	.detail-skel-row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 0;
	}

    .detail-skel-avatar {
        width: 20px;
        height: 20px;
        border-radius: 6px;
        background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
        background-size: 400px 100%;
        animation: detail-skeleton-shimmer 0.8s ease-in-out infinite;
    }

	.detail-skel-texts {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

    .detail-skel-line {
        height: 10px;
        border-radius: 6px;
        background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
        background-size: 400px 100%;
        animation: detail-skeleton-shimmer 0.8s ease-in-out infinite;
    }

	.detail-skel-line.title { width: 220px; }
	.detail-skel-line.desc { width: 160px; opacity: 0.6; }
    .detail-anim {
        will-change: opacity, transform;
    }
	.passwordDetail {
		padding: 20px;
		background-color: var(--main-bg);
		color: var(--white);
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
		gap: 12px;
		outline: none !important;
	}

	/* Smoothen FLIP movement during drag swaps */
	.dnd-item {
		will-change: transform;
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
		color: #bbb;
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
		color: #888;
		margin-top: 50px;
	}

	.passwordDetail {
		display: flex;
		flex-direction: column;
	}

	/* Stack inputs with tighter vertical rhythm */
	.detail-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
		outline: none !important;
	}

	.add-field-button {
		width: 100%;
		height: 56px;
		background-color: var(--near-black);
		border: none;
		border-radius: 10px;
		display: flex;
		justify-content: center;
		align-items: center;
		cursor: pointer;
		margin-top: 12px;
	}

	.add-field-button:hover {
		background-color: #17171b;
	}

	.collapsible-section {
		margin-top: 20px;
		border: 1px solid var(--btn-nav-border);
		border-radius: 8px;
	}

	.collapsible-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 10px 15px;
		background-color: var(--near-black);
		cursor: pointer;
		font-weight: 500;
		color: var(--white);
		border: none;
		width: 100%;
	}

	.collapsible-header:hover {
		background-color: #17171b;
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
		background-color: var(--main-bg);
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

	.password-toggle {
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		margin-left: 10px;
		background: transparent;
		border: none;
		color: white;
	}

	.password-toggle :global(svg) {
		width: 24px;
		height: 24px;
		fill: currentColor;
	}

	.new-field-container {
		display: flex;
		gap: 10px;
		align-items: center;
		margin-top: 12px;
	}

	.field-type-dropdown {
		background-color: var(--near-black);
		color: var(--white);
		border: 1px solid var(--btn-nav-border);
		border-radius: 5px;
		padding: 8px;
		font-size: 14px;
	}

	.new-field-actions {
		display: flex;
		justify-content: flex-end;
		gap: 10px;
		margin-top: 12px;
	}

	.cancel-button,
	.confirm-button {
		padding: 8px 16px;
		border-radius: 5px;
		border: none;
		cursor: pointer;
		font-size: 14px;
	}

	.cancel-button {
		background-color: #444;
		color: var(--white);
	}

	.confirm-button {
		background-color: #007bff;
		color: var(--white);
	}

	.icon-button {
		background: transparent;
		border: none;
		color: var(--white);
		cursor: pointer;
		padding: 0;
		display: flex;
		align-items: center;
	}

	.drag-handle {
		cursor: grab;
		margin-left: 10px;
	}

	.dnd-item {
		touch-action: none;
	}

    /* Hide live content while skeleton overlay is shown */
    .detail-content[aria-hidden="true"] { visibility: hidden; pointer-events: none; }

    /* Overlay skeleton covering entire detail page */
    .detail-skeleton-overlay {
        position: absolute;
        inset: 0;
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 0; /* control spacing explicitly via header/sections margins */
        background: var(--main-bg);
        pointer-events: none;
    }
    .sk-header-block { display: flex; flex-direction: column; gap: 5px; margin-bottom: 20px; }
    .sk-top { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
    .sk-header { display: flex; align-items: center; gap: 10px; }
    .sk-avatar {
        width: 24px; height: 24px; border-radius: 6px;
        background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
        background-size: 400px 100%;
        animation: detail-skeleton-shimmer 0.8s ease-in-out infinite;
    }
    .sk-texts { display: flex; flex-direction: column; gap: 8px; }
    .sk-line {
        height: 10px; border-radius: 6px;
        background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
        background-size: 400px 100%;
        animation: detail-skeleton-shimmer 0.8s ease-in-out infinite;
    }
    .sk-line.title { width: 260px; }
    .sk-line.subtitle { width: 180px; opacity: 0.7; }
    .sk-tags { display: flex; flex-wrap: wrap; gap: 12px; margin-top: 5px; visibility: hidden; }
    .sk-chip {
        height: 22px; min-width: 38px; border-radius: 11px;
        background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
        background-size: 400px 100%;
        animation: detail-skeleton-shimmer 0.8s ease-in-out infinite;
    }
    .sk-actions { display: flex; align-items: cevnter; gap: 35px;  margin-bottom: -20px;}
    .sk-edit-btn {
        width: 68px; height: 20px; border-radius: 14px;
        background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
        background-size: 400px 100%;
        animation: detail-skeleton-shimmer 0.8s ease-in-out infinite;
    }
    
    .sk-fields { display: flex; flex-direction: column;} /* match .detail-group gap */
    .sk-field-row { display: flex; align-items: center; gap: 14px; min-height: 56px; padding: 0px 12px; }
    .sk-icon {
        width: 19px; height: 19px; border-radius: 5px;
        background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%);
        background-size: 400px 100%;
        animation: detail-skeleton-shimmer 0.8s ease-in-out infinite;
    }
    .sk-lines { display: flex; flex-direction: column; gap: 8px; }
    .sk-line.desc { width: 200px; opacity: 0.6; }
    .sk-sections { display: flex; flex-direction: column; gap: 8px; margin-top: 20px; visibility: hidden; }
    .sk-section-title { width: 120px; height: 10px; border-radius: 6px; background: linear-gradient(90deg, #1f1f24 25%, #2a2a30 37%, #1f1f24 63%); background-size: 400px 100%; animation: detail-skeleton-shimmer 0.8s ease-in-out infinite; visibility: hidden; }
    .sk-line.wide { width: 60%; height: 10px; }
    .sk-line.wide.faint { width: 45%; height: 10px; opacity: 0.5; }
</style>
