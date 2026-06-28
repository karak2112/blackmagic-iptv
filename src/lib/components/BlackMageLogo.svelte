<script lang="ts">
  interface Props {
    size?: number;
    showLabel?: boolean;
  }

  let { size = 40, showLabel = false }: Props = $props();
</script>

<div class="logo-wrap" style:width="{size}px" style:height="{size}px">
  <svg
    class="mage"
    viewBox="0 0 128 128"
    width={size}
    height={size}
    aria-hidden="true"
  >
    <rect width="128" height="128" rx="22" fill="#120818"/>
    <path d="M64 14 L34 72 L94 72 Z" fill="#3d2066"/>
    <path d="M64 8 L28 66 L100 66 Z" fill="#5a3388"/>
    <ellipse cx="64" cy="72" rx="36" ry="9" fill="#2a1848"/>
    <ellipse cx="64" cy="92" rx="30" ry="24" fill="#0a0510"/>
    <ellipse cx="52" cy="88" rx="7" ry="9" fill="#f0d830"/>
    <ellipse cx="76" cy="88" rx="7" ry="9" fill="#f0d830"/>
    <ellipse cx="52" cy="89" rx="2.5" ry="3.5" fill="#120818"/>
    <ellipse cx="76" cy="89" rx="2.5" ry="3.5" fill="#120818"/>
    <ellipse cx="64" cy="108" rx="8" ry="4" fill="#ff6b1a" opacity="0.55"/>
  </svg>
  {#if showLabel}
    <span class="label">Black Magic</span>
  {/if}
</div>

<style>
  .logo-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .mage {
    filter: drop-shadow(0 0 6px rgba(155, 92, 255, 0.45));
  }

  .label {
    font-size: 8px;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    text-transform: uppercase;
    white-space: nowrap;
  }
</style>
