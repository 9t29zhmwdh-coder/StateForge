use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::models::{StateMachine, State, Transition, Language, AnalysisSource};
use super::{CodeParser, helpers};

// enum State { case idle, case loading, case success(Model), case error(Error) }
static ENUM_STATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)enum\s+(\w+)\s*(?::\s*\w+\s*)?\{([^}]+)\}").unwrap()
});
static ENUM_CASE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"case\s+(\w+)(?:\(([^)]*)\))?").unwrap()
});

// switch state { case .idle: ... case .loading: ... }
static SWITCH_STATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"switch\s+(\w+)\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap()
});
static SWITCH_CASE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"case\s+\.(\w+)(?:\(([^)]*)\))?:").unwrap()
});

// TCA / Reducer: case .event: return .run / state = .nextState
static TCA_REDUCE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"case\s+\.(\w+).*?state\s*[=.]\s*[=.]*(\w+)").unwrap()
});

// @Observable / ObservableObject state property
static OBSERVABLE_STATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:@Published\s+)?var\s+(?:state|currentState)\s*[=:]\s*(\w+)\.(\w+)").unwrap()
});

pub struct SwiftParser;

impl CodeParser for SwiftParser {
    fn language(&self) -> Language { Language::Swift }

    fn can_parse(&self, content: &str) -> bool {
        content.contains("enum ") && content.contains("case ")
            || content.contains("@Observable")
            || content.contains("Reducer")
    }

    fn parse(&self, content: &str, source_path: Option<&str>) -> Result<StateMachine> {
        let source = match source_path {
            Some(p) => AnalysisSource::CodeFile { path: p.to_string(), language: Language::Swift },
            None => AnalysisSource::CodeSnippet { content: content.chars().take(200).collect(), language: Language::Swift },
        };

        let mut sm = StateMachine::new("SwiftStateMachine", source);
        let mut found_states = std::collections::HashMap::new();

        // 1. Extract state enums
        for cap in ENUM_STATE.captures_iter(content) {
            let enum_name = &cap[1];
            if !looks_like_state_enum(enum_name, &cap[2]) {
                continue;
            }
            sm.name = enum_name.to_string();

            for case_cap in ENUM_CASE.captures_iter(&cap[2]) {
                let case_name = case_cap[1].to_string();
                let kind = helpers::state_kind_from_name(&case_name);
                let mut state = State::new(&case_name, kind);

                if let Some(payload) = case_cap.get(2) {
                    state.metadata.insert("payload".to_string(), payload.as_str().to_string());
                }

                found_states.insert(case_name.clone(), state.id.clone());
                sm.add_state(state);
            }
        }

        // 2. Extract transitions from switch statements
        for switch_cap in SWITCH_STATE.captures_iter(content) {
            let body = &switch_cap[2];
            let cases: Vec<_> = SWITCH_CASE.captures_iter(body).collect();

            // Try to find next-state assignments within each case
            for i in 0..cases.len() {
                let from_name = &cases[i][1];

                // Look for state = .X or state = .newState within the case block
                let case_start = cases[i].get(0).unwrap().end();
                let case_end = cases.get(i + 1)
                    .map(|c| c.get(0).unwrap().start())
                    .unwrap_or(body.len());
                let case_body = &body[case_start.min(body.len())..case_end.min(body.len())];

                for to_cap in Regex::new(r"(?:state|currentState)\s*=\s*\.(\w+)").unwrap().captures_iter(case_body) {
                    let to_name = &to_cap[1];
                    if let (Some(&from_id), Some(&to_id)) = (
                        found_states.get(from_name), found_states.get(to_name)
                    ) {
                        let mut t = Transition::new(from_id, to_id, Some(from_name.to_string()));
                        t.kind = helpers::transition_kind_from_names(from_name, to_name, Some(from_name));
                        sm.add_transition(t);
                    }
                }
            }
        }

        // 3. TCA Reducer transitions
        for cap in TCA_REDUCE.captures_iter(content) {
            let event = &cap[1];
            let next_state = &cap[2];

            if let Some(&to_id) = found_states.get(next_state) {
                // If we know the current state from context, use it; otherwise create "any → next"
                for (from_name, from_id) in &found_states {
                    if from_name != next_state {
                        let mut t = Transition::new(from_id.clone(), to_id.clone(), Some(event.to_string()));
                        t.kind = helpers::transition_kind_from_names(from_name, next_state, Some(event));
                        sm.add_transition(t);
                        break; // one representative transition per TCA action
                    }
                }
            }
        }

        if sm.states.is_empty() {
            // Fallback: extract any case patterns
            for cap in ENUM_CASE.captures_iter(content) {
                let name = cap[1].to_string();
                if name.len() > 1 {
                    let kind = helpers::state_kind_from_name(&name);
                    let state = State::new(&name, kind);
                    found_states.insert(name, state.id.clone());
                    sm.add_state(state);
                }
            }
        }

        Ok(sm)
    }
}

fn looks_like_state_enum(name: &str, body: &str) -> bool {
    let lower = name.to_lowercase();
    let has_state_word = lower.contains("state") || lower.contains("status") || lower.contains("phase") || lower.contains("mode");
    let has_multiple_cases = body.matches("case ").count() >= 2;
    has_state_word || has_multiple_cases
}

#[cfg(test)]
mod tests {
    use super::*;

    const SWIFT_CODE: &str = r#"
enum LoadingState: Equatable {
    case idle
    case loading
    case success(User)
    case error(Error)
}

func reduce(_ state: inout LoadingState, _ action: Action) -> Effect<Action> {
    switch state {
    case .idle:
        state = .loading
        return .none
    case .loading:
        state = .success(user)
        return .none
    case .success:
        return .none
    case .error:
        state = .idle
        return .none
    }
}
    "#;

    #[test]
    fn parses_swift_states() {
        let parser = SwiftParser;
        let sm = parser.parse(SWIFT_CODE, None).unwrap();
        assert!(sm.states.len() >= 4);
        let names: Vec<_> = sm.states.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"idle"));
        assert!(names.contains(&"loading"));
        assert!(names.contains(&"error"));
    }
}
