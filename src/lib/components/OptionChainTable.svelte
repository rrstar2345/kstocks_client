<script lang="ts">
  import type { OptionChainRow } from "$lib/types";

  let {
    rows,
    nearestStrike,
    showNearestOnly = false,
    nearestCount = 10,
  }: {
    rows: OptionChainRow[];
    nearestStrike?: number | null;
    showNearestOnly?: boolean;
    nearestCount?: number;
  } = $props();

  const displayRows = $derived.by(() => {
    if (!showNearestOnly || nearestStrike == null || rows.length === 0) return rows;
    const sorted = [...rows].sort(
      (a, b) => Math.abs(a.strike_price - nearestStrike) - Math.abs(b.strike_price - nearestStrike)
    );
    const nearest = new Set(sorted.slice(0, nearestCount).map((r) => r.strike_price));
    return rows.filter((r) => nearest.has(r.strike_price));
  });
</script>

<div class="chain-table-wrap">
  <table class="chain-table">
    <thead>
      <tr>
        <th colspan="3" class="ce-header">Calls (CE)</th>
        <th class="strike-header">Strike</th>
        <th colspan="3" class="pe-header">Puts (PE)</th>
      </tr>
      <tr class="sub-header">
        <th>OI</th>
        <th>Vol</th>
        <th>LTP</th>
        <th></th>
        <th>LTP</th>
        <th>Vol</th>
        <th>OI</th>
      </tr>
    </thead>
    <tbody>
      {#each displayRows as row (row.strike_price)}
        <tr class:is-atm={row.strike_price === nearestStrike}>
          <td>{row.ce_oi_close?.toFixed(0) ?? "—"}</td>
          <td>{row.ce_volume ?? "—"}</td>
          <td class="ltp">{row.ce_close?.toFixed(2) ?? "—"}</td>
          <td class="strike">{row.strike_price}</td>
          <td class="ltp">{row.pe_close?.toFixed(2) ?? "—"}</td>
          <td>{row.pe_volume ?? "—"}</td>
          <td>{row.pe_oi_close?.toFixed(0) ?? "—"}</td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if displayRows.length === 0}
    <p class="muted pad">No option chain data yet.</p>
  {/if}
</div>

<style>
  .chain-table-wrap {
    overflow: auto;
    height: 100%;
  }

  .pad {
    padding: var(--space-4);
  }

  .chain-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.78rem;
  }

  .chain-table th,
  .chain-table td {
    padding: 0.3em 0.5em;
    text-align: right;
    white-space: nowrap;
  }

  .ce-header {
    text-align: center;
    color: var(--color-positive);
  }

  .pe-header {
    text-align: center;
    color: var(--color-negative);
  }

  .strike-header,
  .strike {
    text-align: center;
    font-weight: 600;
  }

  .sub-header th {
    color: var(--color-text-muted);
    font-weight: 500;
    border-bottom: 1px solid var(--color-border);
  }

  .ltp {
    font-family: var(--font-mono);
  }

  tr.is-atm {
    background-color: var(--color-bg-inset);
  }
</style>
