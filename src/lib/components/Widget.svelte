<script lang="ts">
  import { getRecentIndexBars } from "$lib/api/tauri";
  import type { ChartWidgetConfig, OhlcBar } from "$lib/types";
  import Chart from "./Chart.svelte";
  import SymbolPicker from "./SymbolPicker.svelte";

  let {
    config,
    onupdate,
    onremove,
  }: {
    config: ChartWidgetConfig;
    onupdate: (id: string, patch: Partial<Omit<ChartWidgetConfig, "id">>) => void;
    onremove: (id: string) => void;
  } = $props();

  let bars = $state<OhlcBar[]>([]);
  let loading = $state(true);
  let errorMsg = $state("");

  const POLL_MS = 15_000;

  async function load() {
    loading = true;
    errorMsg = "";
    try {
      bars = await getRecentIndexBars(config.symbol, config.interval, 200);
    } catch (e) {
      errorMsg = `${e}`;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    // Re-fetch whenever symbol/interval changes.
    void config.symbol;
    void config.interval;
    load();

    const timer = setInterval(load, POLL_MS);
    return () => clearInterval(timer);
  });

  const lastBar = $derived(bars.at(-1));
  const changePct = $derived.by(() => {
    if (bars.length < 2) return null;
    const first = bars[0].open;
    const last = bars[bars.length - 1].close;
    if (!first) return null;
    return ((last - first) / first) * 100;
  });
</script>

<div class="widget card">
  <div class="widget-header">
    <div class="widget-title">
      <strong>{config.symbol}</strong>
      {#if lastBar}
        <span class="price">{lastBar.close.toFixed(2)}</span>
        {#if changePct !== null}
          <span class={changePct >= 0 ? "text-positive" : "text-negative"}>
            {changePct >= 0 ? "+" : ""}{changePct.toFixed(2)}%
          </span>
        {/if}
      {/if}
    </div>
    <div class="widget-controls">
      <SymbolPicker
        symbol={config.symbol}
        interval={config.interval}
        onchange={(next) => onupdate(config.id, next)}
      />
      <button class="icon-btn" onclick={() => onremove(config.id)} aria-label="Remove widget">✕</button>
    </div>
  </div>

  {#if errorMsg}
    <p class="error-banner">{errorMsg}</p>
  {:else}
    <Chart {bars} />
  {/if}

  {#if loading && bars.length === 0}
    <p class="muted">Loading…</p>
  {/if}
</div>

<style>
  .widget {
    padding: var(--space-3) var(--space-4) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
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
    font-size: 0.95rem;
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
</style>
