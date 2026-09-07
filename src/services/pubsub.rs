use rhai::{Array, Dynamic, Map};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct PubSubHub {
    topics: Arc<RwLock<HashMap<String, Vec<Map>>>>,
}

impl PubSubHub {
    pub fn new() -> Self {
        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn publish(&self, topic: &str, message: Dynamic) {
        let mut msg_map = Map::new();
        msg_map.insert("topic".into(), Dynamic::from(topic.to_string()));
        msg_map.insert("message".into(), message);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        msg_map.insert("timestamp".into(), Dynamic::from(timestamp as i64));

        let mut lock = self.topics.write().unwrap();
        let history = lock.entry(topic.to_string()).or_default();
        if history.len() >= 50 {
            history.remove(0);
        }
        history.push(msg_map);
    }

    pub fn history(&self, topic: &str, limit: i64) -> Array {
        let lock = self.topics.read().unwrap();
        if let Some(list) = lock.get(topic) {
            let take_count = if limit <= 0 { 20 } else { limit as usize };
            list.iter()
                .rev()
                .take(take_count)
                .cloned()
                .map(Dynamic::from)
                .collect()
        } else {
            Array::new()
        }
    }

    pub fn list_topics(&self) -> Array {
        let lock = self.topics.read().unwrap();
        lock.keys()
            .cloned()
            .map(Dynamic::from)
            .collect()
    }
}
