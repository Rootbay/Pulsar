<script lang="ts">
	import type { PasswordItem } from '../../../routes/+layout.ts';
	import { createEventDispatcher } from 'svelte';
	import TagList from './TagList.svelte';
	import Icon from '../ui/Icon.svelte';
	import { iconPaths } from '$lib/icons';

	const dispatch = createEventDispatcher();

	export let selectedPasswordItem: PasswordItem | null;
	export let isEditing: boolean;
	
	
	export let displayColor: string;
	export let buttons: any[];

	let showMoreDropdown = false;

    let prevColor: string | null = null;
    let pulse = false;

    $: if (selectedPasswordItem) {
        if (prevColor === null) {
            prevColor = displayColor;
        } else if (prevColor !== displayColor) {
            pulse = false;
            setTimeout(() => { pulse = true; prevColor = displayColor; setTimeout(() => (pulse = false), 360); }, 0);
        }
    }

    function enterEditMode() {
        dispatch('enterEditMode');
    }

	function handleReset() {
		dispatch('handleReset');
	}
</script>

{#if selectedPasswordItem}
    <div class="detail-header" style="--display-color: {displayColor};">
        <div class="title-and-tags">
            <div class="title-container">
                {#if selectedPasswordItem.img}
                    <img src={selectedPasswordItem.img} alt={selectedPasswordItem.title} class="title-image" />
                {:else}
                    <Icon path={iconPaths.default} size="24" color={displayColor} className="header-icon" />
                {/if}
                <h2 class="header-title" style="color: {displayColor}">{selectedPasswordItem.title}</h2>
                <span class="color-pulse-bg" aria-hidden="true" class:pulsing={pulse}></span>
            </div>
    <TagList
      bind:selectedPasswordItem
      bind:isEditing
      {buttons}
      on:reorderPending={(e) => dispatch('tagsReorderedPending', e.detail)}
      on:tagsSaved={(e) => dispatch('tagsSaved', e.detail)}
    />
        </div>
		<div class="detail-actions">
			<button class="edit-button" on:click={isEditing ? () => dispatch('save') : enterEditMode}>
				{isEditing ? 'Save' : 'Modify'}
			</button>
			<div class="more-dropdown-container">
				<button class="more-button" on:click={() => (showMoreDropdown = !showMoreDropdown)}>
					<Icon path={iconPaths.more} size="24" color="currentColor" />
				</button>
				{#if showMoreDropdown}
					<div class="more-dropdown-menu">
						<button on:click={() => dispatch('removeEntry', selectedPasswordItem?.id)}
							>Delete Entry</button
						>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.detail-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 20px;
	}

	.title-container {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 5px;
	}

	.title-image {
		width: 24px;
		height: 24px;
		border-radius: 4px;
		object-fit: contain;
	}

    .detail-header h2 {
        margin: 0;
        font-weight: 400;
        transition: color 260ms ease;
    }

    .title-and-tags {
        display: flex;
        flex-direction: column;
    }

    /* Smooth color transition for icon */
    .header-icon { transition: fill 260ms ease; }

    /* Ambient pulse background when color changes */
    .title-container { position: relative; }
    .color-pulse-bg {
        position: absolute;
        left: -6px;
        top: -6px;
        width: 44px;
        height: 44px;
        border-radius: 12px;
        background: var(--display-color);
        filter: blur(8px);
        opacity: 0;
        pointer-events: none;
        transform: scale(0.85);
        transition: opacity 360ms ease, transform 360ms ease, background-color 260ms ease;
    }
    .color-pulse-bg.pulsing {
        opacity: 0.25;
        transform: scale(1.05);
    }

	.detail-actions {
		display: flex;
		align-items: center;
	}

	.edit-button {
		background-color: var(--near-black);
		border: 2px solid var(--btn-nav-border);
		border-radius: 13.5px;
		font-family: inherit;
		font-style: normal;
		font-weight: 500;
		font-size: 0.8rem;
		color: rgba(247, 219, 209, 0.5);
		cursor: pointer;
		width: 68px;
		height: 27px;
        margin-right: 10px;
	}

	.edit-button:hover {
		background-color: #17171b;
	}

	.more-dropdown-container {
		position: relative;
		display: inline-block;
	}

	.more-button {
		background: none;
		border: none;
		padding: 4px 0 0 0;
		color: #f7dbd1;
		width: 24px;
		height: 24px;
		cursor: pointer;
	}

	.more-button:hover {
		color: #ffbfa8;
	}

	.more-dropdown-menu {
		position: absolute;
		top: 100%;
		right: 0;
		background-color: var(--near-black);
		border: 2px solid var(--btn-nav-border);
		border-radius: 8px;
		box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
		z-index: 200;
		display: flex;
		flex-direction: column;
		padding: 5px 0;
		min-width: 150px;
	}

	.more-dropdown-menu button {
		background: none;
		border: none;
		color: var(--white);
		padding: 8px 15px;
		text-align: left;
		cursor: pointer;
		width: 100%;
		white-space: nowrap;
	}

	.more-dropdown-menu button:hover {
		background-color: #3a3a3a;
	}
</style>
