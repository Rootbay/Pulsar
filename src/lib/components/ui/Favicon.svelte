<svelte:options runes />

<script lang="ts">
  import { tick } from 'svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  interface Props {
    url?: string;
    fallbackIcon: string;
    fallbackColor: string;
    size?: number;
    title: string;
    variant?: 'default' | 'list';
    useStroke?: boolean;
    strokeWidth?: number;
  }

  let {
    url = undefined,
    fallbackIcon,
    fallbackColor,
    size = 30,
    title,
    variant = 'default',
    useStroke = false,
    strokeWidth = 2
  }: Props = $props();

  let prevColor: string | null = null;

  let iconSrc = $state<string | null>(null);
  let hasError = $state(false);
  let pulseActive = $state(false);
  
  // Standardize size for all variants to ensure perfect centering
  const innerIconSize = 19;

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
      return null;
    }
  }

  $effect(() => {
    iconSrc = null;
    hasError = false;

    if (!url) {
      hasError = true;
      return;
    }

    const domain = getDomain(url);
    if (domain) {
      if (faviconCache.has(domain)) {
        iconSrc = faviconCache.get(domain)!;
      } else {
        const faviconUrl = `https://www.google.com/s2/favicons?domain=${encodeURIComponent(domain)}&sz=128`;
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
  });

  $effect(() => {
    if (hasError || !iconSrc) {
      if (prevColor === null) {
        prevColor = fallbackColor;
      } else if (prevColor !== fallbackColor) {
        prevColor = fallbackColor;
        
        if (!pulseActive) {
          pulseActive = true;
          setTimeout(() => {
            pulseActive = false;
          }, 500);
        }
      }
    }
  });
</script>

<div
  class="itemImgContainer"
  class:list-variant={variant === 'list'}
  class:has-icon={!hasError && iconSrc}
  class:color-pulse={pulseActive}
  style="--favicon-size: {size}px; --tag-color: {fallbackColor}; --bg-color: {fallbackColor === '#94a3b8' ? '#ffffff' : fallbackColor};"
>
  {#if hasError || !iconSrc}
    <div class="tag-icon-bg"></div>
    {#if fallbackIcon}
      <Icon
        path={fallbackIcon}
        size={String(innerIconSize)}
        color={fallbackColor === '#94a3b8' ? '#ffffff' : fallbackColor}
        stroke={useStroke ? fallbackColor : undefined}
        strokeWidth={useStroke ? strokeWidth : undefined}
        viewBox="0 0 48 48"
      />
    {/if}
  {:else}
    <img
      src={iconSrc}
      alt={title}
      class="itemImg raster"
      draggable="false"
      onerror={() => (hasError = true)}
    />
  {/if}
</div>

<style>
  .itemImgContainer {
    width: var(--favicon-size, 30px);
    height: var(--favicon-size, 30px);
    min-width: var(--favicon-size, 30px);
    max-width: var(--favicon-size, 30px);
    min-height: var(--favicon-size, 30px);
    max-height: var(--favicon-size, 30px);
    aspect-ratio: 1 / 1;
    border-radius: 50%;
    display: grid;
    place-items: center;
    overflow: hidden;
    position: relative;
    transition: background-color 200ms ease;
    box-sizing: border-box;
    flex-shrink: 0;
  }

  .itemImgContainer.has-icon {
    background: #ffffff;
  }

  .itemImg.raster {
    width: 19px;
    height: 19px;
    border-radius: 50%;
    object-fit: contain;
    display: block;
    user-select: none;
    z-index: 1;
  }

  .tag-icon-bg {
    position: absolute;
    inset: 0;
    background: var(--bg-color);
    opacity: 0.25;
    transition: background-color 260ms ease;
    z-index: 0;
  }

  .itemImgContainer :global(svg) {
    display: block;
    position: relative;
    z-index: 1;
  }

  .color-pulse :global(svg) {
    animation: smoothIconPulse 500ms ease-in-out;
  }

  @keyframes smoothIconPulse {
    0% { transform: scale(1); }
    30% { transform: scale(1.18); }
    100% { transform: scale(1); }
  }
</style>