# ⚡ `Titanium` (Ti22) — Version 10.0.0 (Titanium X)

> **The Native Multi-Paradigm Web Runtime & Distributed AI Engine.**  
> Zero `node_modules`. Zero external runtime dependencies. Compiles to a single standalone bare-metal executable.

[![CI & Deploy Documentation](https://github.com/jins-coder/TI22/actions/workflows/deploy-docs.yml/badge.svg)](https://github.com/jins-coder/TI22/actions/workflows/deploy-docs.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-10.0.0--Supercluster-sky.svg)](https://github.com/jins-coder/TI22)

---

## 🌟 What is Titanium?

**Titanium (Ti22)** is an all-in-one native web platform written in **Rust**. It eliminates the complexity of fragmented web stacks by combining frontend templating, backend scripting, embedded high-concurrency database storage, real-time WebSocket pub/sub, agentic AI reasoning, and multi-node cluster mesh into a single high-performance binary.

---

## 🚀 Key Features (v10.0.0 Supercluster X)

* ⚡ **Dual-Engine Architecture:** Run file-based Single-File Pages (`pages/`) and Enterprise Domain-Driven MVC (`app/controllers/`, `app/models/`, `app/views/`) simultaneously on the same engine.
* 🗄️ **Embedded SQLite & ActiveRecord ORM:** Ultra-fast WAL mode with fluent chained query API (`model("products").where("stock", ">", 0).order_by("price", "DESC").get()`).
* 📡 **Hyperdrive Realtime WebSockets:** Multi-channel pub/sub routing, Server-Sent Events (SSE), and declarative `data-live` attribute soft-DOM morphing.
* 🧠 **Singularity AI & Semantic Vector RAG:** Native local reasoning engine, token-by-token streaming (`ai_stream`), and vector cosine similarity knowledge retrieval (`ai_rag`).
* 🌐 **Supercluster Multi-Node Mesh:** Auto-discovering peer nodes, Primary-Replica cluster leadership, and distributed SQLite WAL commit replication.
* 🎨 **Visual Web Studio GUI:** Interactive web dashboard at `http://127.0.0.1:8080/__titanium_studio` with live SQLite table explorer, SQL query console, AI playground, and cluster topology map.
* ⚡ **Zero-Flicker Turbo SPA Engine:** Debounced progress loader and intelligent DOM tree diffing preserving active inputs, cursor positions, and scroll state.
* 🔐 **Built-in Session Authentication & Security:** Cryptographic password hashing (`crypto_hash()`), CSRF token verification, and role-based route middleware.
* 🧩 **Official VS Code Extension:** Syntax highlighting for `.titanium` and `.ti` files, rich snippets, and Web Studio controller.

---

## ⚡ Quick Start

### 1. Prerequisites
Install [Rust & Cargo](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Run Development Server
```bash
# Clone repository
git clone https://github.com/jins-coder/TI22.git
cd TI22

# Start development server with live reload
cargo run -- dev ./example
```

* 🌐 **Storefront:** [http://127.0.0.1:8080](http://127.0.0.1:8080)
* 🎨 **Ti22 Web Studio:** [http://127.0.0.1:8080/__titanium_studio](http://127.0.0.1:8080/__titanium_studio)
* 🧠 **AI Store Copilot:** [http://127.0.0.1:8080/ai_copilot](http://127.0.0.1:8080/ai_copilot)

---

## 📝 Code Examples

### 1. Single-File Page (`pages/product/[id].titanium`)
```titanium
---
// Server Scripting Block (Rhai)
let id = params.id;
let product = model("products").find(id);

if product == () {
  return redirect("/404");
}
---
<!DOCTYPE html>
<html lang="en">
<head>
  <title>{{ product.name }} | TITANSTORE</title>
  <link rel="stylesheet" href="/app.css">
</head>
<body class="bg-slate-950 text-white">
  <h1>{{ product.name }}</h1>
  <p class="text-sky-400 font-bold">${{ product.price }}</p>
  <span data-live="inventory" data-live-key="stock">{{ product.stock }} in stock</span>
</body>
</html>
```

### 2. MVC Controller (`app/controllers/products.rhai`)
```rust
// Product Domain Controller
fn index(req, session) {
  let items = model("products")
    .where("stock", ">", 0)
    .order_by("price", "DESC")
    .limit(10)
    .get();

  return view("products/index", #{
    title: "All Products",
    products: items
  });
}
```

### 3. Realtime WebSockets & Live Subscriptions
```javascript
// Browser JavaScript
Titanium.subscribe('orders', (order) => {
  console.log('⚡ Live Order Received:', order);
});

// Broadcast from Rhai server
ws_broadcast("orders", #{
  order_number: "TI-9912",
  total: 349.99
});
```

### 4. Singularity AI & Semantic Vector RAG
```rust
// Ingest knowledge document
vector_upsert("faq", "return_policy", "30-day no-hassle return window on all titanium products.");

// Semantic RAG retrieval and grounded generation
let answer = ai_rag("What is your return policy?", "faq", 3);
```

---

## 🧭 CLI Commands

```bash
titanium dev [dir]               # Start live development server
titanium studio [dir]            # Start server & launch Web Studio GUI
titanium build [dir]             # Validate & package project for production
titanium migrate [dir]           # Execute database migrations
titanium info [dir]              # Display project telemetry & diagnostics
titanium new <dir>               # Scaffold new Titanium project
```

---

## 📖 Documentation

* 📖 **[Interactive HTML Documentation (GitHub Pages)](index.html)** — Comprehensive architecture guide, API cheatsheet & visual diagrams.
* 🧩 **[VS Code Extension (`vscode-titanium`)](vscode-titanium/)** — TextMate syntax grammars and snippets for `.titanium` files.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
