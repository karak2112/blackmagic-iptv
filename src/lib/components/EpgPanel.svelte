<script lang="ts">
  import type { Channel, NowNext } from "$lib/types";

  interface Props {
    channels: Channel[];
    nowNext: Map<string, NowNext>;
    highlightedId: string | null;
    previewActive: boolean;
    onHighlight: (channel: Channel) => void;
    onPlay: (channel: Channel) => void;
  }

  let { channels, nowNext, highlightedId, previewActive, onHighlight, onPlay }: Props =
    $props();

  let listEl = $state<HTMLDivElement | null>(null);

  function formatTime(iso: string): string {
    try {
      return new Date(iso).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return "";
    }
  }

  function formatRange(start: string, stop: string): string {
    return `${formatTime(start)} – ${formatTime(stop)}`;
  }

  function scrollToHighlighted(id: string | null) {
    if (!id || !listEl) return;
    const row = listEl.querySelector(`[data-channel-id="${id}"]`);
    row?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (channels.length === 0) return;
    const idx = channels.findIndex((c) => c.id === highlightedId);
    const currentIdx = idx >= 0 ? idx : 0;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = channels[Math.min(currentIdx + 1, channels.length - 1)];
      if (next) {
        onHighlight(next);
        scrollToHighlighted(next.id);
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      const prev = channels[Math.max(currentIdx - 1, 0)];
      if (prev) {
        onHighlight(prev);
        scrollToHighlighted(prev.id);
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      const ch = channels[currentIdx];
      if (ch) onPlay(ch);
    }
  }

  $effect(() => {
    scrollToHighlighted(highlightedId);
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="guide" class:preview-active={previewActive} role="region" aria-label="Program guide">
  <header>
    <h2>TV Guide</h2>
    <p class="subtitle">Hover or ↑↓ to preview · Enter or double-click to watch</p>
  </header>

  <div class="guide-list" bind:this={listEl}>
    {#each channels as channel (channel.id)}
      {@const nn = nowNext.get(channel.id)}
      {@const highlighted = highlightedId === channel.id}
      <button
        class="guide-row"
        class:highlighted
        data-channel-id={channel.id}
        onmouseenter={() => onHighlight(channel)}
        onclick={() => onHighlight(channel)}
        ondblclick={() => onPlay(channel)}
      >
        <div class="channel-col">
          <span class="ch-name">{channel.name}</span>
          {#if channel.group}
            <span class="ch-group">{channel.group}</span>
          {/if}
        </div>
        <div class="prog-col">
          {#if nn?.now}
            <div class="prog now">
              <span class="badge">NOW</span>
              <span class="title">{nn.now.title}</span>
              <span class="time">{formatRange(nn.now.start, nn.now.stop)}</span>
            </div>
          {:else}
            <div class="prog empty">No current programme</div>
          {/if}
        </div>
        <div class="prog-col">
          {#if nn?.next}
            <div class="prog next">
              <span class="badge">NEXT</span>
              <span class="title">{nn.next.title}</span>
              <span class="time">{formatRange(nn.next.start, nn.next.stop)}</span>
            </div>
          {:else}
            <div class="prog empty">—</div>
          {/if}
        </div>
      </button>
    {/each}

    {#if channels.length === 0}
      <p class="empty-state">Load a playlist and EPG in Settings to see the guide.</p>
    {/if}
  </div>
</div>

<style>
  .guide {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0 340px 0 20px;
    background-color: var(--bg-primary);
    background-image:
      radial-gradient(
        ellipse at 15% 0%,
        rgba(155, 92, 255, 0.07) 0%,
        rgb(13, 15, 20) 55%
      ),
      radial-gradient(
        ellipse at 85% 100%,
        rgba(107, 63, 191, 0.05) 0%,
        rgb(13, 15, 20) 50%
      ),
      repeating-linear-gradient(
        -45deg,
        rgba(255, 255, 255, 0.015),
        rgba(255, 255, 255, 0.015) 12px,
        rgba(255, 255, 255, 0.005) 12px,
        rgba(255, 255, 255, 0.005) 24px
      );
  }

  /* Shrink opaque guide away from the PIP column so video can show through the webview hole. */
  .guide.preview-active {
    padding-right: 20px;
    margin-right: 336px;
    max-width: calc(100% - 336px);
  }

  header {
    padding: 16px 0 12px;
    border-bottom: 1px solid var(--border);
  }

  h2 {
    margin: 0 0 4px;
    font-size: 18px;
  }

  .subtitle {
    margin: 0;
    color: var(--text-muted);
    font-size: 13px;
  }

  .guide-list {
    overflow-y: auto;
    flex: 1;
    padding: 8px 0;
  }

  .guide-row {
    display: grid;
    grid-template-columns: 200px 1fr 1fr;
    gap: 16px;
    width: 100%;
    padding: 14px 12px;
    border-radius: var(--radius);
    text-align: left;
    border-bottom: 1px solid var(--border);
    align-items: start;
    color: inherit;
  }

  .guide-row:hover,
  .guide-row.highlighted {
    background: var(--bg-elevated);
  }

  .guide-row.highlighted {
    box-shadow: inset 0 0 0 1px var(--accent-dim);
  }

  .channel-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .ch-name {
    font-weight: 600;
  }

  .ch-group {
    font-size: 12px;
    color: var(--text-muted);
  }

  .prog {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .prog.empty {
    color: var(--text-muted);
    font-size: 13px;
  }

  .badge {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.05em;
    color: var(--accent);
  }

  .prog.next .badge {
    color: var(--text-muted);
  }

  .title {
    font-size: 14px;
  }

  .time {
    font-size: 12px;
    color: var(--text-muted);
  }

  .empty-state {
    text-align: center;
    color: var(--text-muted);
    padding: 48px;
  }
</style>
