<script lang="ts">
  import PlayerInfoBanner from "$lib/components/PlayerInfoBanner.svelte";
  import PlayerMiniEpg from "$lib/components/PlayerMiniEpg.svelte";
  import type { Channel, NowNext, StreamStats } from "$lib/types";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  interface Props {
    channel: Channel | null;
    nowNext: NowNext | null;
    streamStats: StreamStats | null;
    playing: boolean;
    paused: boolean;
    volume: number;
    muted: boolean;
    visible: boolean;
    guidePreview?: boolean;
    engineName: string;
    videoAvailable: boolean;
    playbackError: string | null;
    showInfoBanner: boolean;
    onClose: () => void;
    onTogglePlay: () => void;
    onStop: () => void;
    onVolumeChange: (v: number) => void;
    onVolumeDelta: (delta: number) => void;
    onToggleMute: () => void;
    onReload: () => void;
    onChannelDelta: (delta: number) => void;
    recordAvailable?: boolean;
    recording?: boolean;
    onToggleRecord?: () => void;
  }

  let {
    channel,
    nowNext,
    streamStats,
    playing,
    paused,
    volume,
    muted,
    visible,
    guidePreview = false,
    engineName,
    videoAvailable,
    playbackError,
    showInfoBanner,
    onClose,
    onTogglePlay,
    onStop,
    onVolumeChange,
    onVolumeDelta,
    onToggleMute,
    onReload,
    onChannelDelta,
    recordAvailable = false,
    recording = false,
    onToggleRecord,
  }: Props = $props();

  let showControls = $state(true);
  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let isFullscreen = $state(false);
  let showMiniEpg = $state(false);

  const nativeVideoEngine = $derived(
    engineName === "libmpv" || engineName === "exoplayer",
  );
  const isExoPlayer = $derived(engineName === "exoplayer");

  function bumpControls() {
    showControls = true;
    if (hideTimer) clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (playing) showControls = false;
    }, 3000);
  }

  async function toggleFullscreen() {
    const win = getCurrentWindow();
    isFullscreen = !isFullscreen;
    await win.setFullscreen(isFullscreen);
    bumpControls();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (isFullscreen) {
        toggleFullscreen();
        return;
      }
      onClose();
    } else if (e.key === " " || e.code === "Space") {
      e.preventDefault();
      onTogglePlay();
      bumpControls();
    } else if (e.key === "m" || e.key === "M") {
      onToggleMute();
      bumpControls();
    } else if (e.key === "f" || e.key === "F") {
      e.preventDefault();
      toggleFullscreen();
    } else if (
      e.key === "PageUp" ||
      e.code === "ChannelUp"
    ) {
      e.preventDefault();
      onChannelDelta(1);
      bumpControls();
    } else if (
      e.key === "PageDown" ||
      e.code === "ChannelDown"
    ) {
      e.preventDefault();
      onChannelDelta(-1);
      bumpControls();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      onVolumeDelta(5);
      bumpControls();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      onVolumeDelta(-5);
      bumpControls();
    } else if (e.key === "+" || e.key === "=") {
      e.preventDefault();
      onVolumeDelta(5);
      bumpControls();
    } else if (e.key === "-" || e.key === "_") {
      e.preventDefault();
      onVolumeDelta(-5);
      bumpControls();
    } else if (e.key === "g" || e.key === "G") {
      e.preventDefault();
      showMiniEpg = !showMiniEpg;
      bumpControls();
    }
  }
</script>

<svelte:window onkeydown={visible && !guidePreview ? handleKeydown : undefined} />

