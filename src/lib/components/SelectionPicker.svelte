<script lang="ts">
  import { onMount } from "svelte";
  import { listOptionExpiries, listOptionStrikes, listOptionSymbols } from "$lib/api/tauri";
  import type { ChartInterval, InstrumentSelection, OptionLeg } from "$lib/types";

  const KNOWN_INDICES = ["NIFTY", "BANKNIFTY", "FINNIFTY", "SENSEX"];

  let {
    selection,
    interval,
    showInterval = true,
    onchange,
  }: {
    selection: InstrumentSelection;
    interval?: ChartInterval;
    showInterval?: boolean;
    onchange: (next: { selection: InstrumentSelection; interval?: ChartInterval }) => void;
  } = $props();

  let optionSymbols = $state<string[]>([]);
  let expiries = $state<string[]>([]);
  let strikes = $state<number[]>([]);

  const mode = $derived(selection.kind);

  onMount(async () => {
    try {
      optionSymbols = await listOptionSymbols();
    } catch {
      optionSymbols = [];
    }
  });

  async function loadExpiries(symbol: string) {
    try {
      expiries = await listOptionExpiries(symbol);
    } catch {
      expiries = [];
    }
  }

  async function loadStrikes(symbol: string, expiry: string) {
    try {
      strikes = await listOptionStrikes(symbol, expiry);
    } catch {
      strikes = [];
    }
  }

  $effect(() => {
    if (selection.kind === "option") {
      loadExpiries(selection.symbol);
    }
  });

  $effect(() => {
    if (selection.kind === "option" && selection.expiry) {
      loadStrikes(selection.symbol, selection.expiry);
    }
  });

  function switchMode(next: "index" | "option") {
    if (next === "index") {
      onchange({ selection: { kind: "index", symbol: "NIFTY" }, interval });
    } else {
      const symbol = optionSymbols[0] ?? "NIFTY";
      onchange({
        selection: { kind: "option", symbol, expiry: "", strike: 0, leg: "CE" },
        interval,
      });
    }
  }

  function updateIndexSymbol(symbol: string) {
    onchange({ selection: { kind: "index", symbol }, interval });
  }

  function updateOptionField(patch: Partial<{ symbol: string; expiry: string; strike: number; leg: OptionLeg }>) {
    if (selection.kind !== "option") return;
    onchange({ selection: { ...selection, ...patch, kind: "option" }, interval });
  }
</script>

<div class="picker row">
  <select value={mode} onchange={(e) => switchMode(e.currentTarget.value as "index" | "option")}>
    <option value="index">Index</option>
    <option value="option">Option chain</option>
  </select>

  {#if selection.kind === "index"}
    <select value={selection.symbol} onchange={(e) => updateIndexSymbol(e.currentTarget.value)}>
      {#each KNOWN_INDICES as s (s)}
        <option value={s}>{s}</option>
      {/each}
    </select>
  {:else}
    <select
      value={selection.symbol}
      onchange={(e) => updateOptionField({ symbol: e.currentTarget.value, expiry: "", strike: 0 })}
    >
      {#each optionSymbols as s (s)}
        <option value={s}>{s}</option>
      {/each}
    </select>
    <select
      value={selection.expiry}
      onchange={(e) => updateOptionField({ expiry: e.currentTarget.value, strike: 0 })}
    >
      <option value="" disabled>Expiry</option>
      {#each expiries as e (e)}
        <option value={e}>{e}</option>
      {/each}
    </select>
    <select
      value={selection.strike || ""}
      onchange={(e) => updateOptionField({ strike: Number(e.currentTarget.value) })}
    >
      <option value="" disabled>Strike</option>
      {#each strikes as s (s)}
        <option value={s}>{s}</option>
      {/each}
    </select>
    <select value={selection.leg} onchange={(e) => updateOptionField({ leg: e.currentTarget.value as OptionLeg })}>
      <option value="CE">CE</option>
      <option value="PE">PE</option>
    </select>
  {/if}

  {#if showInterval && interval}
    <select value={interval} onchange={(e) => onchange({ selection, interval: e.currentTarget.value as ChartInterval })}>
      <option value="1m">1m</option>
      <option value="1d">1d</option>
    </select>
  {/if}
</div>

<style>
  .picker select {
    font-size: 0.78rem;
    padding: 0.25em 0.4em;
  }
</style>