<svelte:options runes />

<script lang="ts">
  import { untrack } from 'svelte';
  import { dndzone } from 'svelte-dnd-action';
  import { flip } from 'svelte/animate';
  import { cubicOut } from 'svelte/easing';
  import {
    Eye,
    EyeOff,
    ArrowDownUp,
    Copy,
    ShieldAlert,
    ShieldCheck,
    Shield,
    WandSparkles,
    Trash2
  } from '@lucide/svelte';
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger
  } from '$lib/components/ui/context-menu';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
  } from '$lib/components/ui/dialog';
  import Input from '$lib/components/ui/FieldInput.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import type { DisplayField } from '$lib/types/password-fields';
  import type { PasswordItem } from '$lib/types/password';
  import { SecurityService } from '$lib/utils/security';
  import { copyPassword, copyText, copyUrl, copyUsername } from '$lib/utils/copyHelper';
  import { toast } from '$lib/components/ui/sonner';
  import { securityDashboard } from '$lib/stores/security-dashboard.svelte';
  import PasswordStrength from '$lib/components/password/PasswordStrength.svelte';
  import PasswordGeneratorPopup from '$lib/components/password/PasswordGeneratorPopup.svelte';

  interface Props {
    isEditing: boolean;
    displayFields: DisplayField[];
    editingFields: DisplayField[];
    displayColor: string;
    showPassword: boolean;
    showSkeleton: boolean;
    viewSkeletonPlaceholders: unknown[];
    editSkeletonPlaceholders: unknown[];
    passwordItem: PasswordItem | null;
    onconsider?: (detail: { items: DisplayField[] }) => void;
    onfinalize?: (detail: { items: DisplayField[] }) => void;
  }

  let {
    isEditing,
    displayFields,
    editingFields = $bindable<DisplayField[]>([]),
    displayColor,
    showPassword = $bindable(false),
    showSkeleton,
    viewSkeletonPlaceholders = [],
    editSkeletonPlaceholders = [],
    passwordItem = null,
    onconsider,
    onfinalize
  }: Props = $props();

  let breachCheckTimeout: ReturnType<typeof setTimeout> | null = null;
  let showGeneratorPopup = $state(false);
  let activeGeneratorFieldId = $state<string | null>(null);
  let isDeleteDialogOpen = $state(false);
  let fieldToDelete = $state<DisplayField | null>(null);
  let localBreachData = $state<Record<string, { isBreached: boolean; count: number }>>({});

  $effect(() => {
    const fields = isEditing ? editingFields : displayFields;
    const passwordValues = fields
      .filter((f) => f.id === 'password' || f.type === 'password')
      .map((f) => f.value)
      .filter((v): v is string => !!v && v !== 'N/A');

    const uniquePasswords = [...new Set(passwordValues)];
    const timers: ReturnType<typeof setTimeout>[] = [];

    uniquePasswords.forEach((pwd) => {
      if (localBreachData[pwd] !== undefined) return;

      const timer = setTimeout(async () => {
        try {
          const count = await SecurityService.checkBreach(pwd);
          // Only update if not already set (to avoid racing with other effects)
          if (localBreachData[pwd] === undefined) {
            localBreachData[pwd] = { isBreached: count > 0, count };
          }
        } catch (e) {
          console.error('Breach check failed', e);
        }
      }, 800);
      timers.push(timer);
    });

    return () => {
      timers.forEach(t => clearTimeout(t));
    };
  });

  $effect(() => {
    const item = passwordItem;
    if (item && item.id && item.password) {
      untrack(() => {
        securityDashboard.assessStrength(item);
      });
    }
  });

  $effect(() => {
    const item = passwordItem;
    if (item && item.id && item.password) {
      const health = untrack(() => securityDashboard.items[item.id!]);
      if (!health || health.isBreached === null) {
        if (breachCheckTimeout) clearTimeout(breachCheckTimeout);
        breachCheckTimeout = setTimeout(() => {
          untrack(() => {
            if (passwordItem) securityDashboard.checkBreach(passwordItem);
          });
        }, 1000);
      }
    }

    return () => {
      if (breachCheckTimeout) clearTimeout(breachCheckTimeout);
    };
  });

  function handlePasswordGenerated(newPass: string) {
    const targetId = activeGeneratorFieldId || 'password';
    const passField = editingFields.find((f) => f.id === targetId);
    if (passField) {
      passField.value = newPass;
      showPassword = true;
    }
    activeGeneratorFieldId = null;
  }

  function openGenerator(fieldId: string) {
    activeGeneratorFieldId = fieldId;
    showGeneratorPopup = true;
  }

  function handleConsider(event: CustomEvent<{ items: DisplayField[] }>) {
    editingFields = event.detail.items;
    onconsider?.(event.detail);
  }

  function handleFinalize(event: CustomEvent<{ items: DisplayField[] }>) {
    editingFields = event.detail.items;
    onfinalize?.(event.detail);
  }

  function togglePasswordVisibility() {
    showPassword = !showPassword;
  }

  function getIconName(field: DisplayField): string {
    if (field.id === 'username') return 'user';
    if (field.id === 'password') return 'key';
    return field.type;
  }

  function getDisplayValue(field: DisplayField): string {
    if (field.id === 'password') {
      return field.value && field.value.length ? field.value : 'N/A';
    }
    return field.value && field.value.length > 0 ? field.value : 'N/A';
  }

  function getInputType(field: DisplayField, isViewMode: boolean): 'text' | 'password' | 'url' {
    const isPassword = field.id === 'password' || field.type === 'password';
    const isUrl = field.id === 'url' || field.type === 'url';

    if (isUrl) return 'url';

    if (isPassword) {
      if (isViewMode) {
        const hasValue = field.value && field.value.length > 0 && field.value !== 'N/A';
        if (!hasValue) return 'text';
      }
      return showPassword ? 'text' : 'password';
    }

    return 'text';
  }

  function canCopyField(field: DisplayField): boolean {
    if (!field || field.type === 'totp') {
      return false;
    }

    if (field.id === 'password') {
      return typeof field.value === 'string' && field.value.length > 0;
    }

    if (typeof field.value !== 'string') {
      return false;
    }

    return field.value.trim().length > 0;
  }

  function getCopySuccessMessage(field: DisplayField): string {
    const messages: Record<string, string> = {
      password: 'Password copied to clipboard.',
      username: 'Username copied to clipboard.',
      url: 'URL copied to clipboard.',
      notes: 'Notes copied to clipboard.'
    };

    return messages[field.id] ?? `${field.name} copied to clipboard.`;
  }

  function getCopyErrorMessage(field: DisplayField): string {
    const messages: Record<string, string> = {
      password: 'Failed to copy password.',
      username: 'Failed to copy username.',
      url: 'Failed to copy URL.',
      notes: 'Failed to copy notes.'
    };

    return messages[field.id] ?? `Failed to copy ${field.name}.`;
  }

  async function handleCopyField(field: DisplayField) {
    if (!canCopyField(field)) {
      toast.error(`Nothing to copy for ${field.name}.`);
      return;
    }

    try {
      if (!isEditing && passwordItem) {
        switch (field.id) {
          case 'password':
            await copyPassword(passwordItem);
            break;
          case 'username':
            await copyUsername(passwordItem);
            break;
          case 'url':
            await copyUrl(passwordItem);
            break;
          default:
            await copyText(field.value, field.name);
            break;
        }
      } else {
        await copyText(field.value, field.name);
      }

      toast.success(getCopySuccessMessage(field));
    } catch (error) {
      console.error('Failed to copy field value', error);
      toast.error(getCopyErrorMessage(field));
    }
  }

  function handleDeleteField(field: DisplayField) {
    const staticFields = ['username', 'password', 'url', 'notes'];
    if (staticFields.includes(field.id)) {
      toast.error(`Default field "${field.name}" cannot be deleted.`);
      return;
    }

    fieldToDelete = field;
    isDeleteDialogOpen = true;
  }

  function confirmDeleteField() {
    if (fieldToDelete) {
      editingFields = editingFields.filter((f) => f.id !== fieldToDelete!.id);
      toast.success(`Field "${fieldToDelete.name}" removed.`);
      isDeleteDialogOpen = false;
      fieldToDelete = null;
    }
  }
