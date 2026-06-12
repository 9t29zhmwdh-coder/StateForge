use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::state_machine::Language;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    EnumState,
    SwitchCase,
    ReducerPattern,
    ViewModel,
    SealedClass,
    UnionType,
    IotaConst,
    XState,
    Redux,
    Tca,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub kind: PatternKind,
    pub language: Language,
    pub name: String,
    pub states_found: Vec<String>,
    pub transitions_found: usize,
    pub source_file: Option<String>,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub confidence: f32,
    pub raw_snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: String,
    pub source_path: Option<String>,
    pub language: Language,
    pub patterns: Vec<PatternMatch>,
    pub state_machine_id: String,
    pub analyzed_at: DateTime<Utc>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub ai_insight: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSequence {
    pub id: String,
    pub events: Vec<LogEvent>,
    pub duration_ms: Option<u64>,
    pub final_state: Option<String>,
    pub is_error_path: bool,
    pub occurrence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp: Option<DateTime<Utc>>,
    pub name: String,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub payload: Option<String>,
    pub level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGroup {
    pub id: String,
    pub name: String,
    pub sequences: Vec<EventSequence>,
    pub template: String,
    pub frequency: usize,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSequence {
    pub id: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: Option<u64>,
    pub next_calls: Vec<String>,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
}