{#if visible && channel}
  <div
    class="player"
    class:has-video={videoAvailable || nativeVideoEngine}
    class:guide-preview={guidePreview}
    onmousemove={guidePreview ? undefined : bumpControls}
    onclick={guidePreview ? undefined : bumpControls}
    role={guidePreview ? "presentation" : "dialog"}
    aria-label={guidePreview ? undefined : "Player"}
    aria-hidden={guidePreview ? "true" : undefined}
  >
    {#snippet videoSurface()}
      <div class="video-placeholder" class:has-video={videoAvailable || nativeVideoEngine}>
        {#if guidePreview}
          <!-- Full-window transparent layer: mpv shows through (same path as full player). -->
        {:else if !videoAvailable && !nativeVideoEngine}
          <p class="channel-name">{channel.name}</p>
          <p class="hint warn">
            {playbackError ??
              "Video engine not active. Install libmpv (see README) and restart the app."}
          </p>
          <p class="hint engine">Engine: {engineName}</p>
        {:else if isExoPlayer && playbackError}
          <p class="hint warn">{playbackError}</p>
          <p class="hint engine">Engine: {engineName}</p>
        {:else if playing && !paused}
          <!-- Native video (libmpv / ExoPlayer) renders behind this transparent layer -->
        {:else if paused}
          <p class="hint">Paused</p>
        {:else}
          <p class="hint">Starting stream…</p>
        {/if}
      </div>

      <PlayerInfoBanner
        {channel}
        {nowNext}
        {streamStats}
        visible={showInfoBanner && videoAvailable}
      />

      <PlayerMiniEpg {channel} {nowNext} visible={showMiniEpg} />
    {/snippet}

    {#if guidePreview}
      <div class="video-area">
        {@render videoSurface()}
      </div>
    {:else}
      <button
        type="button"
        class="video-area"
        aria-label="Toggle fullscreen"
        ondblclick={toggleFullscreen}
      >
        {@render videoSurface()}
      </button>
    {/if}

    {#if showControls && !guidePreview}
      <div class="overlay">
        <div class="top-bar">
          <button class="back-btn" onclick={onClose} aria-label="Back to browse">
            ← Back
          </button>
          <div class="channel-info">
            <h1>{channel.name}</h1>
            {#if nowNext?.now}
              <p class="now-playing">
                <span class="live">LIVE</span>
                {nowNext.now.title}
              </p>
            {/if}
          </div>
        </div>

        <div class="bottom-bar">
          <div class="controls-left">
            <button onclick={onTogglePlay} aria-label={playing && !paused ? "Pause" : "Play"}>
              {playing && !paused ? "⏸" : "▶"}
            </button>
            <button onclick={onStop} aria-label="Stop">⏹</button>
            <button onclick={onReload} aria-label="Reload stream">↻</button>
            <button onclick={() => onChannelDelta(-1)} aria-label="Previous channel">CH−</button>
            <button onclick={() => onChannelDelta(1)} aria-label="Next channel">CH+</button>
            {#if recordAvailable && onToggleRecord}
              <button
                class="record-btn"
                class:active={recording}
                onclick={onToggleRecord}
                aria-label={recording ? "Stop recording" : "Start recording"}
                aria-pressed={recording}
              >
                {recording ? "⏹ REC" : "⏺ REC"}
              </button>
            {/if}
          </div>

          <div class="controls-right">
            <button onclick={onToggleMute} aria-label={muted ? "Unmute" : "Mute"}>
              {muted ? "🔇" : "🔊"}
            </button>
            <button onclick={() => onVolumeDelta(-5)} aria-label="Volume down">−</button>
            <input
              type="range"
              min="0"
              max="100"
              value={volume}
              oninput={(e) => onVolumeChange(Number(e.currentTarget.value))}
              aria-label="Volume"
            />
            <button onclick={() => onVolumeDelta(5)} aria-label="Volume up">+</button>
            <span class="vol-label">{Math.round(volume)}</span>
            <button onclick={toggleFullscreen} aria-label="Toggle fullscreen">⛶</button>
          </div>
        </div>

        <div class="shortcuts-hint">
          PgUp/Dn CH · ↑↓ Vol · +− Vol · Space Play · F Fullscreen · G Guide · M Mute · Esc Back
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .player {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: #000;
    display: flex;
    flex-direction: column;
  }

  .player.has-video {
    background: transparent;
  }

  .player.guide-preview {
    z-index: 100;
    pointer-events: none;
  }

  .player.guide-preview .video-area {
    cursor: default;
  }

  .video-area {
    flex: 1;
    position: relative;
    cursor: pointer;
  }

  button.video-area {
    display: flex;
    flex-direction: column;
    width: 100%;
    border: none;
    margin: 0;
    padding: 0;
    background: none;
    font: inherit;
    color: inherit;
    text-align: inherit;
  }

  .video-placeholder {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: transparent;
    pointer-events: none;
  }

  .video-placeholder:not(.has-video) {
    background: radial-gradient(ellipse at center, #1a1f2e 0%, #000 70%);
    pointer-events: auto;
  }

  .channel-name {
    font-size: 28px;
    font-weight: 700;
    margin: 0 0 8px;
  }

  .hint {
    color: var(--text-muted);
    margin: 0;
  }

  .hint.warn {
    color: #fbbf24;
    max-width: 480px;
    text-align: center;
    line-height: 1.5;
    padding: 0 24px;
  }

  .hint.engine {
    font-size: 12px;
    margin-top: 8px;
  }

  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    background: linear-gradient(
      to bottom,
      rgba(0, 0, 0, 0.75) 0%,
      transparent 30%,
      transparent 70%,
      rgba(0, 0, 0, 0.85) 100%
    );
    pointer-events: none;
  }

  .overlay * {
    pointer-events: auto;
  }

  .top-bar {
    display: flex;
    align-items: flex-start;
    gap: 20px;
    padding: 24px 28px;
  }

  .back-btn {
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: var(--radius);
    font-size: 15px;
  }

  .channel-info h1 {
    margin: 0 0 6px;
    font-size: 24px;
  }

  .now-playing {
    margin: 0;
    color: var(--text-secondary);
    font-size: 15px;
  }

  .live {
    display: inline-block;
    background: var(--danger);
    color: white;
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 3px;
    margin-right: 8px;
    vertical-align: middle;
  }

  .bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 28px;
    gap: 12px;
    flex-wrap: wrap;
  }

  .controls-left,
  .controls-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .controls-left button,
  .controls-right button {
    font-size: 18px;
    padding: 8px 10px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: var(--radius);
  }

  .record-btn {
    font-size: 13px !important;
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  .record-btn.active {
    background: rgba(239, 68, 68, 0.35);
    box-shadow: inset 0 0 0 1px rgba(239, 68, 68, 0.7);
    color: #fecaca;
  }

  input[type="range"] {
    width: 100px;
    accent-color: var(--accent);
  }

  .vol-label {
    font-size: 13px;
    color: var(--text-secondary);
    min-width: 28px;
  }

  .shortcuts-hint {
    position: absolute;
    bottom: 8px;
    left: 28px;
    right: 28px;
    text-align: center;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
    pointer-events: none;
  }
</style>
