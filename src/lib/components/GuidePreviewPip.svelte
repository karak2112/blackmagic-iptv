<script lang="ts">
  import type { Channel, NowNext } from "$lib/types";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  export type PreviewBounds = {
    clientX: number;
    clientY: number;
    width: number;
    height: number;
    windowWidth: number;
    windowHeight: number;
  };

  export type PreviewStatus =
    | "idle"
    | "waiting"
    | "loading"
    | "live"
    | "error"
    | "unavailable";

  interface Props {
    channel: Channel | null;
    nowNext: NowNext | null;
    visible: boolean;
    status: PreviewStatus;
    error: string | null;
    onBoundsChange: (bounds: PreviewBounds) => void;
  }

  let { channel, nowNext, visible, status, error, onBoundsChange }: Props = $props();

  let videoSlot = $state<HTMLDivElement | null>(null);
  let observer: ResizeObserver | null = null;
  let lastBoundsKey = "";

  const statusMessage = $derived.by(() => {
    switch (status) {
      case "unavailable":
        return "Preview unavailable";
      case "loading":
        return "Tuning…";
      case "waiting":
        return "Starting preview…";
      case "error":
        return error ?? "Preview failed";
      case "idle":
        return "Select a channel";
      default:
        return null;
    }
  });

  function boundsKey(bounds: PreviewBounds) {
    return [
      bounds.clientX,
      bounds.clientY,
      bounds.width,
      bounds.height,
      bounds.windowWidth,
      bounds.windowHeight,
    ]
      .map((v) => Math.round(v))
      .join(":");
  }

  function reportBounds() {
    if (!videoSlot) return;
    const rect = videoSlot.getBoundingClientRect();
    if (rect.width < 1 || rect.height < 1) return;
    const bounds: PreviewBounds = {
      clientX: rect.left,
      clientY: rect.top,
      width: rect.width,
      height: rect.height,
      windowWidth: window.innerWidth,
      windowHeight: window.innerHeight,
    };
    const key = boundsKey(bounds);
    if (key === lastBoundsKey) return;
    lastBoundsKey = key;
    onBoundsChange(bounds);
  }

  const onResize = () => reportBounds();

  $effect(() => {
    if (!visible || !videoSlot) return;

    lastBoundsKey = "";
    const report = () => reportBounds();
    report();
    requestAnimationFrame(report);

    observer = new ResizeObserver(report);
    observer.observe(videoSlot);
    window.addEventListener("resize", onResize);

    let unlistenMove: (() => void) | undefined;
    let unlistenScale: (() => void) | undefined;
    const win = getCurrentWindow();
    win
      .onMoved(onResize)
      .then((fn) => (unlistenMove = fn))
      .catch(() => {});
    win
      .onScaleChanged(onResize)
      .then((fn) => (unlistenScale = fn))
      .catch(() => {});

    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", onResize);
      unlistenMove?.();
      unlistenScale?.();
    };
  });
</script>

{#if visible}
  <!-- Opaque mask below the PIP frame (above mpv layer, below PIP chrome). -->
  <div class="pip-column-mask" aria-hidden="true"></div>
  <aside class="pip" aria-label="Channel preview">
    <div class="pip-shell">
      <div class="video-slot" bind:this={videoSlot} aria-hidden="true">
        {#if statusMessage && status !== "live"}
          <span class="status-chip">{statusMessage}</span>
        {/if}
      </div>
      {#if channel}
        <div class="pip-label">
          <span class="pip-name">{channel.name}</span>
          {#if nowNext?.now}
            <span class="pip-epg">{nowNext.now.title}</span>
          {/if}
        </div>
      {/if}
    </div>
    <p class="pip-hint">Hover or ↑↓ · Enter to watch</p>
  </aside>
{/if}

<style>
  /* Covers the right column below the PIP — mpv layer (z-100) shows through the video frame only. */
  .pip-column-mask {
    position: fixed;
    top: 280px;
    right: 0;
    bottom: 0;
    width: 336px;
    background: var(--bg-primary);
    z-index: 115;
    pointer-events: none;
  }

  .pip {
    position: fixed;
    top: 12px;
    right: 16px;
    z-index: 200;
    width: 320px;
    pointer-events: none;
  }

  .pip-shell {
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow:
      0 0 0 2px var(--accent-dim),
      0 8px 32px rgba(0, 0, 0, 0.55),
      0 0 24px var(--accent-glow);
  }

  .video-slot {
    aspect-ratio: 16 / 9;
    position: relative;
    min-height: 180px;
    /* No background — mpv is visible through PlayerOverlay beneath guide UI. */
    background: transparent;
  }

  .status-chip {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    padding: 6px 12px;
    border-radius: 999px;
    background: rgba(10, 5, 16, 0.82);
    font-size: 12px;
    color: var(--text-muted);
    pointer-events: none;
    white-space: nowrap;
  }

  .pip-label {
    padding: 8px 10px;
    background: rgba(18, 8, 24, 0.98);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .pip-name {
    font-size: 13px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pip-epg {
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pip-hint {
    margin: 6px 0 0;
    text-align: center;
    font-size: 10px;
    color: var(--text-muted);
  }
</style>
