<script lang="ts">
  import { addWidget, removeWidget, updateWidget, widgetsStore, MAX_WIDGETS } from "$lib/stores/widgets.svelte";
  import Widget from "./Widget.svelte";

  // Grid slot layout, per spec:
  //   w1) 1 widget  -> fills all 4 cells
  //   w2) 2 widgets -> widget1 = left column (top+bottom), widget2 = right column (top+bottom)
  //   w3) 3 widgets -> widget1 = left column (top+bottom), widget2 = right-top, widget3 = right-bottom
  //   w4) 4 widgets -> each widget takes its own cell: 1=left-top, 2=left-bottom, 3=right-bottom, 4=right-top
  const SLOT_LABELS = ["Left-top", "Left-bottom", "Right-bottom", "Right-top"];

  function areaFor(count: number, index: number): string {
    if (count === 1) return "one";
    if (count === 2) return index === 0 ? "left" : "right";
    if (count === 3) {
      if (index === 0) return "left";
      if (index === 1) return "right-top";
      return "right-bottom";
    }
    // count === 4
    return ["left-top", "left-bottom", "right-bottom", "right-top"][index];
  }

  function gridTemplateFor(count: number): string {
    switch (count) {
      case 1:
        return `"one one" 1fr / 1fr 1fr`;
      case 2:
        return `"left right" 1fr / 1fr 1fr`;
      case 3:
        return `"left right-top" 1fr "left right-bottom" 1fr / 1fr 1fr`;
      default:
        return `"left-top right-top" 1fr "left-bottom right-bottom" 1fr / 1fr 1fr`;
    }
  }

  const count = $derived(widgetsStore.list.length);
  const templateAreas = $derived(gridTemplateFor(count));
</script>

<div class="grid-toolbar">
  <h2>Workspace</h2>
  <button class="primary" onclick={() => addWidget()} disabled={!widgetsStore.canAdd}>
    + Add widget ({widgetsStore.list.length}/{MAX_WIDGETS})
  </button>
</div>

{#if widgetsStore.list.length === 0}
  <p class="muted">No widgets yet. Add one to start watching a symbol.</p>
{:else}
  <div class="widget-grid" style={`grid-template: ${templateAreas};`}>
    {#each widgetsStore.list as config, i (config.id)}
      <div class="widget-slot" style={`grid-area: ${areaFor(count, i)};`}>
        <Widget {config} slotLabel={SLOT_LABELS[i]} onupdate={updateWidget} onremove={removeWidget} />
      </div>
    {/each}
  </div>
{/if}

<style>
  .grid-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-3);
  }

  .widget-grid {
    display: grid;
    gap: var(--space-3);
    height: 100%;
    min-height: 0;
  }

  .widget-slot {
    min-height: 0;
    min-width: 0;
  }
</style>
