<script lang="ts">
  import { addWidget, removeWidget, updateWidget, widgetsStore } from "$lib/stores/widgets.svelte";
  import Widget from "./Widget.svelte";
</script>

<div class="grid-toolbar">
  <h2>Dashboard</h2>
  <button class="primary" onclick={() => addWidget("NIFTY", "1m")}>+ Add widget</button>
</div>

{#if widgetsStore.list.length === 0}
  <p class="muted">No widgets yet. Add one to start watching a symbol.</p>
{/if}

<div class="widget-grid">
  {#each widgetsStore.list as config (config.id)}
    <Widget {config} onupdate={updateWidget} onremove={removeWidget} />
  {/each}
</div>

<style>
  .grid-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-4);
  }

  .widget-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
    gap: var(--space-4);
  }
</style>
