# Titanium Changelog & Version History

All notable changes to the Titanium Native Web Runtime are documented here and in detailed per-version documents under [`versions/`](file:///e:/afterquery/shopify/themes/pa/titanium/versions).

---

## Versions Index

| Version | Codename | Milestone | Status | Documentation |
| :--- | :--- | :--- | :--- | :--- |
| **v6.0.0** | *Omniverse* | Next-Gen Platform | **Production / Latest** | [v6.0.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v6.0.0.md) |
| **v5.0.0** | *Studio & Edge* | Milestone 8 | Complete | [v5.0.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v5.0.0.md) |
| **v4.0.0** | *Packager* | Milestone 7 | Stable | [v4.0.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v4.0.0.md) |
| **v3.0.0** | *Pulsar* | Milestone 6 | Stable | [v3.0.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v3.0.0.md) |
| **v2.0.0** | *Supernova* | Milestone 5 | Stable | [v2.0.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v2.0.0.md) |
| **v1.0.0** | *Foundations* | Milestone 4 | Stable | [v1.0.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v1.0.0.md) |
| **v0.3.0** | *Shield & Stream* | Milestone 3 | Stable | [v0.3.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v0.3.0.md) |
| **v0.2.0** | *Velocity & Ergonomics* | Milestone 2 | Stable | [v0.2.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v0.2.0.md) |
| **v0.1.0** | *Genesis* | Milestone 1 | Initial Release | [v0.1.0 Release Notes](file:///e:/afterquery/shopify/themes/pa/titanium/versions/v0.1.0.md) |

---

## [v6.0.0] — September 2026

### Added
- **Elemental Rebrand to Titanium (Ti22)**: Renamed native binary, package, syntax (`.titanium`, `.ti`), configuration (`titanium.toml`), and VS Code extension (`vscode-titanium`).
- **In-Memory TTL Cache**: `cache_set`, `cache_get`, `cache_has`, `cache_delete`, `cache_clear`, `cache_keys`, `cache_stats`.
- **In-Process Job Queue**: Async worker thread pool with `defer_job` and `queue_stats`.
- **Embedded Vector & Cosine Search**: `vector_cosine_similarity` and `vector_rank` for local AI / RAG knowledge bases.
- **Realtime PubSub Event Hub**: `pubsub_publish`, `pubsub_history`, `pubsub_topics`.
- **Pure-Rust Media & SVG Identicons**: `svg_identicon`, `media_info`, `media_data_uri`.
- **Modern PHP 8.1 - 8.5 Primitives**: `json_validate`, `json_encode`, `json_decode`, `redact`, `mask`, `array_pluck`, `array_chunk`, `array_group_by`, `str_slug`, `clamp`.
- **Titanium Studio v6 API Endpoints**: `/api/cache`, `/api/queue`, `/api/pubsub`.

---

## [v5.0.0] — September 2026

### Added
- **Titanium Web Studio (`/__titanium_studio`)**: Built-in developer visual admin GUI.
- **Interactive SQL Runner**: Direct query execution and table inspection in browser.
- **Studio APIs**: Added `GET /__titanium_studio/api/tables` and `POST /__titanium_studio/api/query`.
- **100% Milestone Completion**: Completed all 8 roadmap milestones.

---

## [v4.0.0] — September 2026

### Added
- **Sliding-Window Rate Limiter**: Built-in `rate_limit(key, max_reqs, window_secs)` helper.
- **Production Packager Command (`titanium build`)**: Package validation and build optimization.
- **Standalone CLI Migrations (`titanium migrate`)**: Execute database migrations directly without web server boot.
- **Diagnostic CLI Command (`titanium info`)**: Project diagnostics and route structure reporting.

---

## [v3.0.0] — September 2026

### Added
- **Real-Time SSE & AI Token Streaming**: Built-in `sse_event` and `sse_stream` response primitives.
- **Server-Side File I/O**: Added `file_write`, `file_read`, and `file_exists` to Rhai scope.
- **IDE-Grade Error Screen**: Enhanced developer exception screen with route inspection and clear stack frames.

---

## [v2.0.0] — September 2026

### Added
- **Native Server-Side HTTP Client**: Built-in `http_get` and `http_post` for calling external APIs directly in scripts.
- **Multi-Column & Fuzzy Search**: Added `db_search(table, query, columns)` for fast multi-column querying.
- **Cryptographic Hashing & Tokens**: Added `crypto_hash`, `crypto_verify`, and `crypto_random_token` for password authentication and API key generation.
- **Automatic SQL Migrations (`migrations/*.sql`)**: Sequential transaction migrations tracked in `_titanium_migrations` table.
- **Production Health Diagnostics (`/__titanium_health`)**: JSON diagnostics endpoint for monitors and load balancers.

---

## [v1.0.0] — September 2026

### Added
- Production-ready core hardening and health diagnostics.
- Sequential migrations runner foundation.

---

## [v0.3.0] — September 2026

### Added
- **Global & Route Middleware (`_middleware.titanium`)**: Pre-execution request interceptors for Auth guards and custom routing logic.
- **Native CSRF Protection**: Added `session.csrf()` / `{{ csrf_token }}` context injection.
- **Custom Template Filters**: Built-in `money` and `slugify` template filters.
- **Static Asset Caching**: Added `Cache-Control` headers for public static assets.

---

## [v0.2.0] — September 2026

### Added
- **Live Reload**: Built-in file monitoring & auto-refresh client script (`/__titanium_live`).
- **Layouts & Slots**: Dedicated layout engine supporting `layouts/default.titanium` with `<slot />` or `{{ content | safe }}`.
- **SQLite WAL Mode**: Enabled `PRAGMA journal_mode = WAL` and `PRAGMA synchronous = NORMAL` for high concurrency.
- **Database CRUD Helpers**: Added `db_insert`, `db_update`, `db_delete`, `db_find` to Rhai scope.
- **Form Validation Engine**: Added `validate(data, rules)` helper supporting `required`, `email`, `min:N`, and `max:N`.

---

## [v0.1.0] — September 2026

### Added
- Core Titanium native runtime binary in Rust.
- Embedded SQLite with `db_exec`, `db_query`, `db_first`, and `db_run`.
- Single-file `.titanium` component support (Rhai scripting + MiniJinja templates).
- File-system router with dynamic parameters `[id].titanium` and JSON API endpoints.
- Zero-dependency Turbo SPA navigation and session flash messaging.
