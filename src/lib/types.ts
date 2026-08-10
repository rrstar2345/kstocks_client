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

export type OptionLeg = "CE" | "PE";

export type OptionLegBar = {
  bucket_start: string;
  open: number;
  high: number;
  low: number;
  close: number;
  tick_count: number;
};

export type OptionChainRow = {
  strike_price: number;
  ce_close: number | null;
  ce_volume: number | null;
  ce_oi_close: number | null;
  pe_close: number | null;
  pe_volume: number | null;
  pe_oi_close: number | null;
};

export type IndexSnapshot = {
  index_name: string;
  current_price: number | null;
  change: number | null;
  per_change: number | null;
  open: number | null;
  low: number | null;
  high: number | null;
  previous_close: number | null;
  time: string;
};

/** What a widget/list-view is currently showing: an index, or one leg of
 * a specific option-chain contract. */
export type InstrumentSelection =
  | { kind: "index"; symbol: string }
  | { kind: "option"; symbol: string; expiry: string; strike: number; leg: OptionLeg };

/** Which visual a widget currently renders: candlestick chart, or a
 * tabular/list view (option chain table, or index watch-style row). */
export type WidgetView = "chart" | "list";

/** A single dashboard widget: its view mode + what instrument it targets.
 * `interval` only applies in chart mode. */
export type ChartWidgetConfig = {
  id: string;
  view: WidgetView;
  selection: InstrumentSelection;
  interval: ChartInterval;
};

export type TradingMode = "paper" | "live";
