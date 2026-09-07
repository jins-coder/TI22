use rhai::{Dynamic, Map};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, HashMap<String, Dynamic>>>>,
    flashes: Arc<Mutex<HashMap<String, HashMap<String, Dynamic>>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_session(&self, session_id: &str) -> SessionHandle {
        SessionHandle {
            session_id: session_id.to_string(),
            store: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SessionHandle {
    pub session_id: String,
    store: SessionStore,
}

impl SessionHandle {
    pub fn get(&self, key: &str) -> Dynamic {
        let sessions = self.store.sessions.lock().unwrap();
        if let Some(map) = sessions.get(&self.session_id) {
            map.get(key).cloned().unwrap_or(Dynamic::UNIT)
        } else {
            Dynamic::UNIT
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) {
        let mut sessions = self.store.sessions.lock().unwrap();
        let map = sessions.entry(self.session_id.clone()).or_default();
        map.insert(key.to_string(), value);
    }

    pub fn flash_set(&self, key: &str, value: Dynamic) {
        let mut flashes = self.store.flashes.lock().unwrap();
        let map = flashes.entry(self.session_id.clone()).or_default();
        map.insert(key.to_string(), value);
    }

    pub fn flash_get(&self, key: &str) -> Dynamic {
        let mut flashes = self.store.flashes.lock().unwrap();
        if let Some(map) = flashes.get_mut(&self.session_id) {
            map.remove(key).unwrap_or(Dynamic::UNIT)
        } else {
            Dynamic::UNIT
        }
    }

    pub fn csrf_token(&self) -> String {
        let current = self.get("_csrf");
        if current.is_string() {
            current.clone_cast::<String>()
        } else {
            // Generate a deterministic yet random token for the session
            let token = format!("{:x}", md5_simple(&format!("titanium_csrf_{}", self.session_id)));
            self.set("_csrf", Dynamic::from(token.clone()));
            token
        }
    }

    #[allow(dead_code)]
    pub fn to_map(&self) -> Map {
        let mut out = Map::new();
        let sessions = self.store.sessions.lock().unwrap();
        if let Some(map) = sessions.get(&self.session_id) {
            for (k, v) in map {
                out.insert(k.clone().into(), v.clone());
            }
        }
        out
    }
}

fn md5_simple(input: &str) -> u128 {
    let mut hash: u128 = 0xcbf29ce484222325;
    for byte in input.bytes() {
        hash ^= byte as u128;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
