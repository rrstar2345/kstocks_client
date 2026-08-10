// Mirrors src-tauri/src/commands and their underlying storage/api types.
// Keep in sync with the Rust side by hand — no codegen yet.

export type ServerConfigView = {
  base_url: string;
  has_api_key: boolean;
};

export type RegisterResponse = {
  status: string;
  api_key: string;
};

export type ValidateResponse = {
  approved: boolean;
  status: string;
};

export type HealthResponse = {
  db_connected: boolean;
  last_index_tick_at: string | null;
  last_option_tick_at: string | null;
  aggregation_watermarks: unknown;
  session_mode: string;
};

export type OhlcBar = {
  bucket_start: string;
  open: number;
  high: number;
  low: number;
  close: number;
  tick_count: number;
};

export type Watchlist = {
  id: number;
  name: string;
  sort_order: number;
  created_at: string;
};

export type WatchlistItem = {
  id: number;
  watchlist_id: number;
  symbol: string;
  instrument_type: string;
  sort_order: number;
  added_at: string;
};

export type PaperTrade = {
  id: number;
  symbol: string;
  instrument_type: string;
  side: string;
  quantity: number;
  entry_price: number;
  exit_price: number | null;
  status: string;
  notes: string | null;
  opened_at: string;
  closed_at: string | null;
};

export type Theme = "light" | "dark";

export type ChartInterval = "1m" | "1d";

/** A single chart widget on the dashboard: its own symbol + interval. */
export type ChartWidgetConfig = {
  id: string;
  symbol: string;
  interval: ChartInterval;
};
