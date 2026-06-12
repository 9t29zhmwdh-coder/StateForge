use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::models::{StateMachine, State, Transition, Language, AnalysisSource};
use super::{CodeParser, helpers};

// sealed class UiState { data class Loading : UiState(); object Idle : UiState() }
static SEALED_CLASS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)sealed\s+class\s+(\w+)\s*\{([^}]+(?:\{[^}]*\}[^}]*)*)\}").unwrap()
});
static SEALED_SUBCLASS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:data class|class|object)\s+(\w+)(?:\s*\([^)]*\))?\s*:\s*\w+").unwrap()
});

// when (state) { is UiState.Loading -> ... }
static WHEN_EXPR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)when\s*\(\s*(\w+)\s*\)\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap()
});
static WHEN_BRANCH_IS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"is\s+\w+\.(\w+)\s*->").unwrap()
});
static WHEN_BRANCH_ELSE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"else\s*->").unwrap()
});

// _state.value = UiState.Loading
static STATE_ASSIGN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:_state|state|_uiState|uiState)\.(?:value|emit)\s*[=\(]\s*\w+\.(\w+)").unwrap()
});

// MVI: intent/action sealed class
static SEALED_INTENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)sealed\s+class\s+(\w*(?:Intent|Action|Event)\w*)\s*\{([^}]+)\}").unwrap()
});

pub struct KotlinParser;

impl CodeParser for KotlinParser {
    fn language(&self) -> Language { Language::Kotlin }

    fn can_parse(&self, content: &str) -> bool {
        content.contains("sealed class") || content.contains("data class") && content.contains("when")
    }

    fn parse(&self, content: &str, source_path: Option<&str>) -> Result<StateMachine> {
        let source = match source_path {
            Some(p) => AnalysisSource::CodeFile { path: p.to_string(), language: Language::Kotlin },
            None => AnalysisSource::CodeSnippet { content: content.chars().take(200).collect(), language: Language::Kotlin },
        };

        let mut sm = StateMachine::new("KotlinStateMachine", source);
        let mut found_states = std::collections::HashMap::new();

        // 1. Extract sealed class states
        for cap in SEALED_CLASS.captures_iter(content) {
            let class_name = &cap[1];
            let class_lower = class_name.to_lowercase();

            let is_state_class = class_lower.contains("state") || class_lower.contains("ui") || class_lower.contains("status");
            let is_intent_class = class_lower.contains("intent") || class_lower.contains("action") || class_lower.contains("event");

            if is_intent_class { continue; }

            sm.name = format!("{}Machine", class_name);

            for sub_cap in SEALED_SUBCLASS.captures_iter(&cap[2]) {
                let sub_name = sub_cap[1].to_string();
                let kind = helpers::state_kind_from_name(&sub_name);
                let mut state = State::new(&sub_name, kind);
                found_states.insert(sub_name.clone(), state.id.clone());
                sm.add_state(state);
            }
        }

        // 2. Extract intents/actions as events from sealed intent classes
        let mut events: Vec<String> = Vec::new();
        for cap in SEALED_INTENT.captures_iter(content) {
            for sub_cap in SEALED_SUBCLASS.captures_iter(&cap[2]) {
                events.push(sub_cap[1].to_string());
            }
        }

        // 3. Extract transitions from when expressions
        for when_cap in WHEN_EXPR.captures_iter(content) {
            let body = &when_cap[2];
            let branches: Vec<_> = WHEN_BRANCH_IS.captures_iter(body).collect();

            for (i, branch) in branches.iter().enumerate() {
                let from_state = &branch[1];
                let branch_start = branch.get(0).unwrap().end();
                let branch_end = branches.get(i + 1)
                    .map(|b| b.get(0).unwrap().start())
                    .unwrap_or(body.len());
                let branch_body = &body[branch_start.min(body.len())..branch_end.min(body.len())];

                for assign_cap in STATE_ASSIGN.captures_iter(branch_body) {
                    let to_state = &assign_cap[1];
                    if let (Some(&from_id), Some(&to_id)) = (
                        found_states.get(from_state), found_states.get(to_state)
                    ) {
                        let event = events.first().cloned();
                        let mut t = Transition::new(from_id, to_id, event.clone());
                        t.kind = helpers::transition_kind_from_names(from_state, to_state, event.as_deref());
                        sm.add_transition(t);
                    }
                }
            }
        }

        // 4. Direct state assignments (ViewModel pattern)
        if sm.transitions.is_empty() {
            for assign_cap in STATE_ASSIGN.captures_iter(content) {
                let to_state = &assign_cap[1];
                if found_states.contains_key(to_state) {
                    // create transitions from all non-final states to this one
                    let to_id = found_states[to_state].clone();
                    let froms: Vec<_> = found_states.iter()
                        .filter(|(n, _)| *n != to_state)
                        .map(|(n, id)| (n.clone(), id.clone()))
                        .take(2)
                        .collect();
                    for (from_name, from_id) in froms {
                        let mut t = Transition::new(from_id, to_id.clone(), None);
                        t.kind = helpers::transition_kind_from_names(&from_name, to_state, None);
                        sm.add_transition(t);
                    }
                }
            }
        }

        if sm.name == "KotlinStateMachine" {
            if let Some(first) = sm.states.first() {
                sm.name = format!("{}Machine", first.name);
            }
        }

        Ok(sm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KOTLIN_CODE: &str = r#"
sealed class LoginUiState {
    object Idle : LoginUiState()
    object Loading : LoginUiState()
    data class Success(val user: User) : LoginUiState()
    data class Error(val message: String) : LoginUiState()
}
sealed class LoginIntent {
    object Login : LoginIntent()
    object Logout : LoginIntent()
}
    "#;

    #[test]
    fn parses_kotlin_sealed_class() {
        let parser = KotlinParser;
        let sm = parser.parse(KOTLIN_CODE, None).unwrap();
        assert!(sm.states.len() >= 4);
    }
}
