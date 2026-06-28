<script lang="ts">
  import BlackMageLogo from "$lib/components/BlackMageLogo.svelte";
  import type { View } from "$lib/types";

  interface Props {
    current: View;
    onNavigate: (view: View) => void;
  }

  let { current, onNavigate }: Props = $props();

  const items: { id: View; label: string }[] = [
    { id: "browse", label: "Browse" },
    { id: "guide", label: "Guide" },
    { id: "settings", label: "Settings" },
  ];
</script>

<nav class="nav" aria-label="Main navigation">
  <div class="logo" title="Black Magic IPTV · BlackMagicSoftware.net">
    <BlackMageLogo size={36} />
    <span class="logo-sub">Black Magic</span>
  </div>
  {#each items as item}
    <button
      class="nav-item"
      class:active={current === item.id}
      onclick={() => onNavigate(item.id)}
      aria-current={current === item.id ? "page" : undefined}
      aria-label={item.label}
    >
      <span class="nav-icon" aria-hidden="true">
        {#if item.id === "browse"}📺{:else if item.id === "guide"}📋{:else}⚙️{/if}
      </span>
      <span class="label">{item.label}</span>
    </button>
  {/each}
</nav>

<style>
  .nav {
    width: var(--nav-width);
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 16px 0;
    gap: 8px;
    flex-shrink: 0;
  }

  .logo {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: 16px;
    padding: 8px;
    line-height: 1.1;
  }

  .logo-sub {
    font-size: 8px;
    letter-spacing: 0.1em;
    color: var(--accent);
    margin-top: 4px;
    text-transform: uppercase;
  }

  .nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 10px 8px;
    width: 56px;
    border-radius: var(--radius);
    color: var(--text-secondary);
    transition: background 0.15s, color 0.15s;
  }

  .nav-item:hover,
  .nav-item.active {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    box-shadow: inset 0 0 0 1px var(--accent-dim);
  }

  .nav-icon {
    font-size: 20px;
    line-height: 1;
  }

  .label {
    font-size: 10px;
    font-weight: 500;
  }
</style>
