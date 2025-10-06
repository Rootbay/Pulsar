<script lang="ts">
  import Icon from "../ui/Icon.svelte";
  import { onMount, tick } from "svelte";
  import type { Snippet } from "svelte";

  interface $$Slots {
    default?: Snippet;
    rightIcon?: Snippet;
  }

  interface Props {
    selectedColor?: string;
    inputValue?: string | null;
    placeholder?: string;
    initialInput?: HTMLInputElement | HTMLTextAreaElement | null;
    title: string;
    showTitle?: boolean;
    selectedIconPath?: string;
    selectedIconName?: string;
    readOnly?: boolean;
    isMultiline?: boolean;
    type?: string;
    isExpandable?: boolean;
    rightIcon?: Snippet;
    children?: Snippet;
  }

  let {
    selectedColor = undefined,
    inputValue = $bindable<string | null>(null),
    placeholder = "",
    initialInput = null,
    title,
    showTitle = true,
    selectedIconPath = undefined,
    selectedIconName = undefined,
    readOnly = false,
    isMultiline = false,
    type = "text",
    isExpandable = false,
    rightIcon
  }: Props = $props();

  let expanded =  $state(false); 

  $effect(() => {
    if (inputValue == null) {
      inputValue = '';
    }
  });

  // Treat empty or literal "N/A" values in read-only mode as placeholders
  const isPlaceholderValue = $derived(() => readOnly && (!inputValue || inputValue === 'N/A'));

  function resizeTextarea() {
    if (!isMultiline) return;
    const node = initialInput as HTMLTextAreaElement | null;
    if (!node) return;
    node.style.height = "auto";
    node.style.height = node.scrollHeight + "px";
  }

  onMount(() => {
    resizeTextarea();
  });

  $effect(() => {
    if (isMultiline) {
      tick().then(() => resizeTextarea());
    }
  });

  function focusInnerInput() {
    const node = initialInput as HTMLInputElement | HTMLTextAreaElement | null;
    if (!node) return;
    node.focus();
    try {
      const len = (node as HTMLInputElement | HTMLTextAreaElement).value?.length ?? 0;
      if (typeof (node as any).setSelectionRange === 'function') {
        (node as any).setSelectionRange(len, len);
      }
    } catch {}
  }

  async function handleContainerClick(event: MouseEvent) {
    if (isExpandable && isMultiline) {
      expanded = !expanded;
      await tick();
    }
    if (readOnly) return;
    const target = event.target as HTMLElement;
    if (target && target.closest('button')) return;
    focusInnerInput();
  }
</script>

<div
  class="inputContainer"
  class:singleline={!isMultiline}
  class:editable={!readOnly}
  class:has-placeholder-value={isPlaceholderValue}
  style="color: {selectedColor || '#8aa0ff'}"
  class:multiline-expanded={expanded && isMultiline}
  class:multiline-collapsed={!expanded && isMultiline}
  onclick={handleContainerClick}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (!isExpandable || !isMultiline) return;
    if (e.target !== e.currentTarget) return;
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      expanded = !expanded;
    }
  }}
>
  <span class="titleIcon" style="color: {selectedColor || '#fff'}" title={selectedIconName || ''}>
    <Icon path={selectedIconPath || ''} color="currentColor" size="24" viewBox='0 0 48 48' />
  </span>
  <div class="titleAndInputContainer">
    {#if showTitle}
      <div class="inputTitle" style="color: {selectedColor || '#fff'}">{title}</div>
    {/if}
    {#if isMultiline}
      <div class="textarea-clip">
        <textarea
          bind:this={initialInput}
          placeholder={placeholder}
          bind:value={inputValue}
          oninput={resizeTextarea}
          readonly={readOnly}
        ></textarea>
      </div>
    {:else}
      <input
        bind:this={initialInput}
        type={type}
        placeholder={placeholder}
        bind:value={inputValue}
        readonly={readOnly}
      />
    {/if}
  </div>
  <div class="right-icon-wrapper">
    {@render rightIcon?.()}
  </div>
</div>

<style>
  .inputContainer {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 48px;
    /* Keep standard padding; extra left space added via .titleIcon margin-left */
    padding: 10px 12px;
    border-radius: var(--radius, 12px);
    background: #151516;
    border: none;
    box-sizing: border-box;
    transition: color 260ms ease, background-color 150ms ease-out, box-shadow 150ms ease-out;
  }

  .inputContainer:hover {
    background: #17171B;
  }

  .inputContainer:focus-within {
    background: #17171B;
    box-shadow: none;
  }

  .inputContainer input,
  .inputContainer textarea {
    width: 100%;
    display: block;
    background: transparent;
    border: none;
    outline: none;
    color: #D9D9D9;
    font-size: 14px;
    padding: 0;
    min-height: 17px;
    line-height: 17px;
    overflow-y: hidden;
    box-sizing: border-box;
    font-family: inherit;
    resize: none;
    cursor: inherit;
  }

  /* When showing placeholder-like value (empty or "N/A") */
  .inputContainer.has-placeholder-value input,
  .inputContainer.has-placeholder-value textarea {
    color: #9aa0a6;
    font-size: 12px;
  }

  .textarea-clip {
    width: 100%;
    box-sizing: border-box;
  }

  .inputContainer.multiline-collapsed .textarea-clip {
    max-height: 50px;
    overflow: hidden;
  }

  .inputContainer.multiline-expanded .textarea-clip {
    max-height: none;
    overflow: visible;
  }

  .textarea-clip textarea {
    width: 100%;
    display: block;
    box-sizing: border-box;
    overflow-y: hidden;
  }

  .inputContainer input::placeholder,
  .inputContainer textarea::placeholder {
    color: #D9D9D9;
  }

  .titleAndInputContainer {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .inputTitle {
    width: 100%;
    text-align: left;
    font-size: 12px;
    font-weight: 500;
    margin-bottom: 4px;
    transition: color 260ms ease;
  }

  .titleIcon :global(svg) {
    width: 19px;
    height: 19px;
    display: block;
    transition: fill 220ms ease;
  }

  .titleIcon {
    align-self: center;
    margin-top: 0;
    /* Additive offsets: +2px from border, +3px before input */
    margin-left: 2px;
    /* Extra spacing between SVG and input (adds 3px beyond the standard gap) */
    margin-right: 3px;
    display: flex;
    align-items: center;
    transition: color 260ms ease;
  }

  /* For multiline inputs (e.g., Notes), keep icon fixed at top */
  .inputContainer.multiline-expanded .titleIcon,
  .inputContainer.multiline-collapsed .titleIcon {
    align-self: flex-start;
    margin-top: 6px;
  }

  .right-icon-wrapper {
    display: flex;
    align-items: center;
  }

  .inputContainer.editable {
    cursor: text;
  }

  /* Fixed height for single-line inputs */
  .inputContainer.singleline {
    height: 56px;
  }

  /* Non-editable (read-only) inputs should not show text cursor */
  .inputContainer:not(.editable) {
    cursor: default;
  }
  .inputContainer:not(.editable):hover {
    cursor: default;
  }
  .inputContainer:not(.editable):not(.has-placeholder-value):hover {
    cursor: pointer;
  }
  .inputContainer:not(.editable) input,
  .inputContainer:not(.editable) textarea {
    cursor: inherit;
  }
</style>
