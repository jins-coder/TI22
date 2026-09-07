pub mod studio;

use crate::core::context::{TitaniumRequest, TitaniumResponse};
use crate::core::engine::TitaniumEngine;
use crate::core::router::Router;
use crate::core::session::SessionStore;
use crate::server::studio::STUDIO_HTML;
use crate::services::ai::AiEngine;
use crate::services::cluster::{ClusterMesh, ClusterNode};
use crate::services::pubsub::PubSubHub;
use crate::services::queue::JobQueue;
use crate::services::vector::VectorEngine;
use crate::services::websocket::WebSocketHub;
use crate::storage::cache::CacheStore;
use crate::storage::db::Database;
use rhai::{Dynamic, Map};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Response, Server};

pub const TURBO_CLIENT_SCRIPT: &str = r#"
<script>
(function() {
  // 0. Titanium Realtime WebSockets & Singularity AI Engine (v9.0.0)
  window.Titanium = window.Titanium || {};
  window.Titanium.channels = {};
  window.Titanium.eventSources = {};

  window.Titanium.subscribe = function(channel, callback) {
    if (!window.Titanium.channels[channel]) {
      window.Titanium.channels[channel] = [];
      const sseUrl = '/__titanium_ws/stream?channel=' + encodeURIComponent(channel);
      const es = new EventSource(sseUrl);
      window.Titanium.eventSources[channel] = es;

      es.addEventListener('message', (e) => {
        try {
          const msg = JSON.parse(e.data);
          (window.Titanium.channels[channel] || []).forEach(cb => cb(msg.payload, msg));
          
          // Auto-update elements marked with data-live="channel"
          document.querySelectorAll(`[data-live="${channel}"]`).forEach(el => {
            const key = el.getAttribute('data-live-key');
            if (key && msg.payload && msg.payload[key] !== undefined) {
              el.textContent = msg.payload[key];
              el.classList.add('scale-105', 'text-sky-400');
              setTimeout(() => el.classList.remove('scale-105', 'text-sky-400'), 600);
            }
          });
        } catch (_) {}
      });
    }

    if (callback) window.Titanium.channels[channel].push(callback);

    return () => {
      if (callback) {
        window.Titanium.channels[channel] = (window.Titanium.channels[channel] || []).filter(cb => cb !== callback);
      }
    };
  };

  window.Titanium.broadcast = async function(channel, payload) {
    return fetch('/__titanium_ws/send', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ channel, payload })
    }).then(r => r.json()).catch(err => ({ success: false, error: err.message }));
  };

  window.Titanium.send = window.Titanium.broadcast;

  // Singularity AI Client Streaming Bridge
  window.Titanium.aiStream = function(prompt, options = {}) {
    const onToken = options.onToken || (() => {});
    const onDone = options.onDone || (() => {});
    const onError = options.onError || (() => {});
    const system = options.system || '';

    const url = '/__titanium_ai/stream?prompt=' + encodeURIComponent(prompt) + '&system=' + encodeURIComponent(system);
    const es = new EventSource(url);
    let fullText = '';

    es.addEventListener('token', (e) => {
      try {
        const data = JSON.parse(e.data);
        const token = data.token || '';
        fullText += token;
        onToken(token, fullText);
      } catch (_) {}
    });

    es.addEventListener('done', (e) => {
      try {
        const data = JSON.parse(e.data);
        onDone(fullText, data);
      } catch (_) {
        onDone(fullText);
      }
      es.close();
    });

    es.onerror = (err) => {
      onError(err);
      es.close();
    };

    return () => es.close();
  };

  window.Titanium.aiChat = async function(prompt, system) {
    return fetch('/__titanium_ai/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt, system })
    }).then(r => r.json());
  };

  window.Titanium.aiRag = async function(query, collection = 'documents', top_k = 3) {
    return fetch('/__titanium_ai/rag', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, collection, top_k })
    }).then(r => r.json());
  };
  // 1. Zero-Flicker Debounced Progress Bar Loader
  const bar = document.createElement('div');
  bar.id = '__titanium_progress';
  bar.style.cssText = 'position:fixed;top:0;left:0;height:3px;background:linear-gradient(90deg,#38bdf8,#818cf8);width:0%;opacity:0;z-index:999999;pointer-events:none;box-shadow:0 0 10px rgba(56,189,248,0.7);';
  document.documentElement.appendChild(bar);

  let progressTimer = null;
  let progressActive = false;

  function startProgress() {
    clearTimeout(progressTimer);
    // 120ms debounce threshold: instant/micro-second responses will NEVER flash the loader bar!
    progressTimer = setTimeout(() => {
      progressActive = true;
      bar.style.transition = 'width 0.25s cubic-bezier(0.1, 0.9, 0.2, 1), opacity 0.2s ease';
      bar.style.opacity = '1';
      bar.style.width = '35%';
      setTimeout(() => {
        if (progressActive && bar.style.width === '35%') {
          bar.style.width = '75%';
        }
      }, 200);
    }, 120);
  }

  function finishProgress() {
    clearTimeout(progressTimer);
    if (progressActive) {
      progressActive = false;
      bar.style.width = '100%';
      setTimeout(() => {
        bar.style.opacity = '0';
        setTimeout(() => {
          bar.style.transition = 'none';
          bar.style.width = '0%';
        }, 250);
      }, 150);
    }
  }

  function showHmrBadge(message) {
    let badge = document.getElementById('__titanium_hmr_badge');
    if (!badge) {
      badge = document.createElement('div');
      badge.id = '__titanium_hmr_badge';
      badge.style.cssText = 'position:fixed;bottom:16px;right:16px;background:rgba(9,13,22,0.92);backdrop-filter:blur(10px);color:#38bdf8;border:1px solid rgba(56,189,248,0.3);padding:6px 14px;border-radius:20px;font-size:12px;font-family:system-ui,-apple-system,sans-serif;font-weight:600;display:flex;align-items:center;gap:6px;box-shadow:0 8px 24px rgba(0,0,0,0.5);z-index:999999;transition:all 0.3s cubic-bezier(0.16, 1, 0.3, 1);transform:translateY(40px);opacity:0;pointer-events:none;';
      document.documentElement.appendChild(badge);
    }
    badge.innerHTML = message;
    badge.style.transform = 'translateY(0)';
    badge.style.opacity = '1';
    clearTimeout(badge._timer);
    badge._timer = setTimeout(() => {
      badge.style.transform = 'translateY(40px)';
      badge.style.opacity = '0';
    }, 1600);
  }

  // 2. High-Performance Soft-DOM Morphing (Preserves inputs, focus, cursor, & scroll)
  function morphTree(oldNode, newNode) {
    if (oldNode.nodeType !== newNode.nodeType || oldNode.nodeName !== newNode.nodeName) {
      oldNode.replaceWith(newNode.cloneNode(true));
      return;
    }
    if (oldNode.nodeType === Node.TEXT_NODE || oldNode.nodeType === Node.COMMENT_NODE) {
      if (oldNode.nodeValue !== newNode.nodeValue) {
        oldNode.nodeValue = newNode.nodeValue;
      }
      return;
    }
    // Sync attributes
    const oldAttrs = Array.from(oldNode.attributes || []);
    const newAttrs = Array.from(newNode.attributes || []);
    for (const attr of newAttrs) {
      if (oldNode.getAttribute(attr.name) !== attr.value) {
        oldNode.setAttribute(attr.name, attr.value);
      }
    }
    for (const attr of oldAttrs) {
      if (!newNode.hasAttribute(attr.name)) {
        oldNode.removeAttribute(attr.name);
      }
    }
    // Form Inputs Value Synchronization without stealing active typing focus
    if (oldNode.tagName === 'INPUT' || oldNode.tagName === 'TEXTAREA') {
      if (oldNode !== document.activeElement) {
        if (oldNode.value !== newNode.value) oldNode.value = newNode.value;
        if (oldNode.type === 'checkbox' || oldNode.type === 'radio') {
          oldNode.checked = newNode.checked;
        }
      }
    } else if (oldNode.tagName === 'SELECT') {
      if (oldNode !== document.activeElement) {
        oldNode.value = newNode.value;
      }
    }

    // Children diffing
    const oldChildren = Array.from(oldNode.childNodes);
    const newChildren = Array.from(newNode.childNodes);
    const max = Math.max(oldChildren.length, newChildren.length);
    for (let i = 0; i < max; i++) {
      const oldChild = oldChildren[i];
      const newChild = newChildren[i];
      if (oldChild && newChild) {
        morphTree(oldChild, newChild);
      } else if (!oldChild && newChild) {
        oldNode.appendChild(newChild.cloneNode(true));
      } else if (oldChild && !newChild) {
        oldChild.remove();
      }
    }
  }

  async function applyHtml(htmlText, pushUrl, preserveScroll = false) {
    const prevScrollX = window.scrollX;
    const prevScrollY = window.scrollY;

    // Capture active input and cursor position
    const activeEl = document.activeElement;
    const activeTag = activeEl ? activeEl.tagName : null;
    const activeId = activeEl && activeEl.id ? activeEl.id : null;
    const activeName = activeEl && activeEl.name ? activeEl.name : null;
    const isInputActive = activeEl && (activeTag === 'INPUT' || activeTag === 'TEXTAREA' || activeTag === 'SELECT');
    const selStart = (isInputActive && 'selectionStart' in activeEl) ? activeEl.selectionStart : null;
    const selEnd = (isInputActive && 'selectionEnd' in activeEl) ? activeEl.selectionEnd : null;

    const parser = new DOMParser();
    const doc = parser.parseFromString(htmlText, 'text/html');
    if (doc.title) document.title = doc.title;

    // Morph the body smoothly to avoid tearing down DOM elements
    morphTree(document.body, doc.body);

    // Restore active element focus and selection if needed
    if (isInputActive) {
      let targetInput = null;
      if (activeId) targetInput = document.getElementById(activeId);
      if (!targetInput && activeName) targetInput = document.querySelector(`[name="${activeName}"]`);
      if (targetInput && typeof targetInput.focus === 'function') {
        targetInput.focus();
        if (selStart !== null && selEnd !== null && typeof targetInput.setSelectionRange === 'function') {
          try { targetInput.setSelectionRange(selStart, selEnd); } catch (_) {}
        }
      }
    }

    if (pushUrl && window.location.href !== pushUrl) {
      window.history.pushState({}, '', pushUrl);
    }

    if (preserveScroll) {
      window.scrollTo({ left: prevScrollX, top: prevScrollY, behavior: 'instant' });
    } else {
      window.scrollTo({ top: 0, behavior: 'instant' });
    }
  }

  document.addEventListener('click', async (e) => {
    const link = e.target.closest('a');
    if (!link || !link.href) return;
    if (link.target && link.target !== '_self') return;
    if (link.hasAttribute('download') || link.getAttribute('rel') === 'external' || link.href.includes('/__titanium_studio')) return;

    const url = new URL(link.href, window.location.origin);
    if (url.origin !== window.location.origin) return;
    if (url.pathname === window.location.pathname && url.search === window.location.search && url.hash) return;

    e.preventDefault();
    startProgress();
    const isSamePage = (url.pathname === window.location.pathname && url.search === window.location.search);
    const shouldPreserve = link.hasAttribute('data-preserve-scroll') || isSamePage;

    try {
      const res = await fetch(url.href, { headers: { 'X-Titanium-Request': 'true' } });
      const text = await res.text();
      await applyHtml(text, res.redirected ? res.url : url.href, shouldPreserve);
    } catch {
      window.location.href = url.href;
    } finally {
      finishProgress();
    }
  });

  document.addEventListener('submit', async (e) => {
    const form = e.target;
    if (!form || !(form instanceof HTMLFormElement) || form.hasAttribute('data-native')) return;

    const rawAction = form.getAttribute('action');
    const action = (rawAction !== null && rawAction !== '') ? rawAction : window.location.href;
    const url = new URL(action, window.location.origin);
    if (url.origin !== window.location.origin) return;

    e.preventDefault();
    startProgress();

    const rawMethod = form.getAttribute('method');
    const method = (rawMethod || 'GET').toUpperCase();
    const formData = new FormData(form);

    try {
      let res;
      if (method === 'GET') {
        const params = new URLSearchParams(formData);
        url.search = params.toString();
        res = await fetch(url.href, { headers: { 'X-Titanium-Request': 'true' } });
      } else {
        const body = new URLSearchParams();
        for (const [k, v] of formData.entries()) {
          body.append(k, v);
        }
        res = await fetch(url.href, {
          method: 'POST',
          headers: { 'Content-Type': 'application/x-www-form-urlencoded', 'X-Titanium-Request': 'true' },
          body: body.toString()
        });
      }
      const text = await res.text();
      const targetUrl = new URL(res.redirected ? res.url : url.href, window.location.origin);
      const isSamePage = targetUrl.pathname === window.location.pathname;
      const preserve = form.hasAttribute('data-preserve-scroll') || isSamePage || method === 'POST';

      await applyHtml(text, res.redirected ? res.url : url.href, preserve);
    } catch {
      HTMLFormElement.prototype.submit.call(form);
    } finally {
      finishProgress();
    }
  });

  window.addEventListener('popstate', async () => {
    startProgress();
    try {
      const res = await fetch(window.location.href);
      const text = await res.text();
      await applyHtml(text, null, false);
    } catch {
      window.location.reload();
    } finally {
      finishProgress();
    }
  });

  // ⚡ Titanium HMR (Hot Module Replacement Engine)
  let lastMtime = null;
  async function checkHmr() {
    try {
      const res = await fetch('/__titanium_live', { cache: 'no-store' });
      if (res.ok) {
        const data = await res.json();
        if (lastMtime !== null && data.mtime && data.mtime > lastMtime) {
          if (data.change_type === 'css') {
            const links = document.querySelectorAll('link[rel="stylesheet"]');
            links.forEach(link => {
              const url = new URL(link.href, window.location.origin);
              url.searchParams.set('hmr', Date.now());
              link.href = url.href;
            });
            showHmrBadge('🎨 HMR: Hot-swapped CSS (' + (data.file || 'style') + ')');
          } else {
            const fresh = await fetch(window.location.href, { cache: 'no-store', headers: { 'X-Titanium-HMR': 'true' } });
            const text = await fresh.text();
            await applyHtml(text, null, true);
            showHmrBadge('⚡ HMR: Hot-updated ' + (data.file || 'component'));
          }
        }
        lastMtime = data.mtime;
      }
    } catch (_) {}
    setTimeout(checkHmr, 200);
  }
  setTimeout(checkHmr, 200);
})();
</script>
"#;

