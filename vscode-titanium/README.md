# Titanium Web Runtime (Ti22) for Visual Studio Code

Official VS Code extension for **Titanium (Ti22)** — the ultra-fast, native web runtime with embedded SQLite, single-file SSR components, built-in key-value cache, async background job queues, local vector search, and Web Studio GUI.

---

## 🚀 Features

- **⚡ Syntax Highlighting:** Full semantic highlighting for `.titanium` and `.ti` single-file components (frontmatter script logic + HTML/Jinja template interpolation).
- **💡 Rich Snippets:** Instant scaffolding for pages (`tipage`), REST APIs (`tiapi`), SSE streaming (`tisse`), database queries (`tidb`), caching (`ticache`), and background jobs (`tijob`).
- **🎯 One-Click Dev Server:** Run `titanium dev .` with instant soft-DOM live reload from the status bar or command palette.
- **🛠️ Web Studio Integration:** Quick link to launch the interactive SQLite & query console at `http://127.0.0.1:8080/__titanium_studio`.
- **🗄️ SQL Migrations & Builds:** Run automatic migrations and production bundling directly inside VS Code.

---

## ⌨️ Code Snippets

| Prefix | Description |
| :--- | :--- |
| `tipage` | Scaffold complete Single-File Component SSR page |
| `tiapi` | Scaffold JSON REST API endpoint with input validation |
| `tisse` | Scaffold Real-Time Server-Sent Events (SSE) live stream |
| `tidb` | Parameterized SQLite query (`db_query`) |
| `ticache` | In-memory key-value cache get/set pattern with TTL |
| `tijob` | Dispatch asynchronous task to native threadpool worker queue |
| `tivector` | Vector cosine similarity AI embedding search |

---

## 🛠️ Commands

Open the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):
- `Titanium: Quick Actions Menu` (`titanium.showMenu`)
- `Titanium: Start Live Dev Server` (`titanium.startDev`)
- `Titanium: Open Web Studio` (`titanium.openStudio`)
- `Titanium: Run SQL Migrations` (`titanium.runMigrations`)
- `Titanium: Build Standalone Package` (`titanium.build`)
- `Titanium: Project Diagnostics` (`titanium.info`)
