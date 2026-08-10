<script lang="ts">
  import "../styles/global.css";
  import { onMount } from "svelte";
  import { initTheme } from "$lib/stores/theme.svelte";
  import { initWidgets } from "$lib/stores/widgets.svelte";
  import { refreshServerConfig, refreshValidation } from "$lib/stores/auth.svelte";
  import Nav from "$lib/components/Nav.svelte";

  let { children } = $props();

  onMount(async () => {
    await initTheme();
    await initWidgets();
    await refreshServerConfig();
    await refreshValidation();
  });
</script>

<Nav />
<main class="app-main">
  {@render children()}
</main>

<style>
  .app-main {
    max-width: 1100px;
    margin: 0 auto;
    padding: var(--space-5);
  }
</style>
