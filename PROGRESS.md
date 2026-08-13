# kstocks_client — Project Progress

## Latest session updates (most recent first)

### Session: Chart engine, real-time transport, select-box staleness, watchlist collapse, chart x-axis/timezone

**Status: Implemented — unverified**

1. **Chart engine replaced with lightweight-charts.** `src/lib/components/Chart.svelte` now renders candlesticks via TradingView's open-source `lightweight-charts` library instead of the old hand-written `<canvas>` renderer. This gives real axes, crosshair/tooltip, pan/zoom, and an API surface for future indicator overlays.
2. **Real-time data via WSS instead of polling.** Widgets and the watchlist now receive live prices through Tauri events (`index-tick`, `option-tick`, see `src/lib/api/events.ts`) fed by the Rust-side WSS market-data streamers, replacing the previous 5-second polling loop. `Widget.svelte` patches the in-progress last bar directly from ticks; `Watchlist.svelte` updates snapshots in place from the same event stream.
3. **Fixed stale/inconsistent select-box values (`SelectionPicker.svelte`).** Previously, index/option symbol, expiry, and strike lists were fetched only once per mount or once per symbol/expiry change via `$effect`, so newly available expiries/strikes (or symbols that only started streaming after mount) wouldn't show up until the user changed a *different* field to re-trigger the effect — and each `SelectionPicker` instance refreshed independently, so one widget could show fresh values while a sibling widget stayed stale. Fixed by:
   - Adding a 5-second background refresh timer (`REFRESH_INTERVAL_MS`) that re-pulls whichever lists are relevant to the current selection, per picker instance, so every mounted picker converges independently instead of waiting on user interaction.
   - Listening to `option-tick` events to opportunistically grow the `strikes` list live when a tick for the current symbol/expiry references a strike not yet in the list (mirrors the existing `index-tick` handling already used to grow `indexSymbols`).
   - Merging (rather than overwriting) `indexSymbols` on refresh so a symbol added live by a tick isn't dropped by a backend list call that hasn't caught up yet.
4. **Watchlist pane is now collapsible.** `Watchlist.svelte` takes a bindable `collapsed` prop with a toggle button in its header; `src/routes/home/+page.svelte` binds to it and shrinks the watchlist grid column to 44px (vertical label, no list body) when collapsed, with a matching collapsed height on the stacked/narrow layout.
5. **Fixed missing chart x-axis and timezone handling (`Chart.svelte`).**
   - The chart is now created with an explicit initial `width` (measured synchronously from the container at mount) instead of relying solely on the first `ResizeObserver` callback, and `chart.timeScale().fitContent()` is called after both initial creation and every `setData()`, so the time axis reliably has room and ticks to lay out labels immediately instead of only after a resize/interaction.
   - `bucket_start` values from the backend are confirmed to already be genuine UTC instants (`Utc::now()` at tick-arrival time, floored to the minute, serialized with a trailing `Z` — see `src-tauri/src/storage/ohlc.rs`), so no additional timezone conversion is needed when parsing them into the chart's `UTCTimestamp`.
   - The actual bug was **display-side**: lightweight-charts formats axis/crosshair labels using the browser's local timezone by default, which silently relabels NSE bars into the viewer machine's timezone instead of NSE's IST wall-clock time whenever the two differ. Fixed by supplying an explicit fixed-offset (UTC+5:30) `tickMarkFormatter` and `localization.timeFormatter`, so labels always read as true IST regardless of the host machine's configured timezone — with no double conversion, since the underlying instant was already correct UTC.

---


> **Purpose:** This document is the current source of truth for implementation status.
>
> **Status terminology**
> - **Implemented — unverified:** The code/configuration has been added according to the previous development sessions, but the project has not yet been compiled or run successfully in this environment.
> - **Verified:** Confirmed by a successful build/run or explicit runtime test.
> - **Pending:** Not implemented yet.
> - **Deferred:** Intentionally left for a later phase.

---

## 1. Current overall status

The project has progressed from an empty Tauri v2 + Svelte + TypeScript shell into a substantially structured desktop market-analysis application with:

