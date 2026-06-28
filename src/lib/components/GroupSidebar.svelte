<script lang="ts">
  import type { GroupInfo } from "$lib/types";

  interface Props {
    groups: GroupInfo[];
    selectedGroup: string | null;
    search: string;
    hiddenGroups: string[];
    onSelectGroup: (group: string | null) => void;
    onSearch: (query: string) => void;
  }

  let {
    groups,
    selectedGroup,
    search,
    hiddenGroups,
    onSelectGroup,
    onSearch,
  }: Props = $props();

  let visibleGroups = $derived(
    groups.filter((g) => !hiddenGroups.includes(g.name)),
  );
</script>

<aside class="sidebar" aria-label="Channel groups">
  <div class="search-wrap">
    <input
      type="search"
      placeholder="Search channels..."
      value={search}
      oninput={(e) => onSearch(e.currentTarget.value)}
      aria-label="Search channels"
    />
  </div>

  <button
    class="group-item"
    class:active={selectedGroup === null}
    onclick={() => onSelectGroup(null)}
  >
    All Channels
  </button>

  <ul class="group-list" role="list">
    {#each visibleGroups as group}
      <li>
        <button
          class="group-item"
          class:active={selectedGroup === group.name}
          onclick={() => onSelectGroup(group.name)}
        >
          <span class="name">{group.name}</span>
          <span class="count">{group.channel_count}</span>
        </button>
      </li>
    {/each}
  </ul>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
  }

  .search-wrap {
    padding: 12px;
    border-bottom: 1px solid var(--border);
  }

  input {
    width: 100%;
    padding: 10px 12px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 14px;
  }

  input::placeholder {
    color: var(--text-muted);
  }

  .group-list {
    list-style: none;
    margin: 0;
    padding: 8px;
    overflow-y: auto;
    flex: 1;
  }

  .group-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 10px 12px;
    border-radius: var(--radius);
    text-align: left;
    color: var(--text-secondary);
    transition: background 0.12s, color 0.12s;
  }

  .group-item:hover,
  .group-item.active {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .group-item.active {
    border-left: 3px solid var(--accent);
    padding-left: 9px;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .count {
    font-size: 12px;
    color: var(--text-muted);
    margin-left: 8px;
    flex-shrink: 0;
  }
</style>
