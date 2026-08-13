<script lang="ts">
  import { onMount } from "svelte";
  import { getAllIndexSnapshots } from "$lib/api/tauri";
  import { onIndexTick } from "$lib/api/events";
  import type { IndexSnapshot, IndexTick } from "$lib/types";

  let { collapsed = $bindable(false) }: { collapsed?: boolean } = $props();

  let snapshots = $state<IndexSnapshot[]>([]);
  let loading = $state(true);

  async function load() {
    try {
      snapshots = await getAllIndexSnapshots();
    } catch {
      // keep last state
    } finally {
      loading = false;
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

  function formatIndianNumberWithDecimals(value: number | null | undefined) {
    if (value === null || value === undefined || !Number.isFinite(value))
      return "—";

    const isNeg = value < 0;
    const absFixed = Math.abs(value).toFixed(2); // keep 2 decimals
    const [intPart, decPart] = absFixed.split(".");

    // Indian commas: first comma after 3 digits from right, then every 2 digits
    const last3 = intPart.slice(-3);
    const rest = intPart.slice(0, -3);

    const intWithCommas = rest
      ? `${rest.replace(/\B(?=(\d{2})+(?!\d))/g, ",")},${last3}`
      : last3;

    return `${isNeg ? "-" : ""}${intWithCommas}.${decPart}`;
  }

  function signedFixed2(value: number | null | undefined) {
    if (value === null || value === undefined || !Number.isFinite(value))
      return { text: "—", isPositive: true };

    const isPositive = value >= 0;
    const sign = isPositive ? "+" : "";
    return {
      text: `${sign}${formatIndianNumberWithDecimals(value)}`,
      isPositive,
    };
  }
</script>

<div class="watchlist card" class:collapsed>
  <div class="watchlist-header">
    <h2>Watchlist</h2>
    <button
      class="collapse-btn"
      onclick={() => (collapsed = !collapsed)}
      aria-label={collapsed ? "Expand watchlist" : "Collapse watchlist"}
      title={collapsed ? "Expand watchlist" : "Collapse watchlist"}
    >
      {collapsed ? "»" : "«"}
    </button>
  </div>

  {#if !collapsed}
  {#if loading && snapshots.length === 0}
    <p class="muted pad">Loading…</p>
  {:else if snapshots.length === 0}
    <p class="muted pad">No live data yet.</p>
  {:else}
    <ul class="watchlist-items">
      {#each snapshots as s (s.index_name)}
        {@const changeVal = s.change ?? 0}
        {@const perChangeVal = s.per_change ?? 0}
        {@const isPositive = changeVal >= 0}
        {@const priceVal = s.current_price ?? null}

        <li class="watchlist-item">
          <div class="name-cell">
            {s.index_name}
          </div>

          <div class="metrics-cell">
            <div
              class="price-pill {isPositive
                ? 'pill-positive'
                : 'pill-negative'}"
            >
              {formatIndianNumberWithDecimals(priceVal)}
            </div>

            <div
              class="change-text {isPositive
                ? 'text-positive'
                : 'text-negative'}"
            >
              {isPositive ? "+" : ""}{formatIndianNumberWithDecimals(changeVal)}
              ({perChangeVal >= 0 ? "+" : ""}{formatIndianNumberWithDecimals(
                perChangeVal,
              )}%)
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
  {/if}
</div>

<style>
  .watchlist {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    transition: width 0.15s ease;
  }

  .watchlist.collapsed {
    width: 100%;
  }

  .watchlist-header {
    padding: var(--space-3) var(--space-4) var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .watchlist.collapsed .watchlist-header {
    flex-direction: column;
    padding: var(--space-3) var(--space-2);
  }

  .watchlist.collapsed .watchlist-header h2 {
    writing-mode: vertical-rl;
    font-size: 0.85rem;
  }

  .watchlist-header h2 {
    margin: 0;
  }

  .collapse-btn {
    flex: none;
    padding: 0.2em 0.5em;
    line-height: 1;
    font-family: var(--font-mono);
    color: var(--color-text-muted);
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .collapse-btn:hover {
    color: var(--color-text);
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
    grid-template-columns: 55% 45%;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.85rem;
    align-items: center; /* vertically middle for the row */
  }

  .watchlist-item:last-child {
    border-bottom: none;
  }

  /* Column 1 */
  .name-cell {
    font-weight: 600;
    align-self: center;

    /* 2-line clamp + ellipsis */
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    overflow: hidden;
  }

  /* Column 2 */
  .metrics-cell {
    display: grid;
    grid-template-rows: auto auto;
    gap: var(--space-2);
    justify-items: end; /* keeps pill + text aligned to the right within column */
    align-items: center;
  }

  .price-pill {
    padding: 0.2rem var(--space-2);
    border-radius: 5px;
    color: #fff;
    font-family: var(--font-mono);
    line-height: 1.2;
    white-space: nowrap;
  }

  /* Use your existing “positive/negative” colors if you already have them.
     Otherwise replace these with your theme variables. */
  .pill-positive {
    background: var(--color-positive);
  }
  .pill-negative {
    background: var(--color-negative);
  }

  /* Text color logic (same as before) */
  .text-positive {
    color: var(--color-positive);
  }
  .text-negative {
    color: var(--color-negative);
  }
</style>