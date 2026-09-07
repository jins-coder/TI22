use rhai::{Array, Dynamic, Map};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsMessage {
    pub id: String,
    pub channel: String,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct WebSocketHub {
    // Channel name -> list of client message channels / queues
    channels: Arc<Mutex<HashMap<String, Vec<std::sync::mpsc::Sender<WsMessage>>>>>,
    // Broadcast all listeners
    global_listeners: Arc<Mutex<Vec<std::sync::mpsc::Sender<WsMessage>>>>,
    // Message history log for debugging & Studio inspector (ring buffer)
    history: Arc<Mutex<VecDeque<WsMessage>>>,
    max_history: usize,
}

impl WebSocketHub {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            global_listeners: Arc::new(Mutex::new(Vec::new())),
            history: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            max_history: 100,
        }
    }

    pub fn subscribe(&self, channel: &str) -> std::sync::mpsc::Receiver<WsMessage> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut map = self.channels.lock().unwrap();
        map.entry(channel.to_string()).or_default().push(tx);
        rx
    }

    pub fn subscribe_global(&self) -> std::sync::mpsc::Receiver<WsMessage> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut list = self.global_listeners.lock().unwrap();
        list.push(tx);
        rx
    }

    pub fn broadcast(&self, channel: &str, payload: serde_json::Value) -> WsMessage {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let msg = WsMessage {
            id: format!("ws_{}", now),
            channel: channel.to_string(),
            payload,
            timestamp: now,
        };

        // 1. Record in history
        {
            let mut hist = self.history.lock().unwrap();
            if hist.len() >= self.max_history {
                hist.pop_front();
            }
            hist.push_back(msg.clone());
        }

        // 2. Dispatch to channel subscribers
        {
            let mut map = self.channels.lock().unwrap();
            if let Some(senders) = map.get_mut(channel) {
                senders.retain(|tx| tx.send(msg.clone()).is_ok());
            }
        }

        // 3. Dispatch to global listeners
        {
            let mut list = self.global_listeners.lock().unwrap();
            list.retain(|tx| tx.send(msg.clone()).is_ok());
        }

        msg
    }

    pub fn broadcast_dynamic(&self, channel: &str, data: Dynamic) -> WsMessage {
        let val = rhai::serde::from_dynamic::<serde_json::Value>(&data).unwrap_or(serde_json::Value::Null);
        self.broadcast(channel, val)
    }

    pub fn client_count(&self, channel: &str) -> usize {
        let mut map = self.channels.lock().unwrap();
        if let Some(senders) = map.get_mut(channel) {
            // Filter out disconnected senders
            senders.retain(|tx| tx.send(WsMessage {
                id: "ping".to_string(),
                channel: "ping".to_string(),
                payload: serde_json::Value::Null,
                timestamp: 0,
            }).is_ok());
            senders.len()
        } else {
            0
        }
    }

    pub fn total_clients(&self) -> usize {
        let global = self.global_listeners.lock().unwrap().len();
        let mut channel_total = 0;
        let map = self.channels.lock().unwrap();
        for (_, senders) in map.iter() {
            channel_total += senders.len();
        }
        global + channel_total
    }

    pub fn channels_list(&self) -> Vec<String> {
        let map = self.channels.lock().unwrap();
        map.keys().cloned().collect()
    }

    pub fn recent_history(&self) -> Vec<WsMessage> {
        let hist = self.history.lock().unwrap();
        hist.iter().cloned().collect()
    }

    pub fn stats_map(&self) -> Map {
        let mut m = Map::new();
        m.insert("total_clients".into(), Dynamic::from(self.total_clients() as i64));
        m.insert("channels_count".into(), Dynamic::from(self.channels_list().len() as i64));
        
        let mut channels_arr = Array::new();
        for c in self.channels_list() {
            channels_arr.push(Dynamic::from(c));
        }
        m.insert("channels".into(), Dynamic::from(channels_arr));
        m
    }
}
