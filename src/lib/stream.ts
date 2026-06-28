import type { StreamStats } from "./types";

export type QualityBadge = "4K" | "HD" | "SD" | null;

export function qualityBadge(stats: StreamStats | null): QualityBadge {
  const h = stats?.height;
  if (!h) return null;
  if (h >= 2160) return "4K";
  if (h >= 720) return "HD";
  return "SD";
}

export function formatBitrate(kbps: number | null | undefined): string | null {
  if (kbps == null || kbps <= 0) return null;
  if (kbps >= 1000) return `${(kbps / 1000).toFixed(1)} Mbps`;
  return `${Math.round(kbps)} kbps`;
}

export function formatResolution(stats: StreamStats | null): string | null {
  if (!stats?.width || !stats?.height) return null;
  return `${stats.width}×${stats.height}`;
}

export function formatFps(fps: number | null | undefined): string | null {
  if (fps == null || fps <= 0) return null;
  return `${fps.toFixed(fps >= 10 ? 0 : 1)} fps`;
}
