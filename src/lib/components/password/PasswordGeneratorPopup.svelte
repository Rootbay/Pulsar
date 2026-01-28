<script lang="ts">
  import { settings } from '$lib/stores/appSettings.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { Badge } from '$lib/components/ui/badge';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { RefreshCcw, Check, X, Key } from '@lucide/svelte';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { GeneratorService } from '$lib/utils/generator';
  import PasswordStrength from '$lib/components/password/PasswordStrength.svelte';

  interface Props {
    onselect: (password: string) => void;
    onclose: () => void;
  }

  let { onselect, onclose }: Props = $props();

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as I18nKey, vars);

  let passwordLength = $state(settings.state.generator.passwordLength);
  let options = $state(JSON.parse(JSON.stringify(settings.state.generator.options)));
  let generatedPassword = $state('');

  const mode = $derived(options.mode || 'password');
  const wordCount = $derived(options.wordCount || 4);
  const separator = $derived(options.separator || '-');

  function generate() {
    const pass = GeneratorService.generate(passwordLength, options);
    if (pass) generatedPassword = pass;
  }

  function handleModeChange(newMode: string) {
    options.mode = newMode as 'password' | 'passphrase';
    generate();
  }

  $effect(() => {
    generate();
  });

  function selectAndClose() {
    onselect(generatedPassword);
    onclose();
  }
</script>

<div
  class="fixed inset-0 z-100 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
>
  <div class="border-border bg-card w-full max-w-lg overflow-hidden rounded-2xl border shadow-2xl">
    <div class="border-border bg-muted/30 flex items-center justify-between border-b p-4">
      <div class="flex items-center gap-2">
        <div
          class="bg-primary/10 text-primary flex h-8 w-8 items-center justify-center rounded-full"
        >
          <Key class="h-4 w-4" />
        </div>
        <h3 class="font-semibold">{t('Password Generator')}</h3>
      </div>
      <Button variant="ghost" size="icon" onclick={onclose} class="h-8 w-8">
        <X class="h-4 w-4" />
      </Button>
    </div>

    <div class="space-y-6 p-6">
      <div class="group relative">
        <div
          class="border-border bg-muted/20 flex min-h-20 items-center justify-center rounded-xl border p-4 text-center font-mono text-lg break-all"
        >
          {generatedPassword}
        </div>
        <Button
          variant="ghost"
          size="icon"
          onclick={generate}
          class="absolute top-2 right-2 h-8 w-8 opacity-0 transition-opacity group-hover:opacity-100"
        >
          <RefreshCcw class="h-4 w-4" />
        </Button>
      </div>

      <div class="mt-2">
        <PasswordStrength password={generatedPassword} showDetails={false} />
      </div>

      <div class="grid gap-6 sm:grid-cols-2">
        <div class="space-y-4">
          <div class="space-y-2">
            <Label class="text-muted-foreground text-xs font-semibold uppercase">{t('Type')}</Label>
            <Select type="single" value={mode} onValueChange={handleModeChange}>
              <SelectTrigger class="h-9 w-full">
                <span data-slot="select-value" class="text-sm">
                  {mode === 'password' ? t('Password') : t('Passphrase')}
                </span>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="password">{t('Password')}</SelectItem>
                <SelectItem value="passphrase">{t('Passphrase')}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {#if mode === 'password'}
            <div class="space-y-3">
              <div class="flex items-center justify-between">
                <Label class="text-muted-foreground text-xs font-semibold uppercase"
                  >{t('Length')}</Label
                >
                <Badge variant="outline" class="h-5 text-[10px]">{passwordLength}</Badge>
              </div>
              <input
                type="range"
                min="8"
                max="64"
                bind:value={passwordLength}
                class="bg-muted accent-primary h-1.5 w-full cursor-pointer appearance-none rounded-full"
              />
            </div>
          {:else}
            <div class="space-y-3">
              <div class="flex items-center justify-between">
                <Label class="text-muted-foreground text-xs font-semibold uppercase"
                  >{t('Words')}</Label
                >
                <Badge variant="outline" class="h-5 text-[10px]">{wordCount}</Badge>
              </div>
              <input
                type="range"
                min="3"
                max="12"
                bind:value={options.wordCount}
                class="bg-muted accent-primary h-1.5 w-full cursor-pointer appearance-none rounded-full"
              />
            </div>
          {/if}
        </div>

        <div class="border-border space-y-4 border-l pl-6">
          {#if mode === 'password'}
            <Label class="text-muted-foreground text-xs font-semibold uppercase"
              >{t('Complexity')}</Label
            >
            <div class="space-y-2">
              {#each ['uppercase', 'lowercase', 'digits', 'symbols'] as key (key)}
                <div class="flex items-center justify-between">
                  <span class="text-xs capitalize">{key}</span>
                  <Switch
                    checked={options[key]}
                    onCheckedChange={(val) => {
                      options[key] = val;
                      generate();
                    }}
                    class="origin-right scale-75"
                  />
                </div>
              {/each}
            </div>
          {:else}
            <Label class="text-muted-foreground text-xs font-semibold uppercase"
              >{t('Separator')}</Label
            >
            <Select
              type="single"
              value={separator}
              onValueChange={(val) => {
                options.separator = val;
                generate();
              }}
            >
              <SelectTrigger class="h-9 w-full">
                <span data-slot="select-value" class="text-sm">
                  {separator === '-'
                    ? '-'
                    : separator === '.'
                      ? '.'
                      : separator === '_'
                        ? '_'
                        : 'Space'}
                </span>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="-">-</SelectItem>
                <SelectItem value=".">.</SelectItem>
                <SelectItem value="_">_</SelectItem>
                <SelectItem value=" ">Space</SelectItem>
              </SelectContent>
            </Select>
          {/if}
        </div>
      </div>
    </div>

    <div class="border-border bg-muted/10 flex items-center justify-end gap-3 border-t p-4">
      <Button variant="ghost" onclick={onclose}>{t('Cancel')}</Button>
      <Button onclick={selectAndClose} class="gap-2">
        <Check class="h-4 w-4" />
        {t('Use Password')}
      </Button>
    </div>
  </div>
</div>
