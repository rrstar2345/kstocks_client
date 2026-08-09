//! Backfill: pulls historical `index_ohlc_1m` / `index_ohlc_1d` bars from
//! kstocks-server's read-only `/ohlc/index` API and upserts them into the
//! client's own local tables, so the chart has continuity across periods
//! the app wasn't running to stream live (app closed overnight, laptop
//! asleep, etc.).
//!
//! Deliberately narrow scope, matching the server's role as described in
//! its README: "the desktop app's job once it connects directly to NSE's
//! WSS for gap-fill" — this module IS that gap-fill, and only that.
//! It never touches raw ticks (`index_ticks`/`option_ticks`) or
//! `aggregation_state`; those are exclusively owned by the local streaming
//! + aggregation pipeline (`market::streamers`, `storage::ohlc`). Option
//! backfill is not wired in yet — see PROGRESS.md.

use anyhow::{anyhow, Result};
use sqlx::SqlitePool;
use tracing::{error, info, warn};

use crate::api::ApiClient;

/// Upsert one page of `/ohlc/index` bars into `index_ohlc_1m`. Same
/// `ON CONFLICT ... DO UPDATE` shape as the server's own aggregation, so a
/// re-run (e.g. overlapping ranges) is always safe to repeat.
async fn upsert_index_1m_bars(
    pool: &SqlitePool,
    index_name: &str,
    bars: &[crate::api::client::IndexBar],
) -> Result<usize> {
    let mut tx = pool.begin().await?;
    let mut count = 0;

    for bar in bars {
        sqlx::query(
            r#"
            INSERT INTO index_ohlc_1m (index_name, bucket_start, open, high, low, close, tick_count)
            VALUES (?, ?, ?, ?, ?, ?, 0)
            ON CONFLICT (index_name, bucket_start) DO UPDATE SET
                open = excluded.open,
                high = excluded.high,
                low = excluded.low,
                close = excluded.close
            "#,
        )
        .bind(index_name)
        .bind(&bar.bucket_start)
        .bind(bar.open)
        .bind(bar.high)
        .bind(bar.low)
        .bind(bar.close)
        .execute(&mut *tx)
        .await?;
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

/// Upsert one page of `/ohlc/index` bars into `index_ohlc_1d`, same shape.
async fn upsert_index_1d_bars(
    pool: &SqlitePool,
    index_name: &str,
    bars: &[crate::api::client::IndexBar],
) -> Result<usize> {
    let mut tx = pool.begin().await?;
    let mut count = 0;

    for bar in bars {
        sqlx::query(
            r#"
            INSERT INTO index_ohlc_1d (index_name, bucket_start, open, high, low, close, tick_count)
            VALUES (?, ?, ?, ?, ?, ?, 0)
            ON CONFLICT (index_name, bucket_start) DO UPDATE SET
                open = excluded.open,
                high = excluded.high,
                low = excluded.low,
                close = excluded.close
            "#,
        )
        .bind(index_name)
        .bind(&bar.bucket_start)
        .bind(bar.open)
        .bind(bar.high)
        .bind(bar.low)
        .bind(bar.close)
        .execute(&mut *tx)
        .await?;
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

/// Fetch and upsert `index_ohlc_1m` bars for one symbol/range/interval.
/// `interval` should be a 1-minute-tier interval valid for the server's
/// `/ohlc/index` (see server README's range/interval table, e.g. `1m` for
/// `range=1d`).
pub async fn backfill_index_1m(
    api_client: &ApiClient,
    pool: &SqlitePool,
    symbol: &str,
    range: &str,
    interval: &str,
) -> Result<usize> {
    let bars = api_client
        .ohlc_index(symbol, range, interval)
        .await
        .map_err(|e| anyhow!("backfill fetch failed for {} {}/{}: {}", symbol, range, interval, e))?;

    let n = upsert_index_1m_bars(pool, symbol, &bars).await?;
    info!("Backfilled {} index_ohlc_1m bar(s) for {} ({}/{})", n, symbol, range, interval);
    Ok(n)
}

/// Fetch and upsert `index_ohlc_1d` bars for one symbol/range/interval
/// (e.g. `range=1y`, `interval=1mo` or similar daily-tier combination —
/// see server README).
pub async fn backfill_index_1d(
    api_client: &ApiClient,
    pool: &SqlitePool,
    symbol: &str,
    range: &str,
    interval: &str,
) -> Result<usize> {
    let bars = api_client
        .ohlc_index(symbol, range, interval)
        .await
        .map_err(|e| anyhow!("backfill fetch failed for {} {}/{}: {}", symbol, range, interval, e))?;

    let n = upsert_index_1d_bars(pool, symbol, &bars).await?;
    info!("Backfilled {} index_ohlc_1d bar(s) for {} ({}/{})", n, symbol, range, interval);
    Ok(n)
}

/// Run a backfill pass for a set of index symbols on app startup: recent 1m
/// history (gap-fill since last close) plus daily history. Best-effort per
/// symbol — one symbol's failure (e.g. not yet approved, network down)
/// doesn't stop the others.
pub async fn run_startup_backfill(api_client: &ApiClient, pool: &SqlitePool, symbols: &[String]) {
    for symbol in symbols {
        if let Err(e) = backfill_index_1m(api_client, pool, symbol, "1d", "1m").await {
            warn!("Startup 1m backfill failed for {}: {}", symbol, e);
        }
        if let Err(e) = backfill_index_1d(api_client, pool, symbol, "3mo", "1d").await {
            warn!("Startup 1d backfill failed for {}: {}", symbol, e);
        }
    }
}

/// Errors are logged, never propagated — backfill is a best-effort
/// convenience layered on top of local streaming, never a hard dependency
/// for the app to function (per CONTEXT.md: "Keep the UI completely
/// independent of market data providers").
#[allow(dead_code)]
fn log_backfill_error(context: &str, e: &anyhow::Error) {
    error!("{}: {}", context, e);
}