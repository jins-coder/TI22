use crate::core::context::{TitaniumRequest, TitaniumResponse};
use crate::core::session::SessionHandle;
use crate::services::{AiEngine, JobQueue, MediaUtils, PubSubHub, VectorEngine, WebSocketHub};
use crate::storage::{CacheStore, Database, ModelDef, QueryBuilder};
use minijinja::{Environment, Value};
use rhai::{Array, Dynamic, Engine, Map, Scope, AST};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Clone)]
pub struct TitaniumEngine {
    db: Database,
    cache: CacheStore,
    queue: JobQueue,
    pubsub: PubSubHub,
    ws: WebSocketHub,
    ai: AiEngine,
    vector: VectorEngine,
    rhai_engine: Arc<Engine>,
    ast_cache: Arc<Mutex<HashMap<String, AST>>>,
    rate_limiter: Arc<Mutex<HashMap<String, Vec<u64>>>>,
}

impl TitaniumEngine {
    pub fn new(
        db: Database,
        cache: CacheStore,
        queue: JobQueue,
        pubsub: PubSubHub,
        ws: WebSocketHub,
        ai: AiEngine,
        vector: VectorEngine,
    ) -> Self {
        let mut engine = Engine::new();
        let rate_limiter = Arc::new(Mutex::new(HashMap::new()));

        // Custom Types and Helpers
        engine.register_type_with_name::<TitaniumResponse>("TitaniumResponse");
        engine.register_type_with_name::<SessionHandle>("Session");

        // Session methods: session.id(), session.get(), session.set(), session.remove(), session.flash(), session.csrf()
        engine.register_fn("id", |s: &mut SessionHandle| -> String {
            s.id()
        });
        engine.register_get("id", |s: &mut SessionHandle| -> String {
            s.id()
        });
        engine.register_fn("get", |s: &mut SessionHandle, key: &str| -> Dynamic {
            s.get(key)
        });
        engine.register_fn("set", |s: &mut SessionHandle, key: &str, val: Dynamic| {
            s.set(key, val);
        });
        engine.register_fn("remove", |s: &mut SessionHandle, key: &str| {
            s.remove(key);
        });
        engine.register_fn("delete", |s: &mut SessionHandle, key: &str| {
            s.remove(key);
        });
        engine.register_fn("flash", |s: &mut SessionHandle, key: &str| -> Dynamic {
            s.flash_get(key)
        });
        engine.register_fn("flash", |s: &mut SessionHandle, key: &str, val: Dynamic| {
            s.flash_set(key, val);
        });
        engine.register_fn("csrf", |s: &mut SessionHandle| -> String {
            s.csrf_token()
        });

        // ActiveRecord ORM & QueryBuilder (v7.0.0 Dual Engine)
        engine.register_type_with_name::<ModelDef>("Model");
        engine.register_type_with_name::<QueryBuilder>("QueryBuilder");

        let db_orm1 = db.clone();
        engine.register_fn("model", move |table: &str| -> ModelDef {
            ModelDef::new(table, db_orm1.clone())
        });
        let db_orm2 = db.clone();
        engine.register_fn("define_model", move |table: &str, pk: &str| -> ModelDef {
            ModelDef::with_pk(table, pk, db_orm2.clone())
        });

        engine.register_fn("all", |m: &mut ModelDef| -> Array { m.all() });
        engine.register_fn("find", |m: &mut ModelDef, id: Dynamic| -> Dynamic { m.find(id) });
        engine.register_fn("create", |m: &mut ModelDef, data: Map| -> Dynamic { m.create(data) });
        engine.register_fn("count", |m: &mut ModelDef| -> i64 { m.count() });

        engine.register_fn("where", |m: &mut ModelDef, col: &str, val: Dynamic| -> QueryBuilder {
            let mut q = m.query_builder();
            q.where_eq(col, val)
        });
        engine.register_fn("where", |m: &mut ModelDef, col: &str, op: &str, val: Dynamic| -> QueryBuilder {
            let mut q = m.query_builder();
            q.where_op(col, op, val)
        });
        engine.register_fn("order_by", |m: &mut ModelDef, col: &str, dir: &str| -> QueryBuilder {
            let mut q = m.query_builder();
            q.order_by_clause(col, dir)
        });
        engine.register_fn("limit", |m: &mut ModelDef, n: i64| -> QueryBuilder {
            let mut q = m.query_builder();
            q.set_limit(n)
        });
        engine.register_fn("offset", |m: &mut ModelDef, n: i64| -> QueryBuilder {
            let mut q = m.query_builder();
            q.set_offset(n)
        });

        // QueryBuilder chaining
        engine.register_fn("where", |q: &mut QueryBuilder, col: &str, val: Dynamic| -> QueryBuilder {
            q.where_eq(col, val)
        });
        engine.register_fn("where", |q: &mut QueryBuilder, col: &str, op: &str, val: Dynamic| -> QueryBuilder {
            q.where_op(col, op, val)
        });
        engine.register_fn("order_by", |q: &mut QueryBuilder, col: &str, dir: &str| -> QueryBuilder {
            q.order_by_clause(col, dir)
        });
        engine.register_fn("limit", |q: &mut QueryBuilder, n: i64| -> QueryBuilder {
            q.set_limit(n)
        });
        engine.register_fn("offset", |q: &mut QueryBuilder, n: i64| -> QueryBuilder {
            q.set_offset(n)
        });
        engine.register_fn("get", |q: &mut QueryBuilder| -> Array { q.get() });
        engine.register_fn("first", |q: &mut QueryBuilder| -> Dynamic { q.first() });
        engine.register_fn("count", |q: &mut QueryBuilder| -> i64 { q.count() });
        engine.register_fn("delete", |q: &mut QueryBuilder| -> i64 { q.delete() });
        engine.register_fn("update", |q: &mut QueryBuilder, data: Map| -> i64 { q.update(data) });

        // MVC View constructors
        engine.register_fn("view", |view_name: &str, data: Dynamic| -> TitaniumResponse {
            TitaniumResponse::View {
                status: 200,
                view: view_name.to_string(),
                data,
            }
        });
        engine.register_fn("view", |view_name: &str| -> TitaniumResponse {
            TitaniumResponse::View {
                status: 200,
                view: view_name.to_string(),
                data: Dynamic::from(Map::new()),
            }
        });

        // Response constructors
        engine.register_fn("redirect", |url: &str| -> TitaniumResponse {
            TitaniumResponse::Redirect {
                status: 302,
                location: url.to_string(),
            }
        });

        engine.register_fn("redirect", |url: &str, status: i64| -> TitaniumResponse {
            TitaniumResponse::Redirect {
                status: status as u16,
                location: url.to_string(),
            }
        });

        engine.register_fn("json", |data: Dynamic| -> TitaniumResponse {
            TitaniumResponse::Json { status: 200, data }
        });

        engine.register_fn("json", |data: Dynamic, status: i64| -> TitaniumResponse {
            TitaniumResponse::Json {
                status: status as u16,
                data,
            }
        });

        engine.register_fn("html", |content: &str| -> TitaniumResponse {
            TitaniumResponse::Html {
                status: 200,
                body: content.to_string(),
            }
        });

        // Real-Time SSE Streaming Constructor
        engine.register_fn("sse_event", |event_name: &str, data: Dynamic| -> TitaniumResponse {
            let data_str = if data.is_string() {
                data.clone_cast::<String>()
            } else if let Ok(json_val) = rhai::serde::from_dynamic::<serde_json::Value>(&data) {
                serde_json::to_string(&json_val).unwrap_or_default()
            } else {
                data.to_string()
            };

            let formatted = format!("event: {}\ndata: {}\n\n", event_name, data_str);
            TitaniumResponse::Sse {
                status: 200,
                body: formatted,
            }
        });

        engine.register_fn("sse_stream", |events: Array| -> TitaniumResponse {
            let mut body = String::new();
            for ev in events {
                if let Some(map) = ev.try_cast::<Map>() {
                    let name = map.get("event").map(|d| d.to_string()).unwrap_or_else(|| "message".to_string());
                    let data_val = map.get("data").cloned().unwrap_or(Dynamic::UNIT);
                    let data_str = if data_val.is_string() {
                        data_val.clone_cast::<String>()
                    } else if let Ok(json_val) = rhai::serde::from_dynamic::<serde_json::Value>(&data_val) {
                        serde_json::to_string(&json_val).unwrap_or_default()
                    } else {
                        data_val.to_string()
                    };
                    body.push_str(&format!("event: {}\ndata: {}\n\n", name, data_str));
                }
            }

            TitaniumResponse::Sse {
                status: 200,
                body,
            }
        });

        // File I/O Helpers
        engine.register_fn("file_write", |path: &str, content: &str| -> bool {
            std::fs::write(path, content).is_ok()
        });

        engine.register_fn("file_read", |path: &str| -> String {
            std::fs::read_to_string(path).unwrap_or_default()
        });

        engine.register_fn("file_exists", |path: &str| -> bool {
            Path::new(path).exists()
        });

        // Rate Limiting Helper (Sliding Window Algorithm)
        let rl_clone = Arc::clone(&rate_limiter);
        engine.register_fn("rate_limit", move |key: &str, max_reqs: i64, window_secs: i64| -> Map {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let window = if window_secs <= 0 { 60 } else { window_secs as u64 };
            let max = if max_reqs <= 0 { 60 } else { max_reqs };

            let mut map = Map::new();
            let mut rl = rl_clone.lock().unwrap();
            let timestamps: &mut Vec<u64> = rl.entry(key.to_string()).or_default();

            // Evict timestamps older than current window
            let threshold = now.saturating_sub(window);
            timestamps.retain(|&t| t > threshold);

            if (timestamps.len() as i64) >= max {
                let oldest = timestamps.first().copied().unwrap_or(now);
                let reset_in = (oldest + window).saturating_sub(now);
                map.insert("allowed".into(), Dynamic::from(false));
                map.insert("remaining".into(), Dynamic::from(0i64));
                map.insert("reset_in".into(), Dynamic::from(reset_in as i64));
            } else {
                timestamps.push(now);
                let remaining = max - (timestamps.len() as i64);
                map.insert("allowed".into(), Dynamic::from(true));
                map.insert("remaining".into(), Dynamic::from(remaining));
                map.insert("reset_in".into(), Dynamic::from(window as i64));
            }

            map
        });

        // ==========================================
        // Next-Gen Capabilities: Cache Engine (v6.0.0)
        // ==========================================
        let cache_c1 = cache.clone();
        engine.register_fn("cache_set", move |key: &str, val: Dynamic| {
            cache_c1.set(key, val, None);
        });

        let cache_c2 = cache.clone();
        engine.register_fn("cache_set", move |key: &str, val: Dynamic, ttl_seconds: i64| {
            cache_c2.set(key, val, Some(ttl_seconds));
        });

        let cache_c3 = cache.clone();
        engine.register_fn("cache_get", move |key: &str| -> Dynamic {
            cache_c3.get(key)
        });

        let cache_c4 = cache.clone();
        engine.register_fn("cache_has", move |key: &str| -> bool {
            cache_c4.has(key)
        });

        let cache_c5 = cache.clone();
        engine.register_fn("cache_delete", move |key: &str| -> bool {
            cache_c5.delete(key)
        });

        let cache_c6 = cache.clone();
        engine.register_fn("cache_clear", move || {
            cache_c6.clear();
        });

        let cache_c7 = cache.clone();
        engine.register_fn("cache_keys", move || -> Array {
            cache_c7.keys()
        });

        let cache_c8 = cache.clone();
        engine.register_fn("cache_stats", move || -> Map {
            cache_c8.stats()
        });

        // ==========================================
        // Next-Gen Capabilities: Job Queue (v6.0.0)
        // ==========================================
        let queue_c1 = queue.clone();
        engine.register_fn("defer_job", move |name: &str, payload: Dynamic| -> bool {
            queue_c1.dispatch(name, payload)
        });

        let queue_c2 = queue.clone();
        engine.register_fn("queue_stats", move || -> Map {
            queue_c2.stats()
        });

        // ==========================================
        // Next-Gen Capabilities: PubSub Event Hub (v6.0.0)
        // ==========================================
        let pubsub_c1 = pubsub.clone();
        engine.register_fn("pubsub_publish", move |topic: &str, message: Dynamic| {
            pubsub_c1.publish(topic, message);
        });

        let pubsub_c2 = pubsub.clone();
        engine.register_fn("pubsub_history", move |topic: &str| -> Array {
            pubsub_c2.history(topic, 20)
        });

        let pubsub_c3 = pubsub.clone();
        engine.register_fn("pubsub_history", move |topic: &str, limit: i64| -> Array {
            pubsub_c3.history(topic, limit)
        });

        let pubsub_c4 = pubsub.clone();
        engine.register_fn("pubsub_topics", move || -> Array {
            pubsub_c4.list_topics()
        });

        // ==========================================
        // Next-Gen Capabilities: Vector Search (v6.0.0)
        // ==========================================
        engine.register_fn("vector_cosine_similarity", |vec_a: Array, vec_b: Array| -> f64 {
            let va = VectorEngine::to_f64_vec(&vec_a);
            let vb = VectorEngine::to_f64_vec(&vec_b);
            VectorEngine::cosine_similarity(&va, &vb)
        });

        engine.register_fn("vector_rank", |query_vec: Array, docs: Array, vector_field: &str| -> Array {
            let qv = VectorEngine::to_f64_vec(&query_vec);
            let mut maps = Vec::new();
            for item in docs {
                if let Some(m) = item.try_cast::<Map>() {
                    maps.push(m);
                }
            }
            VectorEngine::rank_documents(&qv, &maps, vector_field)
        });

        // ==========================================
        // Next-Gen Capabilities: Media & Image (v6.0.0)
        // ==========================================
        engine.register_fn("media_info", |path: &str| -> Map {
            MediaUtils::file_info(path)
        });

        engine.register_fn("media_data_uri", |path: &str| -> String {
            MediaUtils::to_data_uri(path)
        });

        engine.register_fn("svg_identicon", |seed: &str| -> String {
            MediaUtils::generate_identicon_svg(seed, 120)
        });

        engine.register_fn("svg_identicon", |seed: &str, size: i64| -> String {
            MediaUtils::generate_identicon_svg(seed, size)
        });

        // Raw SQL Database methods
        let db_clone1 = db.clone();
        engine.register_fn("db_exec", move |sql: &str| -> Result<(), Box<rhai::EvalAltResult>> {
            db_clone1.exec(sql).map_err(|e| e.into())
        });

        let db_clone2 = db.clone();
        engine.register_fn("db_query", move |sql: &str| -> Result<Array, Box<rhai::EvalAltResult>> {
            db_clone2.query(sql, Array::new()).map_err(|e| e.into())
        });

        let db_clone3 = db.clone();
        engine.register_fn("db_query", move |sql: &str, params: Array| -> Result<Array, Box<rhai::EvalAltResult>> {
            db_clone3.query(sql, params).map_err(|e| e.into())
        });

        let db_clone4 = db.clone();
        engine.register_fn("db_first", move |sql: &str| -> Result<Dynamic, Box<rhai::EvalAltResult>> {
            db_clone4.first(sql, Array::new()).map_err(|e| e.into())
        });

        let db_clone5 = db.clone();
        engine.register_fn("db_first", move |sql: &str, params: Array| -> Result<Dynamic, Box<rhai::EvalAltResult>> {
            db_clone5.first(sql, params).map_err(|e| e.into())
        });

        let db_clone6 = db.clone();
        engine.register_fn("db_run", move |sql: &str, params: Array| -> Result<Map, Box<rhai::EvalAltResult>> {
            db_clone6.run(sql, params).map_err(|e| e.into())
        });

        // Ergonomic CRUD Database Helpers
        let db_clone7 = db.clone();
        engine.register_fn("db_insert", move |table: &str, data: Map| -> Result<Map, Box<rhai::EvalAltResult>> {
            db_clone7.insert(table, data).map_err(|e| e.into())
        });

        let db_clone8 = db.clone();
        engine.register_fn("db_update", move |table: &str, id: Dynamic, data: Map| -> Result<Map, Box<rhai::EvalAltResult>> {
            db_clone8.update(table, id, data).map_err(|e| e.into())
        });

        let db_clone9 = db.clone();
        engine.register_fn("db_delete", move |table: &str, id: Dynamic| -> Result<Map, Box<rhai::EvalAltResult>> {
            db_clone9.delete(table, id).map_err(|e| e.into())
        });

        let db_clone10 = db.clone();
        engine.register_fn("db_find", move |table: &str, id: Dynamic| -> Result<Dynamic, Box<rhai::EvalAltResult>> {
            db_clone10.find(table, id).map_err(|e| e.into())
        });

        let db_clone11 = db.clone();
        engine.register_fn("db_search", move |table: &str, query: &str, columns: Array| -> Result<Array, Box<rhai::EvalAltResult>> {
            db_clone11.search(table, query, columns).map_err(|e| e.into())
        });

        // Form Validation Helper
        engine.register_fn("validate", |data: Map, rules: Map| -> Map {
            run_validation(&data, &rules)
        });

        // HTTP Client Helpers (Server-side API calls)
        engine.register_fn("http_get", |url: &str| -> Map {
            run_http_get(url, Map::new())
        });

        engine.register_fn("http_get", |url: &str, headers: Map| -> Map {
            run_http_get(url, headers)
        });

        engine.register_fn("http_post", |url: &str, body: Dynamic| -> Map {
            run_http_post(url, body, Map::new())
        });

        engine.register_fn("http_post", |url: &str, body: Dynamic, headers: Map| -> Map {
            run_http_post(url, body, headers)
        });

        // Crypto & Password Hashing Helpers
        engine.register_fn("crypto_hash", |text: &str| -> String {
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            format!("{:x}", hasher.finalize())
        });

        engine.register_fn("crypto_verify", |text: &str, expected_hash: &str| -> bool {
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let computed = format!("{:x}", hasher.finalize());
            computed == expected_hash
        });

        engine.register_fn("crypto_random_token", |len: i64| -> String {
            let count = if len <= 0 { 32 } else { len as usize };
            let mut bytes = Vec::with_capacity(count);
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let mut seed = time;
            for _ in 0..count {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let byte = (seed >> 32) as u8;
                let char_byte = match byte % 62 {
                    0..=9 => b'0' + (byte % 10),
                    10..=35 => b'a' + (byte % 26),
                    _ => b'A' + (byte % 26),
                };
                bytes.push(char_byte);
            }
            String::from_utf8(bytes).unwrap_or_default()
        });

        // ==========================================
        // PHP 8.1 - 8.5 Modern Ergonomics Primitives
        // ==========================================

        // [PHP 8.3] json_validate
        engine.register_fn("json_validate", |json_str: &str| -> bool {
            serde_json::from_str::<serde_json::Value>(json_str).is_ok()
        });

        engine.register_fn("json_encode", |data: Dynamic| -> String {
            if let Ok(json_val) = rhai::serde::from_dynamic::<serde_json::Value>(&data) {
                serde_json::to_string(&json_val).unwrap_or_default()
            } else {
                data.to_string()
            }
        });

        engine.register_fn("json_decode", |json_str: &str| -> Dynamic {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                rhai::serde::to_dynamic(&val).unwrap_or(Dynamic::UNIT)
            } else {
                Dynamic::UNIT
            }
        });

