<script lang="ts">
  import { openPaperTrade } from "$lib/api/tauri";
  import { brokerStore } from "$lib/stores/broker.svelte";
  import type { TradingMode } from "$lib/types";

  let mode = $state<TradingMode>("paper");
  let symbol = $state("NIFTY");
  let side = $state<"buy" | "sell">("buy");
  let quantity = $state(1);
  let price = $state<number | "">("");
  let loading = $state(false);
  let message = $state("");

  async function submit() {
    message = "";
    if (mode === "live") {
      // Production trading isn't wired yet — the broker key is only
      // captured as a placeholder for now (see Settings).
      message = brokerStore.hasDhanApiKey
        ? "Live trading isn't wired up yet."
        : "Add a Dhan API key in Settings before live trading is enabled.";
      return;
    }

    if (!price || price <= 0) {
      message = "Enter a price.";
      return;
    }

    loading = true;
    try {
      await openPaperTrade({
        symbol,
        instrumentType: "index",
        side,
        quantity,
        entryPrice: Number(price),
      });
      message = "Paper order placed.";
      price = "";
    } catch (e) {
      message = `${e}`;
    } finally {
      loading = false;
    }
  }
</script>

<div class="order-panel card">
  <div class="mode-toggle">
    <button class:active={mode === "paper"} onclick={() => (mode = "paper")}>Paper</button>
    <button class:active={mode === "live"} onclick={() => (mode = "live")}>Live</button>
  </div>

  <form class="stack pad" onsubmit={(e) => { e.preventDefault(); submit(); }}>
    <label class="field">
      <span>Symbol</span>
      <input bind:value={symbol} />
    </label>

    <div class="side-toggle">
      <button type="button" class:active={side === "buy"} class="buy" onclick={() => (side = "buy")}>
        Buy
      </button>
      <button type="button" class:active={side === "sell"} class="sell" onclick={() => (side = "sell")}>
        Sell
      </button>
    </div>

    <label class="field">
      <span>Quantity</span>
      <input type="number" min="1" bind:value={quantity} />
    </label>

    <label class="field">
      <span>Price</span>
      <input type="number" step="0.05" placeholder="Market/limit price" bind:value={price} />
    </label>

    <button type="submit" class="primary submit-btn" disabled={loading}>
      {mode === "paper" ? "Place paper order" : "Place live order"}
    </button>

    {#if message}
      <p class="muted">{message}</p>
    {/if}
  </form>
</div>

<style>
  .order-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }

  .pad {
    padding: var(--space-4);
  }

  .mode-toggle,
  .side-toggle {
    display: flex;
    border-bottom: 1px solid var(--color-border);
  }

  .side-toggle {
    border-bottom: none;
    border-radius: var(--radius-sm);
    overflow: hidden;
    border: 1px solid var(--color-border);
  }

  .mode-toggle button,
  .side-toggle button {
    flex: 1;
    border: none;
    border-radius: 0;
    background: transparent;
  }

  .mode-toggle button.active {
    background-color: var(--color-accent);
    color: var(--color-accent-contrast);
  }

  .side-toggle button.buy.active {
    background-color: var(--color-positive);
    color: var(--color-accent-contrast);
  }

  .side-toggle button.sell.active {
    background-color: var(--color-negative);
    color: var(--color-accent-contrast);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .submit-btn {
    width: 100%;
  }
</style>
