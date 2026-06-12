use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static ERROR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:ERROR|FATAL|EXCEPTION|PANIC|CRASH|FAILED?)\s+(?:in\s+)?(?:state\s+)?['\"]?(\w+)['\"]?").unwrap()
});

static DEAD_STATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:stuck|deadlock|timeout|unreachable|infinite loop)\s+(?:in\s+)?(?:state\s+)?['\"]?(\w+)['\"]?").unwrap()
});

pub struct ErrorPathDetector;

impl ErrorPathDetector {
    pub fn new() -> Self { Self }

    pub fn detect(&self, content: &str) -> HashSet<String> {
        let mut error_states = HashSet::new();

        for cap in ERROR_PATTERN.captures_iter(content) {
            error_states.insert(cap[1].to_string());
        }
        for cap in DEAD_STATE.captures_iter(content) {
            error_states.insert(cap[1].to_string());
        }

        error_states
    }

    pub fn find_error_transitions<'a>(
        &self,
        sm: &'a crate::models::StateMachine,
    ) -> Vec<&'a crate::models::Transition> {
        sm.transitions.iter()
            .filter(|t| {
                use crate::models::TransitionKind;
                t.kind == TransitionKind::Error
                    || sm.state_by_id(&t.to_state)
                        .map(|s| matches!(s.kind, crate::models::StateKind::Error))
                        .unwrap_or(false)
            })
            .collect()
    }
}
