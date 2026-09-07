# Titanium v7.0.0 Dual Engine Architecture Plan

Build **Titanium v7.0.0 (Dual Engine Edition)** supporting both **Single-File Components (`pages/`)** and **Structured MVC Architecture (`app/models/`, `app/controllers/`, `app/views/`, `config/routes.ti`)**, powered by a built-in **ActiveRecord ORM** and **Enhanced Web Studio Database Manager**.

---

## User Review Required

> [!IMPORTANT]
> Titanium v7.0.0 introduces zero breaking changes to existing v6.0.0 single-file applications (`pages/`). The router dynamically evaluates explicit MVC routes first, then automatically falls back to file-based page components.

---

## Proposed Changes

### Core Engine & Routing

#### [MODIFY] [src/core/router.rs](file:///e:/afterquery/shopify/themes/pa/titanium/src/core/router.rs)
- Add MVC route table supporting explicit route definitions (`Route.get(path, "Controller@action")`, `Route.post(...)`, `Route.resource(...)`).
- Support loading `config/routes.ti` or `routes.titanium`.

#### [MODIFY] [src/core/engine.rs](file:///e:/afterquery/shopify/themes/pa/titanium/src/core/engine.rs)
- Implement **ActiveRecord ORM & Fluent Query Builder**:
  - `Model.define(table, config)`
  - `Model.all()`, `Model.find(id)`, `Model.where(...)`, `Model.create(...)`, `Model.count()`
  - `QueryBuilder`: `.where()`, `.order_by()`, `.limit()`, `.offset()`, `.get()`, `.first()`, `.update()`, `.delete()`
- Implement `view(template_name, data_map)` helper for resolving views from `app/views/` or `views/`.
- Implement MVC controller action execution.

#### [MODIFY] [src/server/studio.rs](file:///e:/afterquery/shopify/themes/pa/titanium/src/server/studio.rs)
- Enhance the Web Studio GUI (`/__titanium_studio`) with a visual database table browser, row editor, schema inspector, and migration runner.

#### [MODIFY] [Cargo.toml](file:///e:/afterquery/shopify/themes/pa/titanium/Cargo.toml)
- Bump version from `6.0.0` to `7.0.0`.

---

## Verification Plan

### Automated Tests & Build
- `cargo check` to verify pure Rust compilation.
- `cargo build` to generate `target/debug/titanium.exe` v7.0.0.
- Verify `cargo run -- build ./example` validates both SFC and MVC route paths without errors.

### Manual Verification
- Test ActiveRecord ORM query builder: `Product.where("category", "Peripherals").limit(5).get()`.
- Test Visual Studio GUI at `http://127.0.0.1:8080/__titanium_studio` to inspect tables and execute queries.