        // [PHP 8.2] Sensitive Data Redaction & Masking
        engine.register_fn("redact", |mut map: Map, sensitive_keys: Array| -> Map {
            for key_dyn in sensitive_keys {
                let key_str = key_dyn.to_string();
                if map.contains_key(key_str.as_str()) {
                    map.insert(key_str.into(), Dynamic::from("********"));
                }
            }
            map
        });

        engine.register_fn("mask", |text: &str| -> String {
            mask_string(text, 4)
        });

        engine.register_fn("mask", |text: &str, keep_last: i64| -> String {
            mask_string(text, keep_last.max(0) as usize)
        });

        // [PHP 8.4] Modern Array Primitives: array_find, array_all, array_any, array_pluck, array_chunk, array_group_by
        engine.register_fn("array_pluck", |arr: Array, key: &str| -> Array {
            let mut result = Array::new();
            for item in arr {
                if let Some(map) = item.try_cast::<Map>() {
                    if let Some(val) = map.get(key) {
                        result.push(val.clone());
                    }
                }
            }
            result
        });

        engine.register_fn("array_chunk", |arr: Array, size: i64| -> Array {
            let chunk_size = if size <= 0 { 1 } else { size as usize };
            let mut chunks = Array::new();
            let mut current = Array::new();
            for item in arr {
                current.push(item);
                if current.len() == chunk_size {
                    chunks.push(Dynamic::from(current));
                    current = Array::new();
                }
            }
            if !current.is_empty() {
                chunks.push(Dynamic::from(current));
            }
            chunks
        });

