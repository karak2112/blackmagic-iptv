<script lang="ts">
  import NavBar from "$lib/components/NavBar.svelte";
  import GroupSidebar from "$lib/components/GroupSidebar.svelte";
  import ChannelBrowse from "$lib/components/ChannelBrowse.svelte";
  import EpgPanel from "$lib/components/EpgPanel.svelte";
  import GuidePreviewPip from "$lib/components/GuidePreviewPip.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import PlayerOverlay from "$lib/components/PlayerOverlay.svelte";
  import {
    listGroups,
    listChannels,
    getNowNext,
    getChannel,
    playChannel,
    previewChannel,
    setPreviewBounds,
    hidePreviewSurface,
    stopPlayback,
    pausePlayback,
    resumePlayback,
    setVolume,
    setMuted,
    getSettings,
    saveSettings,
    listFavorites,
    getPlaybackState,
    getStreamStats,
    zapChannel,
    getPlatform,
  } from "$lib/api";
  import type { Channel, GroupInfo, NowNext, StreamStats, View } from "$lib/types";

  let view = $state<View>("browse");
  let groups = $state<GroupInfo[]>([]);
  let channels = $state<Channel[]>([]);
  let guideChannels = $state<Channel[]>([]);
  let total = $state(0);
  let selectedGroup = $state<string | null>(null);
  let search = $state("");
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;
  let loading = $state(false);
  let favorites = $state<Set<string>>(new Set());
  let favoritesOnly = $state(false);
  let nowNext = $state<Map<string, NowNext>>(new Map());
  let hiddenGroups = $state<string[]>([]);
  let fontScale = $state(100);
  let browseScroll = $state(0);

  let highlightedChannel = $state<Channel | null>(null);
  let guideHighlight = $state<Channel | null>(null);
  let selectedChannel = $state<Channel | null>(null);
  let playing = $state(false);
  let paused = $state(false);
  let volume = $state(100);
  let muted = $state(false);
  let playerVisible = $state(false);
  let previewMode = $state(false);
  let engineName = $state("unknown");
  let videoAvailable = $state(false);
  let playbackError = $state<string | null>(null);
  let streamStats = $state<StreamStats | null>(null);
  let showInfoBanner = $state(false);
  let guidePreviewLoading = $state(false);
  let guidePreviewError = $state<string | null>(null);
  let previewSurfaceReady = $state(false);
  let pendingPreviewId = $state<string | null>(null);

  let infoBannerTimer: ReturnType<typeof setTimeout> | null = null;
  let statsTimer: ReturnType<typeof setInterval> | null = null;
  let previewDebounce: ReturnType<typeof setTimeout> | null = null;
  let previewRequestId = 0;
  let boundsDebounce: ReturnType<typeof setTimeout> | null = null;
  let lastPreviewBounds: {
    clientX: number;
    clientY: number;
    width: number;
    height: number;
    windowWidth: number;
    windowHeight: number;
  } | null = null;
  let platform = $state("unknown");

  const mobilePlatform = $derived(platform === "android" || platform === "ios");
  const guidePreviewSupported = $derived(!mobilePlatform);

  const guidePreviewActive = $derived(
    guidePreviewSupported && view === "guide" && (previewMode || guidePreviewLoading),
  );

  const guidePreviewStatus = $derived.by((): "idle" | "waiting" | "loading" | "live" | "error" | "unavailable" => {
    if (engineName === "stub") return "unavailable";
    if (guidePreviewError) return "error";
    if (guidePreviewLoading) return "loading";
    if (previewMode) return "live";
    if (pendingPreviewId) return "waiting";
    return "idle";
  });

  const PAGE_SIZE = 100;

  function applyFontScale(scale: number) {
    document.documentElement.style.fontSize = `${scale}%`;
  }

  function showChannelInfoBanner() {
    showInfoBanner = true;
    if (infoBannerTimer) clearTimeout(infoBannerTimer);
    infoBannerTimer = setTimeout(() => {
      showInfoBanner = false;
    }, 10_000);
  }

  function startStatsPolling() {
    stopStatsPolling();
    const poll = async () => {
      if (!playerVisible || previewMode) return;
      try {
        streamStats = await getStreamStats();
        if (streamStats?.error) {
          playbackError = streamStats.error;
        }
      } catch {
        /* ignore */
      }
    };
    poll();
    statsTimer = setInterval(poll, 1500);
  }

  function stopStatsPolling() {
    if (statsTimer) {
      clearInterval(statsTimer);
      statsTimer = null;
    }
    streamStats = null;
  }

  async function loadChannelPage(offset: number, replace: boolean) {
    loading = true;
    try {
      const page = await listChannels({
        group: selectedGroup ?? undefined,
        search: search || undefined,
        favoritesOnly,
        offset,
        limit: PAGE_SIZE,
      });
      channels = replace ? page.channels : [...channels, ...page.channels];
      total = page.total;
      await refreshNowNext(page.channels.map((c) => c.id));
      if (replace && channels.length > 0 && !highlightedChannel) {
        highlightedChannel = channels[0];
      }
    } finally {
      loading = false;
    }
  }

  async function bootstrap() {
    platform = await getPlatform();
    const settings = await getSettings();
    hiddenGroups = settings.hidden_groups ?? [];
    volume = settings.volume ?? 100;
    fontScale = settings.font_scale ?? 100;
    browseScroll = settings.browse_scroll ?? 0;
    selectedGroup = settings.last_group ?? null;
    applyFontScale(fontScale);
    favorites = new Set(await listFavorites());
    await refreshGroups();
    await loadChannelPage(0, true);
    await syncPlaybackState();

    if (settings.resume_on_startup !== false && settings.last_channel) {
      const ch = await getChannel(settings.last_channel);
      if (ch) await startPlayback(ch);
    }
  }

  async function refreshGroups() {
    groups = await listGroups();
  }

  async function refreshNowNext(channelIds: string[]) {
    if (channelIds.length === 0) return;
    const batch = await getNowNext(channelIds);
    const next = new Map(nowNext);
    for (const [id, nn] of batch) {
      next.set(id, nn);
    }
    nowNext = next;
  }

  function defaultPreviewBounds() {
    const width = 320;
    const height = Math.round((width * 9) / 16);
    return {
      clientX: window.innerWidth - 16 - width,
      clientY: 12,
      width,
      height,
      windowWidth: window.innerWidth,
      windowHeight: window.innerHeight,
    };
  }

  async function loadGuideChannels() {
    const page = await listChannels({ offset: 0, limit: 200 });
    guideChannels = page.channels;
    await refreshNowNext(page.channels.map((c) => c.id));
    previewSurfaceReady = false;
    if (guideChannels.length > 0) {
      const first = guideChannels[0];
      guideHighlight = first;
      pendingPreviewId = first.id;
    }
  }

  async function ensurePreviewSurface() {
    if (previewSurfaceReady) return;
    await handlePreviewBounds(defaultPreviewBounds());
  }

  async function startGuidePreview(channelId: string) {
    const requestId = ++previewRequestId;
    guidePreviewLoading = true;
    guidePreviewError = null;
    playbackError = null;
    pendingPreviewId = null;
    try {
      await ensurePreviewSurface();
      if (requestId !== previewRequestId) return;
      await previewChannel(channelId);
      if (requestId !== previewRequestId) return;
      previewMode = true;
      await syncPlaybackState();
      if (lastPreviewBounds) {
        await setPreviewBounds(lastPreviewBounds);
      }
    } catch (e) {
      if (requestId === previewRequestId) {
        guidePreviewError = String(e);
        console.error("guide preview:", e);
      }
    } finally {
      if (requestId === previewRequestId) {
        guidePreviewLoading = false;
      }
    }
  }

  function kickGuidePreviewIfReady() {
    if (view !== "guide") return;
    const id = pendingPreviewId ?? guideHighlight?.id;
    if (!id || guidePreviewLoading) return;
    void startGuidePreview(id);
  }

  async function handlePreviewBounds(bounds: {
    clientX: number;
    clientY: number;
    width: number;
    height: number;
    windowWidth: number;
    windowHeight: number;
  }) {
    if (view !== "guide") return;
    lastPreviewBounds = bounds;

    const apply = async () => {
      try {
        await setPreviewBounds(bounds);
        previewSurfaceReady = true;
      } catch (e) {
        guidePreviewError = String(e);
        console.error("preview surface:", e);
      }
    };

    if (previewMode) {
      if (boundsDebounce) clearTimeout(boundsDebounce);
      boundsDebounce = setTimeout(() => void apply(), 100);
      return;
    }

    await apply();
  }

  async function stopGuidePreview() {
    if (previewDebounce) {
      clearTimeout(previewDebounce);
      previewDebounce = null;
    }
    previewRequestId++;
    pendingPreviewId = null;
    guidePreviewLoading = false;
    guidePreviewError = null;
    previewSurfaceReady = false;
    try {
      await hidePreviewSurface();
    } catch {
      /* ignore */
    }
    previewMode = false;
    playbackError = null;
    await syncPlaybackState();
  }

  function scheduleGuidePreview(channelId: string) {
    if (!guidePreviewSupported) return;
    if (previewDebounce) clearTimeout(previewDebounce);
    previewDebounce = setTimeout(() => {
      void startGuidePreview(channelId);
    }, 300);
  }

  function handleGuideHighlight(channel: Channel) {
    guideHighlight = channel;
    scheduleGuidePreview(channel.id);
    void refreshNowNext([channel.id]);
  }

  async function handleNavigate(next: View) {
    if (view === "guide" && next !== "guide") {
      await stopGuidePreview();
    }

    if (next === "guide") {
      playerVisible = false;
      stopStatsPolling();
      guidePreviewError = null;
      if (playing && !previewMode) {
        await stopPlayback();
      }
      await syncPlaybackState();
      view = next;
      await loadGuideChannels();
      if (guidePreviewSupported) {
        await handlePreviewBounds(defaultPreviewBounds());
        kickGuidePreviewIfReady();
      }
      return;
    }

    view = next;
  }

  function handleSearch(query: string) {
    search = query;
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => {
      loadChannelPage(0, true);
    }, 250);
  }

  async function handleSelectGroup(group: string | null) {
    selectedGroup = group;
    highlightedChannel = null;
    await saveSettings({ last_group: group ?? "" });
    loadChannelPage(0, true);
  }

  async function syncPlaybackState() {
    const ps = await getPlaybackState();
    playing = ps.playing;
    paused = ps.paused;
    previewMode = ps.preview_mode;
    engineName = ps.engine_name;
    videoAvailable = ps.video_available;
    playbackError = ps.error;
  }

  async function startPlayback(channel: Channel) {
    if (previewMode) {
      await stopGuidePreview();
    }
    selectedChannel = channel;
    highlightedChannel = channel;
    playerVisible = true;
    view = "player";
    playbackError = null;
    try {
      await playChannel(channel.id);
      await syncPlaybackState();
      await refreshNowNext([channel.id]);
      await saveSettings({ last_channel: channel.id, volume });
      showChannelInfoBanner();
      startStatsPolling();
    } catch (e) {
      playbackError = String(e);
      console.error(e);
    }
  }

  async function handleBrowseHighlight(channel: Channel) {
    highlightedChannel = channel;
  }

  async function handleBrowsePlay(channel: Channel) {
    await startPlayback(channel);
  }

  async function handleGuidePlay(channel: Channel) {
    await stopGuidePreview();
    await startPlayback(channel);
  }

  async function handleChannelDelta(delta: number) {
    try {
      const next = await zapChannel(delta);
      if (!next) return;
      selectedChannel = next;
      await syncPlaybackState();
      await refreshNowNext([next.id]);
      await saveSettings({ last_channel: next.id, volume });
      showChannelInfoBanner();
      startStatsPolling();
    } catch (e) {
      playbackError = String(e);
      console.error(e);
    }
  }

  function handleLoadMore() {
    loadChannelPage(channels.length, false);
  }

  async function handleClosePlayer() {
    stopStatsPolling();
    if (infoBannerTimer) clearTimeout(infoBannerTimer);
    showInfoBanner = false;
    await stopPlayback();
    await syncPlaybackState();
    playerVisible = false;
    view = "browse";
  }

  async function handleTogglePlay() {
    if (playing && !paused) {
      await pausePlayback();
      paused = true;
      playing = false;
    } else {
      await resumePlayback();
      paused = false;
      playing = true;
    }
  }

  async function handleVolumeChange(v: number) {
    volume = Math.max(0, Math.min(100, v));
    await setVolume(volume);
    await saveSettings({ volume });
  }

  async function handleVolumeDelta(delta: number) {
    await handleVolumeChange(volume + delta);
  }

  async function handleToggleMute() {
    muted = !muted;
    await setMuted(muted);
  }

  async function handleReload() {
    if (selectedChannel) {
      await startPlayback(selectedChannel);
    }
  }

  function handleSourcesUpdated() {
    refreshGroups();
    highlightedChannel = null;
    loadChannelPage(0, true);
    hiddenGroups = [];
    getSettings().then((s) => {
      hiddenGroups = s.hidden_groups ?? [];
    });
  }

  function handleFontScaleChange(scale: number) {
    fontScale = scale;
    applyFontScale(scale);
  }

  function handleFavoritesOnlyChange(value: boolean) {
    favoritesOnly = value;
    highlightedChannel = null;
    loadChannelPage(0, true);
  }

  function handleBrowseScrollChange(scrollTop: number) {
    browseScroll = scrollTop;
    saveSettings({ browse_scroll: scrollTop });
  }

  $effect(() => {
    bootstrap();
    return () => {
      stopStatsPolling();
      if (infoBannerTimer) clearTimeout(infoBannerTimer);
      if (previewDebounce) clearTimeout(previewDebounce);
    };
  });
