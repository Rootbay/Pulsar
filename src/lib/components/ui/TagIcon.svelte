<script lang="ts">
  import Icon from "../ui/Icon.svelte";
  import { flip } from 'svelte/animate';
  import { fade, scale } from 'svelte/transition';
  import { tick } from 'svelte';

  export let tagNames: string[] = [];
  export let tagMap: Map<string, { color: string, icon: string }>;

  let prevFirst: string | null = null;
  let pulseFirst = false;

  $: if (tagNames) {
    const first = tagNames[0] ?? null;
    if (prevFirst === null) {
      prevFirst = first;
    } else if (first !== prevFirst) {
      pulseFirst = false;
      tick();
      pulseFirst = true;
      prevFirst = first;
      setTimeout(() => (pulseFirst = false), 320);
    }
  }
</script>

{#each tagNames as tagName, i (tagName)}
  {@const tag = tagMap.get(tagName)}
  {#if tag}
    <div
      class="tag-icon-container"
      class:first-pulse={pulseFirst && i === 0}
      style="--tag-color: {tag.color}"
      in:fade={{ duration: 240 }}
      out:fade={{ duration: 180 }}
    >
      <div class="tag-bg"></div>
      <Icon path={tag.icon} color={tag.color} size="17" />
    </div>
  {/if}
{/each}

  <style>
  .tag-icon-container {
    position: relative;
    width: 23px;
    height: 23px;
    border-radius: 11.5px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: filter 260ms ease;
  }

  .tag-bg {
    position: absolute;
    inset: 0;
    border-radius: 11.5px;
    background: transparent;
    opacity: 0;
    transition: background-color 280ms ease, opacity 280ms ease;
  }

  .tag-icon-container :global(svg) {
    position: relative;
    z-index: 1;
    width: 17px;
    height: 17px;
    display: block;
      transition: fill 280ms ease, transform 220ms ease;
  }

  /* Pulse the first tag briefly when it changes */
  .tag-icon-container.first-pulse {
    animation: firstTagPulse 320ms ease-out;
  }
  @keyframes firstTagPulse {
    0% { transform: scale(0.92); filter: saturate(0.9); }
    60% { transform: scale(1.06); filter: saturate(1.05); }
    100% { transform: scale(1); filter: none; }
  }
</style>