- Tauri v2 + Rust backend
- Svelte/SvelteKit frontend
- Local SQLite storage
- kstocks-server API integration
- Local market-data read paths
- Index and option-chain data support
- Configurable dashboard widgets
- Chart/list views
- Watchlist-style index display
- Paper trading
- Username-based registration and approval status
- Session-only broker API-key placeholder
- Light/dark theme support

### Important verification status

**Nothing in the recent implementation passes has been compiled or run successfully.** The progress notes explicitly state that `cargo check`/`cargo build`, `pnpm install`, and `pnpm tauri dev` were intentionally left for manual execution. Therefore, the current implementation should be treated as **implemented in source but unverified**, not as production-ready or build-verified. fileciteturn0file0L5-L6

The **first immediate task is therefore a full compile/run validation** before adding another large feature set.

---

# 2. Implemented functionality

## 2.1 Project foundation

**Status: Implemented — unverified**

### Rust/Tauri backend

- Tauri v2 application structure established.
- Rust backend organized into:
  - application state
  - settings/configuration
  - API client
  - storage
  - Tauri commands
- Shared `AppState` contains:
  - SQLite pool
  - application paths
  - application configuration
  - server API client
- Tauri commands are registered through `invoke_handler!`.

### Local application configuration

- Application data directory follows the server's `setup_app_folders()` pattern.
- `~/.kstocks/` is used when available, with a current-working-directory fallback.
- Separate client configuration file:
  - `settings_client.json`
- Database and log directories are created during startup.
- Server URL and server API-key configuration are supported internally.

The original foundation established the local storage and server API integration layers. fileciteturn0file0L321-L332

---

## 2.2 Local SQLite storage

**Status: Implemented — unverified**

SQLite storage is initialized through `sqlx` with:

- WAL mode
- `synchronous=NORMAL`
- foreign keys enabled
- idempotent schema creation

Current tables include:

- `app_settings`
- `watchlists`
- `watchlist_items`
- `layouts`
- `paper_trades`

Storage modules exist for:

- application settings
- watchlists
- paper trades
- OHLC/market data

The database foundation and query modules were established in the initial implementation. fileciteturn0file0L334-L346

---

## 2.3 kstocks-server API integration

**Status: Implemented — unverified**

The client API layer supports the server contract for:

- registration
- validation/approval
- health check
- index OHLC
- option OHLC

The API client supports runtime server URL/API-key changes and distinguishes transport, server-response, and missing-key errors. fileciteturn0file0L348-L359

---

## 2.4 Registration and authentication UX

**Status: Implemented — unverified**

The registration flow has been substantially simplified.

### Current behavior

- `/register` was renamed to `/settings`.
- Registration is now username-only.
- The client handles the server API key internally.
- The UI no longer exposes a "reveal your key" workflow.
- Username is stored locally for display.
- Approval status is displayed as pending/approved.
- Manual approval-status checking was removed.
- `validate_client` runs automatically once during application startup.
- Root `/` redirects to `/home` regardless of registration status.
- Unregistered users can still access live local streamer data.
- Registered users additionally receive server-side backfill.

The current settings behavior is described in the latest implementation pass. fileciteturn0file0L9-L31

### API-key security change

The previous implementation exposed the raw server API key across the Tauri/frontend boundary. This was changed so the frontend receives only a `ServerConfigView` containing:

- `base_url`
- `has_api_key`

The raw key no longer crosses the IPC boundary after registration. fileciteturn0file0L230-L240

---

## 2.5 Settings page

**Status: Implemented — unverified**

`/settings` currently contains:

- username registration before registration
- persisted username display after registration
- approval status
- broker connection placeholder
- Dhan API-key input

The Dhan key is intentionally:

- held only in an in-memory Svelte `$state`
- not written to `app_settings`
- not passed to persisting Tauri commands
- discarded when the app/tab reloads

Live broker trading is not connected yet; the field is explicitly a placeholder. fileciteturn0file0L20-L38

---

# 3. Market-data implementation

## 3.1 Local index OHLC

**Status: Implemented — unverified**

