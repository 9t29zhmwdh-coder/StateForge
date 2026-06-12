use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::models::{StateMachine, State, Transition, Language, AnalysisSource};
use super::{CodeParser, helpers};

// type State = 'idle' | 'loading' | 'success' | 'error'
static UNION_TYPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"type\s+(\w*[Ss]tate\w*)\s*=\s*([^;]+);").unwrap()
});
static UNION_MEMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"['"](\w+)['"]"#).unwrap()
});

// Redux reducer: switch (state) { case 'loading': ... case 'success': ... }
static REDUCER_SWITCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)switch\s*\((?:state|action\.type|action\.payload\.?type)\s*\)\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap()
});
static REDUCER_CASE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"case\s+['"]?(\w+)['"]?:"#).unwrap()
});

// return { ...state, status: 'loading' }  /  return { ...state, phase: newPhase }
static STATE_RETURN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:status|state|phase|mode)\s*:\s*['"](\w+)['"]"#).unwrap()
});

// XState createMachine
static XSTATE_MACHINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)createMachine\s*\(\s*\{[^{]*states\s*:\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap()
});
static XSTATE_STATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+)\s*:\s*\{[^{}]*(?:type\s*:\s*['\"](final|parallel|history)['\"])?").unwrap()
});
static XSTATE_ON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"on\s*:\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap()
});
static XSTATE_TRANSITION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(\w+)\s*:\s*['"]?(\w+)['"]?"#).unwrap()
});

pub struct TypeScriptParser;

impl CodeParser for TypeScriptParser {
    fn language(&self) -> Language { Language::TypeScript }

    fn can_parse(&self, content: &str) -> bool {
        content.contains("createMachine")
            || (content.contains("type ") && content.contains("| '"))
            || (content.contains("switch") && content.contains("case '"))
    }

    fn parse(&self, content: &str, source_path: Option<&str>) -> Result<StateMachine> {
        let source = match source_path {
            Some(p) => AnalysisSource::CodeFile { path: p.to_string(), language: Language::TypeScript },
            None => AnalysisSource::CodeSnippet { content: content.chars().take(200).collect(), language: Language::TypeScript },
        };
        let mut sm = StateMachine::new("TypeScriptStateMachine", source);
        let mut found_states = std::collections::HashMap::new();

        // 1. XState createMachine — most structured
        if let Some(cap) = XSTATE_MACHINE.captures(content) {
            sm.name = "XStateMachine".to_string();
            let states_body = &cap[1];

            for state_cap in XSTATE_STATE.captures_iter(states_body) {
                let sname = state_cap[1].to_string();
                if sname == "on" || sname == "entry" || sname == "exit" || sname == "always" { continue; }

                let kind = if let Some(t) = state_cap.get(2) {
                    match t.as_str() {
                        "final" => crate::models::StateKind::Final,
                        "parallel" => crate::models::StateKind::Parallel,
                        _ => helpers::state_kind_from_name(&sname),
                    }
                } else {
                    helpers::state_kind_from_name(&sname)
                };

                let state = State::new(&sname, kind);
                found_states.insert(sname, state.id.clone());
                sm.add_state(state);
            }

            // Extract transitions from on: blocks
            for on_cap in XSTATE_ON.captures_iter(states_body) {
                for t_cap in XSTATE_TRANSITION.captures_iter(&on_cap[1]) {
                    let event = t_cap[1].to_string();
                    let to_state = t_cap[2].to_string();
                    if let Some(&to_id) = found_states.get(&to_state) {
                        for (from_name, from_id) in found_states.iter().take(1) {
                            let mut t = Transition::new(from_id.clone(), to_id.clone(), Some(event.clone()));
                            t.kind = helpers::transition_kind_from_names(from_name, &to_state, Some(&event));
                            sm.add_transition(t);
                        }
                    }
                }
            }

            return Ok(sm);
        }

        // 2. Union type states
        for cap in UNION_TYPE.captures_iter(content) {
            sm.name = format!("{}Machine", &cap[1]);
            for member_cap in UNION_MEMBER.captures_iter(&cap[2]) {
                let sname = member_cap[1].to_string();
                let kind = helpers::state_kind_from_name(&sname);
                let state = State::new(&sname, kind);
                found_states.insert(sname, state.id.clone());
                sm.add_state(state);
            }
        }

        // 3. Redux switch reducer transitions
        for switch_cap in REDUCER_SWITCH.captures_iter(content) {
            let body = &switch_cap[1];
            let cases: Vec<_> = REDUCER_CASE.captures_iter(body).collect();

            for (i, case_cap) in cases.iter().enumerate() {
                let event = case_cap[1].to_string();
                let case_start = case_cap.get(0).unwrap().end();
                let case_end = cases.get(i + 1)
                    .map(|c| c.get(0).unwrap().start())
                    .unwrap_or(body.len());
                let case_body = &body[case_start.min(body.len())..case_end.min(body.len())];

                for ret_cap in STATE_RETURN.captures_iter(case_body) {
                    let to_state = ret_cap[1].to_string();
                    if let Some(&to_id) = found_states.get(&to_state) {
                        for (from_name, from_id) in found_states.iter().take(1) {
                            let mut t = Transition::new(from_id.clone(), to_id.clone(), Some(event.clone()));
                            t.kind = helpers::transition_kind_from_names(from_name, &to_state, Some(&event));
                            sm.add_transition(t);
                        }
                    }
                }
            }
        }

        Ok(sm)
    }
}
