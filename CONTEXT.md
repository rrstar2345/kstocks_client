Build a **professional-grade, cross-platform desktop stock trading platform** using a **Rust + Tauri + Svelte** architecture, optimized for **low latency, high performance, modularity, and long-term maintainability**.

**Technology Stack**

* Backend: Rust (Tokio, async, event-driven architecture)
* Desktop Shell: Tauri
* Frontend: Svelte + TypeScript
* IPC: Typed Tauri commands and event streams
* Database: SQLite for application data, settings, layouts, watchlists, simulated trades; DuckDB/Apache Arrow/Parquet for historical market data and analytics
* Chart Rendering: High-performance Canvas/WebGL-based financial charting engine

**Architecture Requirements**

* Follow Clean Architecture, SOLID principles, and a modular, config-driven design.
* Keep the UI completely independent of market data providers and broker implementations.
* Implement an internal event bus where all components communicate through typed events rather than direct dependencies.
* Separate the application into independent modules, including:

  * Market Data Engine
  * WebSocket Manager
  * Broker Adapter Layer
  * Order Manager
  * Strategy Engine
  * Trade Simulation Engine
  * Indicator Engine
  * Chart Engine
  * Workspace/Layout Manager
  * Local Storage
  * Plugin System

**Functional Requirements**

* Consume real-time market data from both REST APIs and WebSocket streams (including NSE public WebSocket and my own server APIs).
* Display streaming charts with customizable overlays, indicators, drawing tools, synchronized crosshairs, multiple chart layouts, and responsive updates.
* Support paper trading using live market prices while executing simulated orders independently of live broker execution.
* Support live trading through interchangeable broker adapters behind a common interface.
* Design all providers (market data, brokers, indicators, strategies) as pluggable modules to allow future expansion without modifying core components.

**Non-Functional Goals**

* Prioritize low latency, scalability, extensibility, maintainability, testability, and cross-platform compatibility (Windows and Linux first, macOS optional).
* Avoid monolithic classes, tightly coupled components, duplicated business logic, and direct communication between the UI and external services.
* Optimize for long-term evolution into a professional trading terminal rather than rapid prototyping.
