# ⚡ `Titanium` (Ti22)
### Standalone Native Web Runtime Engine — Friendly Syntax, Embedded SQLite & Turbo SPA.

**Titanium** is a standalone, native web runtime written in **Rust** (compiling directly to `titanium.exe` with zero Node.js/external runtime dependencies). It combines the productivity and single-file simplicity of modern web paradigms with high-performance native threadpool execution.

---

## 🗺️ Milestones & Documentation
* 🧩 **[Official VS Code Extension](file:///e:/afterquery/shopify/themes/pa/titanium/vscode-titanium)** — Dual-syntax highlighting (`.titanium`, `.ti`), rich snippets & Web Studio controller.
* 💡 **[Ecosystem Comparison & Advantages](file:///e:/afterquery/shopify/themes/pa/titanium/ECOSYSTEM_ADVANTAGES.md)** — What Node, PHP, Python, Go, and Rails lack vs. what Titanium solves.
* 📍 **[Project Milestones & Roadmap](file:///e:/afterquery/shopify/themes/pa/titanium/MILESTONES.md)** — Completed achievements (v0.1.0 → v6.0.0).
* 📋 **[Master CHANGELOG](file:///e:/afterquery/shopify/themes/pa/titanium/CHANGELOG.md)** — Detailed changelog for every release.
* 📦 **[Versions Directory](file:///e:/afterquery/shopify/themes/pa/titanium/versions)** — Individual release notes for each version.

---

## 🚀 Key Features

* 💎 **Friendly, Clean Syntax**: Frontmatter script blocks (`--- ... ---`) and simple `{{ ... }}` / `{% ... %}` templates.
* 🦀 **Native Compiled Binary (`titanium.exe`)**: Executes directly on bare-metal machine code with sub-millisecond response times ($<1\text{ ms}$).
* 🗄️ **Embedded Native SQLite (WAL Mode)**: `db_exec()`, `db_query()`, `db_first()`, `db_run()`, `db_insert()`, `db_update()`, `db_delete()`, `db_find()`, `db_search()`.
* ⚡ **Turbo SPA Navigation & Live Reload**: Zero full-page reloads on navigation and instant soft-DOM live reload during development.
* 📦 **In-Memory TTL Cache & Job Queue**: Native Redis alternative (`cache_set`, `cache_get`) and background async workers (`defer_job`).
* 🧠 **Local Vector & RAG Search**: Pure-Rust cosine similarity matching (`vector_rank`, `vector_cosine_similarity`).
* 📡 **Real-Time PubSub & SSE**: Channel broadcasting (`pubsub_publish`, `pubsub_history`) and live event streaming (`sse_event`, `sse_stream`).
* 🔐 **Built-in Cryptography & Security**: `crypto_hash()`, `crypto_verify()`, `crypto_random_token()`, and CSRF protection.
* 🗄️ **Automated SQL Migrations**: Sequential `.sql` transactions tracked in `_titanium_migrations`.
* 🛡️ **Global Middleware**: Pre-render request interceptors (`pages/_middleware.titanium`).
* 📁 **File-Based Routing**: Drop `.titanium` or `.ti` files into `pages/` (e.g. `pages/index.titanium`, `pages/users/[id].titanium`, `pages/api/users.titanium`).
* 🍪 **Built-in Session & Flash**: `session.flash("msg", "...")` and `session.get()`.

---

## ⚡ Quick Start

```bash
# 1. Build the standalone native binary
cargo build

# 2. Run the development server with live reload
./target/debug/titanium.exe dev ./example
```

Open **`http://127.0.0.1:8080`** in your browser!
Access the interactive Web Studio GUI at **`http://127.0.0.1:8080/__titanium_studio`**.

---

## 📝 Example: Single-File SSR + SQLite (`pages/index.titanium`)

```titanium
---
// 1. Ensure SQLite Table exists
db_exec("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, email TEXT, role TEXT);");

// 2. Handle POST Form Submission
if req.method == "POST" {
  if body.name != () && body.email != () {
    db_insert("users", #{ name: body.name, email: body.email, role: body.role });
    session.flash("msg", "✨ New user created in native Titanium engine!");
  }
  return redirect("/");
}

// 3. Query Database
let users = db_query("SELECT * FROM users ORDER BY id DESC");
let count = users.len();
let msg = session.flash("msg");
---
<!DOCTYPE html>
<html>
<head>
  <title>Titanium App</title>
  <link rel="stylesheet" href="/app.css">
</head>
<body>
  <h1>⚡ Titanium Web Runtime</h1>

  {% if msg %}
    <div class="alert alert-success">{{ msg }}</div>
  {% endif %}

  <form method="POST" action="/">
    <input type="text" name="name" placeholder="Full Name" required>
    <input type="email" name="email" placeholder="Email Address" required>
    <button type="submit">Create User</button>
  </form>

  <ul>
    {% for u in users %}
      <li><a href="/users/{{ u.id }}">{{ u.name }}</a> ({{ u.email }})</li>
    {% endfor %}
  </ul>
</body>
</html>
```

---

## 📂 File-Based Routing

| File Path | HTTP Route | Description |
|---|---|---|
| `pages/index.titanium` | `/` | Home dashboard (SSR) |
| `pages/about.titanium` | `/about` | About page |
| `pages/users/[id].titanium` | `/users/:id` | Dynamic route with `params.id` |
| `pages/api/users.titanium` | `/api/users` | REST API with `GET` and `POST` |
