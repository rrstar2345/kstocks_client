<script lang="ts">
  import { onMount } from "svelte";
  import { getAllIndexSnapshots } from "$lib/api/tauri";
  import type { IndexSnapshot } from "$lib/types";

  let snapshots = $state<IndexSnapshot[]>([]);
  let loading = $state(true);

  const POLL_MS = 5_000;

  async function load() {
    try {
      snapshots = await getAllIndexSnapshots();
    } catch {
      // Local data not available yet (e.g. streamers still starting up);
      // keep showing whatever we last had.
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    load();
    const timer = setInterval(load, POLL_MS);
    return () => clearInterval(timer);
  });
</script>

<div class="watchlist card">
  <div class="watchlist-header">
    <h2>Watchlist</h2>
  </div>

  {#if loading && snapshots.length === 0}
    <p class="muted pad">Loading…</p>
  {:else if snapshots.length === 0}
    <p class="muted pad">No live data yet.</p>
  {:else}
    <ul class="watchlist-items">
      {#each snapshots as s (s.index_name)}
        <li class="watchlist-item">
          <span class="symbol">{s.index_name}</span>
          <span class="price">{s.current_price?.toFixed(2) ?? "—"}</span>
          <span class={(s.change ?? 0) >= 0 ? "text-positive" : "text-negative"}>
            {(s.change ?? 0) >= 0 ? "+" : ""}{s.change?.toFixed(2) ?? "0.00"}
          </span>
          <span class={(s.per_change ?? 0) >= 0 ? "text-positive" : "text-negative"}>
            {(s.per_change ?? 0) >= 0 ? "+" : ""}{s.per_change?.toFixed(2) ?? "0.00"}%
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .watchlist {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .watchlist-header {
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--color-border);
  }

  .pad {
    padding: var(--space-4);
  }

  .watchlist-items {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
  }

  .watchlist-item {
    display: grid;
    grid-template-columns: 1fr auto auto auto;
    gap: var(--space-2);
    align-items: baseline;
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.85rem;
  }

  .watchlist-item:last-child {
    border-bottom: none;
  }

  .symbol {
    font-weight: 600;
  }

  .price {
    font-family: var(--font-mono);
  }
</style>
