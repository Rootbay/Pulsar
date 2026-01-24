<script lang="ts">
  import { fade } from 'svelte/transition';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    onsave?: () => void;
    onreset?: () => void;
  }

  let { onsave, onreset }: Props = $props();

  const handleReset = () => {
    onreset?.();
  };

  const handleSave = () => {
    onsave?.();
  };
</script>

<div
  class="border-border/80 bg-card/95 supports-backdrop-filter:bg-card/80 fixed bottom-5 left-1/2 z-50 w-[calc(100%-40px)] max-w-xl -translate-x-1/2 rounded-xl border px-4 py-3 shadow-lg backdrop-blur"
  transition:fade={{ duration: 150 }}
>
  <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
    <span class="text-foreground text-sm font-medium"> Careful - you have unsaved changes! </span>
    <div class="flex items-center gap-2 sm:gap-3">
      <Button variant="outline" size="sm" onclick={handleReset}>Reset</Button>
      <Button size="sm" onclick={handleSave}>Save changes</Button>
    </div>
  </div>
</div>
