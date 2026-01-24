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
    size = 29,
    title,
    variant = 'default',
    useStroke = false,
    strokeWidth = 2
  }: Props = $props();

  let prevColor: string | null = null;
  let colorChangeKey = 0;

  let iconSrc = $state<string | null>(null);
  let hasError = $state(false);
  let pulseActive = $state(false);
  const innerIconSize = $derived(() => (variant === 'list' ? 18 : 17));

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
        tick();
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
        pulseActive = false;
        tick();
        pulseActive = true;
        prevColor = fallbackColor;
        setTimeout(() => (pulseActive = false), 320);
      }
    }
  });
</script>

<div
  class="itemImgContainer {variant === 'list' ? 'list-variant' : ''}"
  style="--favicon-size: {size}px;"
>
  {#if hasError || !iconSrc}
    <div
      class="tag-icon-container"
      class:color-pulse={pulseActive}
      style="--tag-color: {fallbackColor};"
    >
      <div class="tag-icon-bg"></div>
      <div class="tag-icon-plate {variant === 'list' ? 'list' : 'default'}">
        <div class="tag-icon-content">
          {#if fallbackIcon}
            <Icon
              path={fallbackIcon}
              size={String(innerIconSize)}
              color={fallbackColor}
              stroke={useStroke ? fallbackColor : undefined}
              strokeWidth={useStroke ? strokeWidth : undefined}
              viewBox="0 0 44 44"
            />
          {/if}
        </div>
      </div>
    </div>
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
    /* Force a perfectly square box regardless of layout stretching */
    width: var(--favicon-size, 29px);
    height: var(--favicon-size, 29px);
    min-width: var(--favicon-size, 29px);
    max-width: var(--favicon-size, 29px);
    min-height: var(--favicon-size, 29px);
    max-height: var(--favicon-size, 29px);
    flex: 0 0 auto;
    border-radius: calc(var(--favicon-size, 29px) / 2);
    /* Ensure a consistent white background behind favicons (including SVGs) */
    background: #ffffff;
    display: grid;
    place-items: center;
    overflow: hidden;
    line-height: 0;
    box-sizing: border-box;
  }

  /* Ensure nav/list variant background matches header size exactly */
  .itemImgContainer.list-variant {
    width: 30px;
    height: 30px;
    min-width: 30px;
    max-width: 30px;
    min-height: 30px;
    max-height: 30px;
    flex: 0 0 auto;
  }

  .itemImg.raster {
    width: calc(var(--favicon-size, 29px) * 0.586);
    height: calc(var(--favicon-size, 29px) * 0.586);
    border-radius: calc(var(--favicon-size, 29px) * 0.293);
    object-fit: cover;
    object-position: center;
    display: block;
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
  }

  /* Nudge raster URL icons in main (default) variant for visual centering */
  .itemImgContainer:not(.list-variant) .itemImg.raster {
    width: 18px;
    height: 18px;
    border-radius: 9px;
    margin-top: 0;
    margin-left: 0;
  }

  /* Match icon size/position in nav list (list variant) */
  .itemImgContainer.list-variant .itemImg.raster {
    width: 18px;
    height: 18px;
    border-radius: 9px;
    margin-top: 0;
    margin-left: 0;
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
    z-index: 0;
  }

  /* Fallback inline SVG plate */
  .tag-icon-plate.default {
    width: 17px;
    height: 17px;
    border-radius: 8.5px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    z-index: 1;
  }

  .tag-icon-plate.list {
    width: 18px;
    height: 18px;
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    z-index: 1;
  }

  .tag-icon-content :global(svg) {
    display: block;
  }

  /* offset for list variant */
  .list-variant .tag-icon-content :global(svg) {
    margin-top: -1px;
    margin-left: -1px;
  }

  /* Subtle pulse when color changes */
  .tag-icon-container:global(.color-pulse) .tag-icon-content {
    animation: colorPulse 280ms ease-out;
  }
  @keyframes colorPulse {
    0% {
      transform: scale(0.92);
      opacity: 0.85;
    }
    60% {
      transform: scale(1.04);
      opacity: 1;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }
</style>
