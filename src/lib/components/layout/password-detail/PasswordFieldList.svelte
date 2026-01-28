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
    WandSparkles
  } from '@lucide/svelte';
  import Input from '$lib/components/ui/FieldInput.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import type { DisplayField } from '$lib/types/password-fields';
  import type { PasswordItem } from '$lib/types/password';
  import { SecurityService } from '$lib/utils/security';
  import { copyPassword, copyText, copyUrl, copyUsername } from '$lib/utils/copyHelper';
  import { toast } from '$lib/components/ui/sonner';
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
    const passField = editingFields.find((f) => f.id === 'password');
    if (passField) {
      passField.value = newPass;
      showPassword = true;
    }
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
    switch (field.id) {
      case 'username':
        return 'user';
      case 'password':
        return 'key';
      case 'url':
        return 'link';
      case 'notes':
        return 'notes';
      default:
        return field.id;
    }
  }

  function getDisplayValue(field: DisplayField): string {
    if (field.id === 'password') {
      return field.value && field.value.length ? field.value : 'N/A';
    }
    if (field.id === 'username' || field.id === 'url' || field.id === 'notes') {
      return field.value ?? 'N/A';
    }
    return field.value ?? '';
  }

  function getInputType(field: DisplayField, isViewMode: boolean): 'text' | 'password' | 'url' {
    if (field.id === 'url') {
      return 'url';
    }
    if (field.id === 'password') {
      if (isViewMode) {
        return field.value && field.value.length && field.value !== 'N/A'
          ? showPassword
            ? 'text'
            : 'password'
          : 'text';
      }
      return showPassword ? 'text' : 'password';
    }
    return field.type === 'password' ? 'password' : 'text';
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
          selectedColor={displayColor}
          isMultiline={field.type === 'multiline'}
          type={getInputType(field, true)}
          isExpandable
        >
          {#snippet rightIcon()}
            {@const hasCopy = canCopyField(field)}
            {@const canToggle =
              field.id === 'password' && field.value && field.value.length && field.value !== 'N/A'}

            <div class="flex items-center gap-2">
              {#if field.id === 'password' && passwordItem}
                {@const health = securityDashboard.items[passwordItem.id]}
                {#if health}
                  <Badge
                    variant="outline"
                    class={`h-6 gap-1.5 px-2 transition-colors ${SecurityService.getStrengthBadgeClass(health.score, health.isBreached)}`}
                  >
                    {#if health.isBreached}
                      <ShieldAlert class="h-3.5 w-3.5" />
                    {:else if health.score === 4}
                      <ShieldCheck class="h-3.5 w-3.5" />
                    {:else}
                      <Shield class="h-3.5 w-3.5" />
                    {/if}
                    <span class="text-[10px] font-semibold tracking-wider uppercase"
                      >{SecurityService.getStrengthLabel(
                        health.score,
                        health.isBreached,
                        health.breachCount
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
          <Input
            title={field.name}
            bind:inputValue={field.value}
            readOnly={!isEditing}
            selectedColor={displayColor}
            selectedIconPath={field.icon}
            selectedIconName={field.id}
            isMultiline={field.type === 'multiline'}
            type={getInputType(field, false)}
          >
            {#snippet rightIcon()}
              {@const hasCopy = canCopyField(field)}
              {@const showToggle = field.id === 'password'}
              {@const showControls = hasCopy || showToggle || isEditing}

              {#if showControls}
                <div class="flex items-center gap-2">
                  {#if field.id === 'password'}
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      class="text-muted-foreground hover:text-foreground h-6 w-6 p-0"
                      aria-label="Generate password"
                      title="Generate password"
                      onclick={() => (showGeneratorPopup = true)}
                    >
                      <WandSparkles class="h-4.5 w-4.5" />
                    </Button>
                  {/if}

                  {#if field.id === 'password' && passwordItem}
                    {@const health = securityDashboard.items[passwordItem.id]}
                    {#if health}
                      <Badge
                        variant="outline"
                        class={`h-6 gap-1.5 px-2 transition-colors ${SecurityService.getStrengthBadgeClass(health.score, health.isBreached)}`}
                      >
                        {#if health.isBreached}
                          <ShieldAlert class="h-3.5 w-3.5" />
                        {:else if health.score === 4}
                          <ShieldCheck class="h-3.5 w-3.5" />
                        {:else}
                          <Shield class="h-3.5 w-3.5" />
                        {/if}
                        <span class="text-[10px] font-semibold tracking-wider uppercase"
                          >{SecurityService.getStrengthLabel(
                            health.score,
                            health.isBreached,
                            health.breachCount
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
          {/Input}
          {#if isEditing && field.id === 'password'}
            <div class="mt-2 px-3 pb-2">
              <PasswordStrength password={field.value ?? ''} showDetails={true} />
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
