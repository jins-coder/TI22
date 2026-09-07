use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: Option<String>,
    pub wal_mode: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub default_ttl: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub worker_threads: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TitaniumConfig {
    pub server: Option<ServerConfig>,
    pub database: Option<DatabaseConfig>,
    pub cache: Option<CacheConfig>,
    pub queue: Option<QueueConfig>,
}

impl TitaniumConfig {
    pub fn load_from_dir(root_dir: &Path) -> Self {
        for filename in &["titanium.toml", "cobalt.toml", "carbon.toml"] {
            let toml_path = root_dir.join(filename);
            if toml_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&toml_path) {
                    if let Ok(cfg) = toml::from_str::<TitaniumConfig>(&content) {
                        return cfg;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn host(&self, default: &str) -> String {
        self.server
            .as_ref()
            .and_then(|s| s.host.clone())
            .unwrap_or_else(|| default.to_string())
    }

    pub fn port(&self, default: u16) -> u16 {
        self.server
            .as_ref()
            .and_then(|s| s.port)
            .unwrap_or(default)
    }

    pub fn workers(&self, default: usize) -> usize {
        self.server
            .as_ref()
            .and_then(|s| s.workers)
            .unwrap_or(default)
    }

    pub fn db_path(&self, root_dir: &Path) -> String {
        if let Some(ref db) = self.database {
            if let Some(ref p) = db.path {
                return root_dir.join(p).to_string_lossy().to_string();
            }
        }
        root_dir
            .join(".titanium")
            .join("app.sqlite")
            .to_string_lossy()
            .to_string()
    }

    pub fn queue_workers(&self, default: usize) -> usize {
        self.queue
            .as_ref()
            .and_then(|q| q.worker_threads)
            .unwrap_or(default)
    }
}
