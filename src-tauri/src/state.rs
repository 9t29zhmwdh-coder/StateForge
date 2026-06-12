use sqlx::SqlitePool;
use tokio::sync::RwLock;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            ai_backend: "claude".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3".to_string(),
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
