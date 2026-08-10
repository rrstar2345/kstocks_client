<script lang="ts">
  import { onMount } from "svelte";
  import { authStore, refreshServerConfig } from "$lib/stores/auth.svelte";
  import WidgetGrid from "$lib/components/WidgetGrid.svelte";

  onMount(refreshServerConfig);
</script>

{#if !authStore.hasApiKey}
  <p class="muted">
    No API key stored yet. Go to <a href="/register">Connection</a> to register this device.
  </p>
{:else}
  <WidgetGrid />
{/if}
