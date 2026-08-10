<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { authStore } from "$lib/stores/auth.svelte";
  import { serverHealth } from "$lib/api/tauri";
  import ThemeToggle from "./ThemeToggle.svelte";

  const links = [
    { href: "/home", label: "Home" },
    { href: "/settings", label: "Settings" },
  ];

  // Server health is display-only chrome, shown here in the nav rather
  // than as an action on the Settings page. Polled quietly; failures just
  // mean "server unreachable", which is a normal, non-fatal state — the
  // app still works fully on local NSE data.
  let serverUp = $state(false);

  async function pollHealth() {
    try {
      await serverHealth();
      serverUp = true;
    } catch {
      serverUp = false;
    }
  }

  onMount(() => {
    pollHealth();
    const timer = setInterval(pollHealth, 30_000);
    return () => clearInterval(timer);
  });
</script>

<nav>
  <div class="brand">kstocks</div>
  <div class="links">
    {#each links as link (link.href)}
      <a href={link.href} class:active={$page.url.pathname === link.href}>{link.label}</a>
    {/each}
  </div>
  <div class="right">
    <span
      class="status-dot"
      class:ready={serverUp}
      title={serverUp ? "Server reachable" : "Server unreachable — using local data only"}
    ></span>
    <span class="status-dot" class:ready={authStore.isReady} title={`Account: ${authStore.status}`}></span>
    <ThemeToggle />
  </div>
</nav>

<style>
  nav {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--color-border);
    background-color: var(--color-bg-elevated);
  }

  .brand {
    font-weight: 700;
    letter-spacing: 0.02em;
  }

  .links {
    display: flex;
    gap: var(--space-4);
    flex: 1;
  }

  .links a {
    color: var(--color-text-muted);
    text-decoration: none;
    font-size: 0.9rem;
    padding: 0.25em 0;
    border-bottom: 2px solid transparent;
  }

  .links a.active {
    color: var(--color-text);
    border-bottom-color: var(--color-accent);
  }

  .right {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--color-negative);
  }

  .status-dot.ready {
    background-color: var(--color-positive);
  }
</style>
