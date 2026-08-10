// Broker (Dhan) API key store. Deliberately in-memory only — a plain
// module-level `$state` variable, never written to `app_settings`, never
// sent to a Tauri command that persists it. It exists only for the
// current session and is forgotten the moment the app closes (or the
// page reloads). Production trading wiring will consume this later; for
// now it's a placeholder input on the Settings page.

export type BrokerId = "dhan";

let dhanApiKey = $state<string>("");

export function setDhanApiKey(value: string): void {
  dhanApiKey = value;
}

export function clearDhanApiKey(): void {
  dhanApiKey = "";
}

export const brokerStore = {
  get dhanApiKey() {
    return dhanApiKey;
  },
  get hasDhanApiKey() {
    return dhanApiKey.trim().length > 0;
  },
};
