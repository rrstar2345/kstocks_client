// Theme store using Svelte 5 runes. Module-level `$state` acts as a
// singleton store — imported anywhere, always the same reactive value.
// Persisted to the local `app_settings` table so it survives restarts.

import { getSetting, setSetting } from "$lib/api/tauri";
import type { Theme } from "$lib/types";

const STORAGE_KEY = "ui.theme";

let theme = $state<Theme>("dark");
let initialized = false;

function applyToDocument(value: Theme) {
  if (typeof document !== "undefined") {
    document.documentElement.setAttribute("data-theme", value);
  }
}

/** Load the persisted theme (falls back to OS preference, then dark) and
 * apply it. Safe to call once at app startup (see +layout.svelte). */
export async function initTheme(): Promise<void> {
  if (initialized) return;
  initialized = true;

  try {
    const stored = await getSetting(STORAGE_KEY);
    if (stored === "light" || stored === "dark") {
      theme = stored;
    } else if (typeof window !== "undefined" && window.matchMedia) {
      theme = window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
    }
  } catch {
    // Local DB not reachable yet (shouldn't happen post-setup) — keep default.
  }

  applyToDocument(theme);
}

export function getTheme(): Theme {
  return theme;
}

export async function setTheme(value: Theme): Promise<void> {
  theme = value;
  applyToDocument(value);
  try {
    await setSetting(STORAGE_KEY, value);
  } catch {
    // Best-effort persistence; theme still applies for this session.
  }
}

export async function toggleTheme(): Promise<void> {
  await setTheme(theme === "dark" ? "light" : "dark");
}

/** Reactive accessor for use in component markup, e.g. `{themeStore.value}`. */
export const themeStore = {
  get value() {
    return theme;
  },
};
