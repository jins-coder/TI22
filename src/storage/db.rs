use rusqlite::{types::ValueRef, Connection, ToSql};
use rhai::{Array, Dynamic, Map};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = if path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            Connection::open(path)?
        };

        // Enable WAL mode & performance optimizations
        let _ = conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        );

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn exec(&self, sql: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(sql).map_err(|e| e.to_string())
    }

    pub fn run_migrations(&self, migrations_dir: &Path) -> Result<usize, String> {
        if !migrations_dir.exists() {
            return Ok(0);
        }

        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // 1. Ensure migrations table exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _titanium_migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
        )
        .map_err(|e| e.to_string())?;

        // 2. Read all .sql files sorted
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(migrations_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("sql") {
                    files.push(p);
                }
            }
        }
        files.sort();

        let mut applied_count = 0;

        for file in files {
            let filename = file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            let already_applied: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM _titanium_migrations WHERE name = ?)",
                    [&filename],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if !already_applied {
                if let Ok(sql) = std::fs::read_to_string(&file) {
                    conn.execute_batch(&sql)
                        .map_err(|e| format!("Migration error in {}: {}", filename, e))?;

                    conn.execute(
                        "INSERT INTO _titanium_migrations (name) VALUES (?)",
                        [&filename],
                    )
                    .map_err(|e| e.to_string())?;

                    println!("  🗄️ Applied migration: {}", filename);
                    applied_count += 1;
                }
            }
        }

        Ok(applied_count)
    }

    pub fn query(&self, sql: &str, params: Array) -> Result<Array, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        let sql_params: Vec<Box<dyn ToSql>> = params
            .iter()
            .map(|p| dynamic_to_sql(p))
            .collect();

        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|b| b.as_ref()).collect();

        let column_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mut rows = stmt.query(&param_refs[..]).map_err(|e| e.to_string())?;
        let mut results = Array::new();

        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let mut map = Map::new();
            for (idx, col) in column_names.iter().enumerate() {
                let val_ref = row.get_ref(idx).map_err(|e| e.to_string())?;
                let dyn_val = match val_ref {
                    ValueRef::Null => Dynamic::UNIT,
                    ValueRef::Integer(i) => Dynamic::from(i),
                    ValueRef::Real(r) => Dynamic::from(r),
                    ValueRef::Text(t) => {
                        let s = std::str::from_utf8(t).unwrap_or("");
                        Dynamic::from(s.to_string())
                    }
                    ValueRef::Blob(b) => Dynamic::from(format!("<blob: {} bytes>", b.len())),
                };
                map.insert(col.clone().into(), dyn_val);
            }
            results.push(Dynamic::from(map));
        }

        Ok(results)
    }

    pub fn first(&self, sql: &str, params: Array) -> Result<Dynamic, String> {
        let rows = self.query(sql, params)?;
        if let Some(first) = rows.into_iter().next() {
            Ok(first)
        } else {
            Ok(Dynamic::UNIT)
        }
    }

    pub fn run(&self, sql: &str, params: Array) -> Result<Map, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        let sql_params: Vec<Box<dyn ToSql>> = params
            .iter()
            .map(|p| dynamic_to_sql(p))
            .collect();

        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|b| b.as_ref()).collect();

        let changes = stmt.execute(&param_refs[..]).map_err(|e| e.to_string())?;
        let last_id = conn.last_insert_rowid();

        let mut map = Map::new();
        map.insert("changes".into(), Dynamic::from(changes as i64));
        map.insert("last_insert_id".into(), Dynamic::from(last_id));
        Ok(map)
    }

    pub fn insert(&self, table: &str, data: Map) -> Result<Map, String> {
        if data.is_empty() {
            return Err("Cannot insert empty data map".to_string());
        }

        let mut cols = Vec::new();
        let mut placeholders = Vec::new();
        let mut params = Array::new();

        for (k, v) in data.into_iter() {
            cols.push(k.to_string());
            placeholders.push("?".to_string());
            params.push(v);
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            cols.join(", "),
            placeholders.join(", ")
        );

        self.run(&sql, params)
    }

    pub fn update(&self, table: &str, id: Dynamic, data: Map) -> Result<Map, String> {
        if data.is_empty() {
            return Err("Cannot update with empty data map".to_string());
        }

        let mut set_clauses = Vec::new();
        let mut params = Array::new();

        for (k, v) in data.into_iter() {
            set_clauses.push(format!("{} = ?", k));
            params.push(v);
        }

        params.push(id);

        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?",
            table,
            set_clauses.join(", ")
        );

        self.run(&sql, params)
    }

    pub fn delete(&self, table: &str, id: Dynamic) -> Result<Map, String> {
        let mut params = Array::new();
        params.push(id);
        let sql = format!("DELETE FROM {} WHERE id = ?", table);
        self.run(&sql, params)
    }

    pub fn find(&self, table: &str, id: Dynamic) -> Result<Dynamic, String> {
        let mut params = Array::new();
        params.push(id);
        let sql = format!("SELECT * FROM {} WHERE id = ? LIMIT 1", table);
        self.first(&sql, params)
    }

    pub fn search(&self, table: &str, query: &str, columns: Array) -> Result<Array, String> {
        let pattern = format!("%{}%", query);
        let mut clauses = Vec::new();
        let mut params = Array::new();

        for col in columns {
            let col_name = col.to_string();
            clauses.push(format!("{} LIKE ?", col_name));
            params.push(Dynamic::from(pattern.clone()));
        }

        let where_clause = if clauses.is_empty() {
            "1=1".to_string()
        } else {
            clauses.join(" OR ")
        };

        let sql = format!("SELECT * FROM {} WHERE {} ORDER BY id DESC LIMIT 50", table, where_clause);
        self.query(&sql, params)
    }
}

fn dynamic_to_sql(dyn_val: &Dynamic) -> Box<dyn ToSql> {
    if dyn_val.is_string() {
        Box::new(dyn_val.clone_cast::<String>())
    } else if dyn_val.is_int() {
        Box::new(dyn_val.clone_cast::<i64>())
    } else if dyn_val.is_float() {
        Box::new(dyn_val.clone_cast::<f64>())
    } else if dyn_val.is_bool() {
        Box::new(dyn_val.clone_cast::<bool>())
    } else if dyn_val.is_unit() {
        Box::new(rusqlite::types::Null)
    } else {
        Box::new(dyn_val.to_string())
    }
}
