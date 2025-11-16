<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import { profileSettings } from '$lib/stores/profile';
  import { User } from '@lucide/svelte';

  type ProfileField = 'name' | 'country' | 'timezone' | 'about' | 'phone' | null;
  let editing: ProfileField = null;
  let tempValue = '';

  function startEdit(field: Exclude<ProfileField, null>, value: string) {
    editing = field;
    tempValue = value ?? '';
  }
  function saveEdit() {
    if (!editing) return;
    profileSettings.update((p) => ({ ...p, [editing!]: tempValue }));
    editing = null;
  }
  function cancelEdit() {
    editing = null;
    tempValue = '';
  }
  const roleLabel = 'Vault owner';
</script>

<h1 class="text-xl font-semibold">Personal info</h1>

<section class="mt-6 flex items-center gap-4">
  <div class="relative">
    <div class="h-20 w-20 shrink-0 overflow-hidden rounded-full border border-border/60 bg-muted"></div>
    <button class="absolute -right-2 -bottom-2 rounded-full border border-border/60 bg-background p-2 shadow-sm" aria-label="Edit avatar">
      <User class="size-4" />
    </button>
  </div>
  <div>
    <p class="text-base font-semibold">{$profileSettings.name}</p>
    <p class="text-sm text-muted-foreground">{roleLabel}</p>
  </div>
</section>

<section class="mt-6 divide-y divide-border/60">
  <div class="grid grid-cols-3 items-center gap-4 py-4">
    <div class="text-sm text-muted-foreground">Name</div>
    <div class="col-span-2 flex items-center justify-between gap-3">
      {#if editing === 'name'}
        <div class="flex w-full max-w-[420px] items-center gap-2">
          <Input class="w-full" bind:value={tempValue} placeholder="Your name" />
          <Button size="sm" class="h-8 px-3" on:click={saveEdit}>Save</Button>
          <Button size="sm" variant="ghost" class="h-8 px-3" on:click={cancelEdit}>Cancel</Button>
        </div>
      {:else}
        <span class="truncate text-sm">{$profileSettings.name}</span>
        <button class="text-xs text-primary hover:underline" on:click={() => startEdit('name', $profileSettings.name)}>Edit</button>
      {/if}
    </div>
  </div>
  <div class="grid grid-cols-3 items-center gap-4 py-4">
    <div class="text-sm text-muted-foreground">Country</div>
    <div class="col-span-2 flex items-center justify-between gap-3">
      {#if editing === 'country'}
        <div class="flex w-full max-w-[420px] items-center gap-2">
          <Input class="w-full" bind:value={tempValue} placeholder="Country" />
          <Button size="sm" class="h-8 px-3" on:click={saveEdit}>Save</Button>
          <Button size="sm" variant="ghost" class="h-8 px-3" on:click={cancelEdit}>Cancel</Button>
        </div>
      {:else}
        <span class="truncate text-sm">{$profileSettings.country}</span>
        <button class="text-xs text-primary hover:underline" on:click={() => startEdit('country', $profileSettings.country)}>Edit</button>
      {/if}
    </div>
  </div>
  <div class="grid grid-cols-3 items-center gap-4 py-4">
    <div class="text-sm text-muted-foreground">Timezone</div>
    <div class="col-span-2 flex items-center justify-between gap-3">
      {#if editing === 'timezone'}
        <div class="flex w-full max-w-[420px] items-center gap-2">
          <Input class="w-full" bind:value={tempValue} placeholder="Region/City" />
          <Button size="sm" class="h-8 px-3" on:click={saveEdit}>Save</Button>
          <Button size="sm" variant="ghost" class="h-8 px-3" on:click={cancelEdit}>Cancel</Button>
        </div>
      {:else}
        <span class="truncate text-sm">{$profileSettings.timezone}</span>
        <button class="text-xs text-primary hover:underline" on:click={() => startEdit('timezone', $profileSettings.timezone)}>Edit</button>
      {/if}
    </div>
  </div>
  <div class="grid grid-cols-3 items-center gap-4 py-4">
    <div class="text-sm text-muted-foreground">About</div>
    <div class="col-span-2 flex items-center justify-between gap-3">
      {#if editing === 'about'}
        <div class="flex w-full max-w-[420px] items-center gap-2">
          <Input class="w-full" bind:value={tempValue} placeholder="Short bio" />
          <Button size="sm" class="h-8 px-3" on:click={saveEdit}>Save</Button>
          <Button size="sm" variant="ghost" class="h-8 px-3" on:click={cancelEdit}>Cancel</Button>
        </div>
      {:else}
        <span class="truncate text-sm">{$profileSettings.about}</span>
        <button class="text-xs text-primary hover:underline" on:click={() => startEdit('about', $profileSettings.about)}>Edit</button>
      {/if}
    </div>
  </div>
  <div class="grid grid-cols-3 items-center gap-4 py-4">
    <div class="text-sm text-muted-foreground">Phone number</div>
    <div class="col-span-2 flex items-center justify-between gap-3">
      {#if editing === 'phone'}
        <div class="flex w-full max-w-[420px] items-center gap-2">
          <Input class="w-full" bind:value={tempValue} placeholder="+39 ..." />
          <Button size="sm" class="h-8 px-3" on:click={saveEdit}>Save</Button>
          <Button size="sm" variant="ghost" class="h-8 px-3" on:click={cancelEdit}>Cancel</Button>
        </div>
      {:else}
        <span class="truncate text-sm">{$profileSettings.phone}</span>
        <button class="text-xs text-primary hover:underline" on:click={() => startEdit('phone', $profileSettings.phone)}>Edit</button>
      {/if}
    </div>
  </div>
</section>

<div class="mt-6 grid grid-cols-1 gap-2">
  <a class="text-xs text-muted-foreground hover:underline" href="/settings/security">Go to Login & Security</a>
  <a class="text-xs text-muted-foreground hover:underline" href="/settings/appearance">Go to Appearance</a>
  <a class="text-xs text-muted-foreground hover:underline" href="/settings/general">Go to General</a>
</div>
