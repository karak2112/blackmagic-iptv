<script lang="ts">
  import "../app.css";

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href="/tauri.svg" />
  <title>IPTV Player</title>
</svelte:head>

{@render children?.()}
