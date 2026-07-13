export interface Channel {
  id: string;
  source_id: string;
  name: string;
  group: string | null;
  logo_url: string | null;
  stream_url: string;
  tvg_id: string | null;
  tvg_name: string | null;
}

export interface Programme {
  channel_epg_id: string;
  start: string;
  stop: string;
  title: string;
  description: string | null;
  category: string | null;
}

export interface NowNext {
  now: Programme | null;
  next: Programme | null;
}

export interface GroupInfo {
  name: string;
  channel_count: number;
}

export interface ChannelListPage {
  channels: Channel[];
  total: number;
  offset: number;
  limit: number;
}

export interface Source {
  id: string;
  name: string;
  source_type: string;
  path_or_url: string;
  last_loaded: string | null;
}

export interface PlaylistSummary {
  source_id: string;
  channel_count: number;
  group_count: number;
}

export interface EpgSummary {
  channel_count: number;
  programme_count: number;
  matched_count: number;
}

export interface RecordingStatus {
  active: boolean;
  starting: boolean;
  stopping: boolean;
  path: string | null;
  available: boolean;
}

export interface PlaybackState {
  channel_id: string | null;
  playing: boolean;
  paused: boolean;
  volume: number;
  muted: boolean;
  fullscreen: boolean;
  error: string | null;
  engine_name: string;
  video_available: boolean;
  preview_mode: boolean;
}

export interface StreamStats {
  width: number | null;
  height: number | null;
  fps: number | null;
  video_bitrate_kbps: number | null;
  audio_bitrate_kbps: number | null;
  video_codec: string | null;
  audio_codec: string | null;
  error: string | null;
}

export interface AppSettings {
  last_channel: string | null;
  hidden_groups: string[];
  volume: number;
  font_scale: number;
  last_group: string | null;
  browse_scroll: number;
  resume_on_startup: boolean;
}

export type View = "browse" | "guide" | "settings" | "player";
