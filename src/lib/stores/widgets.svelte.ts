// Dashboard widget layout store. Each widget is an independent chart with
// its own symbol + interval, so users can place e.g. NIFTY 1m next to
// BANKNIFTY 1d side by side. Persisted as a single JSON blob under
// `app_settings` (key below) — simple, and fine at this scale; the
// `layouts` table exists for a richer future workspace/layout manager.

import { getSetting, setSetting } from "$lib/api/tauri";
import type { ChartWidgetConfig } from "$lib/types";

const STORAGE_KEY = "ui.dashboard_widgets";

const DEFAULT_WIDGETS: ChartWidgetConfig[] = [
  { id: crypto.randomUUID(), symbol: "NIFTY", interval: "1m" },
];

let widgets = $state<ChartWidgetConfig[]>([]);
let initialized = false;

async function persist() {
  try {
    await setSetting(STORAGE_KEY, JSON.stringify(widgets));
  } catch {
    // Best-effort; layout still reflects current session state.
  }
}

export async function initWidgets(): Promise<void> {
  if (initialized) return;
  initialized = true;

  try {
    const stored = await getSetting(STORAGE_KEY);
    widgets = stored ? (JSON.parse(stored) as ChartWidgetConfig[]) : DEFAULT_WIDGETS;
  } catch {
    widgets = DEFAULT_WIDGETS;
  }
}

export function addWidget(symbol: string, interval: ChartWidgetConfig["interval"] = "1m"): void {
  widgets = [...widgets, { id: crypto.randomUUID(), symbol, interval }];
  void persist();
}

export function removeWidget(id: string): void {
  widgets = widgets.filter((w) => w.id !== id);
  void persist();
}

export function updateWidget(id: string, patch: Partial<Omit<ChartWidgetConfig, "id">>): void {
  widgets = widgets.map((w) => (w.id === id ? { ...w, ...patch } : w));
  void persist();
}

export const widgetsStore = {
  get list() {
    return widgets;
  },
};
