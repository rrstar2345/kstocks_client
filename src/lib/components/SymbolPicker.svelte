<script lang="ts">
  import type { ChartInterval } from "$lib/types";

  // Placeholder list until this reads from the user's watchlists / a
  // symbols endpoint. Kept short and index-focused to match what the
  // backend currently streams (see market/streamers).
  const KNOWN_SYMBOLS = ["NIFTY", "BANKNIFTY", "FINNIFTY", "SENSEX"];

  let {
    symbol,
    interval,
    onchange,
  }: {
    symbol: string;
    interval: ChartInterval;
    onchange: (next: { symbol: string; interval: ChartInterval }) => void;
  } = $props();
</script>

<div class="row">
  <select
    value={symbol}
    onchange={(e) => onchange({ symbol: e.currentTarget.value, interval })}
  >
    {#each KNOWN_SYMBOLS as s (s)}
      <option value={s}>{s}</option>
    {/each}
  </select>
  <select
    value={interval}
    onchange={(e) =>
      onchange({ symbol, interval: e.currentTarget.value as ChartInterval })}
  >
    <option value="1m">1m</option>
    <option value="1d">1d</option>
  </select>
</div>

<style>
  select {
    font-size: 0.85rem;
    padding: 0.3em 0.5em;
  }
</style>
