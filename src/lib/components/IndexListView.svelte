<script lang="ts">
  import { onMount } from "svelte";
  import { getAllIndexSnapshots } from "$lib/api/tauri";
  import { onIndexTick } from "$lib/api/events";
  import type { IndexSnapshot, IndexTick } from "$lib/types";
  import RangeBar from "./RangeBar.svelte";

  let snapshots = $state<IndexSnapshot[]>([]);

  async function load() {
    try {
      snapshots = await getAllIndexSnapshots();
    } catch {
      // keep last-known values
    }
  }

  function applyTick(tick: IndexTick) {
    const next: IndexSnapshot = {
      index_name: tick.index_name,
      current_price: tick.current_price,
      change: tick.change,
      per_change: tick.per_change,
      open: tick.open,
      low: tick.low,
      high: tick.high,
      previous_close: tick.previous_close,
      time: tick.time,
    };

    const idx = snapshots.findIndex((s) => s.index_name === tick.index_name);
    if (idx === -1) {
      snapshots = [...snapshots, next];
    } else {
      snapshots = snapshots.map((s, i) => (i === idx ? next : s));
    }
  }

  onMount(() => {
    load();

    let unlisten: (() => void) | undefined;
    onIndexTick(applyTick).then((fn) => (unlisten = fn));

    return () => unlisten?.();
  });
</script>

<div class="index-list">
  {#if snapshots.length === 0}
    <p class="muted pad">No live data yet.</p>
  {:else}
    {#each snapshots as s (s.index_name)}
      <div class="index-row">
        <div class="index-row-top">
          <span class="symbol">{s.index_name}</span>
          <span class="price">{s.current_price?.toFixed(2) ?? "—"}</span>
          <span class={(s.per_change ?? 0) >= 0 ? "text-positive" : "text-negative"}>
            {(s.per_change ?? 0) >= 0 ? "+" : ""}{s.per_change?.toFixed(2) ?? "0.00"}%
          </span>
        </div>
        {#if s.low != null && s.high != null && s.current_price != null}
          <RangeBar low={s.low} high={s.high} current={s.current_price} />
        {/if}
      </div>
    {/each}
  {/if}
</div>

<style>
  .index-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-3);
    overflow-y: auto;
    height: 100%;
  }

  .pad {
    padding: var(--space-4);
  }

  .index-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .index-row-top {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    font-size: 0.85rem;
  }

  .symbol {
    font-weight: 600;
    flex: 1;
  }

  .price {
    font-family: var(--font-mono);
  }
</style>