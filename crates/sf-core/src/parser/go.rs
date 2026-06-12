use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::models::{StateMachine, State, Transition, Language, AnalysisSource};
use super::{CodeParser, helpers};

// type State int / type State string
static STATE_TYPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"type\s+(\w*[Ss]tate\w*)\s+(?:int|string|uint\d*)").unwrap()
});

// const ( StateIdle State = iota ... )
static IOTA_BLOCK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)const\s*\(\s*((?:\w+\s+\w+\s*=\s*iota|[A-Z]\w*[^)]*)+)\)").unwrap()
});
static IOTA_CONST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([A-Z]\w*)(?:\s+\w+)?(?:\s*=\s*iota|\s*\n)").unwrap()
});

// switch state { case StateIdle: ... case StateLoading: ... }
static GO_SWITCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)switch\s+(\w+)\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap()
});
static GO_CASE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"case\s+(State\w+|[A-Z]\w+):").unwrap()
});

// sm.SetState(StateLoading) / fsm.Transition(target)
static FSM_TRANSITION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:SetState|Transition|transitTo|setState)\s*\(\s*([A-Z]\w+|State\w+)").unwrap()
});

pub struct GoParser;

impl CodeParser for GoParser {
    fn language(&self) -> Language { Language::Go }

    fn can_parse(&self, content: &str) -> bool {
        content.contains("iota") || (content.contains("type ") && content.contains("State"))
    }

    fn parse(&self, content: &str, source_path: Option<&str>) -> Result<StateMachine> {
        let source = match source_path {
            Some(p) => AnalysisSource::CodeFile { path: p.to_string(), language: Language::Go },
            None => AnalysisSource::CodeSnippet { content: content.chars().take(200).collect(), language: Language::Go },
        };
        let mut sm = StateMachine::new("GoStateMachine", source);
        let mut found_states = std::collections::HashMap::new();

        // 1. Find state type name
        if let Some(cap) = STATE_TYPE.captures(content) {
            sm.name = format!("{}Machine", &cap[1]);
        }

        // 2. Extract iota constants as states
        for block_cap in IOTA_BLOCK.captures_iter(content) {
            let block = &block_cap[1];
            for const_cap in IOTA_CONST.captures_iter(block) {
                let sname = const_cap[1].to_string();
                // Strip common prefix like "State"
                let display = sname.trim_start_matches("State").to_string();
                let kind = helpers::state_kind_from_name(&display);
                let mut state = State::new(&display, kind);
                state.metadata.insert("const_name".to_string(), sname.clone());
                found_states.insert(sname, state.id.clone());
                found_states.insert(display.clone(), state.id.clone());
                sm.add_state(state);
            }
        }

        // 3. Extract transitions from switch statements
        for switch_cap in GO_SWITCH.captures_iter(content) {
            let body = &switch_cap[2];
            let cases: Vec<_> = GO_CASE.captures_iter(body).collect();

            for (i, case_cap) in cases.iter().enumerate() {
                let from_const = &case_cap[1];
                let case_start = case_cap.get(0).unwrap().end();
                let case_end = cases.get(i + 1)
                    .map(|c| c.get(0).unwrap().start())
                    .unwrap_or(body.len());
                let case_body = &body[case_start.min(body.len())..case_end.min(body.len())];

                for trans_cap in FSM_TRANSITION.captures_iter(case_body) {
                    let to_const = &trans_cap[1];
                    if let (Some(&from_id), Some(&to_id)) = (
                        found_states.get(from_const), found_states.get(to_const)
                    ) {
                        let mut t = Transition::new(from_id, to_id, Some(from_const.to_string()));
                        t.kind = helpers::transition_kind_from_names(from_const, to_const, Some(from_const));
                        sm.add_transition(t);
                    }
                }
            }
        }

        Ok(sm)
    }
}
