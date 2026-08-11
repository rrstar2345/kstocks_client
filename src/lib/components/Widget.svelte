<script lang="ts">
  import { getOptionChain, getRecentIndexBars, getRecentOptionBars } from "$lib/api/tauri";
  import { onIndexTick, onOptionTick } from "$lib/api/events";
  import type {
    ChartWidgetConfig,
    IndexTick,
    OhlcBar,
    OptionChainRow,
    OptionLegBar,
    OptionTick,
  } from "$lib/types";
  import Chart from "./Chart.svelte";
  import SelectionPicker from "./SelectionPicker.svelte";
  import OptionChainTable from "./OptionChainTable.svelte";
  import IndexListView from "./IndexListView.svelte";

  let {
    config,
    slotLabel,
    onupdate,
    onremove,
  }: {
    config: ChartWidgetConfig;
    slotLabel: string;
    onupdate: (id: string, patch: Partial<Omit<ChartWidgetConfig, "id">>) => void;
    onremove: (id: string) => void;
  } = $props();

  let bars = $state<OhlcBar[] | OptionLegBar[]>([]);
  let chainRows = $state<OptionChainRow[]>([]);
  let loading = $state(true);
  let errorMsg = $state("");

  // Bumped on every selection/view/interval change so an in-flight fetch
  // that resolves after a newer one started can detect it's stale and
  // discard itself, instead of clobbering fresher data (or racing with a
  // tick patch for the new selection landing on the old bars array).
  let loadToken = 0;

  async function loadChart(token: number) {
    loading = true;
    errorMsg = "";
    try {
      let next: OhlcBar[] | OptionLegBar[];
      if (!config.selection) {
        next = [];
      } else if (config.selection.kind === "index") {
        next = await getRecentIndexBars(config.selection.symbol, config.interval, 200);
      } else if (config.selection.expiry && config.selection.strike) {
        next = await getRecentOptionBars(
          config.selection.symbol,
          config.selection.expiry,
          config.selection.strike,
          config.selection.leg,
          200
        );
      } else {
        next = [];
      }
      if (token !== loadToken) return; // a newer load superseded this one
      bars = next;
      barsKey = selectionKey(config.selection);
    } catch (e) {
      if (token !== loadToken) return;
      errorMsg = `${e}`;
    } finally {
      if (token === loadToken) loading = false;
    }
  }

  async function loadChain(token: number) {
    if (!config.selection || config.selection.kind !== "option" || !config.selection.expiry) {
      chainRows = [];
      return;
    }
    loading = true;
    errorMsg = "";
    try {
      const next = await getOptionChain(config.selection.symbol, config.selection.expiry);
      if (token !== loadToken) return;
      chainRows = next;
    } catch (e) {
      if (token !== loadToken) return;
      errorMsg = `${e}`;
    } finally {
      if (token === loadToken) loading = false;
    }
  }

  function load() {
    // Any pending fetch from the previous selection is now stale, and so
    // is `bars`/`chainRows` until the new fetch resolves — invalidate the
    // key immediately so a tick that arrives mid-fetch can't patch data
    // belonging to the old selection.
    const token = ++loadToken;
    barsKey = null;
    if (config.view === "chart") {
      void loadChart(token);
    } else if (config.selection?.kind === "option") {
      void loadChain(token);
    } else {
      loading = false;
    }
  }

  /** Tracks which selection `bars` currently belongs to, so a tick that
   * arrives while a fetch for a *different* selection is in flight can't
   * patch onto data it doesn't match (see `loadToken` above for the fetch
   * side of the same race). */
  let barsKey = $state<string | null>(null);

  function selectionKey(sel: typeof config.selection): string | null {
    if (!sel) return null;
    return sel.kind === "index"
      ? `index:${sel.symbol}`
      : `option:${sel.symbol}:${sel.expiry}:${sel.strike}:${sel.leg}`;
  }

  /** Live-patch the in-progress last bar's OHLC from a tick, rather than
   * re-fetching the whole series. The 1m aggregation job (server-side)
   * still periodically finalizes bars; this just keeps the visible bar
   * moving in between those rollups. No-ops if `bars` is empty (initial
   * fetch hasn't landed yet) or the tick doesn't match this widget's
   * current selection. */
  function patchLastBar(price: number) {
    if (bars.length === 0) return;
    if (barsKey !== selectionKey(config.selection)) return;
    const last = bars[bars.length - 1] as OhlcBar | OptionLegBar;
    const updated = {
      ...last,
      close: price,
      high: Math.max(last.high, price),
      low: Math.min(last.low, price),
    };
    bars = [...bars.slice(0, -1), updated];
  }

  function handleIndexTick(tick: IndexTick) {
    if (config.view !== "chart" || config.selection?.kind !== "index") return;
    if (tick.index_name !== config.selection.symbol) return;
    patchLastBar(tick.current_price);
  }

  function handleOptionTick(tick: OptionTick) {
    if (!config.selection || config.selection.kind !== "option") return;
    if (tick.symbol !== config.selection.symbol || tick.expiry !== config.selection.expiry) return;

    if (config.view === "chart") {
      if (tick.strike_price !== config.selection.strike) return;
      const price = config.selection.leg === "CE" ? tick.ce_last_price : tick.pe_last_price;
      if (price != null) patchLastBar(price);
    } else {
      // List view: patch the matching strike row in the option chain
      // in place instead of refetching the whole chain on every tick.
      const idx = chainRows.findIndex((r) => r.strike_price === tick.strike_price);
      if (idx === -1) return;
      const row = chainRows[idx];
      const updated: OptionChainRow = {
        ...row,
        ce_close: tick.ce_last_price ?? row.ce_close,
        ce_volume: tick.ce_volume ?? row.ce_volume,
        ce_oi_close: tick.ce_oi ?? row.ce_oi_close,
        pe_close: tick.pe_last_price ?? row.pe_close,
        pe_volume: tick.pe_volume ?? row.pe_volume,
        pe_oi_close: tick.pe_oi ?? row.pe_oi_close,
      };
      chainRows = chainRows.map((r, i) => (i === idx ? updated : r));
    }
  }

  $effect(() => {
    void config.view;
    void config.selection;
    void config.interval;
    load();
  });

  $effect(() => {
    let unlistenIndex: (() => void) | undefined;
    let unlistenOption: (() => void) | undefined;
    onIndexTick(handleIndexTick).then((fn) => (unlistenIndex = fn));
    onOptionTick(handleOptionTick).then((fn) => (unlistenOption = fn));
    return () => {
      unlistenIndex?.();
      unlistenOption?.();
    };
  });

  const lastBar = $derived(bars.at(-1) as { close: number } | undefined);
  const changePct = $derived.by(() => {
    if (bars.length < 2) return null;
    const first = (bars[0] as { open: number }).open;
    const last = (bars[bars.length - 1] as { close: number }).close;
    if (!first) return null;
    return ((last - first) / first) * 100;
  });

  const titleText = $derived(
    !config.selection
      ? "—"
      : config.selection.kind === "index"
        ? config.selection.symbol
        : `${config.selection.symbol} ${config.selection.strike || ""} ${config.selection.leg}`.trim()
  );

  function toggleView() {
    onupdate(config.id, { view: config.view === "chart" ? "list" : "chart" });
  }
