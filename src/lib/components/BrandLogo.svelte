<script lang="ts">
  interface Props {
    variant?: "icon" | "full";
    size?: number;
    alt?: string;
  }

  let {
    variant = "icon",
    size = 36,
    alt = "Black Magic Software",
  }: Props = $props();

  const iconSrc = "/brand/logo-icon.png?v=8";
  const fullSrc = "/brand/logo-full-trimmed.png";
</script>

{#if variant === "icon"}
  <div
    class="icon-frame"
    style:width="{size}px"
    style:height="{size}px"
    aria-hidden="true"
  >
    <img class="brand-logo icon" src={iconSrc} {alt} />
  </div>
{:else}
  <img
    class="brand-logo full"
    src={fullSrc}
    {alt}
    style:max-width="{size}px"
  />
{/if}

<style>
  .icon-frame {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    padding: 2px;
    box-sizing: content-box;
  }

  .brand-logo {
    display: block;
    object-fit: contain;
    object-position: center;
  }

  .brand-logo.icon {
    width: 100%;
    height: 100%;
  }

  .brand-logo.full {
    width: 100%;
    height: auto;
  }
</style>
