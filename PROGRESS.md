# kstocks_client — Progress

## Latest pass — frontend restructure + API key fix

**Not compiled/built** (per instructions, `cargo check`/`cargo build` and
`pnpm install`/`pnpm tauri dev` were intentionally left for manual run).
Treat as a careful desk-check.

### Fixed: API key was visible in the UI
- `commands::server::get_server_config` used to return the raw
  `ServerConfig` (including `api_key: Option<String>`) straight to the
  frontend, and the old single-page UI printed "API key: stored" but the
  value was sitting in reactive state and Tauri IPC in plaintext.
- Now returns a new `ServerConfigView { base_url, has_api_key }` —
  the key itself never crosses the IPC boundary after registration.
- `register_client` still returns the key once (`RegisterResponse.api_key`,
  unchanged, required so the user can see/copy it at all). The new
  `/register` route holds it in a route-local `$state` only, shows it in a
  one-time "acknowledge and hide" panel, and never stores it in any shared
  store or persists it client-side.
- Added `clear_api_key` command ("forget API key" / re-register flow).

### Backend additions (`src-tauri/src/`)
- `commands/app_settings.rs` — `get_setting`/`set_setting`, exposing the
  already-existing (but previously unwired) `storage/settings_store.rs`
  get/set. Used by the frontend for theme + dashboard layout persistence.
- `commands/market_data.rs` — `get_recent_index_bars(symbol, interval,
  limit)`, backed by a new `storage::ohlc::recent_index_bars()` query
  (new `OhlcBar` struct). Reads straight from the locally-populated
  `index_ohlc_1m` / `index_ohlc_1d` tables — fast, offline-capable, no
  network round-trip — as the read path for chart widgets. The existing
  REST `fetch_index_ohlc` remains for explicit historical backfill.
- `commands::server::clear_api_key` (see above).
- All four new commands registered in `lib.rs`'s `invoke_handler!`.

### Frontend: monolith → routes + components + runes
Previously a single `+page.svelte` with inline `<style>`. Now:

- **Routes** (`src/routes/`):
  - `/` — redirects to `/dashboard` or `/register` based on whether an
    API key is stored.
  - `/register` — server URL config, register, one-time key reveal,
    validate/health checks. (Was the API-key leak; now fixed here.)
  - `/dashboard` — the widget grid.
  - `+layout.svelte` — global stylesheet import, theme/widgets/auth init
    on mount, shared `Nav`.
- **Stores** (`src/lib/stores/`, Svelte 5 runes, module-level `$state`
  singletons):
  - `theme.svelte.ts` — light/dark, persisted via `app_settings`,
    applied as `data-theme` on `<html>`.
  - `auth.svelte.ts` — `hasApiKey` / approval `status` only, never the
    key itself.
  - `widgets.svelte.ts` — list of dashboard chart widgets (`{id, symbol,
    interval}`), persisted as one JSON blob under `app_settings` key
    `ui.dashboard_widgets`. `addWidget` / `removeWidget` / `updateWidget`.
- **API layer** (`src/lib/api/tauri.ts`) — typed wrapper around every
  `invoke(...)` call, one function per Tauri command, matching
  `src-tauri/src/commands/*` 1:1. Components no longer call `invoke`
  directly.
- **Components** (`src/lib/components/`):
  - `Chart.svelte` — dependency-free canvas candlestick renderer (no new
    npm packages added, so nothing to `pnpm install` for this pass).
  - `Widget.svelte` — one dashboard tile: header (symbol/interval picker,
    remove button, last price + % change) + `Chart`, polls
    `get_recent_index_bars` every 15s.
  - `WidgetGrid.svelte` — CSS grid of `Widget`s + "add widget" button,
    reads/writes the `widgets` store.
  - `SymbolPicker.svelte` — symbol + interval `<select>`s (symbol list is
    currently a hardcoded placeholder — see Not done).
  - `Nav.svelte`, `ThemeToggle.svelte` — shared chrome.
