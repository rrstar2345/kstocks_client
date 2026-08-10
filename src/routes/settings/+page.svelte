<script lang="ts">
  import { onMount } from "svelte";
  import { registerClient } from "$lib/api/tauri";
  import {
    authStore,
    logOut,
    refreshServerConfig,
    refreshValidation,
    rememberUsername,
  } from "$lib/stores/auth.svelte";
  import { brokerStore, clearDhanApiKey, setDhanApiKey } from "$lib/stores/broker.svelte";

  let username = $state("");
  let loading = $state(false);
  let errorMsg = $state("");

  // Broker key input is local, uncontrolled-ish state that's mirrored into
  // the in-memory-only broker store on every keystroke — never persisted,
  // never sent anywhere yet (wiring for production trading comes later).
  let dhanKeyInput = $state("");

  onMount(async () => {
    await refreshServerConfig();
    dhanKeyInput = brokerStore.dhanApiKey;
  });

  async function handleRegister() {
    if (!username.trim()) return;
    loading = true;
    errorMsg = "";
    try {
      await registerClient(username.trim());
      await rememberUsername(username.trim());
      await refreshServerConfig();
      await refreshValidation();
    } catch (e) {
      errorMsg = `${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleForget() {
    await logOut();
    username = "";
  }

  function handleDhanKeyInput(value: string) {
    dhanKeyInput = value;
    setDhanApiKey(value);
  }

  function handleClearDhanKey() {
    dhanKeyInput = "";
    clearDhanApiKey();
  }

  function statusLabel(status: string): string {
    switch (status) {
      case "approved":
        return "Approved";
      case "pending":
        return "Pending approval";
      case "rejected":
        return "Rejected";
      case "unregistered":
        return "Not registered";
      default:
        return "Unknown";
    }
  }
</script>

<div class="stack">
  <h1>Settings</h1>

  <section class="card pad">
    <h2>Account</h2>

    {#if authStore.hasApiKey}
      <div class="stack">
        <div class="field-row">
          <span class="field-label">Username</span>
          <span class="field-value">{authStore.username ?? "—"}</span>
        </div>
        <div class="field-row">
          <span class="field-label">Status</span>
          <span class="status-pill" class:approved={authStore.status === "approved"}>
            {statusLabel(authStore.status)}
          </span>
        </div>
        <p class="muted">
          Approval status is checked automatically each time the app starts.
        </p>
        <div class="row">
          <button onclick={handleForget} disabled={loading}>Forget this device</button>
        </div>
      </div>
    {:else}
      <form class="row" onsubmit={(e) => { e.preventDefault(); handleRegister(); }}>
        <input placeholder="username" bind:value={username} disabled={loading} />
        <button type="submit" class="primary" disabled={loading || !username.trim()}>
          Register
        </button>
      </form>
      <p class="muted">
        Registration is optional. Without it, the app still works using live
        data streamed directly from NSE. Registering unlocks server-side
        backfill so charts have continuity even when the app wasn't running.
      </p>
    {/if}

    {#if errorMsg}
      <p class="error-banner">{errorMsg}</p>
    {/if}
  </section>

  <section class="card pad">
    <h2>Broker connection</h2>
    <p class="muted">
      Placeholder for wiring production trading via Dhan. This key is used
      only for the current session — it is never saved to disk and is
      forgotten as soon as the app closes.
    </p>
    <div class="row">
      <input
        type="password"
        placeholder="Dhan API key"
        value={dhanKeyInput}
        oninput={(e) => handleDhanKeyInput(e.currentTarget.value)}
        autocomplete="off"
      />
      <button onclick={handleClearDhanKey} disabled={!brokerStore.hasDhanApiKey}>Clear</button>
    </div>
    {#if brokerStore.hasDhanApiKey}
      <p class="muted">Key set for this session.</p>
    {/if}
  </section>
</div>

<style>
  .pad {
    padding: var(--space-4);
  }

  .field-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .field-label {
    width: 90px;
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }

  .field-value {
    font-weight: 600;
  }

  .status-pill {
    display: inline-block;
    padding: 0.2em 0.7em;
    border-radius: 999px;
    font-size: 0.8rem;
    background-color: var(--color-bg-inset);
    color: var(--color-text-muted);
  }

  .status-pill.approved {
    background-color: var(--color-positive);
    color: var(--color-accent-contrast);
  }
</style>
