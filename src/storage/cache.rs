use rhai::{Array, Dynamic, Map};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
struct CacheEntry {
    value: Dynamic,
    expires_at: Option<u64>,
}

#[derive(Clone)]
pub struct CacheStore {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl CacheStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn set(&self, key: &str, value: Dynamic, ttl_seconds: Option<i64>) {
        let now = Self::now_secs();
        let expires_at = ttl_seconds.filter(|&t| t > 0).map(|t| now + (t as u64));

        let mut lock = self.store.write().unwrap();
        lock.insert(
            key.to_string(),
            CacheEntry {
                value,
                expires_at,
            },
        );
    }

    pub fn get(&self, key: &str) -> Dynamic {
        let now = Self::now_secs();
        let mut lock = self.store.write().unwrap();

        if let Some(entry) = lock.get(key) {
            if let Some(exp) = entry.expires_at {
                if now >= exp {
                    lock.remove(key);
                    return Dynamic::UNIT;
                }
            }
            return entry.value.clone();
        }
        Dynamic::UNIT
    }

    pub fn has(&self, key: &str) -> bool {
        let now = Self::now_secs();
        let mut lock = self.store.write().unwrap();

        if let Some(entry) = lock.get(key) {
            if let Some(exp) = entry.expires_at {
                if now >= exp {
                    lock.remove(key);
                    return false;
                }
            }
            return true;
        }
        false
    }

    pub fn delete(&self, key: &str) -> bool {
        let mut lock = self.store.write().unwrap();
        lock.remove(key).is_some()
    }

    pub fn clear(&self) {
        let mut lock = self.store.write().unwrap();
        lock.clear();
    }

    pub fn keys(&self) -> Array {
        let now = Self::now_secs();
        let mut lock = self.store.write().unwrap();
        let mut active_keys = Array::new();

        lock.retain(|k, v| {
            if let Some(exp) = v.expires_at {
                if now >= exp {
                    return false;
                }
            }
            active_keys.push(Dynamic::from(k.clone()));
            true
        });

        active_keys
    }

    pub fn stats(&self) -> Map {
        let now = Self::now_secs();
        let mut lock = self.store.write().unwrap();
        lock.retain(|_, v| {
            if let Some(exp) = v.expires_at {
                now < exp
            } else {
                true
            }
        });

        let mut map = Map::new();
        map.insert("total_keys".into(), Dynamic::from(lock.len() as i64));
        map
    }
}
