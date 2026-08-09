# kstocks_client — Progress

Client repo: https://github.com/rrstar2345/kstocks_client
Server repo (reused for patterns/API contract): https://github.com/rrstar2345/kstocks-server

This client was an empty Tauri v2 + Svelte + TypeScript shell. This pass
added the Rust backend foundation: app folders, local SQLite storage, and
an HTTP client for the kstocks-server API, all wired into Tauri commands.

**Rust toolchain was not available in the sandbox** (no `cargo`, and
`rustup.rs` is blocked by the network allowlist), so none of this has been
compiled yet. Treat it as a careful desk-check, not a verified build —
run `cargo build` (or `pnpm tauri dev`) first thing next session.

---

## What's done

### App folders / settings (`src-tauri/src/settings.rs`)
- Mirrors the server's `setup_app_folders()` pattern (`dirs::data_local_dir()`
  → `~/.kstocks/`, falls back to cwd).
- `db/` and `logs/` subdirs created on startup.
- Client uses its own `settings_client.json` (server uses
  `settings_server.json`) so both processes can run on the same machine
  without fighting over one file.
- `AppConfig` = `DatabaseConfig` (path to `kstocks.db`) + `ServerConfig`
  (`base_url`, defaults to `http://localhost:8787`; `api_key`, `None` until
  `/register` succeeds).
- `load_or_create_config` / `save_config` — same read-or-write-defaults
  pattern as the server's `load_or_create_config`.

### Local storage — `kstocks.db` (`src-tauri/src/storage/`)
- `db.rs` — sqlx `SqlitePool` init, `create_if_missing`, WAL +
  `synchronous=NORMAL` pragmas (same as server), plus `foreign_keys=ON`.
- Schema created on every startup (idempotent `CREATE TABLE IF NOT EXISTS`):
  - `app_settings` (key/value, for arbitrary UI prefs without migrations)
  - `watchlists` / `watchlist_items` (named symbol groups)
  - `layouts` (opaque JSON blob per saved workspace layout — frontend owns
    the shape)
  - `paper_trades` (simulated orders: symbol, side, qty, entry/exit price,
    status, timestamps)
- Query modules: `settings_store.rs`, `watchlists.rs`, `paper_trades.rs` —
  each a thin `sqlx::query`/`query_as` wrapper, following the server's
  `users/mod.rs` style (plain async fns taking `&SqlitePool`).

### Server API client (`src-tauri/src/api/client.rs`)
Reflects the endpoints documented in kstocks-server's README exactly:
- `POST /register` → `RegisterResponse { status, api_key }`
- `GET /validate` (bearer auth) → `ValidateResponse { approved, status }`
- `GET /health` (no auth) → `HealthResponse`
- `GET /ohlc/index` (bearer auth) → `Vec<IndexBar>`
- `GET /ohlc/option` (bearer auth) → `Vec<OptionBar>`
- `base_url` and `api_key` are mutable at runtime (`set_base_url`,
  `set_api_key`) so switching from localhost to a deployed server IP
  doesn't need a restart.
- Errors modeled as `ApiError` (`thiserror`), distinguishing transport
  failures, server error bodies (`{"error": "..."}`), and "no API key yet."

### Tauri commands (`src-tauri/src/commands/`)
- `server.rs` — `register_client` (persists returned key to
  `settings_client.json` + updates in-memory client), `validate_client`,
  `server_health`, `fetch_index_ohlc`, `fetch_option_ohlc`,
  `set_server_url`, `get_server_config`.
- `watchlists.rs` — CRUD over local watchlists/items.
- `paper_trades.rs` — open/close/list simulated trades.
- All registered in `invoke_handler![...]` in `lib.rs`.

### App state (`src-tauri/src/state.rs`)
- `AppState { db: SqlitePool, paths: AppPaths, config: RwLock<AppConfig>,
  api_client: RwLock<ApiClient> }`, `app.manage()`'d in `lib.rs`'s `setup()`
  hook. DB pool + config are initialized synchronously at startup via
  `tauri::async_runtime::block_on`.

### Frontend (`src/routes/+page.svelte`)
- Replaced the default Tauri/Vite/Svelte demo page.
- Minimal UI: register (username → API key), validate (approval status),
  health check. Enough to confirm the Rust↔Svelte wiring works end to end
  once compiled.
- Not yet wired: watchlists, paper trades, layouts, charting — the Rust
  commands exist but have no frontend calling them yet.

### Cargo.toml additions
Added to match server where the client needs the same capability:
`dirs`, `tokio` (`full`), `anyhow`, `sqlx` (sqlite/runtime-tokio/macros/chrono),
`chrono`, `tracing` + `tracing-subscriber`, `reqwest` (json), `thiserror`.

---

## Not done / next pass

1. **Compile it.** Nothing in this pass has been built. Run `cargo build`
   in `src-tauri/`, fix whatever surfaces (likely candidates: sqlx offline
   mode / `DATABASE_URL` for compile-time macros — this uses runtime
   `query`/`query_as`, not `query!`, specifically to avoid needing a
   pre-existing DB at compile time, but double-check), then `pnpm install`
   + `pnpm tauri dev` for the full app.
2. **Frontend for watchlists / paper trades.** Commands exist
   (`list_watchlists`, `create_watchlist`, `add_watchlist_item`,
   `open_paper_trade`, `list_paper_trades`, etc.) but no Svelte UI calls
   them yet.
3. **`app_settings` store has no commands.** `storage/settings_store.rs`
   (`get`/`set`) isn't exposed via `#[tauri::command]` yet — add when the
   UI needs its first persisted preference (theme, last workspace, etc.).
4. **Layouts table is unused.** Schema exists; no queries module, no
   commands, no UI. Wire up once the chart/workspace layout system exists.
5. **No live market data path yet.** Per `CONTEXT.md`, the desktop app is
   expected to connect directly to NSE's WSS for live/in-progress candles
   (the server API only serves closed historical bars for gap-fill). None
   of the Market Data Engine / WebSocket Manager / event bus from
   `CONTEXT.md`'s architecture list exists yet — this pass only covers the
   historical/REST side (`/ohlc/*`) and local persistence.
6. **No Broker Adapter Layer, Order Manager, Strategy Engine, Indicator
   Engine, Chart Engine, or Plugin System** — all still just names in
   `CONTEXT.md`. Foundation (storage + API client + Tauri command pattern)
   is now in place for these to build on.
7. **`/register` UX gap.** After registering, the account is `pending`
   until an admin approves it server-side (see server README's admin CLI).
   The current UI shows raw status but doesn't poll `/validate` or guide
   the user through waiting for approval.
8. **Server URL is not yet user-editable in the UI.** `set_server_url`
   command exists (for switching from `localhost:8787` to a deployed IP)
   but there's no settings screen calling it.
9. **No tests.** Neither the server nor this client pass has any.
10. **No error/loading state polish** in the Svelte page beyond a single
    shared `statusMsg`/`loading` — fine for a smoke test, not production UI.

## Useful references for next pass
- `kstocks-server/README.md` — full API contract (request/response shapes,
  error codes, auth model) that `api/client.rs` mirrors.
- `kstocks-server/src/settings.rs`, `src/storage/ticks.rs`,
  `src/users/mod.rs` — the patterns this client's `settings.rs`,
  `storage/db.rs`, and query modules were modeled on.
- `CONTEXT.md` (this repo) — overall architecture target; only the
  Local Storage module and a slice of the API/auth surface are built so far.