<script lang="ts">
  import type { Channel, NowNext } from "$lib/types";

  interface Props {
    channel: Channel;
    nowNext: NowNext | null;
    visible: boolean;
  }

  let { channel, nowNext, visible }: Props = $props();

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

  function progressPercent(start: string, stop: string): number {
    const s = new Date(start).getTime();
    const e = new Date(stop).getTime();
    const now = Date.now();
    if (e <= s) return 0;
    return Math.min(100, Math.max(0, ((now - s) / (e - s)) * 100));
  }
</script>

{#if visible}
  <div class="mini-epg" role="dialog" aria-label="Program guide">
    <header>
      <h2>{channel.name}</h2>
      {#if channel.group}
        <span class="group">{channel.group}</span>
      {/if}
    </header>

    {#if nowNext?.now}
      <section class="prog now">
        <div class="prog-head">
          <span class="badge">NOW</span>
          <span class="time">{formatRange(nowNext.now.start, nowNext.now.stop)}</span>
        </div>
        <p class="title">{nowNext.now.title}</p>
        {#if nowNext.now.description}
          <p class="desc">{nowNext.now.description}</p>
        {/if}
        <div class="progress" aria-hidden="true">
          <div
            class="progress-fill"
            style:width="{progressPercent(nowNext.now.start, nowNext.now.stop)}%"
          ></div>
        </div>
      </section>
    {:else}
      <p class="empty">No current programme data</p>
    {/if}

    {#if nowNext?.next}
      <section class="prog next">
        <div class="prog-head">
          <span class="badge">NEXT</span>
          <span class="time">{formatRange(nowNext.next.start, nowNext.next.stop)}</span>
        </div>
        <p class="title">{nowNext.next.title}</p>
        {#if nowNext.next.description}
          <p class="desc">{nowNext.next.description}</p>
        {/if}
      </section>
    {/if}

    <p class="hint">Press G to close</p>
  </div>
{/if}

<style>
  .mini-epg {
    position: absolute;
    top: 72px;
    right: 28px;
    width: min(380px, calc(100vw - 56px));
    max-height: calc(100vh - 160px);
    overflow-y: auto;
    background: rgba(18, 8, 24, 0.94);
    border: 1px solid var(--accent-dim);
    border-radius: var(--radius-lg);
    padding: 16px 18px;
    box-shadow: 0 8px 40px rgba(0, 0, 0, 0.6), 0 0 20px var(--accent-glow);
    pointer-events: auto;
    z-index: 10;
  }

  header {
    margin-bottom: 14px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }

  h2 {
    margin: 0 0 4px;
    font-size: 18px;
  }

  .group {
    font-size: 12px;
    color: var(--text-muted);
  }

  .prog {
    margin-bottom: 14px;
  }

  .prog-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 4px;
  }

  .badge {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--accent);
  }

  .prog.next .badge {
    color: var(--text-muted);
  }

  .time {
    font-size: 12px;
    color: var(--text-muted);
  }

  .title {
    margin: 0 0 6px;
    font-size: 15px;
    font-weight: 600;
  }

  .desc {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.45;
    display: -webkit-box;
    -webkit-line-clamp: 4;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .progress {
    height: 3px;
    background: var(--bg-hover);
    border-radius: 2px;
    margin-top: 8px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
  }

  .empty {
    color: var(--text-muted);
    font-size: 14px;
    margin: 0 0 12px;
  }

  .hint {
    margin: 8px 0 0;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.4);
    text-align: center;
  }
</style>
