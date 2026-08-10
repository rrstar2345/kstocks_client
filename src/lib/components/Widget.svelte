<script lang="ts">
  import { getOptionChain, getRecentIndexBars, getRecentOptionBars } from "$lib/api/tauri";
  import type { ChartWidgetConfig, OhlcBar, OptionChainRow, OptionLegBar } from "$lib/types";
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

  const POLL_MS = 15_000;

  async function loadChart() {
    loading = true;
    errorMsg = "";
    try {
      if (config.selection.kind === "index") {
        bars = await getRecentIndexBars(config.selection.symbol, config.interval, 200);
      } else if (config.selection.expiry && config.selection.strike) {
        bars = await getRecentOptionBars(
          config.selection.symbol,
          config.selection.expiry,
          config.selection.strike,
          config.selection.leg,
          200
        );
      } else {
        bars = [];
      }
    } catch (e) {
      errorMsg = `${e}`;
    } finally {
      loading = false;
    }
  }

  async function loadChain() {
    if (config.selection.kind !== "option" || !config.selection.expiry) {
      chainRows = [];
      return;
    }
    loading = true;
    errorMsg = "";
    try {
      chainRows = await getOptionChain(config.selection.symbol, config.selection.expiry);
    } catch (e) {
      errorMsg = `${e}`;
    } finally {
      loading = false;
    }
  }

  function load() {
    if (config.view === "chart") {
      void loadChart();
    } else if (config.selection.kind === "option") {
      void loadChain();
    } else {
      loading = false;
    }
  }

  $effect(() => {
    void config.view;
    void config.selection;
    void config.interval;
    load();

    const timer = setInterval(load, POLL_MS);
    return () => clearInterval(timer);
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
    config.selection.kind === "index"
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
      <span class="slot-label">{slotLabel}</span>
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
    {:else if config.selection.kind === "option"}
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

  .slot-label {
    font-size: 0.7rem;
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0 0.35em;
  }

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
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
