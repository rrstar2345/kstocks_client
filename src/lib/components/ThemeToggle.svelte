<script lang="ts">
  import { themeStore, toggleTheme } from "$lib/stores/theme.svelte";
</script>

<button onclick={() => toggleTheme()} aria-label="Toggle theme" title="Toggle theme">
  {themeStore.value === "dark" ? "☀️" : "🌙"}
</button>
