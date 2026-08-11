// Dashboard widget layout store. Each widget is an independent chart with
// its own symbol + interval, so users can place e.g. NIFTY 1m next to
// BANKNIFTY 1d side by side. Persisted as a single JSON blob under
// `app_settings` (key below) — simple, and fine at this scale; the
// `layouts` table exists for a richer future workspace/layout manager.

import { getSetting, setSetting } from "$lib/api/tauri";
import type { ChartWidgetConfig } from "$lib/types";

const STORAGE_KEY = "ui.dashboard_widgets";
/** Grid positions: w1) left-top, w2) left-bottom, w3) right-bottom,
 * w4) right-top. Order in the array == this fixed slot order; adding a
 * widget appends the next slot, removing compacts the remaining ones
 * back into slots 1..n so the layout rules (w1..w4) always apply to
 * however many widgets currently exist. */
export const MAX_WIDGETS = 4;

const DEFAULT_WIDGETS: ChartWidgetConfig[] = [
  {
    id: crypto.randomUUID(),
    view: "chart",
    selection: { kind: "index", symbol: "NIFTY" },
    interval: "1m",
  },
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

/** Guards against corrupted/older-schema persisted widgets (e.g. a
 * `selection` missing `kind`, or a stale shape from before the
 * discriminated union existed) crashing the whole grid at render time.
 * Any entry that doesn't look like a valid widget is dropped rather than
 * silently propagated into components that assume it's well-formed. */
function isValidWidget(w: unknown): w is ChartWidgetConfig {
  if (!w || typeof w !== "object") return false;
  const cfg = w as Partial<ChartWidgetConfig>;
  if (typeof cfg.id !== "string") return false;
  if (cfg.view !== "chart" && cfg.view !== "list") return false;
  const sel = cfg.selection as { kind?: string; symbol?: unknown } | undefined;
  if (!sel || typeof sel !== "object") return false;
  if (sel.kind === "index") return typeof sel.symbol === "string";
  if (sel.kind === "option") return typeof sel.symbol === "string";
  return false;
}

function sanitize(raw: unknown): ChartWidgetConfig[] {
  if (!Array.isArray(raw)) return DEFAULT_WIDGETS;
  const valid = raw.filter(isValidWidget).slice(0, MAX_WIDGETS);
  return valid.length > 0 ? valid : DEFAULT_WIDGETS;
}

export async function initWidgets(): Promise<void> {
  if (initialized) return;
  initialized = true;

  try {
    const stored = await getSetting(STORAGE_KEY);
    widgets = stored ? sanitize(JSON.parse(stored)) : DEFAULT_WIDGETS;
  } catch {
    widgets = DEFAULT_WIDGETS;
  }
}

export function addWidget(): void {
  if (widgets.length >= MAX_WIDGETS) return;
  widgets = [
    ...widgets,
    {
      id: crypto.randomUUID(),
      view: "chart",
      selection: { kind: "index", symbol: "NIFTY" },
      interval: "1m",
    },
  ];
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
  get canAdd() {
    return widgets.length < MAX_WIDGETS;
  },
};