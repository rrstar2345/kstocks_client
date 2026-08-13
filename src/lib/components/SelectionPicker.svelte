<script lang="ts">
  import { onMount } from "svelte";
  import { listIndexSymbols, listOptionExpiries, listOptionStrikes, listOptionSymbols } from "$lib/api/tauri";
  import { onIndexTick, onOptionTick } from "$lib/api/events";
  import type { ChartInterval, InstrumentSelection, OptionLeg } from "$lib/types";

  // How often to silently re-poll symbol/expiry/strike lists so a picker
  // that's just sitting on a selection still picks up newly-discovered
  // values (e.g. a new expiry/strike the backend starts tracking after
  // this component mounted). Ticks alone don't cover this: option ticks
  // only arrive for strikes we already know about, so a *brand new*
  // strike/expiry needs a poll, not just an event, to be discovered.
  const REFRESH_INTERVAL_MS = 5000;

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

  let indexSymbols = $state<string[]>([]);
  let optionSymbols = $state<string[]>([]);
  let expiries = $state<string[]>([]);
  let strikes = $state<number[]>([]);

  const mode = $derived(selection?.kind ?? "index");

  async function loadIndexSymbols() {
    try {
      const next = await listIndexSymbols();
      // Merge rather than overwrite so a symbol that only exists because
      // a tick added it live (see onIndexTick below) isn't dropped by a
      // backend list that hasn't caught up yet.
      const merged = new Set([...indexSymbols, ...next]);
      indexSymbols = [...merged].sort();
    } catch {
      // keep last known list on transient failure
    }
  }

  async function loadOptionSymbols() {
    try {
      optionSymbols = await listOptionSymbols();
    } catch {
      // keep last known list on transient failure
    }
  }

  async function loadExpiries(symbol: string) {
    try {
      expiries = await listOptionExpiries(symbol);
    } catch {
      // keep last known list on transient failure
    }
  }

  async function loadStrikes(symbol: string, expiry: string) {
    try {
      strikes = await listOptionStrikes(symbol, expiry);
    } catch {
      // keep last known list on transient failure
    }
  }

  /** Re-pull whichever lists are relevant to the current selection. Used
   * both for the initial load and the periodic refresh so newly
   * discovered expiries/strikes/symbols show up without requiring the
   * user to change a different select box first. */
  function refreshLists() {
    loadIndexSymbols();
    loadOptionSymbols();
    if (selection?.kind === "option") {
      loadExpiries(selection.symbol);
      if (selection.expiry) {
        loadStrikes(selection.symbol, selection.expiry);
      }
    }
  }

  onMount(() => {
    let unlistenIndexTick: (() => void) | undefined;
    let unlistenOptionTick: (() => void) | undefined;

    refreshLists();

    // A new index can start streaming after this widget mounted (e.g.
    // the symbol list was empty at startup before the first ticks
    // arrived). Grow the picker's options live instead of requiring a
    // reload.
    onIndexTick((tick) => {
      if (!indexSymbols.includes(tick.index_name)) {
        indexSymbols = [...indexSymbols, tick.index_name].sort();
      }
    }).then((fn) => (unlistenIndexTick = fn));

    // Same idea for options: a tick for the currently selected
    // symbol/expiry can imply a strike exists that our last strikes
    // fetch didn't have yet.
    onOptionTick((tick) => {
      if (
        selection?.kind === "option" &&
        tick.symbol === selection.symbol &&
        tick.expiry === selection.expiry &&
        !strikes.includes(tick.strike_price)
      ) {
        strikes = [...strikes, tick.strike_price].sort((a, b) => a - b);
      }
    }).then((fn) => (unlistenOptionTick = fn));

    // Periodic background refresh: covers cases ticks don't (new expiry
    // dates, or a picker that isn't the one the user is actively
    // changing — every mounted SelectionPicker refreshes independently
    // on its own timer so all widgets stay in sync with each other).
    const intervalId = setInterval(refreshLists, REFRESH_INTERVAL_MS);

    return () => {
      unlistenIndexTick?.();
      unlistenOptionTick?.();
      clearInterval(intervalId);
    };
  });

  $effect(() => {
    if (selection?.kind === "option") {
      loadExpiries(selection.symbol);
    }
  });

  $effect(() => {
    if (selection?.kind === "option" && selection.expiry) {
      loadStrikes(selection.symbol, selection.expiry);
    }
  });

  function switchMode(next: "index" | "option") {
    if (next === "index") {
      const symbol = indexSymbols[0] ?? "NIFTY";
      onchange({ selection: { kind: "index", symbol }, interval });
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
    if (selection?.kind !== "option") return;
    onchange({ selection: { ...selection, ...patch, kind: "option" }, interval });
  }
</script>

<div class="picker row">
  <select value={mode} onchange={(e) => switchMode(e.currentTarget.value as "index" | "option")}>
    <option value="index">Index</option>
    <option value="option">Option chain</option>
  </select>

  {#if !selection || selection.kind === "index"}
    <select value={selection?.symbol ?? "NIFTY"} onchange={(e) => updateIndexSymbol(e.currentTarget.value)}>
      {#if selection?.symbol && !indexSymbols.includes(selection.symbol)}
        <option value={selection.symbol}>{selection.symbol}</option>
      {/if}
      {#each indexSymbols as s (s)}
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