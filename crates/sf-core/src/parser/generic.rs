use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::models::{StateMachine, State, Language, AnalysisSource};
use super::{CodeParser, helpers};

// Generic: any switch/case pattern with state-like names
static SWITCH_BLOCK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)switch\s*\([^)]*\)\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap()
});
static GENERIC_CASE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"case\s+['"]?([A-Z][a-zA-Z0-9_]+|[a-z][a-zA-Z0-9_]+)['"]?:"#).unwrap()
});

// Any UPPERCASE_CONST that looks state-like
static CONST_STATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(STATE_[A-Z_]+|[A-Z][A-Z_]{3,})\b").unwrap()
});

pub struct GenericParser;

impl CodeParser for GenericParser {
    fn language(&self) -> Language { Language::Generic }
    fn can_parse(&self, _content: &str) -> bool { true }

    fn parse(&self, content: &str, source_path: Option<&str>) -> Result<StateMachine> {
        let source = match source_path {
            Some(p) => AnalysisSource::CodeFile { path: p.to_string(), language: Language::Generic },
            None => AnalysisSource::CodeSnippet { content: content.chars().take(200).collect(), language: Language::Generic },
        };
        let mut sm = StateMachine::new("StateMachine", source);
        let mut seen = std::collections::HashSet::new();

        for switch_cap in SWITCH_BLOCK.captures_iter(content) {
            for case_cap in GENERIC_CASE.captures_iter(&switch_cap[1]) {
                let name = case_cap[1].to_string();
                if name == "default" || name == "else" || seen.contains(&name) { continue; }
                seen.insert(name.clone());
                let kind = helpers::state_kind_from_name(&name);
                sm.add_state(State::new(&name, kind));
            }
        }

        if sm.states.is_empty() {
            let mut const_seen = std::collections::HashSet::new();
            for cap in CONST_STATE.captures_iter(content) {
                let name = cap[1].to_string();
                if const_seen.insert(name.clone()) {
                    let kind = helpers::state_kind_from_name(&name);
                    sm.add_state(State::new(&name, kind));
                }
            }
        }

        Ok(sm)
    }
}
