// Typed wrappers around the live-tick Tauri events emitted by
// src-tauri/src/market/events.rs. Components use these instead of polling
// commands so the UI updates as fast as the WSS delivers ticks.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { IndexTick, OptionTick } from "$lib/types";

const INDEX_TICK_EVENT = "index-tick";
const OPTION_TICK_EVENT = "option-tick";

/** Subscribe to every live index tick. Returns the unlisten function —
 * callers must call it (e.g. in `onDestroy`/`$effect` cleanup) or the
 * listener leaks for the lifetime of the webview. */
export function onIndexTick(handler: (tick: IndexTick) => void): Promise<UnlistenFn> {
  return listen<IndexTick>(INDEX_TICK_EVENT, (event) => handler(event.payload));
}

/** Subscribe to every live option-chain tick. Same cleanup contract as
 * `onIndexTick`. */
export function onOptionTick(handler: (tick: OptionTick) => void): Promise<UnlistenFn> {
  return listen<OptionTick>(OPTION_TICK_EVENT, (event) => handler(event.payload));
}