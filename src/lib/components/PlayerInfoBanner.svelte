<script lang="ts">
  import type { Channel, NowNext, StreamStats } from "$lib/types";
  import {
    formatBitrate,
    formatFps,
    formatResolution,
    qualityBadge,
  } from "$lib/stream";

  interface Props {
    channel: Channel;
    nowNext: NowNext | null;
    streamStats: StreamStats | null;
    visible: boolean;
  }

  let { channel, nowNext, streamStats, visible }: Props = $props();

  const badge = $derived(qualityBadge(streamStats));
  const resolution = $derived(formatResolution(streamStats));
  const fps = $derived(formatFps(streamStats?.fps));
  const videoBitrate = $derived(formatBitrate(streamStats?.video_bitrate_kbps));
  const audioBitrate = $derived(formatBitrate(streamStats?.audio_bitrate_kbps));
</script>

{#if visible}
  <div class="info-banner" aria-live="polite">
    <div class="row primary">
      <span class="channel-name">{channel.name}</span>
      {#if badge}
        <span class="badge quality">{badge}</span>
      {/if}
      {#if channel.tvg_id}
        <span class="meta">ID: {channel.tvg_id}</span>
      {/if}
      {#if channel.group}
        <span class="meta">{channel.group}</span>
      {/if}
    </div>

    {#if nowNext?.now || nowNext?.next}
      <div class="row epg">
        {#if nowNext?.now}
          <span><strong>Now:</strong> {nowNext.now.title}</span>
        {/if}
        {#if nowNext?.next}
          <span class="next"><strong>Next:</strong> {nowNext.next.title}</span>
        {/if}
      </div>
    {/if}

    <div class="row stream">
      {#if resolution}
        <span>{resolution}</span>
      {/if}
      {#if fps}
        <span>{fps}</span>
      {/if}
      {#if videoBitrate}
        <span>Video {videoBitrate}</span>
      {/if}
      {#if audioBitrate}
        <span>Audio {audioBitrate}</span>
      {/if}
      {#if streamStats?.video_codec}
        <span>{streamStats.video_codec.toUpperCase()}</span>
      {/if}
      {#if !resolution && !videoBitrate}
        <span class="meta">Stream info loading…</span>
      {/if}
    </div>
  </div>
{/if}

<style>
  .info-banner {
    position: absolute;
    left: 28px;
    right: 28px;
    bottom: 96px;
    padding: 14px 18px;
    border-radius: var(--radius-lg);
    background: rgba(8, 10, 16, 0.72);
    border: 1px solid rgba(255, 255, 255, 0.08);
    backdrop-filter: blur(8px);
    pointer-events: none;
    animation: fade-in 0.25s ease;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 16px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .row + .row {
    margin-top: 8px;
  }

  .row.primary {
    font-size: 15px;
    color: var(--text-primary);
  }

  .channel-name {
    font-weight: 700;
  }

  .badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.04em;
  }

  .badge.quality {
    background: var(--accent);
    color: white;
  }

  .meta {
    color: var(--text-muted);
    font-size: 12px;
  }

  .row.epg .next {
    opacity: 0.85;
  }

  .row.stream {
    font-size: 12px;
    font-family: ui-monospace, "Cascadia Mono", monospace;
    color: var(--text-muted);
  }
</style>
