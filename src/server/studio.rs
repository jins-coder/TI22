pub const STUDIO_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Titanium Web Studio (Ti22) v9.0.0</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;600;700;800&family=JetBrains+Mono:wght@400;600&display=swap">
  <style>
    :root {
      --bg: #090d16;
      --card: #0f172a;
      --card-border: #1e293b;
      --primary: #38bdf8;
      --accent: #a855f7;
      --text: #f8fafc;
      --muted: #94a3b8;
      --danger: #f43f5e;
      --success: #10b981;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: 'Plus Jakarta Sans', sans-serif; background: var(--bg); color: var(--text); display: flex; height: 100vh; overflow: hidden; }
    
    /* Sidebar */
    .sidebar { width: 260px; background: var(--card); border-right: 1px solid var(--card-border); display: flex; flex-direction: column; padding: 24px 16px; shrink: 0; }
    .brand { display: flex; align-items: center; gap: 10px; font-weight: 800; font-size: 18px; color: var(--primary); margin-bottom: 28px; }
    .nav-item { display: flex; align-items: center; gap: 10px; padding: 12px 14px; border-radius: 8px; color: var(--muted); cursor: pointer; font-weight: 600; font-size: 14px; transition: all 0.2s; margin-bottom: 4px; }
    .nav-item:hover, .nav-item.active { background: rgba(56, 189, 248, 0.1); color: var(--primary); }
    .status-badge { margin-top: auto; padding: 12px; background: rgba(16, 185, 129, 0.1); border: 1px solid rgba(16, 185, 129, 0.2); border-radius: 8px; font-size: 12px; color: var(--success); font-weight: 700; display: flex; align-items: center; gap: 6px; }
    
    /* Main Content */
    .main { flex: 1; display: flex; flex-direction: column; overflow-y: auto; padding: 32px; }
    .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 28px; }
    h1 { font-size: 26px; font-weight: 800; }
    .subtitle { color: var(--muted); font-size: 14px; margin-top: 4px; }
    
    /* Tabs */
    .tab-pane { display: none; }
    .tab-pane.active { display: block; }
    
    /* Cards & Grids */
    .card { background: var(--card); border: 1px solid var(--card-border); border-radius: 12px; padding: 24px; margin-bottom: 24px; }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 20px; margin-bottom: 24px; }
    .stat-card { background: rgba(255, 255, 255, 0.02); border: 1px solid var(--card-border); border-radius: 10px; padding: 20px; }
    .stat-val { font-size: 32px; font-weight: 800; color: var(--primary); margin: 8px 0; }
    .stat-lbl { color: var(--muted); font-size: 13px; font-weight: 600; }
    
    /* SQL Editor */
    .editor-wrapper { display: flex; flex-direction: column; gap: 12px; }
    textarea.sql-input { width: 100%; height: 120px; background: #050811; border: 1px solid var(--card-border); border-radius: 8px; padding: 16px; color: #38bdf8; font-family: 'JetBrains Mono', monospace; font-size: 14px; resize: vertical; }
    textarea.sql-input:focus { outline: none; border-color: var(--primary); }
    .btn { background: var(--primary); color: #000; font-weight: 700; padding: 10px 20px; border-radius: 8px; border: none; cursor: pointer; align-self: flex-start; transition: all 0.2s; }
    .btn:hover { opacity: 0.9; transform: translateY(-1px); }
    .btn-secondary { background: rgba(255,255,255,0.08); color: var(--text); border: 1px solid var(--card-border); }
    
    /* Tables */
    .table-container { overflow-x: auto; margin-top: 16px; }
    table.data-table { width: 100%; border-collapse: collapse; text-align: left; font-size: 13px; }
    table.data-table th, table.data-table td { padding: 12px 16px; border-bottom: 1px solid var(--card-border); }
    table.data-table th { background: rgba(255, 255, 255, 0.02); color: var(--muted); font-weight: 700; }
    table.data-table tr:hover { background: rgba(255, 255, 255, 0.01); }
    
    .tag { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 11px; font-weight: 700; background: rgba(56, 189, 248, 0.1); color: var(--primary); }
    .tag-accent { background: rgba(168, 85, 247, 0.1); color: var(--accent); }
    .tag-success { background: rgba(16, 185, 129, 0.1); color: var(--success); }
    
    /* Live Log Console */
    .log-stream { background: #050811; border: 1px solid var(--card-border); border-radius: 8px; padding: 16px; height: 260px; overflow-y: auto; font-family: 'JetBrains Mono', monospace; font-size: 12px; color: #a5f3fc; }
    .log-entry { margin-bottom: 6px; padding-bottom: 6px; border-bottom: 1px solid rgba(255,255,255,0.05); }
  </style>
</head>
<body>
  <div class="sidebar">
    <div class="brand">
      <span>⚡</span>
      <span>Titanium Studio</span>
    </div>
    <div class="nav-item active" onclick="switchTab('overview', this)">📊 Overview & Metrics</div>
    <div class="nav-item" onclick="switchTab('cluster', this)">🌐 Supercluster Mesh</div>
    <div class="nav-item" onclick="switchTab('ai', this)">🧠 AI Playground & RAG</div>
    <div class="nav-item" onclick="switchTab('realtime', this)">📡 Live WebSockets</div>
    <div class="nav-item" onclick="switchTab('database', this)">🗄️ Database Manager</div>
    <div class="nav-item" onclick="switchTab('sql', this)">⚡ SQL Console</div>
    <div class="nav-item" onclick="switchTab('routes', this)">🧭 Route Map</div>
    <div class="status-badge">
      <span>●</span>
      <span>v10.0.0 Titanium X</span>
    </div>
  </div>

  <div class="main">
    <!-- OVERVIEW TAB -->
    <div id="tab-overview" class="tab-pane active">
      <div class="header">
        <div>
          <h1>Engine Overview</h1>
          <p class="subtitle">Titanium v8.0.0 (Ti22) Hyperdrive — Realtime WebSockets, Dual Engine & Embedded SQLite.</p>
        </div>
      </div>
      <div class="grid">
        <div class="stat-card">
          <div class="stat-lbl">REALTIME CHANNELS</div>
          <div class="stat-val" style="color:var(--primary);">Active</div>
          <div style="color:var(--success); font-size:12px; font-weight:700;">● WebSockets & Live Hub</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">STORAGE & ORM</div>
          <div class="stat-val" style="color:var(--accent);">SQLite WAL</div>
          <div style="color:var(--muted); font-size:12px;">ActiveRecord Query Engine</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">SPA RUNTIME</div>
          <div class="stat-val" style="color:#10b981;">Turbo Soft-DOM</div>
          <div style="color:var(--muted); font-size:12px;">0ms Input Flashes • Morphing</div>
        </div>
      </div>
      <div class="card">
        <h3 style="margin-bottom:12px;">⚡ Platform Capabilities</h3>
        <p style="color:var(--muted); font-size:14px; line-height:1.6;">
          Titanium v8.0.0 integrates native real-time WebSockets and bi-directional live topic broadcasting directly into the multi-threaded Rust execution pipeline alongside single-file components and domain-driven MVC architecture.
        </p>
      </div>
    </div>

    <!-- SUPERCLUSTER TAB -->
    <div id="tab-cluster" class="tab-pane">
      <div class="header">
        <div>
          <h1>Supercluster Mesh & Node Topology</h1>
          <p class="subtitle">Distributed Primary-Replica nodes, SQLite WAL sync, and cluster heartbeats.</p>
        </div>
      </div>

      <div class="grid">
        <div class="stat-card">
          <div class="stat-lbl">PRIMARY NODE</div>
          <div class="stat-val" id="cluster-primary-id" style="color:var(--primary); font-size:20px;">Primary</div>
          <div style="color:var(--success); font-size:12px; font-weight:700;">● Mesh Leader Active</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">DISCOVERED NODES</div>
          <div class="stat-val" id="cluster-nodes-count" style="color:var(--accent);">1</div>
          <div style="color:var(--muted); font-size:12px;">Active Mesh Participants</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">DISTRIBUTED REPLICATION</div>
          <div class="stat-val" style="color:var(--success); font-size:20px;">SQLite WAL</div>
          <div style="color:var(--muted); font-size:12px;">Real-time transaction broadcasting</div>
        </div>
      </div>

      <div class="card">
        <h3 style="margin-bottom:16px;">🌐 Active Cluster Node Topology</h3>
        <div id="cluster-nodes-table">Loading cluster topology...</div>
      </div>

      <div class="card">
        <h3 style="margin-bottom:16px;">⚡ Cross-Node Distributed Broadcast</h3>
        <div style="display:grid; grid-template-columns: 1fr 2fr; gap:12px; margin-bottom:12px;">
          <div>
            <label style="font-size:12px; color:var(--muted); display:block; margin-bottom:6px; font-weight:700;">Sync Event Type</label>
            <input id="cluster-sync-type" value="wal_commit" style="width:100%; background:#050811; border:1px solid var(--card-border); padding:10px; border-radius:8px; color:#fff; font-family:'JetBrains Mono'; font-size:13px;" />
          </div>
          <div>
            <label style="font-size:12px; color:var(--muted); display:block; margin-bottom:6px; font-weight:700;">Sync Payload</label>
            <input id="cluster-sync-payload" value='{"table": "products", "action": "replicate", "version": 10}' style="width:100%; background:#050811; border:1px solid var(--card-border); padding:10px; border-radius:8px; color:#38bdf8; font-family:'JetBrains Mono'; font-size:13px;" />
          </div>
        </div>
        <button class="btn" onclick="syncClusterWal()">Dispatch Cluster Sync</button>
      </div>
    </div>

    <!-- AI PLAYGROUND & RAG TAB -->
    <div id="tab-ai" class="tab-pane">
      <div class="header">
        <div>
          <h1>Singularity AI & RAG Agent Playground</h1>
          <p class="subtitle">Live token streaming, prompt engineering, and semantic vector grounding.</p>
        </div>
      </div>

      <div class="grid">
        <div class="stat-card">
          <div class="stat-lbl">AI ENGINE MODE</div>
          <div class="stat-val" style="color:var(--primary); font-size:24px;">Singularity 1B</div>
          <div style="color:var(--muted); font-size:12px;">Local Embedded / Ollama / OpenAI</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">LATENCY & STREAMING</div>
          <div class="stat-val" style="color:var(--success); font-size:24px;">&lt; 5ms SSE</div>
          <div style="color:var(--muted); font-size:12px;">Zero-allocation token streaming</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">RAG VECTOR EMBEDDINGS</div>
          <div class="stat-val" style="color:var(--accent); font-size:24px;">VectorEngine</div>
          <div style="color:var(--muted); font-size:12px;">Cosine similarity knowledge store</div>
        </div>
      </div>

      <div class="card">
        <h3 style="margin-bottom:16px;">🧠 Interactive AI Token Streamer</h3>
        <div style="display:flex; flex-direction:column; gap:12px; margin-bottom:14px;">
          <div>
            <label style="font-size:12px; color:var(--muted); display:block; margin-bottom:6px; font-weight:700;">System Persona (Optional)</label>
            <input id="ai-system-prompt" value="You are the Titanium Singularity E-Commerce Shopping Concierge." style="width:100%; background:#050811; border:1px solid var(--card-border); padding:10px; border-radius:8px; color:#fff; font-family:'JetBrains Mono'; font-size:13px;" />
          </div>
          <div>
            <label style="font-size:12px; color:var(--muted); display:block; margin-bottom:6px; font-weight:700;">User Prompt</label>
            <input id="ai-user-prompt" value="What titanium products do you recommend for outdoor daily wear?" style="width:100%; background:#050811; border:1px solid var(--card-border); padding:10px; border-radius:8px; color:#38bdf8; font-family:'JetBrains Mono'; font-size:13px;" />
          </div>
        </div>
        <div style="display:flex; gap:10px;">
          <button class="btn" onclick="runStudioAiStream()">⚡ Stream AI Response</button>
          <button class="btn btn-secondary" onclick="runStudioAiRag()">🔍 Test Semantic RAG</button>
        </div>

        <div style="margin-top:16px;">
          <label style="font-size:12px; color:var(--muted); display:block; margin-bottom:6px; font-weight:700;">Live Generated Output</label>
          <div id="ai-stream-output" class="log-stream" style="min-height:140px; color:#f8fafc; font-family:'Plus Jakarta Sans', sans-serif; line-height:1.6; font-size:14px;">
            AI response will stream here in real-time...
          </div>
        </div>
      </div>
    </div>

    <!-- REALTIME WEBSOCKETS TAB -->
    <div id="tab-realtime" class="tab-pane">
      <div class="header">
        <div>
          <h1>Realtime WebSocket & Live Channel Inspector</h1>
          <p class="subtitle">Monitor active topic broadcasts, connected clients, and dispatch live payloads.</p>
        </div>
      </div>

      <div class="grid">
        <div class="stat-card">
          <div class="stat-lbl">LIVE CHANNELS</div>
          <div class="stat-val" id="ws-channels-count" style="color:var(--primary);">0</div>
          <div style="color:var(--muted); font-size:12px;">Active Topics</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">TOTAL CLIENTS</div>
          <div class="stat-val" id="ws-clients-count" style="color:var(--accent);">0</div>
          <div style="color:var(--muted); font-size:12px;">Connected Subscribers</div>
        </div>
      </div>

      <div class="card">
        <h3 style="margin-bottom:16px;">⚡ Live Topic Broadcast Tester</h3>
        <div style="display:grid; grid-template-columns: 1fr 2fr; gap:12px; margin-bottom:12px;">
          <div>
            <label style="font-size:12px; color:var(--muted); display:block; margin-bottom:6px; font-weight:700;">Channel Topic</label>
            <input id="ws-test-channel" value="inventory" style="width:100%; background:#050811; border:1px solid var(--card-border); padding:10px; border-radius:8px; color:#fff; font-family:'JetBrains Mono'; font-size:13px;" />
          </div>
          <div>
            <label style="font-size:12px; color:var(--muted); display:block; margin-bottom:6px; font-weight:700;">JSON Payload</label>
            <input id="ws-test-payload" value='{"item": "Titanium Apex Keyboard", "stock": 14, "action": "live_update"}' style="width:100%; background:#050811; border:1px solid var(--card-border); padding:10px; border-radius:8px; color:#38bdf8; font-family:'JetBrains Mono'; font-size:13px;" />
          </div>
        </div>
        <button class="btn" onclick="sendWsBroadcast()">Broadcast Event</button>
      </div>

      <div class="card">
        <h3 style="margin-bottom:16px;">📡 Live Message Feed Stream</h3>
        <div id="ws-log-stream" class="log-stream">Connecting to live event stream...</div>
      </div>
    </div>

    <!-- DATABASE EXPLORER TAB -->
    <div id="tab-database" class="tab-pane">
      <div class="header">
        <div>
          <h1>Database Manager</h1>
          <p class="subtitle">Inspect tables, view schemas, and query SQLite records in real-time.</p>
        </div>
      </div>
      <div class="card">
        <h3 style="margin-bottom:16px;">SQLite Database Tables</h3>
        <div id="tables-list">Loading tables...</div>
      </div>
    </div>

    <!-- SQL CONSOLE TAB -->
    <div id="tab-sql" class="tab-pane">
      <div class="header">
        <div>
          <h1>Interactive SQL Console</h1>
          <p class="subtitle">Execute raw queries directly against the embedded SQLite database.</p>
        </div>
      </div>
      <div class="card">
        <div class="editor-wrapper">
          <textarea id="sql-query" class="sql-input" placeholder="SELECT * FROM products LIMIT 10;"></textarea>
          <div style="display:flex; gap:10px;">
            <button class="btn" onclick="runSql()">Execute Query</button>
            <button class="btn btn-secondary" onclick="document.getElementById('sql-query').value='SELECT * FROM products;'; runSql();">Products</button>
            <button class="btn btn-secondary" onclick="document.getElementById('sql-query').value='SELECT * FROM orders;'; runSql();">Orders</button>
            <button class="btn btn-secondary" onclick="document.getElementById('sql-query').value='SELECT * FROM users;'; runSql();">Users</button>
          </div>
        </div>
        <div class="table-container" id="query-results" style="margin-top:20px;"></div>
      </div>
    </div>

    <!-- ROUTES TAB -->
    <div id="tab-routes" class="tab-pane">
      <div class="header">
        <div>
          <h1>Active Route Map</h1>
          <p class="subtitle">Discovered .titanium / .ti pages, MVC controllers, and API endpoints.</p>
        </div>
      </div>
      <div class="card">
        <div id="routes-list">Loading routes...</div>
      </div>
    </div>
  </div>

  <script>
    function switchTab(name, el) {
      document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
      document.querySelectorAll('.nav-item').forEach(i => i.classList.remove('active'));
      document.getElementById('tab-' + name).classList.add('active');
      el.classList.add('active');
      if (name === 'cluster') loadCluster();
      if (name === 'database') loadTables();
      if (name === 'routes') loadRoutes();
      if (name === 'realtime') loadWsStats();
    }

    async function loadTables() {
      const target = document.getElementById('tables-list');
      try {
        const res = await fetch('/__titanium_studio/api/tables');
        const data = await res.json();
        if (data.tables && data.tables.length > 0) {
          let html = '<table class="data-table"><thead><tr><th>Table Name</th><th>Type</th><th>Actions</th></tr></thead><tbody>';
          data.tables.forEach(t => {
            html += `<tr><td><strong>${t.name}</strong></td><td><span class="tag">${t.type}</span></td><td><button class="btn" style="padding:4px 10px; font-size:11px;" onclick="queryTable('${t.name}')">Browse Records</button></td></tr>`;
          });
          html += '</tbody></table>';
          target.innerHTML = html;
        } else {
          target.innerHTML = '<p style="color:var(--muted)">No tables found in database.</p>';
        }
      } catch (e) {
        target.innerHTML = '<p style="color:var(--danger)">Failed to fetch tables.</p>';
      }
    }

    function queryTable(name) {
      switchTab('sql', document.querySelectorAll('.nav-item')[3]);
      document.getElementById('sql-query').value = `SELECT * FROM ${name} LIMIT 25;`;
      runSql();
    }

    async function runSql() {
      const sql = document.getElementById('sql-query').value;
      const target = document.getElementById('query-results');
      target.innerHTML = '<p style="color:var(--muted)">Running query...</p>';
      try {
        const res = await fetch('/__titanium_studio/api/query', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ sql })
        });
        const data = await res.json();
        if (data.error) {
          target.innerHTML = `<p style="color:var(--danger); font-weight:700;">Error: ${data.error}</p>`;
          return;
        }
        if (!data.rows || data.rows.length === 0) {
          target.innerHTML = `<p style="color:var(--success); font-weight:700;">${data.message || 'Query executed successfully. 0 rows returned.'}</p>`;
          return;
        }
        const cols = Object.keys(data.rows[0]);
        let html = '<table class="data-table"><thead><tr>';
        cols.forEach(c => html += `<th>${c}</th>`);
        html += '</tr></thead><tbody>';
        data.rows.forEach(r => {
          html += '<tr>';
          cols.forEach(c => html += `<td>${r[c] !== null ? r[c] : '<span style="color:var(--muted)">NULL</span>'}</td>`);
          html += '</tr>';
        });
        html += '</tbody></table>';
        target.innerHTML = html;
      } catch (e) {
        target.innerHTML = `<p style="color:var(--danger);">Error: ${e.message}</p>`;
      }
    }

    async function loadRoutes() {
      const target = document.getElementById('routes-list');
      try {
        const res = await fetch('/__titanium_studio/api/routes');
        const data = await res.json();
        if (data.routes && data.routes.length > 0) {
          let html = '<table class="data-table"><thead><tr><th>Route Pattern</th><th>File Path</th><th>Type</th></tr></thead><tbody>';
          data.routes.forEach(r => {
            html += `<tr><td><code style="color:var(--primary); font-weight:700;">${r.pattern}</code></td><td style="color:var(--muted); font-size:12px;">${r.file}</td><td><span class="tag">${r.is_dynamic ? 'Dynamic Route' : 'Static Route'}</span></td></tr>`;
          });
          html += '</tbody></table>';
          target.innerHTML = html;
        } else {
          target.innerHTML = '<p style="color:var(--muted)">No routes detected.</p>';
        }
      } catch (e) {
        target.innerHTML = '<p style="color:var(--danger)">Failed to fetch routes.</p>';
      }
    }

    async function loadWsStats() {
      try {
        const res = await fetch('/__titanium_ws/stats');
        const data = await res.json();
        document.getElementById('ws-clients-count').textContent = data.total_clients || 0;
        document.getElementById('ws-channels-count').textContent = (data.channels || []).length;
      } catch (_) {}
    }

    async function sendWsBroadcast() {
      const channel = document.getElementById('ws-test-channel').value;
      const rawPayload = document.getElementById('ws-test-payload').value;
      let payload = rawPayload;
      try { payload = JSON.parse(rawPayload); } catch (_) {}

      try {
        const res = await fetch('/__titanium_ws/send', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ channel, payload })
        });
        const data = await res.json();
        if (data.success) {
          appendWsLog(`📤 Broadcasted to [${channel}]: ${JSON.stringify(payload)}`);
          loadWsStats();
        }
      } catch (e) {
        alert('Broadcast failed: ' + e.message);
      }
    }

    function appendWsLog(text) {
      const log = document.getElementById('ws-log-stream');
      const time = new Date().toLocaleTimeString();
      const div = document.createElement('div');
      div.className = 'log-entry';
      div.textContent = `[${time}] ${text}`;
      log.prepend(div);
    }

    // Connect to global live stream
    function initWsLogStream() {
      try {
        const es = new EventSource('/__titanium_ws/stream?channel=*');
        es.addEventListener('connected', () => {
          document.getElementById('ws-log-stream').innerHTML = '<div class="log-entry" style="color:var(--success)">🟢 Connected to Live Titanium WebSocket Hub</div>';
        });
        es.addEventListener('message', (e) => {
          try {
            const msg = JSON.parse(e.data);
            appendWsLog(`📥 Channel [${msg.channel}]: ${JSON.stringify(msg.payload)}`);
            loadWsStats();
          } catch (_) {}
        });
      } catch (_) {}
    }

    // Singularity AI Studio Handlers
    let activeAiStream = null;
    function runStudioAiStream() {
      const prompt = document.getElementById('ai-user-prompt').value;
      const system = document.getElementById('ai-system-prompt').value;
      const out = document.getElementById('ai-stream-output');

      if (activeAiStream) activeAiStream.close();
      out.textContent = '';

      const url = '/__titanium_ai/stream?prompt=' + encodeURIComponent(prompt) + '&system=' + encodeURIComponent(system);
      const es = new EventSource(url);
      activeAiStream = es;

      es.addEventListener('token', (e) => {
        try {
          const data = JSON.parse(e.data);
          out.textContent += (data.token || '');
        } catch (_) {}
      });

      es.addEventListener('done', (e) => {
        es.close();
        activeAiStream = null;
      });

      es.onerror = (e) => {
        out.textContent += '\n[Stream Connection Closed]';
        es.close();
        activeAiStream = null;
      };
    }

    async function runStudioAiRag() {
      const query = document.getElementById('ai-user-prompt').value;
      const out = document.getElementById('ai-stream-output');
      out.textContent = '🔍 Querying Vector Semantic Embeddings & Synthesizing Grounded Answer...\n';

      try {
        const res = await fetch('/__titanium_ai/rag', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query, collection: 'documents', top_k: 3 })
        });
        const data = await res.json();
        if (data.result && data.result.answer) {
          out.textContent = `🎯 Grounded Answer (${data.result.sources_count || 0} vector sources retrieved):\n\n${data.result.answer}`;
        } else {
          out.textContent = `Response: ${JSON.stringify(data)}`;
        }
      } catch (err) {
        out.textContent = `RAG Error: ${err.message}`;
      }
    }

    async function loadCluster() {
      const target = document.getElementById('cluster-nodes-table');
      try {
        const res = await fetch('/__titanium_cluster/nodes');
        const data = await res.json();
        document.getElementById('cluster-primary-id').textContent = (data.node_id || 'Primary');
        document.getElementById('cluster-nodes-count').textContent = (data.nodes || []).length;

        if (data.nodes && data.nodes.length > 0) {
          let html = '<table class="data-table"><thead><tr><th>Node ID</th><th>Bind Address</th><th>Role</th><th>Status</th><th>Latency</th></tr></thead><tbody>';
          data.nodes.forEach(n => {
            const roleBadge = n.role === 'primary' ? '<span class="tag tag-success">Primary Leader</span>' : '<span class="tag tag-accent">Replica</span>';
            const statusBadge = n.status === 'healthy' ? '<span style="color:var(--success); font-weight:700;">● Healthy</span>' : '<span style="color:var(--danger); font-weight:700;">● Degraded</span>';
            html += `<tr><td><strong>${n.id}</strong></td><td><code style="color:var(--primary);">${n.address}</code></td><td>${roleBadge}</td><td>${statusBadge}</td><td><span style="font-family:JetBrains Mono; color:var(--muted);">${n.latency_ms}ms</span></td></tr>`;
          });
          html += '</tbody></table>';
          target.innerHTML = html;
        } else {
          target.innerHTML = '<p style="color:var(--muted)">No nodes registered in cluster.</p>';
        }
      } catch (e) {
        target.innerHTML = `<p style="color:var(--danger);">Error loading cluster topology: ${e.message}</p>`;
      }
    }

    async function syncClusterWal() {
      const event_type = document.getElementById('cluster-sync-type').value;
      const rawPayload = document.getElementById('cluster-sync-payload').value;
      let payload = rawPayload;
      try { payload = JSON.parse(rawPayload); } catch (_) {}

      try {
        const res = await fetch('/__titanium_cluster/sync', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ event_type, payload })
        });
        const data = await res.json();
        if (data.success) {
          alert(`✅ Broadcasted sync [${data.sync_id}] across Supercluster nodes!`);
          loadCluster();
        } else {
          alert('Sync failed: ' + data.error);
        }
      } catch (e) {
        alert('Sync error: ' + e.message);
      }
    }

    loadTables();
    initWsLogStream();
  </script>
</body>
</html>"#;