The client can read index OHLC data from local SQLite storage rather than making a network request for every chart refresh.

Implemented:

- `OhlcBar`
- `recent_index_bars()`
- `get_recent_index_bars(...)`

The existing REST OHLC endpoint remains available for explicit historical backfill. fileciteturn0file0L247-L252

---

## 3.2 Local option OHLC

**Status: Implemented — unverified**

Option-leg OHLC support has now been added.

Implemented:

- `OptionLegBar`
- `recent_option_bars(...)`
- `get_recent_option_bars(...)`

The query reads one option leg (`CE` or `PE`) from `option_ohlc_1m`.

The implementation uses the same basic OHLC structure as index bars, allowing the existing chart renderer to be reused. fileciteturn0file0L95-L104

---

## 3.3 Option-chain snapshot

**Status: Implemented — unverified**

Implemented:

- `OptionChainRow`
- `option_chain_snapshot(...)`
- `get_option_chain(...)`

The resulting structure contains one row per strike with:

- CE OI
- CE volume
- CE LTP
- strike
- PE OI
- PE volume
- PE LTP

The latest bucket for each strike is used. fileciteturn0file0L103-L107

---

## 3.4 Option selection data

**Status: Implemented — unverified**

Cascading selection commands are available for:

- option symbols
- expiries
- strikes

Implemented commands:

- `list_option_symbols`
- `list_option_expiries`
- `list_option_strikes`

These use real local database data rather than hardcoded option-chain values. fileciteturn0file0L100-L102

---

## 3.5 Option backfill

**Status: Implemented — unverified**

Implemented:

- `backfill_option_1m(...)`
- `run_startup_option_backfill(...)`

The startup backfill:

- discovers locally known option contracts
- processes both CE and PE legs
- writes into `option_ohlc_1m`
- runs after index backfill
- is enabled only for registered users with a server API key

The conflict update uses `COALESCE` so fresher locally streamed values are not overwritten by backfill data. fileciteturn0file0L111-L122

---

# 4. Frontend/dashboard implementation

## 4.1 Route structure

**Status: Implemented — unverified**

Current routes:

- `/` → redirects to `/home`
- `/home`
- `/settings`

Global initialization is handled by `+layout.svelte`.

Shared navigation is provided by `Nav.svelte`. fileciteturn0file0L14-L18

---

## 4.2 Home layout

**Status: Implemented — unverified**

The home screen uses three main sections:

1. **Left:** Watchlist
2. **Middle:** Widget grid
3. **Right:** Order panel

The desktop layout uses approximately:

- `260px`
- flexible center
- `300px`

and switches to a stacked layout below roughly 1000px. fileciteturn0file0L41-L63

---

## 4.3 Widget grid

**Status: Implemented — unverified**

The dashboard supports up to four widgets.

Current slot behavior:

- **1 widget:** fills the entire 2×2 area
- **2 widgets:** left and right columns, each spanning both rows
- **3 widgets:** left column plus two right-side widgets
- **4 widgets:** four individual cells

Widget state is persisted through `app_settings`.

`MAX_WIDGETS = 4`.

The widget grid and layout rules are implemented in `WidgetGrid.svelte` and the widgets store. fileciteturn0file0L47-L58

---

## 4.4 Chart/list toggle

**Status: Implemented — unverified**

Each widget can switch between:

- chart view
- list view

The widget does not require a complete reload to change the view.

The configuration model is now:

```text
{
    id,
    view: "chart" | "list",
    selection,
    interval
}
```

The instrument selection is a discriminated union supporting:

- index
- option contract

fileciteturn0file0L65-L78

---

## 4.5 Index chart/list

**Status: Implemented — unverified**

### Chart view

Index charts use:

- `get_recent_index_bars`
- `Chart.svelte`

### List view

Index list view provides:

- symbol
- price
- change
- percentage change
- day low/high range visualization

`RangeBar.svelte` displays the current price relative to the day's range. fileciteturn0file0L74-L81

---

## 4.6 Option chart/list

**Status: Implemented — unverified**

### Chart view

An individual CE/PE option contract can be displayed using the same candlestick renderer as index data.