- **Styles** (`src/styles/`) — pulled out of components entirely:
  - `variables.css` — theme tokens as CSS custom properties, `:root`/
    `[data-theme="light"]` vs `[data-theme="dark"]`.
  - `global.css` — base element styles + a few reusable utility classes
    (`.card`, `.stack`, `.row`, `.error-banner`, etc.), imported once in
    `+layout.svelte`. Components use these tokens/utilities; only
    layout-specific CSS stays in each component's `<style>` block.
- `types.ts` — TS types mirroring every Rust command payload by hand (no
  codegen yet — see Not done).



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

1. **Compile/build it.** Nothing in this pass has been built — neither the
   Rust changes nor the frontend. Run `cargo build` in `src-tauri/`
   (existing sqlx runtime-query approach unchanged, no new compile-time
   macro usage added), then `pnpm install` + `pnpm tauri dev`. The
   `SvelteKit` route restructure (new `/register`, `/dashboard` dirs) and
   the rune stores (`*.svelte.ts` files) are new surface area most likely
   to need small fixes on first build (e.g. `$app/stores`/`$app/navigation`
   import paths, `crypto.randomUUID()` availability in the Tauri webview).
2. **Frontend for watchlists / paper trades still missing.** Commands
   exist (`list_watchlists`, `create_watchlist`, `open_paper_trade`,
   `list_paper_trades`, etc.) and now have typed wrappers in
   `src/lib/api/tauri.ts`, but no route/component calls them yet. Natural
   next screens: a watchlist sidebar feeding `SymbolPicker`, and a paper
   trades panel/route.
3. **`SymbolPicker` symbol list is hardcoded.** Replace the placeholder
   `KNOWN_SYMBOLS` array with a real source once watchlists (above) or a
   symbols endpoint is wired in.
4. **Layouts table (`layouts`) is still unused.** The new
   `ui.dashboard_widgets` app_settings blob covers "which widgets, which
   symbols" for now; migrate to the `layouts` table if/when multiple named
   named/saved workspaces (not just one dashboard) are needed.
5. **No live market data push to the frontend yet.** `Widget.svelte`
   currently polls `get_recent_index_bars` every 15s — it reads whatever
   the backend's streamers have already aggregated into
   `index_ohlc_1m`/`1d` locally. No Tauri event-based push (`emit`/
   `listen`) from the Rust streamers to the frontend yet, so widgets are
   not truly real-time between polls. Wiring `app.emit()` on new bars +
   a `listen()` in `Widget.svelte` is the natural next step for low
   latency.
6. **Only index bars are chart-able.** `get_recent_index_bars` reads
   `index_ohlc_1m`/`1d` only; there's no equivalent local read command for
   `option_ohlc_1m`, so widgets can't yet chart individual option legs.
7. **No Broker Adapter Layer, Order Manager, Strategy Engine, Indicator
   Engine, or Plugin System** — still just names in `CONTEXT.md`.
8. **No tests.** Neither the server nor this client pass has any.
9. **`Chart.svelte` is intentionally minimal** — no zoom/pan, no overlays/
   indicators, no crosshair sync across widgets, no drawing tools. It's a
   dependency-free candlestick canvas so this pass didn't need any new
   npm installs; swap in a real charting engine (per `CONTEXT.md`'s Chart
   Engine requirement) when that becomes the priority.
10. **Types are hand-mirrored, not generated.** `src/lib/types.ts` matches
    the Rust command payloads by hand. Consider `specta`/`tauri-specta` or
    similar if drift becomes a problem.

## Useful references for next pass
- `kstocks-server/README.md` — full API contract (request/response shapes,
  error codes, auth model) that `api/client.rs` mirrors.
- `kstocks-server/src/settings.rs`, `src/storage/ticks.rs`,
  `src/users/mod.rs` — the patterns this client's `settings.rs`,
  `storage/db.rs`, and query modules were modeled on.
- `CONTEXT.md` (this repo) — overall architecture target; only the
  Local Storage module and a slice of the API/auth surface are built so far.