<script lang="ts">
  import "../styles/global.css";
  import { onMount } from "svelte";
  import { initTheme } from "$lib/stores/theme.svelte";
  import { initWidgets } from "$lib/stores/widgets.svelte";
  import { loadUsername, refreshServerConfig, refreshValidation } from "$lib/stores/auth.svelte";
  import Nav from "$lib/components/Nav.svelte";

  let { children } = $props();

  onMount(async () => {
    await initTheme();
    await initWidgets();
    await loadUsername();
    await refreshServerConfig();
    // Validation is never user-triggered — it always runs automatically
    // once, here, on app start.
    await refreshValidation();
  });
</script>

<Nav />
<main class="app-main">
  {@render children()}
</main>

<style>
  .app-main {
    max-width: 1400px;
    margin: 0 auto;
    padding: var(--space-5);
  }
</style>