        engine.register_fn("array_group_by", |arr: Array, key: &str| -> Map {
            let mut groups: HashMap<String, Array> = HashMap::new();
            for item in arr {
                let group_key = if let Some(map) = item.clone().try_cast::<Map>() {
                    map.get(key).map(|v| v.to_string()).unwrap_or_else(|| "unknown".into())
                } else {
                    "unknown".into()
                };
                groups.entry(group_key).or_default().push(item);
            }
            let mut result = Map::new();
            for (k, v) in groups {
                result.insert(k.into(), Dynamic::from(v));
            }
            result
        });

        // [PHP 8.1 - 8.4] String, Logic & Math Primitives
        engine.register_fn("str_contains", |haystack: &str, needle: &str| -> bool {
            haystack.contains(needle)
        });

        engine.register_fn("str_starts_with", |haystack: &str, needle: &str| -> bool {
            haystack.starts_with(needle)
        });

        engine.register_fn("str_ends_with", |haystack: &str, needle: &str| -> bool {
            haystack.ends_with(needle)
        });

        engine.register_fn("trim", |s: &str| -> String {
            s.trim().to_string()
        });

        engine.register_fn("to_lower", |s: &str| -> String {
            s.to_lowercase()
        });

        engine.register_fn("to_upper", |s: &str| -> String {
            s.to_uppercase()
        });