### List view

Option-chain view uses:

- `OptionChainTable.svelte`
- one row per strike
- CE columns on the left
- strike in the center
- PE columns on the right

The component already has support for nearest-strike filtering. fileciteturn0file0L79-L88

---

## 4.7 Instrument selection

**Status: Implemented — unverified**

`SelectionPicker.svelte` replaced the previous `SymbolPicker.svelte`.

It supports:

### Index

- index symbol

### Option chain

- symbol
- expiry
- strike
- CE/PE

Option values are loaded from local database-backed commands. fileciteturn0file0L89-L93

---

## 4.8 Watchlist

**Status: Partially implemented — unverified**

The current home-page watchlist:

- polls all locally available indices
- displays symbol
- price
- change
- percentage change

The older database-backed:

- `watchlists`
- `watchlist_items`
- CRUD commands

already exist, but the current `Watchlist.svelte` does not yet use them.

Therefore:

**Watchlist storage exists, but user-curated watchlist management is not complete.** fileciteturn0file0L165-L170

---

## 4.9 Paper trading

**Status: Backend implemented — frontend partially implemented**

The local database contains simulated trade storage and the existing `open_paper_trade` command is used by `OrderPanel.svelte`.

The current order panel supports:

- Paper/Live mode
- Buy/Sell
- quantity
- price

Paper mode calls `open_paper_trade`. fileciteturn0file0L59-L63

However, the complete paper-trading workflow is not finished because the order panel still lacks market-aware contract selection and integration with widget selections.

---

# 5. Current frontend/backend plumbing

**Status: Implemented — unverified**

The frontend has a typed API wrapper:

`src/lib/api/tauri.ts`

Components no longer call Tauri `invoke()` directly.

The TypeScript type definitions mirror the Rust command payloads manually.

The latest pass added seven market-data commands:

- `get_recent_option_bars`
- `list_option_symbols`
- `list_option_expiries`
- `list_option_strikes`
- `get_option_chain`
- `get_all_index_snapshots`
- `get_index_snapshot`

All are registered through `lib.rs`. fileciteturn0file0L123-L139

---

# 6. Explicitly pending work

## P0 — Must do before continuing feature development

### 6.1 Compile and run the complete application

**Status: Pending — highest priority**

Run:

```bash
cd src-tauri
cargo build

cd ..
pnpm install
pnpm tauri dev
```

Then resolve all compile/runtime issues.

Areas already identified as likely first-build risk:

- Svelte 5 + TypeScript discriminated-union narrowing
- `$derived` / `$effect` interactions
- `sqlx::FromRow` mappings
- option OHLC SQL aliases
- Tauri/Svelte integration

No feature should be considered fully complete until this validation pass succeeds. fileciteturn0file0L148-L157

---

## P1 — Core market-analysis UX

### 6.2 Real-time push from Rust to frontend

**Status: Pending**

Current market-data UI uses polling.

Current polling intervals are approximately 5–15 seconds depending on the component.

Not yet implemented:

- Rust streamer → Tauri `app.emit()`
- frontend `listen()`
- tick/bar event propagation
- low-latency UI updates

This is the main missing piece for a genuinely streaming market-data interface. fileciteturn0file0L181-L187

---

### 6.3 Complete chart engine

**Status: Pending**

`Chart.svelte` is still a minimal dependency-free candlestick canvas.

Missing:

- zoom
- pan
- crosshair
- synchronized crosshair between widgets
- overlays
- indicators
- drawing tools
- richer chart interaction

The current chart should therefore be regarded as a functional prototype renderer, not the final chart engine. fileciteturn0file0L198-L201

---

### 6.4 Option-chain nearest-strike control

**Status: Partially implemented**

The underlying component already supports:

- `showNearestOnly`
- `nearestCount`
- `nearestStrike`

But there is no user-facing control yet.

Pending:

- UI toggle
- nearest-strike count control, if required

fileciteturn0file0L158-L164

---

### 6.5 Dynamic index symbol source

**Status: Pending**

`SelectionPicker` still uses a small hardcoded index list:

- NIFTY
- BANKNIFTY
- FINNIFTY
- SENSEX

It should eventually use:

- watchlist data, or
- a proper symbols/instruments command.

fileciteturn0file0L188-L192

---

### 6.6 User-managed watchlists

**Status: Pending**

The database and CRUD commands exist.

Still required:

- watchlist management UI
- add/remove symbols
- select active watchlist
- feed watchlist membership into the picker/home watchlist

---

# 7. Trading subsystem

## 7.1 Paper trading

**Status: Partially implemented**

Implemented:

- local `paper_trades` storage
- open/close/list backend commands
- basic order-panel paper mode

Pending:

- current-market-price integration
- instrument-aware orders
- option-contract selection
- widget → order-panel integration
- order validation
- richer position/order state
- complete paper-trading UI

---

## 7.2 Live trading

**Status: Deferred / not implemented**

Only the session-only Dhan API-key placeholder exists.

Not implemented:

- broker adapter
- authenticated broker client
- live order command
- order routing
- order status updates
- positions
- holdings
- broker error handling
- live trading risk controls

The current "Live" mode is intentionally inert. fileciteturn0file0L171-L180

---

# 8. Larger architecture components

The following architecture components are currently **not implemented**:

1. **Broker Adapter Layer**
2. **Order Manager**
3. **Strategy Engine**
4. **Indicator Engine**
5. **Plugin System**

They currently exist only as architectural targets/references in `CONTEXT.md`. fileciteturn0file0L193-L196

These should not be described as partially implemented unless actual code is added.

---

# 9. Testing and code quality

## 9.1 Automated tests

**Status: Pending**

No meaningful automated test suite currently exists for the client.

Pending test areas:

- storage/query tests
- API client tests
- Tauri command tests
- market-data aggregation/read tests
- option-chain tests
- frontend component tests
- end-to-end application tests

The current project notes explicitly record the absence of tests. fileciteturn0file0L197-L197

---

## 9.2 Rust ↔ TypeScript type generation

**Status: Pending**

`src/lib/types.ts` currently mirrors Rust command types manually.

This creates a growing risk of frontend/backend type drift.

Potential future improvement:

- `specta`
- `tauri-specta`
- another generated-contract approach

The command surface is already large enough for this to become increasingly useful. fileciteturn0file0L202-L205

---

# 10. UI/layout improvements

## 10.1 Resizable dashboard panes

**Status: Pending**

Current home layout is fixed:

```text
260px | flexible | 300px
```

It is responsive enough to stack below approximately 1000px, but it does not support:

- drag-to-resize
- adjustable sidebar width
- adjustable order-panel width

fileciteturn0file0L206-L208

---

# 11. Historical work that is now superseded

The original progress notes contain several older descriptions that should **not** be treated as the current state.

### Superseded route names

Old:

- `/register`
- `/dashboard`

Current:

- `/settings`
- `/home`

### Superseded registration UX

Old:

- API-key reveal
- API-key-oriented registration screen

Current:

- username-only registration
- API key handled internally
- approval status displayed
- broker API key is a separate session-only placeholder

### Superseded widget model

Old widget configuration:

```text
{id, symbol, interval}
```

Current widget configuration:

```text
{id, view, selection, interval}
```

where `selection` supports both index and option contracts.

### Superseded symbol picker

Old:

- `SymbolPicker.svelte`

Current:

- `SelectionPicker.svelte`

### Superseded market-data scope

Old:

- index OHLC only

Current implementation additionally contains:

- option-leg OHLC
- option-chain snapshots
- option symbol/expiry/strike discovery
- option backfill

These changes are already reflected in the current implementation sections above. fileciteturn0file0L65-L93

---

# 12. Recommended implementation sequence

To avoid repeatedly accumulating unverified code, the next development sessions should proceed in this order.

## Phase 1 — Build validation

1. Compile Rust backend.
2. Install frontend dependencies.
3. Run Tauri development build.
4. Fix all Rust compilation errors.
5. Fix all TypeScript/Svelte compilation errors.
6. Exercise registration/settings.
7. Verify local database initialization.
8. Verify index data.
9. Verify option-chain data.
10. Verify chart/list switching.

