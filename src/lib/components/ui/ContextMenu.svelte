<script lang="ts">
	import { createEventDispatcher, onMount, onDestroy } from 'svelte';

	export let x: number;
	export let y: number;

	const dispatch = createEventDispatcher();

	function close() {
		dispatch('close');
	}

	function handleEdit() {
		dispatch('edit');
	}

	function handleRemove() {
		dispatch('remove');
	}

	let element: HTMLDivElement;

	function handleClickOutside(event: MouseEvent) {
		if (element && !element.contains(event.target as Node)) {
			close();
		}
	}

	onMount(() => {
		window.addEventListener('click', handleClickOutside, true);
	});

	onDestroy(() => {
		window.removeEventListener('click', handleClickOutside, true);
	});
</script>

<div bind:this={element} class="context-menu" style="top: {y}px; left: {x}px;" on:contextmenu|preventDefault role="menu" tabindex="-1">
  <ul>
    <li><button on:click={handleEdit} role="menuitem">Edit</button></li>
    <li><button on:click={handleRemove} role="menuitem">Remove</button></li>
  </ul>
</div>

<style>
  .context-menu {
    position: fixed;
    background: #2a2a2e;
    border: 1px solid #3a3a3e;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    z-index: 200;
    padding: 6px;
    font-size: 14px;
    min-width: 120px;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  li button {
    padding: 8px 12px;
    cursor: pointer;
    color: var(--text-color);
    border-radius: 4px;
    display: flex;
    align-items: center;
    background: none;
    border: none;
    width: 100%;
    text-align: left;
  }
  li button:hover {
    background: #3a3a3e;
    color: var(--white);
  }
</style>