</script>

<div
  class={`flex flex-col gap-1.5 ${showSkeleton ? 'pointer-events-none' : ''}`}
  aria-busy={showSkeleton}
>
  {#if !isEditing}
    {#if showSkeleton}
      {#each viewSkeletonPlaceholders as _, i (i)}
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
                            inputValue={getDisplayValue(field)}
                            readOnly
                            selectedIconPath={field.icon}
                            selectedIconName={getIconName(field)}
                            iconComponent={field.iconComponent}
                            selectedColor={displayColor}
                            isMultiline={field.type === 'multiline'}
                            type={getInputType(field, true)}
                            isExpandable
                          >            {#snippet rightIcon()}
              {@const hasCopy = canCopyField(field)}
              {@const isPassword = field.id === 'password' || field.type === 'password'}
              {@const canToggle =
                isPassword && field.value && field.value.length && field.value !== 'N/A'}

              <div class="flex items-center gap-2">
                {#if isPassword && field.value && field.value !== 'N/A'}
                  {@const localBreach = localBreachData[field.value]}
                  {@const health =
                    field.id === 'password' && passwordItem
                      ? securityDashboard.items[passwordItem.id]
                      : SecurityService.checkStrength(field.value)}
                  {#if health}
                    {@const isBreached = localBreach?.isBreached ?? (health.isBreached || false)}
                    {@const breachCount = localBreach?.count ?? (health.breachCount || 0)}
                    <Badge
                      variant="outline"
                      class={`h-6 gap-1.5 px-2 transition-colors ${SecurityService.getStrengthBadgeClass(health.score, isBreached)}`}
                    >
                      {#if isBreached}
                        <ShieldAlert class="h-3.5 w-3.5" />
                      {:else if health.score === 4}
                        <ShieldCheck class="h-3.5 w-3.5" />
                      {:else}
                        <Shield class="h-3.5 w-3.5" />
                      {/if}
                      <span class="text-[10px] font-semibold tracking-wider uppercase"
                        >{SecurityService.getStrengthLabel(
                          health.score,
                          isBreached,
                          breachCount
                        )}</span
                      >
                    </Badge>
                  {/if}
                {/if}

                {#if hasCopy}
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-foreground h-6 w-6 p-0"
                    aria-label={`Copy ${field.name}`}
                    title={`Copy ${field.name}`}
                    onclick={() => handleCopyField(field)}
                  >
                    <Copy class="h-5 w-5" />
                  </Button>
                {/if}
                {#if canToggle}
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-foreground h-6 w-6 p-0"
                    aria-pressed={showPassword}
                    aria-label={showPassword ? 'Hide password' : 'Show password'}
                    onclick={togglePasswordVisibility}
                  >
                    {#if showPassword}
                      <Eye class="h-5 w-5" />
                    {:else}
                      <EyeOff class="h-5 w-5" />
                    {/if}
                  </Button>
                {/if}
              </div>
            {/snippet}
          </Input>
          {#if (field.id === 'password' || field.type === 'password') && field.value && field.value !== 'N/A'}
            {@const localBreach = localBreachData[field.value]}
            {@const health =
              field.id === 'password' && passwordItem
                ? securityDashboard.items[passwordItem.id]
                : null}
            {@const isBreached = localBreach?.isBreached ?? (health?.isBreached || false)}
            <div class="mt-2 px-3 pb-2">
              <PasswordStrength
                password={field.value ?? ''}
                showDetails={false}
                {isBreached}
              />
            </div>
          {/if}
      {/each}
    {/if}
  {:else if showSkeleton}
    {#each editSkeletonPlaceholders as _, i (i)}
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
      class="flex flex-col gap-1.5"
      use:dndzone={{
        items: editingFields,
        flipDurationMs: 300,
        dropFromOthersDisabled: true
      }}
      onconsider={handleConsider}
      onfinalize={handleFinalize}
    >
      {#each editingFields as field (field.id)}
        <div
          animate:flip={{ duration: 300, easing: cubicOut }}
          class="touch-none will-change-transform"
        >
          <ContextMenu>
            <ContextMenuTrigger class="block w-full">
              <Input
                title={field.name}
                bind:inputValue={field.value}
                readOnly={!isEditing}
                selectedColor={displayColor}
                selectedIconPath={field.icon}
                selectedIconName={getIconName(field)}
                iconComponent={field.iconComponent}
                isMultiline={field.type === 'multiline'}
                type={getInputType(field, false)}
              >
                {#snippet rightIcon()}
                  {@const hasCopy = canCopyField(field)}
                  {@const isPassword = field.id === 'password' || field.type === 'password'}
                  {@const showToggle = isPassword}
                  {@const showControls = hasCopy || showToggle || isEditing}

                  {#if showControls}
                    <div class="flex items-center gap-2">
                      {#if isPassword}
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          class="text-muted-foreground hover:text-foreground h-6 w-6 p-0"
                          aria-label="Generate password"
                          title="Generate password"
                          onclick={() => openGenerator(field.id)}
                        >
                          <WandSparkles class="h-4.5 w-4.5" />
                        </Button>
                      {/if}

                      {#if isPassword && field.value && field.value !== 'N/A'}
                        {@const localBreach = localBreachData[field.value]}
                        {@const health =
                          field.id === 'password' && passwordItem
                            ? securityDashboard.items[passwordItem.id]
                            : SecurityService.checkStrength(field.value)}
                        {#if health}
                          {@const isBreached = localBreach?.isBreached ?? (health.isBreached || false)}
                          {@const breachCount = localBreach?.count ?? (health.breachCount || 0)}
                          <Badge
                            variant="outline"
                            class={`h-6 gap-1.5 px-2 transition-colors ${SecurityService.getStrengthBadgeClass(health.score, isBreached)}`}
                          >
                            {#if isBreached}
                              <ShieldAlert class="h-3.5 w-3.5" />
                            {:else if health.score === 4}
                              <ShieldCheck class="h-3.5 w-3.5" />
                            {:else}
                              <Shield class="h-3.5 w-3.5" />
                            {/if}
                            <span class="text-[10px] font-semibold tracking-wider uppercase"
                              >{SecurityService.getStrengthLabel(
                                health.score,
                                isBreached,
                                breachCount
                              )}</span
                            >
                          </Badge>
                        {/if}
                      {/if}

                      {#if hasCopy}
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          class="text-muted-foreground hover:text-foreground h-6 w-6 p-0"
                          aria-label={`Copy ${field.name}`}
                          title={`Copy ${field.name}`}
                          onclick={() => handleCopyField(field)}
                        >
                          <Copy class="h-5 w-5" />
                        </Button>
                      {/if}
                      {#if showToggle}
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          class="text-muted-foreground hover:text-foreground h-6 w-6 p-0"
                          aria-pressed={showPassword}
                          aria-label={showPassword ? 'Hide password' : 'Show password'}
                          onclick={togglePasswordVisibility}
                        >
                          {#if showPassword}
                            <Eye class="h-5 w-5" />
                          {:else}
                            <EyeOff class="h-5 w-5" />
                          {/if}
                        </Button>
                      {/if}
                      {#if isEditing}
                        <div class="ml-2 cursor-grab" data-dnd-handle>
                          <ArrowDownUp class="h-6 w-6" />
                        </div>
                      {/if}
                    </div>
                  {/if}
                {/snippet}
              </Input>
            </ContextMenuTrigger>
            <ContextMenuContent class="w-48">
              <ContextMenuItem
                onclick={() => handleDeleteField(field)}
                class="text-destructive focus:bg-destructive/10 focus:text-destructive gap-2"
              >
                <Trash2 class="size-4" />
                <span>Delete Field</span>
              </ContextMenuItem>
            </ContextMenuContent>
          </ContextMenu>
          {#if (field.id === 'password' || field.type === 'password') && field.value !== 'N/A'}
            {@const localBreach = localBreachData[field.value ?? '']}
            {@const health =
              field.id === 'password' && passwordItem
                ? securityDashboard.items[passwordItem.id]
                : null}
            {@const isBreached = localBreach?.isBreached ?? (health?.isBreached || false)}
            <div class="mt-2 px-3 pb-2">
              <PasswordStrength
                password={field.value ?? ''}
                showDetails={true}
                {isBreached}
              />
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showGeneratorPopup}
  <PasswordGeneratorPopup
    onselect={handlePasswordGenerated}
    onclose={() => (showGeneratorPopup = false)}
  />
{/if}

<Dialog open={isDeleteDialogOpen} onOpenChange={(open) => (isDeleteDialogOpen = open)}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2 text-destructive">
        <Trash2 class="size-5" />
        Delete Field
      </DialogTitle>
      <DialogDescription>
        Are you sure you want to remove this field? This action can be undone by clicking "Reset"
        before saving.
      </DialogDescription>
    </DialogHeader>

    {#if fieldToDelete}
      <div class="bg-muted/30 border-border/50 mt-2 space-y-3 rounded-lg border p-4">
        <div>
          <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase"
            >Field Name</span
          >
          <p class="text-sm font-medium">{fieldToDelete.name}</p>
        </div>
        <div>
          <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase"
            >Current Content</span
          >
          <p class="mt-0.5 font-mono text-sm break-all">
            {#if fieldToDelete.type === 'password' || fieldToDelete.id === 'password'}
              ••••••••••••
            {:else}
              {fieldToDelete.value || 'Empty'}
            {/if}
          </p>
        </div>
      </div>
    {/if}

    <DialogFooter class="mt-6">
      <Button variant="outline" onclick={() => (isDeleteDialogOpen = false)}>Cancel</Button>
      <Button variant="destructive" onclick={confirmDeleteField}>Delete Field</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
