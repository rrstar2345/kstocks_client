<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type ServerConfig = { base_url: string; api_key: string | null };
  type ValidateResponse = { approved: boolean; status: string };
  type HealthResponse = {
    db_connected: boolean;
    last_index_tick_at: string | null;
    last_option_tick_at: string | null;
    aggregation_watermarks: unknown;
    session_mode: string;
  };

  let username = $state("");
  let serverConfig = $state<ServerConfig | null>(null);
  let validateResult = $state<ValidateResponse | null>(null);
  let healthResult = $state<HealthResponse | null>(null);
  let statusMsg = $state("");
  let loading = $state(false);

  async function loadConfig() {
    serverConfig = await invoke<ServerConfig>("get_server_config");
  }

  async function register() {
    loading = true;
    statusMsg = "";
    try {
      const res = await invoke<{ status: string; api_key: string }>("register_client", { username });
      statusMsg = `Registered. Status: ${res.status}`;
      await loadConfig();
    } catch (e) {
      statusMsg = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function validate() {
    loading = true;
    statusMsg = "";
    try {
      validateResult = await invoke<ValidateResponse>("validate_client");
    } catch (e) {
      statusMsg = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function checkHealth() {
    loading = true;
    statusMsg = "";
    try {
      healthResult = await invoke<HealthResponse>("server_health");
    } catch (e) {
      statusMsg = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  loadConfig();
</script>

<main class="container">
  <h1>kstocks</h1>

  {#if serverConfig}
    <p class="muted">Server: {serverConfig.base_url}</p>
    <p class="muted">API key: {serverConfig.api_key ? "stored" : "not registered yet"}</p>
  {/if}

  <section>
    <h2>Register</h2>
    <form class="row" onsubmit={(e) => { e.preventDefault(); register(); }}>
      <input placeholder="username" bind:value={username} />
      <button type="submit" disabled={loading}>Register</button>
    </form>
  </section>

  <section>
    <h2>Validate</h2>
    <button onclick={validate} disabled={loading}>Check approval status</button>
    {#if validateResult}
      <p>approved: {validateResult.approved} · status: {validateResult.status}</p>
    {/if}
  </section>

  <section>
    <h2>Server health</h2>
    <button onclick={checkHealth} disabled={loading}>Check health</button>
    {#if healthResult}
      <p>db_connected: {healthResult.db_connected} · session: {healthResult.session_mode}</p>
    {/if}
  </section>

  {#if statusMsg}
    <p class="status">{statusMsg}</p>
  {/if}
</main>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  color: #0f0f0f;
  background-color: #f6f6f6;
}

.container {
  max-width: 640px;
  margin: 0 auto;
  padding: 4vh 1.5rem;
}

section {
  margin-top: 2rem;
  text-align: left;
}

.row {
  display: flex;
  gap: 0.5rem;
}

.muted {
  color: #666;
  font-size: 0.9em;
}

.status {
  margin-top: 1rem;
  font-weight: 500;
}

input, button {
  border-radius: 8px;
  border: 1px solid #ccc;
  padding: 0.5em 1em;
  font-size: 1em;
}

button {
  cursor: pointer;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }
  input, button {
    color: #fff;
    background-color: #0f0f0f98;
    border-color: #444;
  }
}
</style>
