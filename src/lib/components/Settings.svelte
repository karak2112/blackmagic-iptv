<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    loadPlaylist,
    loadEpg,
    listSources,
    getSettings,
    saveSettings,
  } from "$lib/api";
  import type { Source } from "$lib/types";

  interface Props {
    onSourcesUpdated: () => void;
    onFontScaleChange: (scale: number) => void;
  }

  let { onSourcesUpdated, onFontScaleChange }: Props = $props();

  let sources = $state<Source[]>([]);
  let playlistName = $state("My Playlist");
  let playlistUrl = $state("");
  let epgUrl = $state("");
  let hiddenGroupsInput = $state("");
  let fontScale = $state(100);
  let resumeOnStartup = $state(true);
  let status = $state("");
  let loading = $state(false);
  let appVersion = $state("");

  const FONT_SCALES = [
    { label: "Small", value: 90 },
    { label: "Default", value: 100 },
    { label: "Large", value: 112 },
    { label: "Extra large", value: 125 },
  ];

  export async function refresh() {
    sources = await listSources();
    const settings = await getSettings();
    hiddenGroupsInput = (settings.hidden_groups ?? []).join(", ");
    fontScale = settings.font_scale ?? 100;
    resumeOnStartup = settings.resume_on_startup !== false;
  }

  $effect(() => {
    refresh();
    getVersion().then((v) => (appVersion = v));
  });

  async function pickM3uFile() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "M3U Playlist", extensions: ["m3u", "m3u8"] }],
    });
    if (!selected || Array.isArray(selected)) return;
    await loadFromPath(selected);
  }

  async function pickXmltvFile() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "XMLTV Guide", extensions: ["xml", "xmltv"] }],
    });
    if (!selected || Array.isArray(selected)) return;
    await loadEpgFromPath(selected);
  }

  async function loadFromPath(path: string) {
    loading = true;
    status = "Loading playlist...";
    try {
      const existing = sources.find(
        (s) => s.path_or_url === path && s.source_type.includes("m3u"),
      );
      const summary = await loadPlaylist({
        name: playlistName,
        localPath: path,
        sourceId: existing?.id,
      });
      status = `Loaded ${summary.channel_count} channels in ${summary.group_count} groups.`;
      await refresh();
      onSourcesUpdated();
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function loadFromUrl() {
    if (!playlistUrl.trim()) return;
    loading = true;
    status = "Fetching playlist...";
    try {
      const summary = await loadPlaylist({
        name: playlistName,
        remoteUrl: playlistUrl.trim(),
      });
      status = `Loaded ${summary.channel_count} channels in ${summary.group_count} groups.`;
      await refresh();
      onSourcesUpdated();
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function loadEpgFromPath(path: string) {
    loading = true;
    status = "Loading EPG...";
    try {
      const summary = await loadEpg({ localPath: path });
      status = `EPG: ${summary.programme_count} programmes, ${summary.matched_count} channels matched.`;
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function loadEpgFromUrl() {
    if (!epgUrl.trim()) return;
    loading = true;
    status = "Fetching EPG...";
    try {
      const summary = await loadEpg({ remoteUrl: epgUrl.trim() });
      status = `EPG: ${summary.programme_count} programmes, ${summary.matched_count} channels matched.`;
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function saveHiddenGroups() {
    const groups = hiddenGroupsInput
      .split(",")
      .map((g) => g.trim())
      .filter(Boolean);
    await saveSettings({ hidden_groups: groups });
    status = "Preferences saved.";
  }

  async function saveResumeOnStartup() {
    await saveSettings({ resume_on_startup: resumeOnStartup });
    status = "Preferences saved.";
  }

  async function saveFontScale() {
    await saveSettings({ font_scale: fontScale });
    onFontScaleChange(fontScale);
    status = "Preferences saved.";
  }
</script>

<div class="settings" role="region" aria-label="Settings">
  <header>
    <h2>Settings</h2>
    <p class="subtitle">Manage playlists, guides, and preferences</p>
  </header>

  <section>
    <h3>Playlist (M3U)</h3>
    <label>
      <span>Name</span>
      <input bind:value={playlistName} />
    </label>
    <label>
      <span>Remote URL</span>
      <input bind:value={playlistUrl} placeholder="https://example.com/playlist.m3u" />
    </label>
    <div class="actions">
      <button onclick={pickM3uFile} disabled={loading}>Open local file...</button>
      <button onclick={loadFromUrl} disabled={loading || !playlistUrl.trim()}>
        Load from URL
      </button>
    </div>
  </section>

  <section>
    <h3>EPG (XMLTV)</h3>
    <label>
      <span>Remote URL</span>
      <input bind:value={epgUrl} placeholder="https://example.com/epg.xml" />
    </label>
    <div class="actions">
      <button onclick={pickXmltvFile} disabled={loading}>Open local file...</button>
      <button onclick={loadEpgFromUrl} disabled={loading || !epgUrl.trim()}>
        Load from URL
      </button>
    </div>
  </section>

  <section>
    <h3>Preferences</h3>
    <label>
      <span>UI font size</span>
      <select bind:value={fontScale} onchange={saveFontScale}>
        {#each FONT_SCALES as opt}
          <option value={opt.value}>{opt.label} ({opt.value}%)</option>
        {/each}
      </select>
    </label>
    <label class="checkbox-row">
      <input
        type="checkbox"
        bind:checked={resumeOnStartup}
        onchange={saveResumeOnStartup}
      />
      <span>Resume last channel on startup</span>
    </label>
    <label>
      <span>Hidden groups (comma-separated)</span>
      <input bind:value={hiddenGroupsInput} placeholder="Adult, XXX" />
    </label>
    <div class="actions">
      <button onclick={saveHiddenGroups}>Save preferences</button>
    </div>
  </section>

  {#if sources.length > 0}
    <section>
      <h3>Saved sources</h3>
      <ul class="source-list">
        {#each sources as source}
          <li>
            <strong>{source.name}</strong>
            <span class="type">{source.source_type}</span>
            <span class="path">{source.path_or_url}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <section class="about">
    <h3>About</h3>
    <p class="about-title">Black Magic IPTV</p>
    {#if appVersion}
      <p class="about-meta">Version {appVersion}</p>
    {/if}
    <p class="about-publisher">
      Published by <strong>BlackMagicSoftware.net</strong>
    </p>
    <p class="about-copy">
      Lightweight open-source IPTV player built with Tauri, Rust, Svelte, and libmpv.
    </p>
    <p class="about-copy muted">
      Playlists and guides are loaded locally; stream URLs are fetched by the app on your behalf.
    </p>
  </section>

  {#if status}
    <p class="status" role="status">{status}</p>
  {/if}
</div>

<style>
  .settings {
    flex: 1;
    overflow-y: auto;
    padding: 20px 28px;
    max-width: 720px;
  }

  header {
    margin-bottom: 24px;
  }

  h2 {
    margin: 0 0 4px;
    font-size: 20px;
  }

  .subtitle {
    margin: 0;
    color: var(--text-muted);
    font-size: 14px;
  }

  section {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 20px;
    margin-bottom: 16px;
  }

  h3 {
    margin: 0 0 16px;
    font-size: 15px;
    font-weight: 600;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  input,
  select {
    padding: 10px 12px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .checkbox-row {
    flex-direction: row;
    align-items: center;
    gap: 10px;
  }

  .checkbox-row input {
    width: auto;
    padding: 0;
    accent-color: var(--accent);
  }

  .actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .actions button {
    padding: 10px 16px;
    background: var(--accent);
    color: white;
    border-radius: var(--radius);
    font-weight: 500;
  }

  .actions button:disabled {
    opacity: 0.5;
  }

  .source-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .source-list li {
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .type {
    font-size: 11px;
    color: var(--accent);
    text-transform: uppercase;
  }

  .path {
    font-size: 12px;
    color: var(--text-muted);
    word-break: break-all;
  }

  .about-title {
    margin: 0 0 4px;
    font-size: 18px;
    font-weight: 700;
  }

  .about-meta,
  .about-publisher,
  .about-copy {
    margin: 0 0 8px;
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .about-copy.muted {
    color: var(--text-muted);
    font-size: 13px;
  }

  .status {
    padding: 12px 16px;
    background: var(--bg-elevated);
    border-radius: var(--radius);
    font-size: 14px;
    color: var(--text-secondary);
  }
</style>
