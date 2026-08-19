//! Hybrid runner: initializes Tauri backend with streamers and DB,
//! then runs egui as the main GUI on the main thread.
//!
//! egui requires the main thread for its event loop on Linux/X11.

use std::sync::mpsc::channel;
use eframe::egui;
use tracing::info;

use kstocks_lib::gui::IndexTickRow;
use kstocks_lib::market::{market_clock, streamers, symbols};
use kstocks_lib::storage;

/// Run the full app: init backend, then run egui on main thread
pub fn run() {
    tracing_subscriber::fmt::init();

    let (tx, rx) = channel::<IndexTickRow>();

    // Spawn backend initialization in background (tokio task)
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            init_backend_with_ticks(tx).await;
            // Keep runtime alive
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        });
    });

    // Small delay for backend to initialize
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Run egui on main thread (required for X11/Wayland)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 700.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "kstocks",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(kstocks_lib::gui::CandlestickApp::new(rx)))
        }),
    );
}

/// Initialize the backend: DB, streamers, aggregation, retention
async fn init_backend_with_ticks(gui_tx: std::sync::mpsc::Sender<IndexTickRow>) {
    let settings = kstocks_lib::settings::setup_app_folders()
        .expect("failed to set up app folders");
    let config = kstocks_lib::settings::load_or_create_config(&settings)
        .expect("failed to load settings");

    info!("App root: {}", settings.root.display());
    info!("Local DB: {}", config.database.connection_string);
    info!("Server URL: {}", config.server.base_url);

    let db = storage::init_pool(&config.database)
        .await
        .expect("failed to initialize local database");
    info!("SQLite connected and schema verified");

    let stats = kstocks_lib::stats::new_shared_stats();
    let session = market_clock::new_shared_session_state();
    market_clock::refresh_if_stale(&config).await;
    {
        let mut s = stats.write().await;
        s.session_mode_label = session.mode().await.label().to_string();
    }

    // Market clock supervisor
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

    // Tick writers
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

    // Create a custom AppHandle wrapper that just holds the GUI tx
    let app_handle = GuiAppHandle { tx: gui_tx };

    // Resolve F&O symbols
    info!("Resolving F&O symbols and nearest expiries...");
    match symbols::resolve_symbol_expiries(&config).await {
        Ok(symbol_expiries) => {
            for se in &symbol_expiries {
                info!("Will stream options for {} / {}", se.symbol, se.expiry);
            }

            // Index streamer
            let handle = app_handle.clone();
            let cfg = config.clone();
            let idx_tx = index_tx.clone();
            let st = stats.clone();
            let ses = session.clone();
            tokio::spawn(async move {
                streamers::indices::run(cfg, idx_tx, st, ses, handle).await;
            });

            // Option streamers
            for se in symbol_expiries {
                let handle = app_handle.clone();
                let cfg = config.clone();
                let opt_tx = option_tx.clone();
                let st = stats.clone();
                let ses = session.clone();
                let sym = se.symbol.clone();
                let exp = se.expiry.clone();
                tokio::spawn(async move {
                    streamers::options::run(cfg, sym, exp, opt_tx, st, ses, handle).await;
                });
            }
        }
        Err(e) => {
            info!("Failed to resolve F&O symbols, indices only: {}", e);
            let handle = app_handle.clone();
            let cfg = config.clone();
            let idx_tx = index_tx.clone();
            let st = stats.clone();
            let ses = session.clone();
            tokio::spawn(async move {
                streamers::indices::run(cfg, idx_tx, st, ses, handle).await;
            });
        }
    }

    // Aggregation & retention
    storage::ohlc::spawn_1m_aggregation_loop(db.clone(), config.aggregation.run_interval_secs);
    storage::ohlc::spawn_daily_rollup_loop(db.clone());
    storage::retention::spawn_retention_loop(db.clone(), config.retention.clone());

    // Backfill
    if config.server.api_key.is_some() {
        let api_client = kstocks_lib::api::ApiClient::new(
            config.server.base_url.clone(),
            config.server.api_key.clone(),
        );
        let backfill_db = db.clone();
        let default_symbols = vec!["NIFTY".to_string(), "BANKNIFTY".to_string()];
        let option_backfill_client = api_client.clone();
        let option_backfill_db = db.clone();
        tokio::spawn(async move {
            storage::backfill::run_startup_backfill(&api_client, &backfill_db, &default_symbols).await;
            storage::backfill::run_startup_option_backfill(&option_backfill_client, &option_backfill_db).await;
        });
    }

    info!("Backend initialized successfully");
}

/// Wrapper to satisfy streamers' AppHandle requirement
#[derive(Clone)]
pub struct GuiAppHandle {
    tx: std::sync::mpsc::Sender<IndexTickRow>,
}

// Minimal trait implementation for Manager (needed by streamers)
impl tauri::Manager for GuiAppHandle {
    // Most methods are unneeded, but we need to impl the trait
}