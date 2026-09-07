pub const STUDIO_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Titanium Web Studio (Ti22)</title>
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
    .sidebar { width: 260px; background: var(--card); border-right: 1px solid var(--card-border); display: flex; flex-direction: column; padding: 24px 16px; }
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
    
    /* Tables */
    .table-container { overflow-x: auto; margin-top: 16px; }
    table.data-table { width: 100%; border-collapse: collapse; text-align: left; font-size: 13px; }
    table.data-table th, table.data-table td { padding: 12px 16px; border-bottom: 1px solid var(--card-border); }
    table.data-table th { background: rgba(255, 255, 255, 0.02); color: var(--muted); font-weight: 700; }
    table.data-table tr:hover { background: rgba(255, 255, 255, 0.01); }
    
    .tag { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 11px; font-weight: 700; background: rgba(56, 189, 248, 0.1); color: var(--primary); }
  </style>
</head>
<body>
  <div class="sidebar">
    <div class="brand">
      <span>⚡</span>
      <span>Titanium Studio</span>
    </div>
    <div class="nav-item active" onclick="switchTab('overview', this)">📊 Overview & Metrics</div>
    <div class="nav-item" onclick="switchTab('database', this)">🗄️ Database Explorer</div>
    <div class="nav-item" onclick="switchTab('sql', this)">⚡ SQL Console</div>
    <div class="nav-item" onclick="switchTab('routes', this)">🧭 Route Map</div>
    <div class="status-badge">
      <span>●</span>
      <span>Runtime: Active WAL</span>
    </div>
  </div>

  <div class="main">
    <!-- OVERVIEW TAB -->
    <div id="tab-overview" class="tab-pane active">
      <div class="header">
        <div>
          <h1>Engine Overview</h1>
          <p class="subtitle">Real-time telemetry and resource usage of Titanium native binary.</p>
        </div>
      </div>
      <div class="grid">
        <div class="stat-card">
          <div class="stat-lbl">ENGINE STATUS</div>
          <div class="stat-val">100%</div>
          <div style="color:var(--success); font-size:12px; font-weight:700;">● Online & Serving</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">STORAGE ENGINE</div>
          <div class="stat-val" style="color:var(--accent);">SQLite WAL</div>
          <div style="color:var(--muted); font-size:12px;">High Concurrent Mode</div>
        </div>
        <div class="stat-card">
          <div class="stat-lbl">SPA ROUTING</div>
          <div class="stat-val" style="color:#10b981;">Turbo Native</div>
          <div style="color:var(--muted); font-size:12px;">Zero Reloads</div>
        </div>
      </div>
      <div class="card">
        <h3 style="margin-bottom:12px;">⚡ Platform Capabilities</h3>
        <p style="color:var(--muted); font-size:14px; line-height:1.6;">
          Titanium (Ti22) is running in native threadpool execution mode. Dynamic file-system routing, Server-Sent Events, encrypted session stores, rate limiting, and soft-DOM live reload are active.
        </p>
      </div>
    </div>

    <!-- DATABASE EXPLORER TAB -->
    <div id="tab-database" class="tab-pane">
      <div class="header">
        <div>
          <h1>Database Explorer</h1>
          <p class="subtitle">Inspect SQLite tables and schemas.</p>
        </div>
      </div>
      <div class="card">
        <h3 style="margin-bottom:16px;">Discovered Tables</h3>
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
          <textarea id="sql-query" class="sql-input" placeholder="SELECT * FROM users LIMIT 10;"></textarea>
          <button class="btn" onclick="runSql()">Execute Query</button>
        </div>
        <div class="table-container" id="query-results" style="margin-top:20px;"></div>
      </div>
    </div>

    <!-- ROUTES TAB -->
    <div id="tab-routes" class="tab-pane">
      <div class="header">
        <div>
          <h1>Active Route Map</h1>
          <p class="subtitle">Discovered .titanium / .ti pages and API endpoints.</p>
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
      if (name === 'database') loadTables();
      if (name === 'routes') loadRoutes();
    }

    async function loadTables() {
      const target = document.getElementById('tables-list');
      try {
        const res = await fetch('/__titanium_studio/api/tables');
        const data = await res.json();
        if (data.tables && data.tables.length > 0) {
          let html = '<table class="data-table"><thead><tr><th>Table Name</th><th>Type</th><th>Actions</th></tr></thead><tbody>';
          data.tables.forEach(t => {
            html += `<tr><td><strong>${t.name}</strong></td><td><span class="tag">${t.type}</span></td><td><button class="btn" style="padding:4px 10px; font-size:11px;" onclick="queryTable('${t.name}')">Browse Data</button></td></tr>`;
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
      switchTab('sql', document.querySelectorAll('.nav-item')[2]);
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
      target.innerHTML = '<table class="data-table"><thead><tr><th>Route Path</th><th>Type</th></tr></thead><tbody><tr><td><code>/</code></td><td><span class="tag">SSR Page</span></td></tr><tr><td><code>/nextgen</code></td><td><span class="tag">Omniverse Showcase</span></td></tr><tr><td><code>/php_features</code></td><td><span class="tag">Modern Primitives</span></td></tr><tr><td><code>/api/users</code></td><td><span class="tag">REST JSON API</span></td></tr></tbody></table>';
    }

    loadTables();
  </script>
</body>
</html>"#;