</script>

<div
  class="app"
  class:player-mode={playerVisible && !previewMode}
  class:guide-preview-mode={view === "guide" && (guidePreviewActive || previewSurfaceReady)}
>
  {#if view !== "player"}
    <NavBar current={view} onNavigate={handleNavigate} />
  {/if}

  {#if view !== "player"}
    <main class="main">
      {#if view === "browse"}
        <GroupSidebar
          {groups}
          {selectedGroup}
          {search}
          {hiddenGroups}
          onSelectGroup={handleSelectGroup}
          onSearch={handleSearch}
        />
        <ChannelBrowse
          {channels}
          {total}
          {favorites}
          {nowNext}
          highlightedChannelId={highlightedChannel?.id ?? null}
          {favoritesOnly}
          {loading}
          initialScrollTop={browseScroll}
          onHighlight={handleBrowseHighlight}
          onPlay={handleBrowsePlay}
          onLoadMore={handleLoadMore}
          onFavoritesChange={(ids) => (favorites = ids)}
          onFavoritesOnlyChange={handleFavoritesOnlyChange}
          onScrollChange={handleBrowseScrollChange}
        />
      {:else if view === "guide"}
        <EpgPanel
          channels={guideChannels}
          {nowNext}
          highlightedId={guideHighlight?.id ?? null}
          previewActive={view === "guide"}
          onHighlight={handleGuideHighlight}
          onPlay={handleGuidePlay}
        />
      {:else if view === "settings"}
        <Settings
          onSourcesUpdated={handleSourcesUpdated}
          onFontScaleChange={handleFontScaleChange}
        />
      {/if}
    </main>
  {/if}

  {#if view === "guide"}
    <GuidePreviewPip
      channel={guideHighlight}
      nowNext={guideHighlight ? (nowNext.get(guideHighlight.id) ?? null) : null}
      visible={true}
      status={guidePreviewStatus}
      error={guidePreviewError}
      onBoundsChange={handlePreviewBounds}
    />
  {/if}

  <PlayerOverlay
    channel={guidePreviewActive ? guideHighlight : selectedChannel}
    nowNext={
      guidePreviewActive && guideHighlight
        ? (nowNext.get(guideHighlight.id) ?? null)
        : selectedChannel
          ? (nowNext.get(selectedChannel.id) ?? null)
          : null
    }
    {streamStats}
    playing={guidePreviewActive ? previewMode : playing}
    {paused}
    volume={guidePreviewActive ? 0 : volume}
    muted={guidePreviewActive ? true : muted}
    visible={(playerVisible && view === "player") || guidePreviewActive}
    guidePreview={guidePreviewActive}
    {engineName}
    {videoAvailable}
    playbackError={guidePreviewActive ? guidePreviewError : playbackError}
    showInfoBanner={guidePreviewActive ? false : showInfoBanner}
    onClose={handleClosePlayer}
    onTogglePlay={handleTogglePlay}
    onStop={handleClosePlayer}
    onVolumeChange={handleVolumeChange}
    onVolumeDelta={handleVolumeDelta}
    onToggleMute={handleToggleMute}
    onReload={handleReload}
    onChannelDelta={handleChannelDelta}
  />
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
    overflow: hidden;
    background: var(--bg-primary);
  }

  .app.player-mode,
  .app.guide-preview-mode {
    background: transparent;
  }

  .app.guide-preview-mode .main {
    background: transparent;
  }

  .app.guide-preview-mode :global(.nav),
  .app.guide-preview-mode .main {
    position: relative;
    z-index: 110;
  }

  .main {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-width: 0;
    background: var(--bg-primary);
  }
</style>