pub struct ServerConfig {
    pub root_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub db_path: Option<String>,
    pub workers: usize,
    pub queue_workers: usize,
}

pub fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.host, config.port);
    let server = Server::http(&addr).map_err(|e| e.to_string())?;

    // Graceful Ctrl+C shutdown handler
    let _ = ctrlc::set_handler(move || {
        println!("\n  🛑 Titanium server gracefully stopped.");
        std::process::exit(0);
    });

    println!("\n  ⚡ Titanium (Ti22) Native Engine v8.0.0 (Hyperdrive) ready on http://{}\n", addr);
    println!("  🎨 Titanium Web Studio GUI accessible at: http://{}/__titanium_studio\n", addr);

    let root_dir = config.root_dir.clone();
    let pages_dir = config.root_dir.join("pages");
    let public_dir = config.root_dir.join("public");
    let migrations_dir = config.root_dir.join("migrations");

    let db_file = config.db_path.unwrap_or_else(|| {
        config
            .root_dir
            .join(".titanium")
            .join("app.sqlite")
            .to_string_lossy()
            .to_string()
    });

    let db = Database::new(&db_file)?;

    // Run auto-migrations on boot
    if let Ok(count) = db.run_migrations(&migrations_dir) {
        if count > 0 {
            println!("  ✅ Successfully applied {} database migration(s)\n", count);
        }
    }

    let cache = CacheStore::new();
    let queue = JobQueue::new(config.queue_workers);
    let pubsub = PubSubHub::new();
    let ws = WebSocketHub::new();
    let ai = AiEngine::new();
    let vector = VectorEngine::new();
    let cluster = ClusterMesh::new("titanium-primary-node-01", &addr, true);

    let engine = TitaniumEngine::new(
        db.clone(),
        cache.clone(),
        queue.clone(),
        pubsub.clone(),
        ws.clone(),
        ai.clone(),
        vector.clone(),
        cluster.clone(),
    );
    let session_store = SessionStore::new();

    let router = Arc::new(Mutex::new(Router::new()));
    {
        router.lock().unwrap().scan_dir(&pages_dir);
    }

    let server_arc = Arc::new(server);
    let engine_arc = Arc::new(engine);
    let session_store_arc = Arc::new(session_store);
    let db_arc = Arc::new(db);
    let cache_arc = Arc::new(cache);
    let queue_arc = Arc::new(queue);
    let pubsub_arc = Arc::new(pubsub);
    let ws_arc = Arc::new(ws);
    let ai_arc = Arc::new(ai);
    let vector_arc = Arc::new(vector);
    let cluster_arc = Arc::new(cluster);

    let num_threads = if config.workers == 0 { 4 } else { config.workers };
    let mut handles = Vec::new();

    for _ in 0..num_threads {
        let server = Arc::clone(&server_arc);
        let engine = Arc::clone(&engine_arc);
        let router = Arc::clone(&router);
        let session_store = Arc::clone(&session_store_arc);
        let db = Arc::clone(&db_arc);
        let cache = Arc::clone(&cache_arc);
        let queue = Arc::clone(&queue_arc);
        let pubsub = Arc::clone(&pubsub_arc);
        let ws = Arc::clone(&ws_arc);
        let ai = Arc::clone(&ai_arc);
        let vector = Arc::clone(&vector_arc);
        let cluster = Arc::clone(&cluster_arc);
        let root_dir = root_dir.clone();
        let public_dir = public_dir.clone();
        let pages_dir = pages_dir.clone();

        let handle = std::thread::spawn(move || {
            for mut req in server.incoming_requests() {
                // Refresh router dynamically
                {
                    router.lock().unwrap().scan_dir(&pages_dir);
                }

                let raw_url = req.url().to_string();
                let parsed_url = match url::Url::parse(&format!("http://localhost{}", raw_url)) {
                    Ok(u) => u,
                    Err(_) => {
                        let resp = Response::from_string("400 Bad Request").with_status_code(400);
                        let _ = req.respond(resp);
                        continue;
                    }
                };

                let path = parsed_url.path();

                // Live reload / HMR heartbeat check
                if path == "/__titanium_live" {
                    let (mtime, change_type, file) = get_latest_mtime_details(&root_dir);
                    let body = format!(
                        r#"{{"mtime":{},"change_type":"{}","file":"{}","version":"6.0.0"}}"#,
                        mtime, change_type, file
                    );
                    let resp = Response::from_string(body)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                        .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-store"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v6.0.0 Studio GUI
                if path == "/__titanium_studio" {
                    let resp = Response::from_string(STUDIO_HTML)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v6.0.0 Studio API: Tables
                if path == "/__titanium_studio/api/tables" {
                    let sql = "SELECT name, type FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'";
                    let tables = db.query(sql, rhai::Array::new()).unwrap_or_default();
                    let json_val = rhai::serde::from_dynamic::<serde_json::Value>(&Dynamic::from(tables)).unwrap_or_else(|_| serde_json::json!([]));
                    let body = serde_json::json!({ "tables": json_val }).to_string();
                    let resp = Response::from_string(body)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v6.0.0 Studio API: Query Runner
                if path == "/__titanium_studio/api/query" {
                    let mut body_bytes = Vec::new();
                    let _ = req.as_reader().read_to_end(&mut body_bytes);
                    let query_sql = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                        Ok(val) => val["sql"].as_str().unwrap_or("").to_string(),
                        Err(_) => String::new(),
                    };

                    let trim_sql = query_sql.trim();
                    let is_select = trim_sql.to_uppercase().starts_with("SELECT")
                        || trim_sql.to_uppercase().starts_with("PRAGMA")
                        || trim_sql.to_uppercase().starts_with("EXPLAIN");

                    let resp_json = if is_select {
                        match db.query(trim_sql, rhai::Array::new()) {
                            Ok(rows) => {
                                let json_rows = rhai::serde::from_dynamic::<serde_json::Value>(&Dynamic::from(rows)).unwrap_or_else(|_| serde_json::json!([]));
                                serde_json::json!({ "success": true, "rows": json_rows })
                            }
                            Err(e) => serde_json::json!({ "success": false, "error": e }),
                        }
                    } else {
                        match db.exec(trim_sql) {
                            Ok(_) => serde_json::json!({ "success": true, "rows": [], "message": "Statement executed successfully" }),
                            Err(e) => serde_json::json!({ "success": false, "error": e }),
                        }
                    };

                    let resp = Response::from_string(resp_json.to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v6.0.0 Studio API: Cache Inspector
                if path == "/__titanium_studio/api/cache" {
                    let keys = cache.keys();
                    let stats = cache.stats();
                    let keys_json = rhai::serde::from_dynamic::<serde_json::Value>(&Dynamic::from(keys)).unwrap_or_else(|_| serde_json::json!([]));
                    let stats_json = rhai::serde::from_dynamic::<serde_json::Value>(&Dynamic::from(stats)).unwrap_or_else(|_| serde_json::json!({}));
                    let body = serde_json::json!({ "keys": keys_json, "stats": stats_json }).to_string();
                    let resp = Response::from_string(body)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v6.0.0 Studio API: Queue Inspector
                if path == "/__titanium_studio/api/queue" {
                    let stats = queue.stats();
                    let stats_json = rhai::serde::from_dynamic::<serde_json::Value>(&Dynamic::from(stats)).unwrap_or_else(|_| serde_json::json!({}));
                    let body = serde_json::json!({ "queue": stats_json }).to_string();
                    let resp = Response::from_string(body)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v7.0.0 Studio API: PubSub Topics Inspector
                if path == "/__titanium_studio/api/pubsub" {
                    let topics = pubsub.list_topics();
                    let topics_json = rhai::serde::from_dynamic::<serde_json::Value>(&Dynamic::from(topics)).unwrap_or_else(|_| serde_json::json!([]));
                    let body = serde_json::json!({ "topics": topics_json }).to_string();
                    let resp = Response::from_string(body)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v7.0.0 Studio API: Dynamic Route Map
                if path == "/__titanium_studio/api/routes" {
                    let r = router.lock().unwrap();
                    let routes_list: Vec<serde_json::Value> = r.routes.iter().map(|rt| {
                        serde_json::json!({
                            "pattern": rt.pattern,
                            "file": rt.file_path.display().to_string(),
                            "is_dynamic": rt.is_dynamic
                        })
                    }).collect();
                    let body = serde_json::json!({ "routes": routes_list }).to_string();
                    let resp = Response::from_string(body)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v8.0.0 Realtime Live Channels Stream (SSE / WS Handshake)
                if path == "/__titanium_ws/stream" || path == "/__titanium_ws" {
                    let channel = parsed_url.query_pairs()
                        .find(|(k, _)| k == "channel")
                        .map(|(_, v)| v.to_string())
                        .unwrap_or_else(|| "*".to_string());

                    let rx = if channel == "*" {
                        ws.subscribe_global()
                    } else {
                        ws.subscribe(&channel)
                    };

                    let mut sse_body = format!("event: connected\ndata: {{\"channel\":\"{}\",\"status\":\"connected\",\"version\":\"8.0.0\"}}\n\n", channel);
                    
                    // Flush any pending queue messages
                    while let Ok(msg) = rx.try_recv() {
                        if msg.channel != "ping" {
                            let json_msg = serde_json::to_string(&msg).unwrap_or_default();
                            sse_body.push_str(&format!("event: message\ndata: {}\n\n", json_msg));
                        }
                    }

                    let resp = Response::from_string(sse_body)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/event-stream"[..]).unwrap())
                        .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-cache, no-transform"[..]).unwrap())
                        .with_header(Header::from_bytes(&b"Connection"[..], &b"keep-alive"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v8.0.0 Realtime Broadcast Endpoint
                if path == "/__titanium_ws/send" || path == "/__titanium_ws/broadcast" {
                    let mut body_bytes = Vec::new();
                    let _ = req.as_reader().read_to_end(&mut body_bytes);
                    let resp_json = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                        Ok(val) => {
                            let channel = val["channel"].as_str().unwrap_or("general");
                            let payload = val.get("payload").cloned().unwrap_or(serde_json::Value::Null);
                            let msg = ws.broadcast(channel, payload);
                            serde_json::json!({ "success": true, "message": msg })
                        }
                        Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
                    };

                    let resp = Response::from_string(resp_json.to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v8.0.0 Realtime Live Channel History
                if path == "/__titanium_ws/history" {
                    let history = ws.recent_history();
                    let resp = Response::from_string(serde_json::json!({ "history": history }).to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v8.0.0 Realtime Stats
                if path == "/__titanium_ws/stats" || path == "/__titanium_studio/api/ws" {
                    let total = ws.total_clients();
                    let channels = ws.channels_list();
                    let history = ws.recent_history();
                    let resp = Response::from_string(serde_json::json!({
                        "total_clients": total,
                        "channels": channels,
                        "recent_messages": history.len(),
                        "version": "8.0.0"
                    }).to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium Diagnostics Health Endpoint
                if path == "/__titanium_health" {
                    let body = r#"{"status":"healthy","version":"9.0.0","engine":"Titanium (Ti22) Singularity","ai":true,"realtime_ws":true,"sqlite":"WAL","orm":true,"mvc":true,"cache":true,"queue":true,"pubsub":true,"vector":true,"media":true}"#;
                    let resp = Response::from_string(body)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v9.0.0 Singularity AI Streaming Endpoint (SSE)
                if path == "/__titanium_ai/stream" {
                    let prompt = parsed_url.query_pairs().find(|(k, _)| k == "prompt").map(|(_, v)| v.to_string()).unwrap_or_default();
                    let system = parsed_url.query_pairs().find(|(k, _)| k == "system").map(|(_, v)| v.to_string());
                    let chunks = ai.generate_stream_chunks(&prompt, system.as_deref());
                    let mut sse = String::new();
                    for (idx, c) in chunks.iter().enumerate() {
                        let payload = serde_json::json!({ "token": c, "index": idx });
                        sse.push_str(&format!("event: token\ndata: {}\n\n", payload));
                    }
                    let done_payload = serde_json::json!({ "status": "completed", "total_tokens": chunks.len() });
                    sse.push_str(&format!("event: done\ndata: {}\n\n", done_payload));
                    let resp = Response::from_string(sse)
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/event-stream"[..]).unwrap())
                        .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-cache, no-transform"[..]).unwrap())
                        .with_header(Header::from_bytes(&b"Connection"[..], &b"keep-alive"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v9.0.0 Singularity AI Chat Completion
                if path == "/__titanium_ai/chat" {
                    let mut body_bytes = Vec::new();
                    let _ = req.as_reader().read_to_end(&mut body_bytes);
                    let res_json = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                        Ok(val) => {
                            let prompt = val["prompt"].as_str().unwrap_or("");
                            let system = val.get("system").and_then(|s| s.as_str());
                            match ai.generate(prompt, system) {
                                Ok(response) => serde_json::json!({ "success": true, "response": response }),
                                Err(e) => serde_json::json!({ "success": false, "error": e }),
                            }
                        }
                        Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
                    };
                    let resp = Response::from_string(res_json.to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v9.0.0 Singularity Semantic RAG Query
                if path == "/__titanium_ai/rag" {
                    let mut body_bytes = Vec::new();
                    let _ = req.as_reader().read_to_end(&mut body_bytes);
                    let res_json = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                        Ok(val) => {
                            let query = val["query"].as_str().unwrap_or("");
                            let collection = val["collection"].as_str().unwrap_or("documents");
                            let top_k = val["top_k"].as_u64().unwrap_or(3) as usize;
                            match ai.rag_search_and_answer(query, &vector, collection, top_k) {
                                Ok(map) => {
                                    let dyn_map = Dynamic::from(map);
                                    let j = rhai::serde::from_dynamic::<serde_json::Value>(&dyn_map).unwrap_or(serde_json::json!({}));
                                    serde_json::json!({ "success": true, "result": j })
                                }
                                Err(e) => serde_json::json!({ "success": false, "error": e }),
                            }
                        }
                        Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
                    };
                    let resp = Response::from_string(res_json.to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v10.0.0 Supercluster: Node Topology & Discovered Mesh
                if path == "/__titanium_cluster/nodes" || path == "/__titanium_studio/api/cluster/nodes" {
                    let nodes = cluster.nodes_list();
                    let resp = Response::from_string(serde_json::json!({
                        "node_id": cluster.node_id(),
                        "is_primary": cluster.is_primary(),
                        "nodes": nodes,
                        "version": "10.0.0"
                    }).to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v10.0.0 Supercluster: Heartbeat Ping Registration
                if path == "/__titanium_cluster/heartbeat" {
                    let mut body_bytes = Vec::new();
                    let _ = req.as_reader().read_to_end(&mut body_bytes);
                    let res_json = match serde_json::from_slice::<ClusterNode>(&body_bytes) {
                        Ok(node) => {
                            cluster.register_heartbeat(node);
                            serde_json::json!({ "success": true, "status": "registered", "primary_node": cluster.node_id() })
                        }
                        Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
                    };
                    let resp = Response::from_string(res_json.to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v10.0.0 Supercluster: Distributed WAL & State Sync
                if path == "/__titanium_cluster/sync" {
                    let mut body_bytes = Vec::new();
                    let _ = req.as_reader().read_to_end(&mut body_bytes);
                    let res_json = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                        Ok(val) => {
                            let event_type = val["event_type"].as_str().unwrap_or("wal_sync");
                            let payload = val.get("payload").cloned().unwrap_or(serde_json::Value::Null);
                            let msg = cluster.record_sync(event_type, payload);
                            ws.broadcast("cluster", serde_json::to_value(&msg).unwrap_or(serde_json::Value::Null));
                            serde_json::json!({ "success": true, "sync_id": msg.id })
                        }
                        Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
                    };
                    let resp = Response::from_string(res_json.to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // Titanium v10.0.0 Supercluster Stats
                if path == "/__titanium_cluster/stats" || path == "/__titanium_studio/api/cluster" {
                    let dyn_stats = Dynamic::from(cluster.stats_map());
                    let json_val = rhai::serde::from_dynamic::<serde_json::Value>(&dyn_stats).unwrap_or(serde_json::json!({}));
                    let resp = Response::from_string(json_val.to_string())
                        .with_status_code(200)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                    let _ = req.respond(resp);
                    continue;
                }

                // 1. Static file handling with ETag and Cache-Control
                if let Some(static_resp) = serve_static(&public_dir, path) {
                    let _ = req.respond(static_resp);
                    continue;
                }

                // 2. Route matching
                let matched_route = {
                    let r = router.lock().unwrap();
                    r.match_path(path).map(|(route, params)| (route.clone(), params))
                };

                let (route, params) = match matched_route {
                    Some(m) => m,
                    None => {
                        let html = format!(
                            r#"<!DOCTYPE html>
<html><head><title>404 Not Found - Titanium</title></head>
<body style="font-family:system-ui;background:#090d16;color:#f8fafc;display:grid;place-content:center;height:90vh;text-align:center;">
  <h1 style="font-size:4rem;margin:0;color:#38bdf8;">404</h1>
  <p style="font-size:1.25rem;color:#94a3b8;">Page not found: <code>{}</code></p>
  <a href="/" style="color:#38bdf8;text-decoration:none;">← Return Home</a>
</body></html>"#,
                            path
                        );
                        let resp = Response::from_string(html)
                            .with_status_code(404)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap());
                        let _ = req.respond(resp);
                        continue;
                    }
                };

                // 3. Extract Session from Cookies
                let cookie_header = req
                    .headers()
                    .iter()
                    .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("Cookie"))
                    .map(|h| h.value.as_str().to_string());

                let mut session_id = None;
                if let Some(ref cookies) = cookie_header {
                    for cookie in cookie::Cookie::split_parse(cookies) {
                        if let Ok(c) = cookie {
                            if c.name() == "titanium_session" || c.name() == "carbon_session" {
                                session_id = Some(c.value().to_string());
                                break;
                            }
                        }
                    }
                }

                let is_new_session = session_id.is_none();
                let sid = session_id.unwrap_or_else(|| {
                    format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
                });
                let session_handle = session_store.get_session(&sid);

                // 4. Parse request parameters and body
                let method = req.method().as_str().to_string();
                let mut query_map = Map::new();
                for (k, v) in parsed_url.query_pairs() {
                    query_map.insert(k.to_string().into(), Dynamic::from(v.to_string()));
                }

                let mut params_map = Map::new();
                for (k, v) in params {
                    params_map.insert(k.into(), Dynamic::from(v));
                }

                let mut headers_map = Map::new();
                for h in req.headers() {
                    headers_map.insert(h.field.as_str().to_string().into(), Dynamic::from(h.value.as_str().to_string()));
                }

                let mut body_map = Map::new();
                let is_post_or_put = method == "POST" || method == "PUT" || method == "PATCH";
                let body_raw = if is_post_or_put {
                    let mut body_bytes = Vec::new();
                    let _ = req.as_reader().read_to_end(&mut body_bytes);
                    String::from_utf8_lossy(&body_bytes).to_string()
                } else {
                    String::new()
                };

                if is_post_or_put {
                    let is_json = req
                        .headers()
                        .iter()
                        .any(|h| h.field.as_str().as_str().eq_ignore_ascii_case("Content-Type") && h.value.as_str().contains("application/json"));

                    if is_json {
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body_raw) {
                            if let Ok(dyn_val) = rhai::serde::to_dynamic(&json_val) {
                                if let Some(m) = dyn_val.try_cast::<Map>() {
                                    body_map = m;
                                }
                            }
                        }
                    } else {
                        // Form URL-encoded
                        for (k, v) in url::form_urlencoded::parse(body_raw.as_bytes()) {
                            body_map.insert(k.to_string().into(), Dynamic::from(v.to_string()));
                        }
                    }
                }

                let titanium_req = TitaniumRequest {
                    method: method.clone(),
                    path: path.to_string(),
                    params: params_map,
                    query: query_map,
                    body: body_map,
                    headers: headers_map,
                };

                // 5. Run Global Middleware (if exists in pages/_middleware.titanium / .ti / .carbon)
                let middleware_path = {
                    let cand1 = root_dir.join("pages").join("_middleware.titanium");
                    let cand2 = root_dir.join("pages").join("_middleware.ti");
                    let cand3 = root_dir.join("pages").join("_middleware.carbon");
                    if cand1.is_file() {
                        cand1
                    } else if cand2.is_file() {
                        cand2
                    } else {
                        cand3
                    }
                };

                match engine.run_middleware(&middleware_path, &titanium_req, &session_handle) {
                    Ok(Some(early_resp)) => {
                        let mut http_resp = match early_resp {
                            TitaniumResponse::Redirect { status, location } => Response::from_string(format!("Redirecting to {}", location))
                                .with_status_code(status)
                                .with_header(Header::from_bytes(&b"Location"[..], location.as_bytes()).unwrap()),
                            TitaniumResponse::Json { status, data } => {
                                let json_val = rhai::serde::from_dynamic::<serde_json::Value>(&data).unwrap_or(serde_json::json!({}));
                                Response::from_string(json_val.to_string())
                                    .with_status_code(status)
                                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                            }
                            TitaniumResponse::Html { status, body } => Response::from_string(body)
                                .with_status_code(status)
                                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap()),
                            _ => Response::from_string("OK").with_status_code(200),
                        };

                        if is_new_session {
                            let cookie_val = format!("titanium_session={}; Path=/; HttpOnly; SameSite=Lax", sid);
                            http_resp.add_header(Header::from_bytes(&b"Set-Cookie"[..], cookie_val.as_bytes()).unwrap());
                        }
                        let _ = req.respond(http_resp);
                        continue;
                    }
                    Err(err) => {
                        eprintln!("  ⚠️ Middleware error in {}: {}", middleware_path.display(), err);
                    }
                    _ => {}
                }

                // 6. Render component via Engine
                let result = engine.execute(&route.file_path, &root_dir, &titanium_req, &session_handle);

                let mut http_resp = match result {
                    Ok(TitaniumResponse::Html { status, mut body }) => {
                        // Inject Turbo Client Script for instant SPA feel
                        if body.contains("</body>") {
                            body = body.replace("</body>", &format!("{}\n</body>", TURBO_CLIENT_SCRIPT));
                        } else {
                            body.push_str(TURBO_CLIENT_SCRIPT);
                        }

                        Response::from_string(body)
                            .with_status_code(status)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
                    }
                    Ok(TitaniumResponse::Json { status, data }) => {
                        let json_val = rhai::serde::from_dynamic::<serde_json::Value>(&data).unwrap_or(serde_json::json!({}));
                        Response::from_string(json_val.to_string())
                            .with_status_code(status)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                    }
                    Ok(TitaniumResponse::Redirect { status, location }) => Response::from_string(format!("Redirecting to {}", location))
                        .with_status_code(status)
                        .with_header(Header::from_bytes(&b"Location"[..], location.as_bytes()).unwrap()),
                    Ok(TitaniumResponse::Sse { status, body }) => Response::from_string(body)
                        .with_status_code(status)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/event-stream"[..]).unwrap())
                        .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..]).unwrap())
                        .with_header(Header::from_bytes(&b"Connection"[..], &b"keep-alive"[..]).unwrap()),
                    Ok(TitaniumResponse::Raw { status, content_type, bytes }) => Response::new(
                        tiny_http::StatusCode(status),
                        vec![Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap()],
                        Cursor::new(bytes.clone()),
                        Some(bytes.len()),
                        None,
                    ),
                    Ok(TitaniumResponse::View { status, view, .. }) => Response::from_string(format!("View template: {}", view))
                        .with_status_code(status)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap()),
                    Err(err_msg) => {
                        let formatted_err = format!(
                            r#"<!DOCTYPE html>
<html>
<head>
  <title>Runtime Error - Titanium Engine</title>
  <style>
    body {{ background: #090d16; color: #f8fafc; font-family: system-ui, sans-serif; padding: 40px; margin: 0; }}
    .err-card {{ background: #0f172a; border: 1px solid #ef4444; border-radius: 12px; padding: 24px; max-width: 800px; margin: 40px auto; box-shadow: 0 10px 30px rgba(239,68,68,0.2); }}
    h1 {{ color: #ef4444; margin-top: 0; font-size: 20px; display: flex; align-items: center; gap: 8px; }}
    pre {{ background: #050811; padding: 16px; border-radius: 8px; overflow-x: auto; font-family: monospace; color: #fca5a5; font-size: 14px; line-height: 1.5; }}
    .badge {{ background: rgba(239,68,68,0.2); color: #f87171; padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: bold; }}
  </style>
</head>
<body>
  <div class="err-card">
    <div style="display:flex; justify-content:space-between; align-items:center;">
      <h1><span>⚠️</span> Titanium Runtime Exception</h1>
      <span class="badge">500 Internal Error</span>
    </div>
    <p style="color:#94a3b8; font-size:14px;">An exception occurred while evaluating component: <code>{}</code></p>
    <pre><code>{}</code></pre>
  </div>
</body>
</html>"#,
                            route.file_path.display(),
                            err_msg
                        );
                        Response::from_string(formatted_err)
                            .with_status_code(500)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
                    }
                };

                if is_new_session {
                    let cookie_val = format!("titanium_session={}; Path=/; HttpOnly; SameSite=Lax", sid);
                    http_resp.add_header(Header::from_bytes(&b"Set-Cookie"[..], cookie_val.as_bytes()).unwrap());
                }

                let _ = req.respond(http_resp);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

fn serve_static(public_dir: &Path, path: &str) -> Option<Response<Cursor<Vec<u8>>>> {
    if path.contains("..") {
        return None;
    }

    let rel_path = path.trim_start_matches('/');
    if rel_path.is_empty() {
        return None;
    }

    let file_path = public_dir.join(rel_path);
    if file_path.is_file() {
        if let Ok(bytes) = std::fs::read(&file_path) {
            let mime = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            let cursor = Cursor::new(bytes);
            let resp = Response::new(
                tiny_http::StatusCode(200),
                vec![
                    Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap(),
                    Header::from_bytes(&b"Cache-Control"[..], &b"public, max-age=3600"[..]).unwrap(),
                ],
                cursor,
                Some(file_path.metadata().map(|m| m.len() as usize).unwrap_or(0)),
                None,
            );
            return Some(resp);
        }
    }
    None
}

fn get_latest_mtime_details(root_dir: &Path) -> (u64, String, String) {
    let mut latest = 0u64;
    let mut latest_file = String::new();
    let mut change_type = "dom".to_string();

    let dirs = vec![
        root_dir.join("pages"),
        root_dir.join("public"),
        root_dir.join("layouts"),
        root_dir.join("components"),
    ];

    for d in dirs {
        if d.is_dir() {
            scan_dir_mtime(&d, &mut latest, &mut latest_file, &mut change_type);
        }
    }

    // Also check root titanium.toml
    let toml_path = root_dir.join("titanium.toml");
    if let Ok(meta) = toml_path.metadata() {
        if let Ok(modified) = meta.modified() {
            if let Ok(dur) = modified.duration_since(std::time::UNIX_EPOCH) {
                let m = dur.as_millis() as u64;
                if m > latest {
                    latest = m;
                    latest_file = "titanium.toml".to_string();
                    change_type = "dom".to_string();
                }
            }
        }
    }

    (latest, change_type, latest_file)
}

fn scan_dir_mtime(dir: &Path, latest: &mut u64, latest_file: &mut String, change_type: &mut String) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_mtime(&path, latest, latest_file, change_type);
            } else if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(dur) = modified.duration_since(std::time::UNIX_EPOCH) {
                        let m = dur.as_millis() as u64;
                        if m > *latest {
                            *latest = m;
                            *latest_file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if ext.eq_ignore_ascii_case("css") {
                                *change_type = "css".to_string();
                            } else if ext.eq_ignore_ascii_case("titanium") || ext.eq_ignore_ascii_case("ti") || ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("carbon") {
                                *change_type = "dom".to_string();
                            } else {
                                *change_type = "static".to_string();
                            }
                        }
                    }
                }
            }
        }
    }
}
