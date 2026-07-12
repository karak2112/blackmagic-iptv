<script lang="ts">
  import type { Channel, NowNext } from "$lib/types";
  import { toggleFavorite } from "$lib/api";
  import { channelInitial } from "$lib/channelDisplay";

  interface Props {
    channels: Channel[];
    total: number;
    favorites: Set<string>;
    nowNext: Map<string, NowNext>;
    highlightedChannelId: string | null;
    favoritesOnly: boolean;
    loading: boolean;
    initialScrollTop: number;
    onHighlight: (channel: Channel) => void;
    onPlay: (channel: Channel) => void;
    onLoadMore: () => void;
    onFavoritesChange: (ids: Set<string>) => void;
    onFavoritesOnlyChange: (value: boolean) => void;
    onScrollChange: (scrollTop: number) => void;
  }

  let {
    channels,
    total,
    favorites,
    nowNext,
    highlightedChannelId,
    favoritesOnly,
    loading,
    initialScrollTop,
    onHighlight,
    onPlay,
    onLoadMore,
    onFavoritesChange,
    onFavoritesOnlyChange,
    onScrollChange,
  }: Props = $props();

  let listEl = $state<HTMLUListElement | null>(null);
  let scrollSaveTimer: ReturnType<typeof setTimeout> | null = null;
  let restoredScroll = $state(false);
  let failedLogos = $state(new Set<string>());

  function markLogoFailed(channelId: string) {
    if (failedLogos.has(channelId)) return;
    failedLogos = new Set([...failedLogos, channelId]);
  }

  async function handleFavorite(e: Event, channelId: string) {
    e.stopPropagation();
    const added = await toggleFavorite(channelId);
    const next = new Set(favorites);
    if (added) next.add(channelId);
    else next.delete(channelId);
    onFavoritesChange(next);
  }

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

  function handleScroll() {
    if (!listEl) return;
    if (scrollSaveTimer) clearTimeout(scrollSaveTimer);
    scrollSaveTimer = setTimeout(() => {
      onScrollChange(listEl!.scrollTop);
    }, 300);
  }

  $effect(() => {
    if (listEl && !restoredScroll && initialScrollTop > 0) {
      listEl.scrollTop = initialScrollTop;
      restoredScroll = true;
    }
  });
</script>

<div class="browse" role="region" aria-label="Channel list">
  <header class="header">
    <h2>Channels</h2>
    <span class="meta">{total.toLocaleString()} total</span>
    <label class="fav-filter">
      <input
        type="checkbox"
        checked={favoritesOnly}
        onchange={(e) => onFavoritesOnlyChange(e.currentTarget.checked)}
      />
      Favorites only
    </label>
  </header>

  <ul
    class="channel-list"
    role="listbox"
    aria-label="Channels"
    bind:this={listEl}
    onscroll={handleScroll}
  >
    {#each channels as channel (channel.id)}
      {@const nn = nowNext.get(channel.id)}
      <li role="option" aria-selected={highlightedChannelId === channel.id}>
        <div
          class="channel-card"
          class:selected={highlightedChannelId === channel.id}
          role="button"
          tabindex="0"
          onclick={() => onHighlight(channel)}
          ondblclick={() => onPlay(channel)}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              onPlay(channel);
            }
          }}
        >
          <div class="logo-wrap">
            {#if channel.logo_url && !failedLogos.has(channel.id)}
              <img
                src={channel.logo_url}
                alt=""
                loading="lazy"
                onerror={() => markLogoFailed(channel.id)}
              />
            {:else}
              <span class="logo-fallback">{channelInitial(channel.name)}</span>
            {/if}
          </div>
          <div class="info">
            <div class="title-row">
              <span class="name">{channel.name}</span>
              {#if channel.group}
                <span class="group">{channel.group}</span>
              {/if}
            </div>
            {#if nn?.now}
              <div class="epg-now">
                <span class="live-dot" aria-hidden="true"></span>
                {nn.now.title}
                {#if nn.next}
                  <span class="epg-next">
                    · Next {formatTime(nn.next.start)}: {nn.next.title}
                  </span>
                {/if}
              </div>
            {:else}
              <div class="epg-now muted">No guide data</div>
            {/if}
          </div>
          <button
            class="fav-btn"
            class:active={favorites.has(channel.id)}
            onclick={(e) => handleFavorite(e, channel.id)}
            aria-label={favorites.has(channel.id) ? "Remove favorite" : "Add favorite"}
          >
            {favorites.has(channel.id) ? "★" : "☆"}
          </button>
        </div>
      </li>
    {/each}
  </ul>

  {#if channels.length < total}
    <div class="load-more">
      <button onclick={onLoadMore} disabled={loading}>
        {loading ? "Loading..." : "Load more"}
      </button>
    </div>
  {/if}

  <p class="browse-hint">Click to select · Double-click or Enter to watch</p>
</div>

<style>
  .browse {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px 12px;
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
  }

  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .meta {
    color: var(--text-muted);
    font-size: 13px;
  }

  .fav-filter {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .fav-filter input {
    accent-color: var(--accent);
  }

  .channel-list {
    list-style: none;
    margin: 0;
    padding: 8px 12px;
    overflow-y: auto;
    flex: 1;
  }

  .channel-card {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    padding: 12px 14px;
    border-radius: var(--radius-lg);
    text-align: left;
    transition: background 0.12s;
    cursor: pointer;
  }

  .channel-card:hover,
  .channel-card.selected {
    background: var(--bg-elevated);
  }

  .channel-card.selected {
    box-shadow: inset 0 0 0 1px var(--accent-dim);
  }

  .logo-wrap {
    width: 48px;
    height: 48px;
    border-radius: var(--radius);
    background: var(--bg-hover);
    overflow: hidden;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo-wrap img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .logo-fallback {
    font-size: 20px;
    font-weight: 700;
    color: var(--accent);
  }

  .info {
    flex: 1;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .name {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .group {
    font-size: 11px;
    color: var(--text-muted);
    background: var(--bg-hover);
    padding: 2px 6px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .epg-now {
    font-size: 13px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .epg-now.muted {
    color: var(--text-muted);
  }

  .live-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--success);
    margin-right: 6px;
    vertical-align: middle;
  }

  .epg-next {
    color: var(--text-muted);
  }

  .fav-btn {
    font-size: 18px;
    padding: 8px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .fav-btn.active {
    color: #fbbf24;
  }

  .load-more {
    padding: 12px;
    text-align: center;
    border-top: 1px solid var(--border);
  }

  .load-more button {
    padding: 10px 24px;
    background: var(--bg-elevated);
    border-radius: var(--radius);
    color: var(--text-primary);
  }

  .load-more button:disabled {
    opacity: 0.5;
  }

  .browse-hint {
    margin: 0;
    padding: 6px 20px 10px;
    font-size: 11px;
    color: var(--text-muted);
    border-top: 1px solid var(--border);
  }
</style>