        engine.register_fn("lower", |s: &str| -> String {
            s.to_lowercase()
        });

        engine.register_fn("upper", |s: &str| -> String {
            s.to_uppercase()
        });

        engine.register_fn("str_slug", |text: &str| -> String {
            text.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .split('-')
                .filter(|p| !p.is_empty())
                .collect::<Vec<&str>>()
                .join("-")
        });

        engine.register_fn("clamp", |val: i64, min: i64, max: i64| -> i64 {
            val.clamp(min, max)
        });

        engine.register_fn("clamp", |val: f64, min: f64, max: f64| -> f64 {
            val.clamp(min, max)
        });

        // Numeric Parsing & Casting Utilities
        engine.register_fn("parse_int", |val: &str| -> i64 {
            val.trim().parse::<i64>().unwrap_or(0)
        });
        engine.register_fn("parse_int", |val: Dynamic| -> i64 {
            if let Ok(num) = val.as_int() {
                num
            } else if let Ok(flt) = val.as_float() {
                flt as i64
            } else {
                val.to_string().trim().parse::<i64>().unwrap_or(0)
            }
        });
        engine.register_fn("to_int", |val: Dynamic| -> i64 {
            if let Ok(num) = val.as_int() {
                num
            } else if let Ok(flt) = val.as_float() {
                flt as i64
            } else {
                val.to_string().trim().parse::<i64>().unwrap_or(0)
            }
        });
        engine.register_fn("parse_float", |val: &str| -> f64 {
            val.trim().parse::<f64>().unwrap_or(0.0)
        });
        engine.register_fn("parse_float", |val: Dynamic| -> f64 {
            if let Ok(flt) = val.as_float() {
                flt
            } else if let Ok(num) = val.as_int() {
                num as f64
            } else {
                val.to_string().trim().parse::<f64>().unwrap_or(0.0)
            }
        });
        engine.register_fn("to_float", |val: Dynamic| -> f64 {
            if let Ok(flt) = val.as_float() {
                flt
            } else if let Ok(num) = val.as_int() {
                num as f64
            } else {
                val.to_string().trim().parse::<f64>().unwrap_or(0.0)
            }
        });

