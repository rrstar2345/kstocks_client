<script lang="ts">
  import { onMount } from "svelte";
  import {
    registerClient,
    serverHealth,
    setServerUrl,
  } from "$lib/api/tauri";
  import {
    authStore,
    logOut,
    refreshServerConfig,
    refreshValidation,
  } from "$lib/stores/auth.svelte";
  import type { HealthResponse } from "$lib/types";

  let username = $state("");
  let baseUrlInput = $state("");
  let loading = $state(false);
  let errorMsg = $state("");
  let healthResult = $state<HealthResponse | null>(null);

  // The API key is held ONLY here, transiently, right after a successful
  // register call — never in the shared auth store, never persisted to a
  // Svelte store that other routes read, and never re-fetchable from the
  // backend afterwards (get_server_config only ever returns has_api_key).
  let revealedApiKey = $state<string | null>(null);
  let keyAcknowledged = $state(false);

  onMount(async () => {
    await refreshServerConfig();
    baseUrlInput = authStore.baseUrl;
  });

  async function handleRegister() {
    if (!username.trim()) return;
    loading = true;
    errorMsg = "";
    try {
      const res = await registerClient(username.trim());
      revealedApiKey = res.api_key;
      keyAcknowledged = false;
      await refreshServerConfig();
      await refreshValidation();
    } catch (e) {
      errorMsg = `${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleValidate() {
    loading = true;
    errorMsg = "";
    try {
      await refreshValidation();
    } catch (e) {
      errorMsg = `${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleHealth() {
    loading = true;
    errorMsg = "";
    try {
      healthResult = await serverHealth();
    } catch (e) {
      errorMsg = `${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleSaveServerUrl() {
    loading = true;
    errorMsg = "";
    try {
      await setServerUrl(baseUrlInput.trim());
      await refreshServerConfig();
    } catch (e) {
      errorMsg = `${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleLogOut() {
    await logOut();
    revealedApiKey = null;
  }

  function dismissKeyReveal() {
    revealedApiKey = null;
  }
</script>

<div class="stack">
  <h1>Connection</h1>

  <section class="card pad">
    <h2>Server</h2>
    <div class="row">
      <input placeholder="http://localhost:8787" bind:value={baseUrlInput} />
      <button onclick={handleSaveServerUrl} disabled={loading}>Save</button>
    </div>
    <p class="muted">Current: {authStore.baseUrl || "…"}</p>
  </section>

  {#if revealedApiKey}
    <section class="card pad key-reveal">
      <h2>Your API key</h2>
      <p>
        This is shown <strong>once</strong>. It's secret — treat it like a password.
        It will not be displayed again anywhere in the app; if lost, re-register.
      </p>
      <code class="key-value">{revealedApiKey}</code>
      <label class="row ack">
        <input type="checkbox" bind:checked={keyAcknowledged} />
        I've copied this key somewhere safe
      </label>
      <button class="primary" disabled={!keyAcknowledged} onclick={dismissKeyReveal}>
        Done — hide it
      </button>
    </section>
  {:else if authStore.hasApiKey}
    <section class="card pad">
      <h2>Registered</h2>
      <p class="muted">
        An API key is stored for this device. Status: <strong>{authStore.status}</strong>
      </p>
      <div class="row">
        <button onclick={handleValidate} disabled={loading}>Check approval status</button>
        <button onclick={handleLogOut} disabled={loading}>Forget API key</button>
      </div>
    </section>
  {:else}
    <section class="card pad">
      <h2>Register this device</h2>
      <form class="row" onsubmit={(e) => { e.preventDefault(); handleRegister(); }}>
        <input placeholder="username" bind:value={username} />
        <button type="submit" class="primary" disabled={loading}>Register</button>
      </form>
      <p class="muted">
        Accounts start <strong>pending</strong> until approved server-side.
      </p>
    </section>
  {/if}

  <section class="card pad">
    <h2>Server health</h2>
    <button onclick={handleHealth} disabled={loading}>Check health</button>
    {#if healthResult}
      <p class="muted">
        db_connected: {healthResult.db_connected} · session: {healthResult.session_mode}
      </p>
    {/if}
  </section>

  {#if errorMsg}
    <p class="error-banner">{errorMsg}</p>
  {/if}
</div>

<style>
  .pad {
    padding: var(--space-4);
  }

  .key-reveal {
    border-color: var(--color-accent);
  }

  .key-value {
    display: block;
    word-break: break-all;
    background-color: var(--color-bg-inset);
    border-radius: var(--radius-sm);
    padding: var(--space-3);
    font-family: var(--font-mono);
    font-size: 0.85rem;
    margin: var(--space-2) 0;
  }

  .ack {
    font-size: 0.85rem;
    color: var(--color-text-muted);
    margin: var(--space-2) 0;
  }
</style>
