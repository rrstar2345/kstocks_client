<script lang="ts">
  import Watchlist from "$lib/components/Watchlist.svelte";
  import WidgetGrid from "$lib/components/WidgetGrid.svelte";
  import OrderPanel from "$lib/components/OrderPanel.svelte";

  // Home is available regardless of registration status: unregistered
  // users see live NSE data streamed directly by this client; registered
  // users additionally benefit from server-side backfill running quietly
  // in the background (see src-tauri lib.rs setup + storage/backfill.rs).
</script>

<div class="home-layout">
  <section class="pane watchlist-pane">
    <Watchlist />
  </section>

  <section class="pane workspace-pane">
    <WidgetGrid />
  </section>

  <section class="pane order-pane">
    <OrderPanel />
  </section>
</div>

<style>
  .home-layout {
    display: grid;
    grid-template-columns: 260px minmax(0, 1fr) 300px;
    gap: var(--space-4);
    height: calc(100vh - 130px);
    min-height: 480px;
  }

  .pane {
    min-width: 0;
    min-height: 0;
  }

  .watchlist-pane,
  .order-pane {
    height: 100%;
  }

  .workspace-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  @media (max-width: 1000px) {
    .home-layout {
      grid-template-columns: 1fr;
      height: auto;
    }

    .watchlist-pane,
    .order-pane {
      height: 320px;
    }

    .workspace-pane {
      height: 600px;
    }
  }
</style>
