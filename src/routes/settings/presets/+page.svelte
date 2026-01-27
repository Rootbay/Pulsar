<script lang="ts">
  import { settings } from '$lib/stores/appSettings.svelte';
  import type { PasswordPreset } from '$lib/config/settings';
  import EditModal from '$lib/components/ui/EditModal.svelte';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { ListChecks, Pencil, Trash2, Plus } from '@lucide/svelte';
  import { i18n, t as translate } from '$lib/i18n.svelte';

  const MAX_ENTROPY_BITS = 128;
  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as any, vars);

  let presets = $derived(settings.state.passwordPresets);

  let showEditModal = $state(false);
  let itemToEdit = $state<PasswordPreset | null>(null);

  function handleEditPreset(preset: PasswordPreset) {
    itemToEdit = JSON.parse(JSON.stringify(preset));
    showEditModal = true;
  }

  function removePreset(name: string) {
    if (confirm(t('Are you sure you want to delete preset "{name}"?', { name }))) {
      settings.state.passwordPresets = settings.state.passwordPresets.filter(
        (p) => p.name !== name
      );
      settings.save();
    }
  }

  function closeModal() {
    showEditModal = false;
    itemToEdit = null;
  }

  function handleSaveEdit(updatedItem: PasswordPreset) {
    if (itemToEdit) {
      const originalName = itemToEdit.name;
      const index = settings.state.passwordPresets.findIndex((p) => p.name === originalName);
      if (index !== -1) {
        settings.state.passwordPresets[index] = updatedItem;
        settings.save();
      }
    }
    closeModal();
  }

  function addNewPreset() {
    const name = prompt(t('Enter a name for the new preset:'));
    if (!name) return;

    const newPreset: PasswordPreset = {
      name,
      length: 16,
      charSet: 'A-Z, a-z, 0-9, !@#$',
      strength: 90,
      settings: {
        uppercase: true,
        lowercase: true,
        digits: true,
        symbols: true,
        ambiguous: false,
        similar: false,
        pronounceable: false
      }
    };

    settings.state.passwordPresets = [...settings.state.passwordPresets, newPreset];
    settings.save();
  }
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
        >
          <ListChecks class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t('Password Presets')}
          </CardTitle>
          <CardDescription>
            {t('Manage and reuse your favourite password configurations.')}
          </CardDescription>
        </div>
      </div>
      <Button type="button" class="gap-2" onclick={addNewPreset}>
        <Plus class="size-4" aria-hidden="true" />
        {t('Add Preset')}
      </Button>
    </CardHeader>
    <CardContent>
      {#if presets.length}
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          {#each presets as preset (preset.name)}
            <div
              class="border-border/60 bg-background/70 flex flex-col gap-3 rounded-xl border p-4"
            >
              <div class="flex items-start justify-between gap-2">
                <div>
                  <p class="text-foreground text-sm font-semibold">{preset.name}</p>
                  <p class="text-muted-foreground text-xs">
                    {t('Length')}
                    {preset.length} · {preset.charSet}
                  </p>
                </div>
                <div class="flex items-center gap-1">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-foreground size-8"
                    aria-label={t('Edit preset {name}', { name: preset.name })}
                    onclick={() => handleEditPreset(preset)}
                  >
                    <Pencil class="size-4" aria-hidden="true" />
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-destructive size-8"
                    aria-label={t('Delete preset {name}', { name: preset.name })}
                    onclick={() => removePreset(preset.name)}
                  >
                    <Trash2 class="size-4" aria-hidden="true" />
                  </Button>
                </div>
              </div>

              <Progress
                value={Math.min(100, Math.round((preset.strength / MAX_ENTROPY_BITS) * 100))}
                class="bg-muted/40 **:data-[slot=progress-indicator]:bg-primary"
              />
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-muted-foreground text-sm">
          {t('No saved presets yet. Click "Add Preset" to create your first one.')}
        </p>
      {/if}
    </CardContent>
  </Card>
</div>

{#if showEditModal && itemToEdit}
  <EditModal
    show={showEditModal}
    item={itemToEdit}
    type="preset"
    onclose={closeModal}
    onsave={handleSaveEdit}
  />
{/if}
