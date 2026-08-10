<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";

  // Home is available regardless of registration status — unregistered
  // users still get live NSE data locally; registered users additionally
  // get server-side backfill. So there's nothing to branch on here.
  onMount(() => {
    goto("/home");
  });
</script>

<p class="muted">Loading…</p>
