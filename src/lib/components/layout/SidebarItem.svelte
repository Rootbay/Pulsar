<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let title: string;
  export let subtitle: string;
  export let active: boolean = false;
  export let type: string;
  export let color: string;

  const dispatch = createEventDispatcher();

  function handleClick() {
    dispatch('click');
  }
</script>

<button class="sidebar-item" class:active on:click={handleClick}>
  <div class="sidebar-icon {type}" style="color: {color};">
    <slot name="icon"></slot>
  </div>
  <div class="sidebar-text">
    <div class="sidebar-title">{title}</div>
    <div class="sidebar-subtitle">{subtitle}</div>
  </div>
  <svg class="sidebar-arrow" viewBox="0 0 24 24">
    <path d="m9 18 6-6-6-6" />
  </svg>
</button>

<style>
  .sidebar-item {
    width: 100%;
    padding: 16px;
    margin-bottom: 8px;
    border-radius: 16px;
    border: 2px solid transparent;
    background: transparent;
    cursor: pointer;
    transition: all 0.3s;
    display: flex;
    align-items: center;
    gap: 16px;
    text-align: left;
  }

  .sidebar-item:hover {
    background: hsl(var(--muted) / 0.3);
    transform: scale(1.02);
  }

  .sidebar-item.active {
    background: hsl(var(--primary) / 0.2);
    border-color: hsl(var(--primary) / 0.3);
    box-shadow: 0 8px 25px -8px hsl(var(--primary) / 0.3);
    transform: scale(1.02);
  }

  .sidebar-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s;
  }

  .sidebar-icon.account {
    background: hsl(210 100% 50% / 0.1);
    color: hsl(210 100% 50%);
  }

  .sidebar-icon.security {
    background: hsl(120 100% 40% / 0.1);
    color: hsl(120 100% 40%);
  }

  .sidebar-icon.appearance {
    background: hsl(280 100% 60% / 0.1);
    color: hsl(280 100% 60%);
  }

  .sidebar-icon.vault {
    background: hsl(50 100% 50% / 0.1);
    color: hsl(50 100% 50%);
  }

  .sidebar-icon.password-generation {
    background: hsl(0 100% 60% / 0.1);
    color: hsl(0 100% 60%);
  }

  .sidebar-icon.sync {
    background: hsl(190 100% 50% / 0.1);
    color: hsl(190 100% 50%);
  }

  .sidebar-icon.notifications {
    background: hsl(30 100% 50% / 0.1);
    color: hsl(30 100% 50%);
  }

  .sidebar-icon.clipboard {
    background: hsl(330 100% 60% / 0.1);
    color: hsl(330 100% 60%);
  }

  .sidebar-icon.advanced {
    background: hsl(30 100% 50% / 0.1);
    color: hsl(30 100% 50%);
  }

  .sidebar-icon.about {
    background: hsl(0 0% 50% / 0.1);
    color: hsl(0 0% 50%);
  }
  
  .sidebar-text {
    flex: 1;
  }

  .sidebar-title {
    font-weight: 600;
    color: hsl(var(--foreground));
    margin-bottom: 2px;
    transition: color 0.3s;
  }

  .sidebar-item.active .sidebar-title {
    color: hsl(var(--primary));
  }

  .sidebar-subtitle {
    font-size: 12px;
    color: hsl(var(--muted-foreground));
  }

  .sidebar-arrow {
    width: 20px;
    height: 20px;
    fill: var(--white);
    color: hsl(var(--muted-foreground));
    transition: all 0.3s;
  }

  .sidebar-item.active .sidebar-arrow {
    color: hsl(var(--primary));
    transform: rotate(90deg);
    fill: hsl(var(--primary));
  }

  .sidebar-icon :global(svg) {
    width: 28px;
    height: 28px;
    stroke: currentColor;
    fill: none;
  }
</style>
