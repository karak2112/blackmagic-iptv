import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  Channel,
  ChannelListPage,
  EpgSummary,
  GroupInfo,
  NowNext,
  PlaybackState,
  PlaylistSummary,
  Source,
  StreamStats,
} from "./types";

export async function loadPlaylist(opts: {
  name: string;
  localPath?: string;
  remoteUrl?: string;
  sourceId?: string;
}): Promise<PlaylistSummary> {
  return invoke("load_playlist", {
    name: opts.name,
    localPath: opts.localPath ?? null,
    remoteUrl: opts.remoteUrl ?? null,
    sourceId: opts.sourceId ?? null,
  });
}

export async function loadEpg(opts: {
  localPath?: string;
  remoteUrl?: string;
}): Promise<EpgSummary> {
  return invoke("load_epg", {
    localPath: opts.localPath ?? null,
    remoteUrl: opts.remoteUrl ?? null,
  });
}

export async function listGroups(sourceId?: string): Promise<GroupInfo[]> {
  return invoke("list_groups", { sourceId: sourceId ?? null });
}

export async function listChannels(opts: {
  sourceId?: string;
  group?: string;
  search?: string;
  favoritesOnly?: boolean;
  offset?: number;
  limit?: number;
}): Promise<ChannelListPage> {
  return invoke("list_channels", {
    sourceId: opts.sourceId ?? null,
    group: opts.group ?? null,
    search: opts.search ?? null,
    favoritesOnly: opts.favoritesOnly ?? false,
    offset: opts.offset ?? 0,
    limit: opts.limit ?? 100,
  });
}

export async function getNowNext(
  channelIds: string[],
): Promise<[string, NowNext][]> {
  return invoke("get_now_next", { channelIds });
}

export async function playChannel(channelId: string): Promise<void> {
  return invoke("play_channel", { channelId });
}

export async function previewChannel(channelId: string): Promise<void> {
  return invoke("preview_channel", { channelId });
}

export async function setPreviewBounds(bounds: {
  clientX: number;
  clientY: number;
  width: number;
  height: number;
  windowWidth: number;
  windowHeight: number;
}): Promise<void> {
  return invoke("set_preview_bounds", bounds);
}

export async function hidePreviewSurface(): Promise<void> {
  return invoke("hide_preview_surface");
}

export async function getChannel(channelId: string): Promise<Channel | null> {
  return invoke("get_channel", { channelId });
}

export async function stopPlayback(): Promise<void> {
  return invoke("stop_playback");
}

export async function pausePlayback(): Promise<void> {
  return invoke("pause_playback");
}

export async function resumePlayback(): Promise<void> {
  return invoke("resume_playback");
}

export async function setVolume(volume: number): Promise<void> {
  return invoke("set_volume", { volume });
}

export async function setMuted(muted: boolean): Promise<void> {
  return invoke("set_muted", { muted });
}

export async function getPlaybackState(): Promise<PlaybackState> {
  return invoke("get_playback_state");
}

export async function toggleFavorite(channelId: string): Promise<boolean> {
  return invoke("toggle_favorite", { channelId });
}

export async function listFavorites(): Promise<string[]> {
  return invoke("list_favorites");
}

export async function getStreamStats(): Promise<StreamStats> {
  return invoke("get_stream_stats");
}

export async function zapChannel(delta: number): Promise<Channel | null> {
  return invoke("zap_channel", { delta });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: Partial<AppSettings>): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function listSources(): Promise<Source[]> {
  return invoke("list_sources");
}