**Do not add another major feature until this phase passes.**

---

## Phase 2 — Real-time market-data pipeline

Implement:

```text
NSE / local streamers
        ↓
Rust market-data ingestion
        ↓
SQLite / in-memory market state
        ↓
Tauri event emission
        ↓
Svelte event listeners
        ↓
Widgets / watchlist / option chain
```

The goal is to eliminate periodic frontend polling for live data.

---

## Phase 3 — Complete analysis UI

Implement:

- production chart engine
- zoom/pan
- crosshair
- synchronized charts
- indicators
- overlays
- option-chain nearest-strike controls
- dynamic symbol/instrument discovery
- curated watchlists
- resizable dashboard panes

---

## Phase 4 — Paper trading

Complete the simulation layer before live trading:

- market-aware order entry
- index/option instrument selection
- current LTP integration
- widget-to-order-panel selection
- order lifecycle
- positions
- P&L
- trade history
- simulation validation

---

## Phase 5 — Trading architecture

Implement the architectural separation:

```text
Broker Adapter
      ↓
Order Manager
      ↓
Portfolio / Position State
      ↓
Strategy Engine
      ↓
Indicator Engine
      ↓
Plugin System
```

The exact boundaries should be finalized before implementing live broker connectivity.

---

## Phase 6 — Live broker integration

Only after paper trading is stable:

- Dhan adapter
- authentication
- order placement
- order updates
- positions/holdings
- broker errors
- rate limits
- risk controls
- live/paper separation

The existing session-only Dhan API-key field can then be connected to the broker subsystem.

---

# 13. Current status summary

| Area | Status |
|---|---|
| Tauri/Rust foundation | Implemented — unverified |
| Local SQLite storage | Implemented — unverified |
| Server API client | Implemented — unverified |
| Registration/auth flow | Implemented — unverified |
| Settings page | Implemented — unverified |
| Home/dashboard | Implemented — unverified |
| Widget grid | Implemented — unverified |
| Index OHLC read path | Implemented — unverified |
| Option OHLC read path | Implemented — unverified |
| Option-chain snapshot | Implemented — unverified |
| Option selection | Implemented — unverified |
| Option backfill | Implemented — unverified |
| Chart/list toggle | Implemented — unverified |
| Index list view | Implemented — unverified |
| Option-chain list view | Implemented — unverified |
| Basic paper trading | Partially implemented |
| User-managed watchlists | Pending |
| Real-time frontend push | Pending |
| Production chart engine | Pending |
| Indicators/overlays | Pending |
| Dynamic symbol discovery | Pending |
| Resizable panes | Pending |
| Complete paper-trading workflow | Pending |
| Broker adapter | Pending |
| Live trading | Deferred |
| Order manager | Pending |
| Strategy engine | Pending |
| Indicator engine | Pending |
| Plugin system | Pending |
| Automated tests | Pending |
| Rust/TS type generation | Pending |
| **Build/runtime verification** | **P0 — Pending** |

---

# 14. References

- `kstocks-server/README.md` — server API contract.
- `kstocks-server/src/settings.rs` — settings/configuration patterns.
- `kstocks-server/src/storage/ticks.rs` — market-data storage patterns.
- `kstocks-server/src/users/mod.rs` — server-side user/auth patterns.
- `CONTEXT.md` — target architecture and long-term design.

## Repository references

- Client repository: `rrstar2345/kstocks_client`
- Server repository: `rrstar2345/kstocks-server`

---

## Bottom line

The project has a **substantial implementation foundation**, including index and option-chain data paths, dashboard widgets, local storage, registration, paper-trading primitives, and the frontend restructuring.

However, the most important distinction is:

> **The implementation is currently source-complete for the features listed as implemented, but it is not build-verified.**

The next session should therefore be a **build-and-integration validation session**, not another feature-heavy implementation session.

Once the application builds and runs, the progress document should be updated again based on actual test results, moving items from **Implemented — unverified** to **Verified** or **Pending** as appropriate.