use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum StateKind {
    Initial,
    Normal,
    Final,
    Error,
    Parallel,
    History,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub id: String,
    pub name: String,
    pub kind: StateKind,
    pub description: Option<String>,
    pub entry_actions: Vec<String>,
    pub exit_actions: Vec<String>,
    pub substates: Vec<State>,
    pub metadata: HashMap<String, String>,
}

impl State {
    pub fn new(name: impl Into<String>, kind: StateKind) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            kind,
            description: None,
            entry_actions: Vec::new(),
            exit_actions: Vec::new(),
            substates: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, StateKind::Final | StateKind::Error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    Normal,
    Auto,
    Error,
    Timeout,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub from_state: String,
    pub to_state: String,
    pub event: Option<String>,
    pub guard: Option<String>,
    pub actions: Vec<String>,
    pub kind: TransitionKind,
    pub probability: Option<f32>,
}

impl Transition {
    pub fn new(from: impl Into<String>, to: impl Into<String>, event: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_state: from.into(),
            to_state: to.into(),
            event,
            guard: None,
            actions: Vec::new(),
            kind: TransitionKind::Normal,
            probability: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Swift,
    Kotlin,
    TypeScript,
    Go,
    Rust,
    Generic,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "swift" => Self::Swift,
            "kt" | "kts" => Self::Kotlin,
            "ts" | "tsx" => Self::TypeScript,
            "go" => Self::Go,
            "rs" => Self::Rust,
            _ => Self::Generic,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Swift => "Swift",
            Self::Kotlin => "Kotlin",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::Rust => "Rust",
            Self::Generic => "Generic",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisSource {
    CodeFile { path: String, language: Language },
    CodeSnippet { content: String, language: Language },
    LogFile { path: String },
    LogContent { content: String },
    ApiSequence { content: String },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    pub id: String,
    pub name: String,
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
    pub initial_state: Option<String>,
    pub context_type: Option<String>,
    pub source: AnalysisSource,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub ai_summary: Option<String>,
}

impl StateMachine {
    pub fn new(name: impl Into<String>, source: AnalysisSource) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            states: Vec::new(),
            transitions: Vec::new(),
            initial_state: None,
            context_type: None,
            source,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            ai_summary: None,
        }
    }

    pub fn add_state(&mut self, state: State) {
        if self.initial_state.is_none() || state.kind == StateKind::Initial {
            if state.kind == StateKind::Initial {
                self.initial_state = Some(state.id.clone());
            } else if self.initial_state.is_none() {
                self.initial_state = Some(state.id.clone());
            }
        }
        self.states.push(state);
    }

    pub fn add_transition(&mut self, t: Transition) {
        self.transitions.push(t);
    }

    pub fn state_by_name(&self, name: &str) -> Option<&State> {
        self.states.iter().find(|s| s.name == name)
    }

    pub fn state_by_id(&self, id: &str) -> Option<&State> {
        self.states.iter().find(|s| s.id == id)
    }

    pub fn outgoing(&self, state_id: &str) -> Vec<&Transition> {
        self.transitions.iter().filter(|t| t.from_state == state_id).collect()
    }

    pub fn incoming(&self, state_id: &str) -> Vec<&Transition> {
        self.transitions.iter().filter(|t| t.to_state == state_id).collect()
    }

    pub fn is_deterministic(&self) -> bool {
        for state in &self.states {
            let out = self.outgoing(&state.id);
            let mut event_counts: HashMap<Option<String>, usize> = HashMap::new();
            for t in out {
                if t.guard.is_none() {
                    *event_counts.entry(t.event.clone()).or_insert(0) += 1;
                    if event_counts[&t.event] > 1 {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn unreachable_states(&self) -> Vec<&State> {
        use std::collections::HashSet;
        let mut reachable = HashSet::new();
        if let Some(ref init) = self.initial_state {
            let mut stack = vec![init.as_str()];
            while let Some(sid) = stack.pop() {
                if reachable.insert(sid) {
                    for t in self.outgoing(sid) {
                        stack.push(t.to_state.as_str());
                    }
                }
            }
        }
        self.states.iter().filter(|s| !reachable.contains(s.id.as_str())).collect()
    }
}
