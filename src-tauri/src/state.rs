use sqlx::SqlitePool;
use tokio::sync::RwLock;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

// `default` keeps stored settings readable when fields change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub ai_backend: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub theme: String,
    pub default_diagram_format: String,
    pub auto_ai_enhance: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ai_backend: "ollama".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "qwen3.5:4b".to_string(),
            theme: "dark".to_string(),
            default_diagram_format: "mermaid".to_string(),
            auto_ai_enhance: false,
        }
    }
}

pub struct AppState {
    pub pool: SqlitePool,
    pub settings: Arc<RwLock<AppSettings>>,
}