        // ==========================================
        // Realtime WebSockets & Live Channels (v8.0.0 Hyperdrive)
        // ==========================================
        let ws_c1 = ws.clone();
        engine.register_fn("ws_broadcast", move |channel: &str, data: Dynamic| -> Dynamic {
            let msg = ws_c1.broadcast_dynamic(channel, data);
            rhai::serde::to_dynamic(&msg).unwrap_or(Dynamic::UNIT)
        });

        let ws_c2 = ws.clone();
        engine.register_fn("ws_broadcast_all", move |data: Dynamic| -> Dynamic {
            let msg = ws_c2.broadcast_dynamic("*", data);
            rhai::serde::to_dynamic(&msg).unwrap_or(Dynamic::UNIT)
        });

        let ws_c3 = ws.clone();
        engine.register_fn("ws_client_count", move |channel: &str| -> i64 {
            ws_c3.client_count(channel) as i64
        });

        let ws_c4 = ws.clone();
        engine.register_fn("ws_channels", move || -> Array {
            let mut arr = Array::new();
            for c in ws_c4.channels_list() {
                arr.push(Dynamic::from(c));
            }
            arr
        });

        let ws_c5 = ws.clone();
        engine.register_fn("ws_stats", move || -> Map {
            ws_c5.stats_map()
        });

        // ==========================================
        // Singularity AI Agentic Runtime (v9.0.0)
        // ==========================================
        let ai_c1 = ai.clone();
        engine.register_fn("ai_generate", move |prompt: &str| -> String {
            ai_c1.generate(prompt, None).unwrap_or_else(|e| format!("AI Error: {}", e))
        });

        let ai_c2 = ai.clone();
        engine.register_fn("ai_generate", move |prompt: &str, system: &str| -> String {
            ai_c2.generate(prompt, Some(system)).unwrap_or_else(|e| format!("AI Error: {}", e))
        });

        let ai_c3 = ai.clone();
        engine.register_fn("ai_chat", move |prompt: &str| -> String {
            ai_c3.generate(prompt, Some("You are the Titanium Singularity AI Assistant.")).unwrap_or_else(|e| format!("AI Error: {}", e))
        });

        let ai_c4 = ai.clone();
        let ws_ai = ws.clone();
        engine.register_fn("ai_stream", move |channel: &str, prompt: &str| -> i64 {
            let chunks = ai_c4.generate_stream_chunks(prompt, None);
            let count = chunks.len() as i64;
            for (idx, chunk) in chunks.into_iter().enumerate() {
                let mut data = Map::new();
                data.insert("type".into(), Dynamic::from("token"));
                data.insert("token".into(), Dynamic::from(chunk));
                data.insert("index".into(), Dynamic::from(idx as i64));
                ws_ai.broadcast_dynamic(channel, Dynamic::from(data));
            }
            let mut end_data = Map::new();
            end_data.insert("type".into(), Dynamic::from("done"));
            end_data.insert("total_tokens".into(), Dynamic::from(count));
            ws_ai.broadcast_dynamic(channel, Dynamic::from(end_data));
            count
        });

        let ai_c5 = ai.clone();
        let vec_ai = vector.clone();
        engine.register_fn("ai_rag", move |query: &str| -> Map {
            ai_c5.rag_search_and_answer(query, &vec_ai, "documents", 3).unwrap_or_default()
        });

        let ai_c6 = ai.clone();
        let vec_ai2 = vector.clone();
        engine.register_fn("ai_rag", move |query: &str, collection: &str| -> Map {
            ai_c6.rag_search_and_answer(query, &vec_ai2, collection, 3).unwrap_or_default()
        });

