<svelte:options runes />

<script lang="ts">
        import type { PasswordItem } from '$lib/types/password';
        import type { DisplayField, TotpDisplayField } from '$lib/types/password-fields';
        import { isTotpDisplayField } from '$lib/types/password-fields';
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
        import { Copy, Plus, RefreshCw, ShieldCheck, Trash2 } from '@lucide/svelte';
        import { buildDisplayFields } from '$lib/utils/passwordFields';
        import { copyText } from '$lib/utils/copyHelper';

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

        const toErrorMessage = (error: unknown): string =>
                error instanceof Error
                        ? error.message
                        : typeof error === 'string'
                        ? error
                        : JSON.stringify(error);

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
        let totpSecondsRemaining = $state(0);
        let totpCode = $state<string | null>(null);
        let totpCodeError = $state<string | null>(null);
        let totpVerificationCode = $state('');
        let totpVerificationError = $state<string | null>(null);
        let totpSuccessMessage = $state<string | null>(null);
        let totpActionError = $state<string | null>(null);
        let isTotpGenerating = $state(false);
        let isTotpVerifying = $state(false);
        let isTotpRemoving = $state(false);
        let totpDigits = $state(6);
        let totpPeriod = $state(30);
        let totpTimer: ReturnType<typeof setInterval> | null = null;
        let lastTotpSecret: string | null = null;
        let lastTotpPeriod = 30;
        let totpTokenRefreshing = false;
        let lastSelectedItemId: number | null = null;

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
                                        ? (isEditing ? dndItems.length : filteredDisplayFields.length)
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
                stopTotpTimer();
        });

        let displayFields = $state<DisplayField[]>([]);
        let filteredDisplayFields = $state<DisplayField[]>([]);
        let totpSecret = $state<string | null>(null);
        let formattedTotpCode = $state('------');
        let currentTotpField: TotpDisplayField | undefined;

        $effect(() => {
                displayFields = buildDisplayFields(selectedPasswordItem, iconPaths);

        });

        $effect(() => {
                const currentId = selectedPasswordItem?.id ?? null;
                if (currentId !== lastSelectedItemId) {
                        lastSelectedItemId = currentId;
                        totpSuccessMessage = null;
                        totpActionError = null;
                        totpVerificationCode = '';
                        totpVerificationError = null;
                        totpCodeError = null;
                }
        });

        $effect(() => {
                filteredDisplayFields = displayFields.filter((field) => !isTotpDisplayField(field));
                currentTotpField = displayFields.find(isTotpDisplayField);
                totpSecret = currentTotpField?.meta.secret ?? null;
        });

        $effect(() => {
                const secret = currentTotpField?.meta.secret ?? null;
                const period = currentTotpField?.meta.period ?? 30;
                const digits = currentTotpField?.meta.digits ?? 6;

                totpDigits = digits;
                totpPeriod = period;

                if (secret !== lastTotpSecret || period !== lastTotpPeriod) {
                        lastTotpSecret = secret;
                        lastTotpPeriod = period;
                        totpVerificationCode = '';
                        totpVerificationError = null;
                        totpCodeError = null;
                        formattedTotpCode = '------';

                        if (secret) {
                                startTotpTimer(secret, period);
                        } else {
                                stopTotpTimer();
                                totpSecondsRemaining = 0;
                                totpCode = null;
                        }
                }
        });

        function formatTotpDisplay(value: string | null, digits: number): string {
                if (!value) {
                        return '------';
                }
                const normalized = value.padStart(Math.max(1, digits), '0');
                return normalized.replace(/(.{3})/g, '$1 ').trim();
        }

        function stopTotpTimer() {
                if (totpTimer) {
                        clearInterval(totpTimer);
                        totpTimer = null;
                }
                totpSecondsRemaining = 0;
                formattedTotpCode = '------';
        }

        function updateTotpRemaining(period: number) {
                if (!period) {
                        totpSecondsRemaining = 0;
                        return;
                }
                const nowSeconds = Math.floor(Date.now() / 1000);
                const remainder = nowSeconds % period;
                const remaining = period - remainder;
                totpSecondsRemaining = remaining === period ? period : remaining;
        }

        async function refreshTotpToken(secret: string) {
                if (totpTokenRefreshing) {
                        return;
                }
                totpTokenRefreshing = true;
                try {
                        const token = await invoke<string>('generate_totp', { secret_b32: secret });
                        const digits = Math.max(1, totpDigits);
                        const padded = token.padStart(digits, '0');
                        totpCode = padded;
                        formattedTotpCode = formatTotpDisplay(padded, digits);
                        totpCodeError = null;
                } catch (error) {
                        totpCode = null;
                        totpCodeError = toErrorMessage(error);
                        formattedTotpCode = '------';
                } finally {
                        totpTokenRefreshing = false;
                }
        }

        function startTotpTimer(secret: string, period: number) {
                stopTotpTimer();
                if (!secret) {
                        return;
                }
                const effectivePeriod = period || 30;
                let lastStep = -1;
                const tickTimer = async () => {
                        const nowSeconds = Math.floor(Date.now() / 1000);
                        const step = Math.floor(nowSeconds / effectivePeriod);
                        updateTotpRemaining(effectivePeriod);
                        if (step !== lastStep) {
                                lastStep = step;
                        await refreshTotpToken(secret);
                        }
                };
                void tickTimer();
                totpTimer = setInterval(() => {
                        void tickTimer();
                }, 1000);
        }

        function handleDndConsider(e: CustomEvent<{ items: DisplayField[] }>) {
                dndItems = e.detail.items;
        }

	function handleDndFinalize(e: CustomEvent<{ items: DisplayField[] }>) {
		const items = e.detail.items;
		dndItems = [...items];
		if (selectedPasswordItem) {
			const updatedOrder = dndItems.map((item) => item.id);
			selectedPasswordItem = {
				...selectedPasswordItem,
				field_order: updatedOrder
			};
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
            currentItemState.password = item.value ?? '';
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
              value: item.value ?? '',
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

      if (normalizedCurrent.password === 'N/A') {
        normalizedCurrent.password = '';
      }
      if (normalizedOriginal.password === 'N/A') {
        normalizedOriginal.password = '';
      }
      if (normalizedCurrent.password == null) {
        normalizedCurrent.password = '';
      }
      if (normalizedOriginal.password == null) {
        normalizedOriginal.password = '';
      }
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

  export function enterEditMode() {
    originalPasswordItem = JSON.parse(
      JSON.stringify(selectedPasswordItem)
    );
    dndItems = [...filteredDisplayFields];
    for (const item of dndItems) {
      if (
        item.id === 'password' &&
        (item.value === 'N/A' || item.value == null || item.value === '')
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
          updated.password = val.length > 0 ? val : '';
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
            value: val,
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
    const trimmedName = newFieldName.trim();
    if (!trimmedName) {
      alert('Please enter a name for the new field.');
      return;
    }

    if (!selectedPasswordItem) {
      return;
    }

    try {
      await invoke('add_custom_field', {
        itemId: selectedPasswordItem.id,
        fieldName: trimmedName,
        fieldType: newFieldType
      });

      const newCustomField = { name: trimmedName, value: '', field_type: newFieldType };
      const updatedCustomFields = [
        ...selectedPasswordItem.custom_fields,
        newCustomField
      ];
      const updatedFieldOrder = [
        ...(selectedPasswordItem.field_order ?? []),
        trimmedName
      ];

      const updatedItem = {
        ...selectedPasswordItem,
        custom_fields: updatedCustomFields,
        field_order: updatedFieldOrder
      };

      selectedPasswordItem = updatedItem;
      displayFields = buildDisplayFields(updatedItem, iconPaths);
      dndItems = [
        ...dndItems,
        {
          id: trimmedName,
          name: trimmedName,
          value: '',
          type: newFieldType,
          icon: iconPaths.plus
        }
      ];

      if (originalPasswordItem) {
        const updatedOriginalFieldOrder =
          originalPasswordItem.field_order == null
            ? originalPasswordItem.field_order
            : [...originalPasswordItem.field_order, trimmedName];

        originalPasswordItem = JSON.parse(
          JSON.stringify({
            ...originalPasswordItem,
            custom_fields: [
              ...originalPasswordItem.custom_fields,
              { ...newCustomField }
            ],
            field_order: updatedOriginalFieldOrder
          })
        );
      }

      handleCancelAddField();
    } catch (error) {
      console.error('Error adding custom field:', error);
      alert(`Failed to add custom field: ${error}`);
    }
  }

        function handleTotpCodeInput(rawValue: string) {
                const digits = Math.max(1, totpDigits);
                const cleaned = rawValue.replace(/\D/g, '').slice(0, digits);
                if (cleaned !== totpVerificationCode) {
                        totpVerificationCode = cleaned;
                }
                if (totpVerificationError && cleaned.length < digits) {
                        totpVerificationError = null;
                }
        }

        async function handleCopyTotpSecret() {
                if (!totpSecret) {
                        return;
                }
                try {
                        await copyText(totpSecret);
                } catch (error) {
                        totpActionError = toErrorMessage(error);
                }
        }

        async function handleCopyTotpCode() {
                if (!totpCode) {
                        return;
                }
                try {
                        await copyText(totpCode);
                } catch (error) {
                        totpActionError = toErrorMessage(error);
                }
        }

        async function handleGenerateTotpSecret(mode: 'create' | 'rotate') {
                if (!selectedPasswordItem) {
                        return;
                }
                isTotpGenerating = true;
                totpActionError = null;
                totpSuccessMessage = null;
                try {
                        const secret = await invoke<string>('generate_totp_secret');
                        await invoke('update_password_item_totp_secret', {
                                id: selectedPasswordItem.id,
                                totpSecret: secret
                        });
                        const updatedItem: PasswordItem = {
                                ...selectedPasswordItem,
                                totp_secret: secret
                        };
                        selectedPasswordItem = updatedItem;
                        if (originalPasswordItem) {
                                originalPasswordItem = JSON.parse(
                                        JSON.stringify({
                                                ...originalPasswordItem,
                                                totp_secret: secret
                                        })
                                );
                        }
                        totpSuccessMessage =
                                mode === 'create'
                                        ? 'TOTP secret generated. Add it to your authenticator and verify below.'
                                        : 'TOTP secret rotated. Update your authenticator and verify below.';
                        totpVerificationCode = '';
                        totpVerificationError = null;
                        totpCodeError = null;
                        formattedTotpCode = '------';
                } catch (error) {
                        totpActionError = toErrorMessage(error);
                } finally {
                        isTotpGenerating = false;
                }
        }

        async function handleRemoveTotpSecret() {
                if (!selectedPasswordItem) {
                        return;
                }
                const confirmed =
                        typeof window === 'undefined'
                                ? true
                                : window.confirm(
                                          'Remove the TOTP secret for this entry? One-time codes will no longer be generated.'
                                  );
                if (!confirmed) {
                        return;
                }
                isTotpRemoving = true;
                totpActionError = null;
                totpSuccessMessage = null;
                try {
                        await invoke('update_password_item_totp_secret', {
                                id: selectedPasswordItem.id,
                                totpSecret: null
                        });
                        const updatedItem: PasswordItem = {
                                ...selectedPasswordItem,
                                totp_secret: null
                        };
                        selectedPasswordItem = updatedItem;
                        if (originalPasswordItem) {
                                originalPasswordItem = JSON.parse(
                                        JSON.stringify({
                                                ...originalPasswordItem,
                                                totp_secret: null
                                        })
                                );
                        }
                        stopTotpTimer();
                        totpCode = null;
                        totpCodeError = null;
                        totpVerificationCode = '';
                        totpVerificationError = null;
                        totpSuccessMessage = 'TOTP secret removed.';
                } catch (error) {
                        totpActionError = toErrorMessage(error);
                } finally {
                        isTotpRemoving = false;
                }
        }

        async function handleVerifyTotp(event?: Event) {
                event?.preventDefault?.();
                if (!selectedPasswordItem?.totp_secret) {
                        totpVerificationError = 'Generate a TOTP secret before verifying a code.';
                        return;
                }
                const digits = Math.max(1, totpDigits);
                if (totpVerificationCode.length !== digits) {
                        totpVerificationError = `Enter the ${digits}-digit code from your authenticator.`;
                        return;
                }
                isTotpVerifying = true;
                totpVerificationError = null;
                totpActionError = null;
                try {
                        const isValid = await invoke<boolean>('verify_totp', {
                                id: selectedPasswordItem.id,
                                token: totpVerificationCode
                        });
                        if (!isValid) {
                                totpVerificationError = 'Code did not match. Try again.';
                                totpSuccessMessage = null;
                                return;
                        }
                        totpSuccessMessage = 'TOTP configuration confirmed.';
                        totpVerificationCode = '';
                } catch (error) {
                        totpVerificationError = toErrorMessage(error);
                        totpSuccessMessage = null;
                } finally {
                        isTotpVerifying = false;
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
                        displayFields={filteredDisplayFields}
                        bind:editingFields={dndItems}
                        displayColor={displayColor}
                        bind:showPassword
                        showSkeleton={showSkeletonDetail}
                        viewSkeletonPlaceholders={
                                showSkeletonDetail
                                        ? createPlaceholders(filteredDisplayFields.length)
                                        : []
                        }
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

                <section class="mt-4 rounded-xl border border-border/60 bg-[color:var(--passworddetail-elevated)] p-4 shadow-sm">
                        <div class="flex flex-wrap items-center justify-between gap-3">
                                <div class="flex flex-col gap-1">
                                        <h2 class="text-sm font-semibold text-[color:var(--passworddetail-strong-text)]">
                                                Authenticator (TOTP)
                                        </h2>
                                        <p class="text-xs text-[color:var(--passworddetail-muted-text)]">
                                                Generate one-time passwords for this entry.
                                        </p>
                                </div>
                                <div class="flex flex-wrap items-center gap-2">
                                        {#if totpSecret}
                                                <Button
                                                        type="button"
                                                        variant="outline"
                                                        size="sm"
                                                        class="gap-1"
                                                        disabled={isTotpGenerating || isTotpRemoving}
                                                        onclick={() => handleGenerateTotpSecret('rotate')}
                                                >
                                                        <RefreshCw class="h-4 w-4" />
                                                        Rotate secret
                                                </Button>
                                                <Button
                                                        type="button"
                                                        variant="destructive"
                                                        size="sm"
                                                        class="gap-1"
                                                        disabled={isTotpGenerating || isTotpRemoving}
                                                        onclick={handleRemoveTotpSecret}
                                                >
                                                        <Trash2 class="h-4 w-4" />
                                                        Remove
                                                </Button>
                                        {:else}
                                                <Button
                                                        type="button"
                                                        class="gap-1"
                                                        disabled={isTotpGenerating}
                                                        onclick={() => handleGenerateTotpSecret('create')}
                                                >
                                                        <Plus class="h-4 w-4" />
                                                        Enable TOTP
                                                </Button>
                                        {/if}
                                </div>
                        </div>
                        {#if totpSecret}
                                <div class="mt-4 flex flex-col gap-4">
                                        <div class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border/60 bg-[color:var(--passworddetail-secondary-surface)] px-3 py-2">
                                                <div class="min-w-0">
                                                        <p class="text-xs uppercase tracking-wide text-[color:var(--passworddetail-subtle-text)]">
                                                                Secret
                                                        </p>
                                                        <p class="break-all font-mono text-sm text-[color:var(--passworddetail-strong-text)]">
                                                                {totpSecret}
                                                        </p>
                                                </div>
                                                <Button
                                                        type="button"
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-8 w-8 text-[color:var(--passworddetail-muted-text)] hover:text-[color:var(--passworddetail-strong-text)]"
                                                        onclick={handleCopyTotpSecret}
                                                        title="Copy TOTP secret"
                                                >
                                                        <Copy class="h-4 w-4" />
                                                </Button>
                                        </div>
                                        <div class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border/60 bg-[color:var(--passworddetail-secondary-surface)] px-3 py-3">
                                                <div>
                                                        <p class="text-xs uppercase tracking-wide text-[color:var(--passworddetail-subtle-text)]">
                                                                Current code
                                                        </p>
                                                        <p class="font-mono text-2xl font-semibold tracking-[0.35em] text-[color:var(--passworddetail-strong-text)]">
                                                                {formattedTotpCode}
                                                        </p>
                                                        <p class="text-xs text-[color:var(--passworddetail-muted-text)]">
                                                                Refreshes in {totpSecondsRemaining > 0 ? `${totpSecondsRemaining}s` : '--'} (every {totpPeriod}s)
                                                        </p>
                                                        {#if totpCodeError}
                                                                <p class="mt-1 text-xs text-destructive">{totpCodeError}</p>
                                                        {/if}
                                                </div>
                                                <Button
                                                        type="button"
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-9 w-9 text-[color:var(--passworddetail-muted-text)] hover:text-[color:var(--passworddetail-strong-text)]"
                                                        onclick={handleCopyTotpCode}
                                                        title="Copy current TOTP code"
                                                        disabled={!totpCode}
                                                >
                                                        <Copy class="h-5 w-5" />
                                                </Button>
                                        </div>
                                        <form class="flex flex-col gap-2" onsubmit={handleVerifyTotp}>
                                                <label
                                                        for="totp-verification"
                                                        class="text-xs font-medium text-[color:var(--passworddetail-muted-text)]"
                                                >
                                                        Confirm with a code
                                                </label>
                                                <div class="flex flex-wrap items-center gap-3">
                                                        <input
                                                                type="text"
                                                                id="totp-verification"
                                                                inputmode="numeric"
                                                                autocomplete="one-time-code"
                                                                pattern="[0-9]*"
                                                                maxlength={totpDigits}
                                                                class="flex-1 rounded-md border border-border/60 bg-[color:var(--passworddetail-secondary-surface)] px-3 py-2 text-sm text-[color:var(--passworddetail-strong-text)] focus:outline-none focus:ring-2 focus:ring-[color:var(--passworddetail-accent)]"
                                                                value={totpVerificationCode}
                                                                oninput={(event) =>
                                                                        handleTotpCodeInput(
                                                                                (event.currentTarget as HTMLInputElement).value
                                                                        )
                                                                }
                                                        />
                                                        <Button
                                                                type="submit"
                                                                class="gap-1"
                                                                disabled={
                                                                        isTotpVerifying ||
                                                                        isTotpGenerating ||
                                                                        isTotpRemoving ||
                                                                        totpVerificationCode.length !== Math.max(1, totpDigits)
                                                                }
                                                        >
                                                                <ShieldCheck class="h-4 w-4" />
                                                                Verify code
                                                        </Button>
                                                </div>
                                                <p class="text-xs text-[color:var(--passworddetail-subtle-text)]">
                                                        Enter the {totpDigits}-digit code from your authenticator to confirm the secret works.
                                                </p>
                                                {#if totpVerificationError}
                                                        <p class="text-sm text-destructive">{totpVerificationError}</p>
                                                {/if}
                                        </form>
                                </div>
                        {:else}
                                <p class="mt-4 text-sm text-[color:var(--passworddetail-muted-text)]">
                                        No authenticator secret configured. Generate one to create time-based one-time passwords for this entry.
                                </p>
                        {/if}
                        {#if totpActionError}
                                <p class="mt-3 text-sm text-destructive">{totpActionError}</p>
                        {/if}
                        {#if totpSuccessMessage}
                                <p class="mt-3 text-sm text-emerald-400">{totpSuccessMessage}</p>
                        {/if}
                </section>

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
                                {#each createPlaceholders(isEditing ? dndItems.length : filteredDisplayFields.length) as _}
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





