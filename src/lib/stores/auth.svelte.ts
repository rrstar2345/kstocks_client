// Auth / server-connection store. Deliberately never holds the API key
// itself in reactive state — only whether one is stored on the backend
// (`has_api_key`) and the last-known approval status. The key is secret
// and confidential: it's shown to the user exactly once, right after
// `register_client` returns it, in a one-time reveal UI, then discarded.

import { clearApiKey, getServerConfig, validateClient } from "$lib/api/tauri";

type AuthStatus = "unknown" | "unregistered" | "pending" | "approved" | "rejected";

let baseUrl = $state("");
let hasApiKey = $state(false);
let status = $state<AuthStatus>("unknown");
let loading = $state(false);

export async function refreshServerConfig(): Promise<void> {
  const cfg = await getServerConfig();
  baseUrl = cfg.base_url;
  hasApiKey = cfg.has_api_key;
  if (!hasApiKey) status = "unregistered";
}

export async function refreshValidation(): Promise<void> {
  if (!hasApiKey) {
    status = "unregistered";
    return;
  }
  loading = true;
  try {
    const res = await validateClient();
    status = res.approved ? "approved" : "pending";
  } catch {
    status = "unknown";
  } finally {
    loading = false;
  }
}

export async function logOut(): Promise<void> {
  await clearApiKey();
  hasApiKey = false;
  status = "unregistered";
}

export const authStore = {
  get baseUrl() {
    return baseUrl;
  },
  get hasApiKey() {
    return hasApiKey;
  },
  get status() {
    return status;
  },
  get loading() {
    return loading;
  },
  get isReady() {
    return hasApiKey && status === "approved";
  },
};
