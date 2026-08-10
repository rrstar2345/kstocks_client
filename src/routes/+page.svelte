<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authStore, refreshServerConfig } from "$lib/stores/auth.svelte";

  onMount(async () => {
    await refreshServerConfig();
    goto(authStore.hasApiKey ? "/dashboard" : "/register");
  });
</script>

<p class="muted">Loading…</p>
