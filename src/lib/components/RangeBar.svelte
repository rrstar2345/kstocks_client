<script lang="ts">
  // A horizontal oval bar spanning [low, high] with a marker positioned at
  // the current price. Used by the index list-view (item 5 in the spec):
  // "an oval bar with day low and day high as the left and right end
  // values, a marker in between placed to show the currently traded price."

  let {
    low,
    high,
    current,
  }: {
    low: number;
    high: number;
    current: number;
  } = $props();

  const range = $derived(high - low || 1);
  const pct = $derived(Math.min(1, Math.max(0, (current - low) / range)));
</script>

<div class="range-bar" title={`Low ${low} · High ${high} · LTP ${current}`}>
  <span class="range-label low">{low.toFixed(1)}</span>
  <div class="oval">
    <div class="marker" style={`left: ${pct * 100}%`}></div>
  </div>
  <span class="range-label high">{high.toFixed(1)}</span>
</div>

<style>
  .range-bar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
  }

  .oval {
    position: relative;
    flex: 1;
    height: 8px;
    border-radius: 999px;
    background-color: var(--color-bg-inset);
    border: 1px solid var(--color-border);
  }

  .marker {
    position: absolute;
    top: 50%;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background-color: var(--color-accent);
    border: 2px solid var(--color-bg-elevated);
    transform: translate(-50%, -50%);
  }

  .range-label {
    font-size: 0.72rem;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    white-space: nowrap;
  }
</style>
