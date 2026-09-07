use rhai::{Array, Dynamic, Map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClusterNode {
    pub id: String,
    pub address: String,
    pub role: String, // "primary", "replica"
    pub status: String, // "healthy", "degraded", "joining"
    pub latency_ms: u64,
    pub last_heartbeat: u64,
    pub active_connections: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClusterMessage {
    pub id: String,
    pub from_node: String,
    pub event_type: String, // "heartbeat", "wal_sync", "broadcast", "session_sync"
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ClusterMesh {
    node_id: String,
    bind_addr: String,
    is_primary: Arc<Mutex<bool>>,
    nodes: Arc<Mutex<HashMap<String, ClusterNode>>>,
    recent_syncs: Arc<Mutex<Vec<ClusterMessage>>>,
}

impl Default for ClusterMesh {
    fn default() -> Self {
        Self::new("node-primary", "127.0.0.1:8080", true)
    }
}

#[allow(dead_code)]
impl ClusterMesh {
    pub fn new(node_id: &str, bind_addr: &str, is_primary: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut nodes = HashMap::new();
        // Register self as initial active node
        nodes.insert(
            node_id.to_string(),
            ClusterNode {
                id: node_id.to_string(),
                address: bind_addr.to_string(),
                role: if is_primary { "primary".into() } else { "replica".into() },
                status: "healthy".into(),
                latency_ms: 1,
                last_heartbeat: now,
                active_connections: 1,
            },
        );

        Self {
            node_id: node_id.to_string(),
            bind_addr: bind_addr.to_string(),
            is_primary: Arc::new(Mutex::new(is_primary)),
            nodes: Arc::new(Mutex::new(nodes)),
            recent_syncs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }

    pub fn is_primary(&self) -> bool {
        *self.is_primary.lock().unwrap()
    }

    pub fn set_primary(&self, val: bool) {
        *self.is_primary.lock().unwrap() = val;
    }

    /// Register or refresh peer heartbeat
    pub fn register_heartbeat(&self, node: ClusterNode) {
        if let Ok(mut map) = self.nodes.lock() {
            map.insert(node.id.clone(), node);
        }
    }

    /// Record a distributed WAL sync or cross-node event
    pub fn record_sync(&self, event_type: &str, payload: serde_json::Value) -> ClusterMessage {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let msg = ClusterMessage {
            id: format!("sync_{}", now),
            from_node: self.node_id.clone(),
            event_type: event_type.to_string(),
            payload,
            timestamp: now,
        };

        if let Ok(mut syncs) = self.recent_syncs.lock() {
            syncs.push(msg.clone());
            if syncs.len() > 100 {
                syncs.remove(0);
            }
        }

        msg
    }

    /// List all discovered active cluster nodes
    pub fn nodes_list(&self) -> Vec<ClusterNode> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut list = Vec::new();
        if let Ok(map) = self.nodes.lock() {
            for (_, mut node) in map.clone() {
                // If heartbeat is older than 30s, mark as degraded
                if now.saturating_sub(node.last_heartbeat) > 30 && node.id != self.node_id {
                    node.status = "degraded".into();
                }
                list.push(node);
            }
        }
        list
    }

    /// Return cluster summary map for Rhai / HTTP JSON APIs
    pub fn stats_map(&self) -> Map {
        let nodes = self.nodes_list();
        let healthy_count = nodes.iter().filter(|n| n.status == "healthy").count() as i64;
        let mut map = Map::new();

        map.insert("node_id".into(), Dynamic::from(self.node_id.clone()));
        map.insert("is_primary".into(), Dynamic::from(self.is_primary()));
        map.insert("total_nodes".into(), Dynamic::from(nodes.len() as i64));
        map.insert("healthy_nodes".into(), Dynamic::from(healthy_count));
        map.insert("version".into(), Dynamic::from("10.0.0"));

        let mut nodes_arr = Array::new();
        for n in &nodes {
            let mut node_map = Map::new();
            node_map.insert("id".into(), Dynamic::from(n.id.clone()));
            node_map.insert("address".into(), Dynamic::from(n.address.clone()));
            node_map.insert("role".into(), Dynamic::from(n.role.clone()));
            node_map.insert("status".into(), Dynamic::from(n.status.clone()));
            node_map.insert("latency_ms".into(), Dynamic::from(n.latency_ms as i64));
            nodes_arr.push(Dynamic::from(node_map));
        }
        map.insert("nodes".into(), Dynamic::from(nodes_arr));

        map
    }
}
