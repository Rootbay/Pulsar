<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade } from 'svelte/transition';
  import { Button } from '$lib/components/ui/button';

  const dispatch = createEventDispatcher();

  const onReset = () => {
    dispatch('reset');
  };

  const onSave = () => {
    dispatch('save');
  };
</script>

<div
  class="fixed left-1/2 bottom-5 z-50 w-[calc(100%-40px)] max-w-xl -translate-x-1/2 rounded-xl border border-border/80 bg-card/95 px-4 py-3 shadow-lg backdrop-blur supports-[backdrop-filter]:bg-card/80"
  transition:fade={{ duration: 150 }}
>
  <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
    <span class="text-sm font-medium text-foreground">
      Careful - you have unsaved changes!
    </span>
    <div class="flex items-center gap-2 sm:gap-3">
      <Button variant="outline" size="sm" onclick={onReset}>
        Reset
      </Button>
      <Button size="sm" onclick={onSave}>
        Save changes
      </Button>
    </div>
  </div>
</div>
