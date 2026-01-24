<script lang="ts">
  import { onMount } from 'svelte';
  import { callBackend } from '$lib/utils/backend';
  import Input from './Input.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import { iconPaths } from '$lib/icons';

  interface Tag {
    id: number;
    text: string;
    icon: string;
    color: string;
  }

  interface Props {
    mode?: 'create' | 'edit';
    tag?: Tag | null;
    onclose?: () => void;
    onsave?: (detail: { mode: 'create' | 'edit'; updatedTag?: any }) => void;
  }

  let { mode = 'create', tag = null, onclose, onsave }: Props = $props();

  let inputValue = $state('');
  let selectedColor = $state('#F29292');
  const colors = [
    '#F29292',
    '#F7D775',
    '#91C799',
    '#92B3F2',
    '#EB8DD6',
    '#CD5A6F',
    '#E4A367',
    '#E6E6E6',
    '#9EE2E6'
  ];

  interface IconType {
    name: string;
    path: string;
  }

  const icons: IconType[] = [
    { name: 'home', path: iconPaths.home },
    { name: 'globe', path: iconPaths.globe },
    { name: 'messenger', path: iconPaths.messenger },
    { name: 'wallet', path: iconPaths.wallet },
    { name: 'lock', path: iconPaths.lock },
    { name: 'folder', path: iconPaths.folder },
    { name: 'paper', path: iconPaths.paper },
    { name: 'card', path: iconPaths.card },
    { name: 'key', path: iconPaths.key }
  ];

  let selectedIcon = $state<IconType>(icons[0]);

  onMount(() => {
    if (mode === 'edit' && tag) {
      inputValue = tag.text;
      selectedColor = tag.color;
      selectedIcon = icons.find((i) => i.path === tag.icon) || icons[0];
    }
  });

  function selectColor(color: string) {
    selectedColor = color;
  }

  function selectIcon(icon: IconType) {
    selectedIcon = icon;
  }

  function onBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).id === 'popup-backdrop') {
      onclose?.();
    }
  }

  function onBackdropKey(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      onclose?.();
    }
  }

  async function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onclose?.();
    } else if (e.key === 'Enter') {
      if (inputValue && selectedIcon && selectedColor) {
        const buttonData = {
          id: tag?.id,
          text: inputValue,
          icon: selectedIcon.path,
          color: selectedColor
        };

        if (mode === 'create') {
          await callBackend('save_button', {
            text: buttonData.text,
            icon: buttonData.icon,
            color: buttonData.color
          });
          onsave?.({ mode: 'create' });
        } else if (mode === 'edit' && buttonData.id) {
          await callBackend('update_button', buttonData);
          onsave?.({ mode: 'edit', updatedTag: buttonData });
        }

        onclose?.();
      }
    }
  }

  let initialInput = $state<HTMLInputElement | null>(null);
  onMount(() => {
    initialInput?.focus();
  });
</script>

<div
  id="popup-backdrop"
  onclick={onBackdropClick}
  onkeydown={onBackdropKey}
  role="presentation"
  aria-label="New Item"
  tabindex="-1"
>
  <div class="popupContent" tabindex="-1" onkeydown={onKeydown} role="dialog">
    <Input
      {selectedColor}
      placeholder="Enter tag name"
      bind:initialInput
      title="Title"
      selectedIconPath={selectedIcon.path}
      selectedIconName={selectedIcon.name}
      bind:inputValue
    />

    <div class="popupItem">
      <div class="formSection">
        <div class="itemColorContainer">
          {#each colors as color (color)}
            <button
              class="colorButton {selectedColor === color ? 'selected' : ''}"
              style="background-color: {color}"
              onclick={() => selectColor(color)}
              aria-label="Select color {color}"
            ></button>
          {/each}
        </div>
      </div>

      <div class="formSection">
        <div class="itemIconContainer">
          {#each icons as icon (icon.name)}
            <button
              class="iconButton {selectedIcon.name === icon.name ? 'selected' : ''}"
              onclick={() => selectIcon(icon)}
              aria-label="Select icon {icon.name}"
              title={icon.name}
            >
              <Icon path={icon.path} color={selectedColor} viewBox="0 0 48 48" />
            </button>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  #popup-backdrop {
    position: fixed;
    inset: 0;
    background:
      radial-gradient(120% 120% at 50% -10%, rgba(255, 255, 255, 0.06) 0%, rgba(0, 0, 0, 0) 60%),
      rgba(6, 8, 12, 0.55);
    backdrop-filter: blur(10px) saturate(120%);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 200;
  }

  .popupContent {
    width: min(80vw, 640px);
    border-radius: var(--radius, 12px);
    background: linear-gradient(135deg, hsl(var(--card)), hsl(var(--muted) / 0.3));
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px;
    gap: 12px;
    border: 1px solid hsl(var(--border) / 0.5);
  }

  .popupItem {
    width: 100%;
  }
  .formSection {
    width: 100%;
  }
  /* sectionTitle removed per request */

  .itemColorContainer {
    width: 100%;
    display: flex;
    justify-content: space-between;
    min-height: 34px;
    gap: 10px;
    flex-wrap: wrap;
  }

  .colorButton {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: none;
    cursor: pointer;
    outline: none;
    position: relative;
    flex: 0 0 34px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.25);
    transition:
      transform 140ms ease,
      box-shadow 140ms ease,
      filter 140ms ease;
  }

  .itemIconContainer {
    margin-top: 8px;
    display: flex;
    gap: 10px;
    width: 100%;
    justify-content: space-between;
    flex-wrap: wrap;
  }

  .iconButton {
    width: 34px;
    height: 34px;
    font-size: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 12px;
    border: 1px solid rgba(80, 80, 92, 0.35);
    background: linear-gradient(180deg, #1a1b20, #14151a);
    cursor: pointer;
    color: var(--white);
    position: relative;
    padding: 0;
    flex: 0 0 34px;
    box-shadow: 0 5px 14px rgba(0, 0, 0, 0.35);
    transition:
      transform 140ms ease,
      box-shadow 140ms ease,
      background-color 140ms ease,
      border-color 140ms ease;
  }

  .colorButton.selected::before,
  .iconButton.selected::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    padding: 3px;
    background: linear-gradient(90deg, #f7dbd1, #c587cb, #8dc5f7);
    -webkit-mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    pointer-events: none;
  }

  .colorButton.selected::before {
    border-radius: 50%;
  }

  .iconButton :global(svg) {
    width: 22px;
    height: 22px;
    display: block;
  }

  .colorButton:hover {
    transform: scale(1.08);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.35);
    filter: brightness(1.05);
  }
  .iconButton:hover {
    transform: translateY(-1px) scale(1.04);
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.45);
  }
  .iconButton:active {
    transform: translateY(0) scale(0.98);
    box-shadow: 0 5px 14px rgba(0, 0, 0, 0.35);
  }
  .iconButton.selected {
    border-color: rgba(120, 120, 220, 0.45);
    background: linear-gradient(180deg, #1d1e24, #16171c);
  }
  .iconButton:focus-visible,
  .colorButton:focus-visible {
    outline: 2px solid rgba(120, 160, 255, 0.65);
    outline-offset: 2px;
  }

  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.985);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @media (max-width: 560px) {
    .popupContent {
      padding: 16px;
      gap: 10px;
      width: min(92vw, 560px);
    }
    .itemColorContainer {
      justify-content: center;
    }
    .itemIconContainer {
      justify-content: center;
    }
  }
</style>
