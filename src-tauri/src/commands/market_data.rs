//! Tauri commands for reading locally-aggregated OHLC data (fast,
//! offline-capable path used by chart widgets). For explicit historical
//! backfill from the server, see `commands::server::fetch_index_ohlc`.

use tauri::State;

use crate::state::AppState;
use crate::storage::ohlc::{self, OhlcBar};

/// Recent bars for a single index/symbol, oldest-first, ready to hand
/// straight to a chart component.
#[tauri::command]
pub async fn get_recent_index_bars(
    state: State<'_, AppState>,
    symbol: String,
    interval: String,
    limit: Option<i64>,
) -> Result<Vec<OhlcBar>, String> {
    ohlc::recent_index_bars(&state.db, &symbol, &interval, limit.unwrap_or(200))
        .await
        .map_err(|e| e.to_string())
}
