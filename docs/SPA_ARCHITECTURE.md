# ⚡ Titanium (Ti22) Turbo SPA Architecture

> **HTML-over-the-Wire • Zero Full-Page Reloads • 0ms White Screen Flash • Client State Preservation**

---

## 📖 1. Overview & Core Philosophy

Titanium uses an integrated **Turbo Single Page Application (SPA)** client engine that bridges the simplicity of Server-Side Rendering (SSR) with the seamless interactivity of client-side frameworks (like Next.js or Remix), with **zero JavaScript build steps or hydration overhead**.

Every page in Titanium (`.titanium`, `.ti`) is a full single-file component that renders pure semantic HTML. On the client, Titanium's runtime intercepts interactions and seamlessly swaps the DOM in milliseconds.

```
+-----------------------------------------------------------------------------+
|                                TITANIUM CLIENT                              |
|                                                                             |
|   [User Click / Form Submit] ---> [Interception & Progress Bar]             |
|                                                  |                          |
|                                                  v                          |
|   [Soft-DOM Swap & History Push] <--- [Fetch HTML with X-Titanium-Request]  |
|                 |                                                           |
|                 v                                                           |
|   [State & Scroll Restored] (Zero White Flash / Zero Framework Hydration)   |
+-----------------------------------------------------------------------------+
```

---

## ⚙️ 2. How the SPA Engine Operates

### 2.1 Link Navigation (`<a>` Clicks)
1. **Interception:** Any standard link `<a href="/product/1">` within the same origin is automatically intercepted.
2. **Progress Telemetry:** An ultra-light top progress bar (`#__titanium_progress`) starts instantly.
3. **Background Fetch:** The target URL is fetched asynchronously with header `X-Titanium-Request: true`.
4. **Soft-DOM Patching:**
   - The `<title>` is updated.
   - `document.body.innerHTML` is replaced with the new server-rendered HTML.
   - Any inline scripts are cleanly re-evaluated.
5. **History Management:** `window.history.pushState` updates the URL for native browser Back/Forward navigation.
6. **Scroll Management:** The window is smoothly scrolled to top for new route navigations, or preserved if targeting the same view.

---

### 2.2 Turbo Form Submissions (`<form method="POST">`)
1. **Asynchronous Form Submission:** Standard HTML forms `<form method="POST" action="/cart">` are intercepted.
2. **Body Serialization:** Form data is encoded into `application/x-www-form-urlencoded` and sent as a background `POST` request.
3. **Server-Side Mutation & Redirect:** The server executes the SQLite transaction, updates the session or flashes, and returns a redirect response (e.g. `return redirect("/cart")`).
4. **In-Place DOM Swap:** The redirected view is fetched and soft-swapped.
5. **Scroll & Input Preservation:** The user's exact scroll position (`window.scrollX` / `window.scrollY`) is **preserved** without jumping to the top of the page.

---

### 2.3 Opting Out of SPA (`data-native`)
When an external link, file download, or hard full-page browser reload is explicitly required, add the `data-native` attribute or `target="_blank"`:

```html
<!-- Traditional Full-Page Reload -->
<a href="/dashboard" data-native>Hard Reload Dashboard</a>

<!-- Traditional Multipart File Upload -->
<form method="POST" action="/upload" data-native enctype="multipart/form-data">
  <input type="file" name="attachment">
  <button type="submit">Upload</button>
</form>
```

---

## 🎨 3. Hot Module Replacement (HMR) & Live State

During development with `titanium dev .`:

| Asset Type | Update Mechanism | Latency | State Behavior |
| :--- | :--- | :--- | :--- |
| **CSS Stylesheet** (`public/*.css`) | Hot-swaps `<link rel="stylesheet">` `href` with cache-busting timestamp | `<5ms` | **100% Preserved** (Zero DOM disruption, zero input loss) |
| **Templates / Pages** (`.titanium`, `.ti`) | Fetches updated HTML, captures active input focus/value, soft-swaps DOM | `<20ms` | **Preserved** (Active input values and scroll positions restored) |
| **Config** (`titanium.toml`) | Auto-reloads server configuration | `<50ms` | Server restarted gracefully |

---

## 📊 4. Architecture Comparison

| Feature | Traditional MPA (PHP/Rails) | Heavy SPA (React/Next.js) | **Titanium Turbo SPA** |
| :--- | :--- | :--- | :--- |
| **Initial Load Speed** | ⚡ Instant (Plain HTML) | 🐢 Heavy JS bundle (2MB+) | **⚡ Instant (Pure Native SSR)** |
| **Page Transitions** | ⚪ White Screen Flash | ⚡ Fast (Client Router) | **⚡ Instant Soft-DOM Swap** |
| **Client Memory** | 🟢 Extremely Low | 🔴 High (Virtual DOM Tree) | **🟢 Extremely Low (Zero VDOM)** |
| **Form Submissions** | ⚪ Full page reload | 🟡 Complex state / Redux | **⚡ Native `<form>` over HTTP** |
| **SEO & Crawlers** | 🟢 100% Native HTML | 🟡 Requires SSR/SSG setup | **🟢 100% Native HTML** |
| **State Retention** | ❌ Lost on reload | 🟢 Preserved in memory | **🟢 Preserved via Session & Turbo** |
