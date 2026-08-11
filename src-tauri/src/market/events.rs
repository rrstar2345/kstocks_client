//! Tauri event emission for live market ticks. The streamers already write
//! every tick to SQLite (via the batched writers in `storage::ticks`); this
//! module additionally pushes each tick straight to the frontend so the UI
//! can update in real time instead of polling the DB.
//!
//! Event payloads are the same row types used for storage
//! (`storage::IndexTickRow` / `storage::OptionTickRow`), serialized as-is,
//! so the wire shape and the DB schema can't drift apart silently.

use tauri::{AppHandle, Emitter};
use tracing::warn;

use crate::storage::{IndexTickRow, OptionTickRow};

/// Emitted once per non-heartbeat index tick, payload = `IndexTickRow`.
pub const INDEX_TICK_EVENT: &str = "index-tick";

/// Emitted once per non-heartbeat option-chain tick, payload =
/// `OptionTickRow`.
pub const OPTION_TICK_EVENT: &str = "option-tick";

pub fn emit_index_tick(app: &AppHandle, row: &IndexTickRow) {
    if let Err(e) = app.emit(INDEX_TICK_EVENT, row) {
        warn!("Failed to emit {} event: {}", INDEX_TICK_EVENT, e);
    }
}

pub fn emit_option_tick(app: &AppHandle, row: &OptionTickRow) {
    if let Err(e) = app.emit(OPTION_TICK_EVENT, row) {
        warn!("Failed to emit {} event: {}", OPTION_TICK_EVENT, e);
    }
}