</script>

<div class="widget card">
  <div class="widget-header">
    <div class="widget-title">
      <!-- <span class="slot-label">{slotLabel}</span> -->
      <strong>{titleText}</strong>
      {#if config.view === "chart" && lastBar}
        <span class="price">{lastBar.close.toFixed(2)}</span>
        {#if changePct !== null}
          <span class={changePct >= 0 ? "text-positive" : "text-negative"}>
            {changePct >= 0 ? "+" : ""}{changePct.toFixed(2)}%
          </span>
        {/if}
      {/if}
    </div>
    <div class="widget-controls">
      <button class="icon-btn" onclick={toggleView} title="Toggle chart / list view" aria-label="Toggle view">
        {config.view === "chart" ? "☰" : "📈"}
      </button>
      <button class="icon-btn" onclick={() => onremove(config.id)} aria-label="Remove widget">✕</button>
    </div>
  </div>

  <div class="widget-picker">
    <SelectionPicker
      selection={config.selection}
      interval={config.interval}
      showInterval={config.view === "chart"}
      onchange={({ selection, interval }) => onupdate(config.id, { selection, interval })}
    />
  </div>

  <div class="widget-body">
    {#if errorMsg}
      <p class="error-banner">{errorMsg}</p>
    {:else if config.view === "chart"}
      <Chart bars={bars as OhlcBar[]} />
    {:else if config.selection?.kind === "option"}
      <OptionChainTable
        rows={chainRows}
        nearestStrike={config.selection.strike || chainRows[Math.floor(chainRows.length / 2)]?.strike_price}
      />
    {:else}
      <IndexListView />
    {/if}

    {#if loading && bars.length === 0 && chainRows.length === 0}
      <p class="muted">Loading…</p>
    {/if}
  </div>
</div>

<style>
  .widget {
    padding: var(--space-3) var(--space-4) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    height: 100%;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .widget-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-2);
  }

  .widget-title {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    font-size: 0.9rem;
    overflow: hidden;
  }

  /* .slot-label {
    font-size: 0.7rem;
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0 0.35em;
  } */

  .price {
    font-family: var(--font-mono);
    color: var(--color-text);
  }

  .widget-controls {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .icon-btn {
    padding: 0.25em 0.55em;
    line-height: 1;
  }

  .widget-body {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>