        let ai_c7 = ai.clone();
        let vec_ai3 = vector.clone();
        engine.register_fn("ai_rag", move |query: &str, collection: &str, top_k: i64| -> Map {
            ai_c7.rag_search_and_answer(query, &vec_ai3, collection, top_k.max(1) as usize).unwrap_or_default()
        });

        let vec_u = vector.clone();
        engine.register_fn("vector_upsert", move |collection: &str, id: &str, content: &str| {
            vec_u.upsert(collection, id, content);
        });

        let vec_s = vector.clone();
        engine.register_fn("vector_search", move |collection: &str, query: &str, top_k: i64| -> Array {
            let mut arr = Array::new();
            if let Ok(results) = vec_s.search(collection, query, top_k.max(1) as usize) {
                for r in results {
                    let mut item = Map::new();
                    item.insert("id".into(), Dynamic::from(r.id));
                    item.insert("content".into(), Dynamic::from(r.content));
                    item.insert("score".into(), Dynamic::from(r.score));
                    arr.push(Dynamic::from(item));
                }
            }
            arr
        });

        Self {
            db,
            cache,
            queue,
            pubsub,
            ws,
            ai,
            vector,
            rhai_engine: Arc::new(engine),
            ast_cache: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter,
        }
    }

    pub fn run_middleware(
        &self,
        middleware_path: &Path,
        req: &TitaniumRequest,
        session: &SessionHandle,
    ) -> Result<Option<TitaniumResponse>, String> {
        if !middleware_path.is_file() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(middleware_path).map_err(|e| e.to_string())?;
        let (script_part, _) = parse_titanium_file(&content, true);

        if let Some(script) = script_part {
            let mut scope = Scope::new();
            scope.push("req", req.to_rhai_map());
            scope.push("params", req.params.clone());
            scope.push("query", req.query.clone());
            scope.push("body", req.body.clone());
            scope.push("session", session.clone());

            let ast = {
                let mut cache = self.ast_cache.lock().unwrap();
                if let Some(cached) = cache.get(&script) {
                    cached.clone()
                } else {
                    let compiled = self
                        .rhai_engine
                        .compile(&script)
                        .map_err(|e| format!("Middleware Syntax Error: {}", e))?;
                    cache.insert(script.clone(), compiled.clone());
                    compiled
                }
            };

            let eval_result: Dynamic = self
                .rhai_engine
                .eval_ast_with_scope(&mut scope, &ast)
                .map_err(|e| format!("Middleware Execution Error: {}", e))?;

            if let Some(resp) = eval_result.try_cast::<TitaniumResponse>() {
                return Ok(Some(resp));
            }
        }

        Ok(None)
    }

    pub fn execute(
        &self,
        file_path: &Path,
        root_dir: &Path,
        req: &TitaniumRequest,
        session: &SessionHandle,
    ) -> Result<TitaniumResponse, String> {
        let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let is_api = file_path.to_string_lossy().replace('\\', "/").contains("/api/");

        let (script_part, template_part) = parse_titanium_file(&content, is_api);

        let mut scope = Scope::new();

        // 1. Build Scope Variables
        scope.push("req", req.to_rhai_map());
        scope.push("params", req.params.clone());
        scope.push("query", req.query.clone());
        scope.push("body", req.body.clone());
        scope.push("session", session.clone());

        // 2. Execute Script if present
        if let Some(script) = script_part {
            let ast = {
                let mut cache = self.ast_cache.lock().unwrap();
                if let Some(cached) = cache.get(&script) {
                    cached.clone()
                } else {
                    let compiled = self
                        .rhai_engine
                        .compile(&script)
                        .map_err(|e| format!("Titanium Script Syntax Error: {}", e))?;
                    cache.insert(script.clone(), compiled.clone());
                    compiled
                }
            };

            let eval_result: Dynamic = self
                .rhai_engine
                .eval_ast_with_scope(&mut scope, &ast)
                .map_err(|e| format!("Titanium Runtime Error: {}", e))?;

            // If script returned an explicit TitaniumResponse
            if let Some(resp) = eval_result.clone().try_cast::<TitaniumResponse>() {
                if let TitaniumResponse::View { status, view, data } = resp {
                    let view_html = render_mvc_view(root_dir, &view, data, session)?;
                    return Ok(TitaniumResponse::Html {
                        status,
                        body: view_html,
                    });
                }
                return Ok(resp);
            }

            // In API routes, returning a Dynamic map/array defaults to JSON
            if is_api && !eval_result.is_unit() {
                return Ok(TitaniumResponse::Json {
                    status: 200,
                    data: eval_result,
                });
            }
        }

        // 3. Render HTML Template
        if let Some(tmpl) = template_part {
            let mut env = Environment::new();
            add_custom_filters(&mut env);

            env.add_template("page", &tmpl)
                .map_err(|e| format!("Template Syntax Error: {}", e))?;

            let template = env.get_template("page").map_err(|e| e.to_string())?;

            // Convert scope variables into JSON context for Minijinja
            let mut context_map = serde_json::Map::new();
            for (name, _, val) in scope.iter() {
                if let Ok(json_val) = rhai::serde::from_dynamic::<serde_json::Value>(&val) {
                    context_map.insert(name.to_string(), json_val);
                }
            }

            // Inject CSRF token into template context
            context_map.insert("csrf_token".to_string(), serde_json::Value::String(session.csrf_token()));

            let mut rendered = template
                .render(serde_json::Value::Object(context_map.clone()))
                .map_err(|e| format!("Template Render Error: {}", e))?;

            // Check for layout wrapping
            let layout_choice = scope
                .get_value::<String>("layout")
                .or_else(|| scope.get_value::<bool>("no_layout").filter(|&b| b).map(|_| "none".to_string()));

            if layout_choice.as_deref() != Some("none") && !rendered.contains("<!DOCTYPE html>") {
                let layout_path = resolve_layout(root_dir, layout_choice.as_deref());
                if let Some(lpath) = layout_path {
                    if let Ok(layout_str) = std::fs::read_to_string(&lpath) {
                        let (_, layout_tmpl) = parse_titanium_file(&layout_str, false);
                        if let Some(ltmpl) = layout_tmpl {
                            let mut l_env = Environment::new();
                            add_custom_filters(&mut l_env);
                            let l_src = ltmpl.replace("<slot />", "{{ content | safe }}").replace("<slot></slot>", "{{ content | safe }}");
                            l_env.add_template("layout", &l_src).map_err(|e| format!("Layout Syntax Error: {}", e))?;
                            let layout_compiled = l_env.get_template("layout").map_err(|e| e.to_string())?;
                            context_map.insert("content".to_string(), serde_json::Value::String(rendered.clone()));
                            rendered = layout_compiled
                                .render(serde_json::Value::Object(context_map))
                                .map_err(|e| format!("Layout Render Error: {}", e))?;
                        }
                    }
                }
            }

            Ok(TitaniumResponse::Html {
                status: 200,
                body: rendered,
            })
        } else {
            Ok(TitaniumResponse::Html {
                status: 200,
                body: "".to_string(),
            })
        }
    }
}

