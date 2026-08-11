// Typed wrappers around `invoke("<command>", ...)` calls. Keeps command
// names and argument shapes in one place instead of scattered through
// components, and matches src-tauri/src/commands/* one-to-one.

import { invoke } from "@tauri-apps/api/core";
import type {
  ChartInterval,
  HealthResponse,
  IndexSnapshot,
  OhlcBar,
  OptionChainRow,
  OptionLeg,
  OptionLegBar,
  PaperTrade,
  RegisterResponse,
  ServerConfigView,
  ValidateResponse,
  Watchlist,
  WatchlistItem,
} from "$lib/types";

// ---- server.rs -------------------------------------------------------

/** Registers this client. Returns the API key exactly once — the caller
 * is responsible for showing it to the user and never persisting it in
 * plain UI state beyond that single reveal. */
export function registerClient(username: string): Promise<RegisterResponse> {
  return invoke("register_client", { username });
}

export function validateClient(): Promise<ValidateResponse> {
  return invoke("validate_client");
}

export function serverHealth(): Promise<HealthResponse> {
  return invoke("server_health");
}

export function fetchIndexOhlc(
  symbol: string,
  range: string,
  interval: string
): Promise<OhlcBar[]> {
  return invoke("fetch_index_ohlc", { symbol, range, interval });
}

export function setServerUrl(baseUrl: string): Promise<void> {
  return invoke("set_server_url", { baseUrl });
}

/** Sanitized: never includes the API key itself, only whether one is
 * stored. See commands/server.rs::ServerConfigView. */
export function getServerConfig(): Promise<ServerConfigView> {
  return invoke("get_server_config");
}

export function clearApiKey(): Promise<void> {
  return invoke("clear_api_key");
}

// ---- app_settings.rs ---------------------------------------------------

export function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting", { key });
}

export function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

// ---- market_data.rs ------------------------------------------------

export function getRecentIndexBars(
  symbol: string,
  interval: ChartInterval,
  limit = 200
): Promise<OhlcBar[]> {
  return invoke("get_recent_index_bars", { symbol, interval, limit });
}

export function getRecentOptionBars(
  symbol: string,
  expiry: string,
  strike: number,
  leg: OptionLeg,
  limit = 200
): Promise<OptionLegBar[]> {
  return invoke("get_recent_option_bars", { symbol, expiry, strike, leg, limit });
}

export function listOptionSymbols(): Promise<string[]> {
  return invoke("list_option_symbols");
}

/** Distinct index names with local data available — powers the index
 * picker in `SelectionPicker` (see market_data::list_index_symbols). */
export function listIndexSymbols(): Promise<string[]> {
  return invoke("list_index_symbols");
}

export function listOptionExpiries(symbol: string): Promise<string[]> {
  return invoke("list_option_expiries", { symbol });
}

export function listOptionStrikes(symbol: string, expiry: string): Promise<number[]> {
  return invoke("list_option_strikes", { symbol, expiry });
}

export function getOptionChain(symbol: string, expiry: string): Promise<OptionChainRow[]> {
  return invoke("get_option_chain", { symbol, expiry });
}

export function getAllIndexSnapshots(): Promise<IndexSnapshot[]> {
  return invoke("get_all_index_snapshots");
}

export function getIndexSnapshot(symbol: string): Promise<IndexSnapshot | null> {
  return invoke("get_index_snapshot", { symbol });
}

// ---- watchlists.rs -----------------------------------------------------

export function listWatchlists(): Promise<Watchlist[]> {
  return invoke("list_watchlists");
}

export function createWatchlist(name: string): Promise<number> {
  return invoke("create_watchlist", { name });
}

export function deleteWatchlist(id: number): Promise<void> {
  return invoke("delete_watchlist", { id });
}

export function listWatchlistItems(watchlistId: number): Promise<WatchlistItem[]> {
  return invoke("list_watchlist_items", { watchlistId });
}

export function addWatchlistItem(
  watchlistId: number,
  symbol: string,
  instrumentType: string
): Promise<number> {
  return invoke("add_watchlist_item", { watchlistId, symbol, instrumentType });
}

export function removeWatchlistItem(itemId: number): Promise<void> {
  return invoke("remove_watchlist_item", { itemId });
}

// ---- paper_trades.rs -----------------------------------------------------

export function openPaperTrade(args: {
  symbol: string;
  instrumentType: string;
  side: string;
  quantity: number;
  entryPrice: number;
  notes?: string;
}): Promise<number> {
  return invoke("open_paper_trade", args);
}

export function closePaperTrade(id: number, exitPrice: number): Promise<void> {
  return invoke("close_paper_trade", { id, exitPrice });
}

export function listPaperTrades(status?: string): Promise<PaperTrade[]> {
  return invoke("list_paper_trades", { status });
}