<script lang="ts">
  import { settings } from '$lib/stores/appSettings.svelte';
  import type { SiteRule } from '$lib/config/settings';
  import EditModal from '$lib/components/ui/EditModal.svelte';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { FileText, Pencil, Trash2, Plus } from '@lucide/svelte';
  import { i18n, t as translate } from '$lib/i18n.svelte';

  import { i18n, t as translate } from '$lib/i18n.svelte';
  import InputDialog from '$lib/components/ui/InputDialog.svelte';

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as any, vars);

  let rules = $derived(settings.state.siteRules);

  let showEditModal = $state(false);
  let itemToEdit = $state<SiteRule | null>(null);
  let showAddDialog = $state(false);

  function handleEditRule(rule: SiteRule) {
    itemToEdit = JSON.parse(JSON.stringify(rule));
    showEditModal = true;
  }

  function removeRule(url: string) {
    if (confirm(t('Are you sure you want to delete rule for "{url}"?', { url }))) {
      settings.state.siteRules = settings.state.siteRules.filter((r) => r.url !== url);
      settings.save();
    }
  }

  function closeModal() {
    showEditModal = false;
    itemToEdit = null;
  }

  function handleSaveEdit(updatedItem: SiteRule) {
    if (itemToEdit) {
      const originalUrl = itemToEdit.url;
      const index = settings.state.siteRules.findIndex((r) => r.url === originalUrl);
      if (index !== -1) {
        settings.state.siteRules[index] = updatedItem;
        settings.save();
      }
    }
    closeModal();
  }

  function handleAddNewRule(url: string) {
    const newRule: SiteRule = {
      url,
      length: 16,
      type: 'Custom',
      desc: '',
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

    settings.state.siteRules = [...settings.state.siteRules, newRule];
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
          <FileText class="size-5" aria-hidden="true" />
        </div>
        <div>
          <CardTitle>
            {t('Site Rule Templates')}
          </CardTitle>
          <CardDescription>
            {t('Maintain site-specific password requirements.')}
          </CardDescription>
        </div>
      </div>
      <Button type="button" class="gap-2" onclick={() => (showAddDialog = true)}>
        <Plus class="size-4" aria-hidden="true" />
        {t('Add Rule')}
      </Button>
    </CardHeader>
    <CardContent class="space-y-3">
      {#if rules.length}
        <div class="grid gap-4 md:grid-cols-2">
          {#each rules as rule (rule.url)}
            <div
              class="border-border/60 bg-background/70 flex flex-col gap-3 rounded-xl border p-4"
            >
              <div class="flex flex-wrap items-start justify-between gap-2">
                <div class="flex-1">
                  <p class="text-foreground text-sm font-semibold break-all">{rule.url}</p>
                  <p class="text-muted-foreground text-xs line-clamp-2">{rule.desc}</p>
                </div>
                <div class="flex items-center gap-1">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-foreground size-8"
                    aria-label={t('Edit rule for {url}', { url: rule.url })}
                    onclick={() => handleEditRule(rule)}
                  >
                    <Pencil class="size-4" aria-hidden="true" />
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-destructive size-8"
                    aria-label={t('Delete rule for {url}', { url: rule.url })}
                    onclick={() => removeRule(rule.url)}
                  >
                    <Trash2 class="size-4" aria-hidden="true" />
                  </Button>
                </div>
              </div>
              <div class="text-muted-foreground flex flex-wrap gap-2 text-xs">
                <Badge variant="secondary">
                  {t('Length')}
                  {rule.length}
                </Badge>
                <Badge variant="outline">{rule.type}</Badge>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-muted-foreground text-sm">
          {t('No site rule templates configured yet. Click "Add Rule" to create your first one.')}
        </p>
      {/if}
    </CardContent>
  </Card>
</div>

{#if showEditModal && itemToEdit}
  <EditModal
    show={showEditModal}
    item={itemToEdit}
    type="rule"
    onclose={closeModal}
    onsave={handleSaveEdit}
  />
{/if}

<InputDialog
  bind:open={showAddDialog}
  title={t('Add Rule')}
  description={t('Enter the domain or URL for this rule:')}
  label="URL"
  placeholder="example.com"
  confirmLabel={t('Add')}
  onConfirm={handleAddNewRule}
/>