fn run_http_get(url: &str, headers: Map) -> Map {
    let mut req = ureq::get(url);
    for (k, v) in headers {
        req = req.set(&k.to_string(), &v.to_string());
    }

    let mut map = Map::new();
    match req.call() {
        Ok(resp) => {
            let status = resp.status();
            let body_text = resp.into_string().unwrap_or_default();
            map.insert("status".into(), Dynamic::from(status as i64));
            map.insert("body".into(), Dynamic::from(body_text.clone()));
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body_text) {
                if let Ok(dyn_val) = rhai::serde::to_dynamic(&json_val) {
                    map.insert("json".into(), dyn_val);
                }
            }
            map.insert("ok".into(), Dynamic::from(true));
        }
        Err(e) => {
            map.insert("status".into(), Dynamic::from(500i64));
            map.insert("error".into(), Dynamic::from(e.to_string()));
            map.insert("ok".into(), Dynamic::from(false));
        }
    }
    map
}

fn run_http_post(url: &str, body_val: Dynamic, headers: Map) -> Map {
    let mut req = ureq::post(url);
    for (k, v) in headers {
        req = req.set(&k.to_string(), &v.to_string());
    }

    let body_str = if body_val.is_string() {
        body_val.clone_cast::<String>()
    } else if let Ok(json_val) = rhai::serde::from_dynamic::<serde_json::Value>(&body_val) {
        req = req.set("Content-Type", "application/json");
        serde_json::to_string(&json_val).unwrap_or_default()
    } else {
        body_val.to_string()
    };

    let mut map = Map::new();
    match req.send_string(&body_str) {
        Ok(resp) => {
            let status = resp.status();
            let body_text = resp.into_string().unwrap_or_default();
            map.insert("status".into(), Dynamic::from(status as i64));
            map.insert("body".into(), Dynamic::from(body_text.clone()));
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body_text) {
                if let Ok(dyn_val) = rhai::serde::to_dynamic(&json_val) {
                    map.insert("json".into(), dyn_val);
                }
            }
            map.insert("ok".into(), Dynamic::from(true));
        }
        Err(e) => {
            map.insert("status".into(), Dynamic::from(500i64));
            map.insert("error".into(), Dynamic::from(e.to_string()));
            map.insert("ok".into(), Dynamic::from(false));
        }
    }
    map
}

fn mask_string(text: &str, keep_last: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= keep_last {
        return text.to_string();
    }
    let mask_len = char_count - keep_last;
    let masked_part: String = "•".repeat(mask_len);
    let visible_part: String = text.chars().skip(mask_len).collect();
    format!("{}{}", masked_part, visible_part)
}

fn add_custom_filters(env: &mut Environment) {
    env.add_filter("money", |val: Value| -> String {
        if let Ok(i) = <Value as TryInto<i64>>::try_into(val.clone()) {
            format!("${:.2}", (i as f64) / 100.0)
        } else if let Ok(f) = <Value as TryInto<f64>>::try_into(val) {
            format!("${:.2}", f)
        } else {
            "$0.00".to_string()
        }
    });

    env.add_filter("slugify", |val: Value| -> String {
        let s = val.to_string().to_lowercase();
        s.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|p| !p.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    });

    // [PHP 8.2] Mask filter
    env.add_filter("mask", |val: Value| -> String {
        mask_string(&val.to_string(), 4)
    });

    // [PHP 8.3] json_validate filter
    env.add_filter("json_validate", |val: Value| -> bool {
        serde_json::from_str::<serde_json::Value>(&val.to_string()).is_ok()
    });
}

fn resolve_layout(root_dir: &Path, custom_name: Option<&str>) -> Option<PathBuf> {
    let name = custom_name.unwrap_or("default");
    let candidates = vec![
        root_dir.join("layouts").join(format!("{}.titanium", name)),
        root_dir.join("layouts").join(format!("{}.ti", name)),
        root_dir.join("layouts").join(format!("{}.html", name)),
        root_dir.join("layouts").join(format!("{}.carbon", name)),
        root_dir.join("pages").join(format!("_layout_{}.titanium", name)),
        root_dir.join("pages").join(format!("_layout_{}.ti", name)),
        root_dir.join("pages").join(format!("_layout_{}.carbon", name)),
        root_dir.join("pages").join("_layout.titanium"),
        root_dir.join("pages").join("_layout.ti"),
        root_dir.join("pages").join("_layout.carbon"),
    ];

    for c in candidates {
        if c.is_file() {
            return Some(c);
        }
    }
    None
}

