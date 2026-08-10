mod api;
mod commands;
mod market;
mod settings;
mod state;
mod stats;
mod storage;

use tauri::Manager;
use tracing::{error, info, warn};

use market::market_clock;
use market::{streamers, symbols};
use state::AppState;
use stats::new_shared_stats;
use storage::{ohlc as aggregation, retention};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let paths = settings::setup_app_folders().expect("failed to set up app folders");
            let config = settings::load_or_create_config(&paths).expect("failed to load settings");

            info!("App root: {}", paths.root.display());
            info!("Local DB: {}", config.database.connection_string);
            info!("Server URL: {}", config.server.base_url);

            // Everything below is async; run it to completion during setup
            // (same `block_on` pattern as the DB init) so the app is fully
            // wired — DB, streamers, aggregation, retention, backfill —
            // before any Tauri command fires. Mirrors kstocks-server's
            // main.rs wiring order.
            let (db, stats, session) = tauri::async_runtime::block_on(async {
                let db = storage::init_pool(&config.database)
                    .await
                    .expect("failed to initialize local database");
                info!("SQLite connected and schema verified");

                let stats = new_shared_stats();

                // Market-hours session state, driven by NSE's own IST clock.
                let session = market_clock::new_shared_session_state();
                market_clock::refresh_if_stale(&config).await;
                {
                    let mut s = stats.write().await;
                    s.session_mode_label = session.mode().await.label().to_string();
                }

                // Background supervisor: periodically re-syncs the NSE time
                // offset and re-evaluates Active/Idle mode.
                let supervisor_config = config.clone();
                let supervisor_stats = stats.clone();
                let supervisor_session = session.clone();
                tokio::spawn(async move {
                    loop {
                        market_clock::refresh_if_stale(&supervisor_config).await;
                        let mode = supervisor_session
                            .tick(supervisor_config.market_runtime.inactive_switch_after_secs)
                            .await;
                        {
                            let mut s = supervisor_stats.write().await;
                            s.session_mode_label = mode.label().to_string();
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    }
                });

                // Batched writers (one per tick type), each backed by an
                // mpsc channel so streamers never block on the DB.
                let (index_tx, _index_writer_handle) = storage::start_index_tick_writer(
                    db.clone(),
                    config.database.clone(),
                    stats.clone(),
                );
                let (option_tx, _option_writer_handle) = storage::start_option_tick_writer(
                    db.clone(),
                    config.database.clone(),
                    stats.clone(),
                );

                // Resolve F&O symbols + nearest expiry dynamically from NSE.
                info!("Resolving F&O symbols and nearest expiries...");
                match symbols::resolve_symbol_expiries(&config).await {
                    Ok(symbol_expiries) => {
                        for se in &symbol_expiries {
                            info!("Will stream options for {} / {}", se.symbol, se.expiry);
                        }

                        // 1 indices streamer.
                        tokio::spawn(streamers::indices::run(
                            config.clone(),
                            index_tx.clone(),
                            stats.clone(),
                            session.clone(),
                        ));

                        // Up to 5 option streamers (one per resolved F&O symbol).
                        for se in symbol_expiries {
                            tokio::spawn(streamers::options::run(
                                config.clone(),
                                se.symbol,
                                se.expiry,
                                option_tx.clone(),
                                stats.clone(),
                                session.clone(),
                            ));
                        }
                    }
                    Err(e) => {
                        // Best-effort: if NSE's symbol endpoints are
                        // unreachable at startup (offline, NSE maintenance),
                        // the app still runs — just without live option
                        // streaming until the next restart. The indices
                        // streamer doesn't depend on this resolution, so
                        // start it regardless.
                        error!("Failed to resolve F&O symbols/expiries, skipping option streamers: {}", e);
                        tokio::spawn(streamers::indices::run(
                            config.clone(),
                            index_tx.clone(),
                            stats.clone(),
                            session.clone(),
                        ));
                    }
                }

                // Aggregation: 1-minute OHLC bars every `run_interval_secs`,
                // plus a once-daily 1m -> 1d rollup after market close.
                aggregation::spawn_1m_aggregation_loop(db.clone(), config.aggregation.run_interval_secs);
                aggregation::spawn_daily_rollup_loop(db.clone());

                // Retention: once-daily purge (raw ticks + 1m tiers +
                // expired options), weekly VACUUM.
                retention::spawn_retention_loop(db.clone(), config.retention.clone());

                // Best-effort startup backfill of index_ohlc_1m/1d from the
                // server, gap-filling any period the app wasn't running to
                // stream live. Only runs if an API key is already stored
                // (i.e. registration previously completed); otherwise
                // skipped silently — the frontend drives register/validate.
                if config.server.api_key.is_some() {
                    let api_client = api::ApiClient::new(
                        config.server.base_url.clone(),
                        config.server.api_key.clone(),
                    );
                    let backfill_db = db.clone();
                    // TODO: source this symbol list from the user's
                    // watchlists once that UI exists, rather than a fixed
                    // set. See PROGRESS.md.
                    let default_symbols = vec!["NIFTY".to_string(), "BANKNIFTY".to_string()];
                    let option_backfill_client = api_client.clone();
                    let option_backfill_db = db.clone();
                    tokio::spawn(async move {
                        storage::backfill::run_startup_backfill(&api_client, &backfill_db, &default_symbols).await;
                        // Option backfill runs after index backfill so it
                        // reads from whatever symbols/expiries/strikes the
                        // local streamers have already started populating.
                        storage::backfill::run_startup_option_backfill(&option_backfill_client, &option_backfill_db).await;
                    });
                } else {
                    warn!("No API key stored yet; skipping startup backfill until registration completes.");
                }

                (db, stats, session)
            });

            app.manage(AppState::new(db, paths, config, stats, session));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::server::register_client,
            commands::server::validate_client,
            commands::server::server_health,
            commands::server::fetch_index_ohlc,
            commands::server::fetch_option_ohlc,
            commands::server::set_server_url,
            commands::server::get_server_config,
            commands::server::clear_api_key,
            commands::app_settings::get_setting,
            commands::app_settings::set_setting,
            commands::market_data::get_recent_index_bars,
            commands::market_data::get_recent_option_bars,
            commands::market_data::list_option_symbols,
            commands::market_data::list_option_expiries,
            commands::market_data::list_option_strikes,
            commands::market_data::get_option_chain,
            commands::market_data::get_all_index_snapshots,
            commands::market_data::get_index_snapshot,
            commands::watchlists::list_watchlists,
            commands::watchlists::create_watchlist,
            commands::watchlists::delete_watchlist,
            commands::watchlists::list_watchlist_items,
            commands::watchlists::add_watchlist_item,
            commands::watchlists::remove_watchlist_item,
            commands::paper_trades::open_paper_trade,
            commands::paper_trades::close_paper_trade,
            commands::paper_trades::list_paper_trades,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}