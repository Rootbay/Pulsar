<script lang="ts">
  import { tick } from 'svelte';
  import Icon from '../ui/Icon.svelte';

  export let url: string | undefined = undefined;
  export let fallbackIcon: string;
  export let fallbackColor: string;
  let prevColor: string | null = null;
  let colorChangeKey = 0;
  export let title: string;

  let iconSrc: string | null = null;
  let hasError = false;
  let pulseActive = false;

  let faviconCache: Map<string, string>;
  if (typeof window !== 'undefined' && !(window as any).faviconCache) {
      (window as any).faviconCache = new Map<string, string>();
  }
  if (typeof window !== 'undefined') {
    faviconCache = (window as any).faviconCache;
  }


  function getDomain(fullUrl: string): string | null {
    try {
      if (!/^https?:\/\//i.test(fullUrl)) {
        fullUrl = 'https://' + fullUrl;
      }
      const urlObject = new URL(fullUrl);
      return urlObject.hostname;
    } catch (e) {
      console.error(`Invalid URL: ${fullUrl}`, e);
      return null;
    }
  }

    $: if (url) {
    hasError = false;
    iconSrc = null;
    const domain = getDomain(url);
    if (domain) {
      if (faviconCache.has(domain)) {
        iconSrc = faviconCache.get(domain)!;
        tick();
      } else {
        const faviconUrl = `https://icon.horse/icon/${domain}`;
        const img = new Image();
        img.src = faviconUrl;
        img.onload = () => {
          if (img.width > 0) {
              iconSrc = faviconUrl;
              faviconCache.set(domain, faviconUrl);
          } else {
              hasError = true;
          }
        };
        img.onerror = () => {
          hasError = true;
        };
      }
    } else {
      hasError = true;
    }
  }

  // Trigger a micro pulse when fallbackColor changes (used when favicon missing)
  $: if (hasError || !iconSrc) {
    if (prevColor === null) {
      prevColor = fallbackColor;
    } else if (prevColor !== fallbackColor) {
      pulseActive = false;
      tick();
      pulseActive = true;
      prevColor = fallbackColor;
      setTimeout(() => (pulseActive = false), 320);
    }
  }

</script>

<div class="itemImgContainer">
  {#if hasError || !iconSrc}
    <div class="tag-icon-container" class:color-pulse={pulseActive} style="--tag-color: {fallbackColor};">
        <div class="tag-icon-bg"></div>
        <div class="tag-icon-content">
            {#if fallbackIcon}
                <Icon path={fallbackIcon} size="16" color="currentColor" />
            {/if}
        </div>
    </div>
  {:else}
    <img src={iconSrc} alt={title} class="itemImg" draggable="false" on:error={() => hasError = true} />
  {/if}
</div>

<style>
  .itemImgContainer {
    width: 29px;
    height: 29px;
    border-radius: 14.5px;
    background: var(--white);
    display: flex;
    justify-content: center;
    align-items: center;
    overflow: hidden;
  }

  .itemImg {
    width: 17px;
    height: 17px;
    border-radius: 8.5px;
    object-fit: cover;
    display: block;
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
  }

  .tag-icon-container {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    position: relative;
    color: var(--tag-color, #000);
  }

  .tag-icon-bg {
    position: absolute;
    inset: 0;
    background: var(--tag-color, #ccc);
    opacity: 0.3;
    transition: background-color 260ms ease;
  }

  .tag-icon-content {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    transition: color 260ms ease, transform 180ms ease;
  }

  /* Subtle pulse when color changes */
  .tag-icon-container:global(.color-pulse) .tag-icon-content {
    animation: colorPulse 280ms ease-out;
  }
  @keyframes colorPulse {
    0%   { transform: scale(0.92); opacity: 0.85; }
    60%  { transform: scale(1.04); opacity: 1; }
    100% { transform: scale(1); opacity: 1; }
  }
</style>