fn run_validation(data: &Map, rules: &Map) -> Map {
    let mut errors = Map::new();
    let mut is_valid = true;

    for (field, rule_dyn) in rules.iter() {
        let rule_str = rule_dyn.to_string();
        let val_opt = data.get(field);
        let val_str = val_opt.map(|d| d.to_string()).unwrap_or_default();
        let is_empty = val_opt.is_none() || val_opt.unwrap().is_unit() || val_str.trim().is_empty();

        for rule in rule_str.split('|') {
            let rule = rule.trim();
            if rule == "required" {
                if is_empty {
                    errors.insert(field.clone(), Dynamic::from(format!("{} is required", field)));
                    is_valid = false;
                    break;
                }
            } else if rule == "email" && !is_empty {
                if !val_str.contains('@') || !val_str.contains('.') {
                    errors.insert(field.clone(), Dynamic::from(format!("{} must be a valid email", field)));
                    is_valid = false;
                    break;
                }
            } else if rule.starts_with("min:") && !is_empty {
                if let Ok(min_len) = rule[4..].parse::<usize>() {
                    if val_str.len() < min_len {
                        errors.insert(field.clone(), Dynamic::from(format!("{} must be at least {} characters", field, min_len)));
                        is_valid = false;
                        break;
                    }
                }
            } else if rule.starts_with("max:") && !is_empty {
                if let Ok(max_len) = rule[4..].parse::<usize>() {
                    if val_str.len() > max_len {
                        errors.insert(field.clone(), Dynamic::from(format!("{} must not exceed {} characters", field, max_len)));
                        is_valid = false;
                        break;
                    }
                }
            }
        }
    }

    let mut res = Map::new();
    res.insert("is_valid".into(), Dynamic::from(is_valid));
    res.insert("errors".into(), Dynamic::from(errors));
    res
}

fn parse_titanium_file(content: &str, is_api: bool) -> (Option<String>, Option<String>) {
    let trimmed = content.trim_start();
    if is_api {
        return (Some(content.to_string()), None);
    }

    if trimmed.starts_with("---") {
        let after_first = &trimmed[3..];
        if let Some(end_idx) = after_first.find("---") {
            let script = after_first[..end_idx].trim().to_string();
            let template = after_first[end_idx + 3..].trim_start().to_string();
            return (Some(script), Some(template));
        }
    }

    (None, Some(content.to_string()))
}

fn render_mvc_view(root_dir: &Path, view_name: &str, data: Dynamic, session: &SessionHandle) -> Result<String, String> {
    let clean_name = view_name
        .trim_end_matches(".html")
        .trim_end_matches(".titanium")
        .trim_end_matches(".ti");

    let candidates = vec![
        root_dir.join("app").join("views").join(format!("{}.html", clean_name)),
        root_dir.join("app").join("views").join(format!("{}.titanium", clean_name)),
        root_dir.join("app").join("views").join(format!("{}.ti", clean_name)),
        root_dir.join("views").join(format!("{}.html", clean_name)),
        root_dir.join("views").join(format!("{}.titanium", clean_name)),
        root_dir.join("views").join(format!("{}.ti", clean_name)),
    ];

    let found_path = candidates.into_iter().find(|p| p.is_file())
        .ok_or_else(|| format!("MVC View not found: {}", view_name))?;

    let tmpl_str = std::fs::read_to_string(&found_path).map_err(|e| e.to_string())?;
    let (_, tmpl_body) = parse_titanium_file(&tmpl_str, false);
    let raw_tmpl = tmpl_body.unwrap_or(tmpl_str);

    let mut env = Environment::new();
    add_custom_filters(&mut env);
    env.add_template("view", &raw_tmpl).map_err(|e| format!("View Syntax Error: {}", e))?;
    let template = env.get_template("view").map_err(|e| e.to_string())?;

    let mut context_map = serde_json::Map::new();
    if let Ok(json_val) = rhai::serde::from_dynamic::<serde_json::Value>(&data) {
        if let serde_json::Value::Object(m) = json_val {
            context_map = m;
        }
    }
    context_map.insert("csrf_token".to_string(), serde_json::Value::String(session.csrf_token()));

    let rendered = template.render(serde_json::Value::Object(context_map.clone()))
        .map_err(|e| format!("View Render Error: {}", e))?;

    // Check if layout exists
    let layout_path = resolve_layout(root_dir, None);
    if !rendered.contains("<!DOCTYPE html>") && layout_path.is_some() {
        if let Some(lpath) = layout_path {
            if let Ok(layout_str) = std::fs::read_to_string(&lpath) {
                let (_, layout_tmpl) = parse_titanium_file(&layout_str, false);
                if let Some(ltmpl) = layout_tmpl {
                    let mut l_env = Environment::new();
                    add_custom_filters(&mut l_env);
                    let l_src = ltmpl.replace("<slot />", "{{ content | safe }}").replace("<slot></slot>", "{{ content | safe }}");
                    l_env.add_template("layout", &l_src).map_err(|e| format!("Layout Syntax Error: {}", e))?;
                    let layout_compiled = l_env.get_template("layout").map_err(|e| e.to_string())?;
                    context_map.insert("content".to_string(), serde_json::Value::String(rendered.clone()));
                    return layout_compiled.render(serde_json::Value::Object(context_map)).map_err(|e| format!("Layout Render Error: {}", e));
                }
            }
        }
    }

    Ok(rendered)
}

