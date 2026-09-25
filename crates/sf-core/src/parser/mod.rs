pub mod flow;
pub mod generic;

use crate::models::{AnalysisSource, Language, State, StateKind, StateMachine, Transition};
use anyhow::{bail, Result};

pub trait CodeParser: Send + Sync {
    fn language(&self) -> Language;
    fn can_parse(&self, content: &str) -> bool;
    fn parse(&self, content: &str, source_path: Option<&str>) -> Result<StateMachine>;
}

pub fn parse_file(path: &str, content: &str) -> Result<StateMachine> {
    let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");
    parse(content, Language::from_extension(ext), Some(path))
}

pub fn parse_with_language(content: &str, lang: Language) -> Result<StateMachine> {
    parse(content, lang, None)
}

fn parse(content: &str, lang: Language, path: Option<&str>) -> Result<StateMachine> {
    let source = match path {
        Some(p) => AnalysisSource::CodeFile { path: p.to_string(), language: lang },
        None => AnalysisSource::CodeSnippet { content: content.chars().take(200).collect(), language: lang },
    };
    if matches!(lang, Language::TypeScript | Language::Generic) {
        if let Some((decl, flow, finals)) = flow::xstate(content) {
            return Ok(assemble(&decl, flow, &finals, source));
        }
    }
    if lang == Language::Generic {
        return generic::GenericParser.parse(content, path);
    }
    let Some(decl) = flow::declaration(lang, content) else {
        bail!("No state type found: expected an enum, sealed class, union type or const block with at least two states");
    };
    let flow = flow::transitions(lang, content, &decl);
    Ok(assemble(&decl, flow, &Default::default(), source))
}

/// Kinds come from the graph, not from the names: a state nothing leaves is
/// final, whatever it is called ("pending" used to be final because it
/// contains "end").
pub(crate) fn assemble(decl: &flow::Decl, flow: flow::Flow, finals: &std::collections::BTreeSet<String>, source: AnalysisSource) -> StateMachine {
    let name = if decl.type_name.to_lowercase().contains("machine") { decl.type_name.clone() } else { format!("{}Machine", decl.type_name) };
    let mut sm = StateMachine::new(name, source);
    let initial = flow.initial.clone().filter(|i| decl.states.contains(i)).unwrap_or_else(|| decl.states[0].clone());
    let mut ids = std::collections::HashMap::new();
    for name in &decl.states {
        let leaves = flow.edges.iter().any(|e| &e.from == name);
        let entered = flow.edges.iter().any(|e| &e.to == name);
        let kind = if *name == initial {
            StateKind::Initial
        } else if helpers::is_error_name(name) {
            StateKind::Error
        } else if finals.contains(name) || (entered && !leaves) {
            StateKind::Final
        } else {
            StateKind::Normal
        };
        let state = State::new(name, kind);
        ids.insert(name.clone(), state.id.clone());
        sm.add_state(state);
    }
    sm.initial_state = ids.get(&initial).cloned();
    for e in flow.edges {
        let (Some(from), Some(to)) = (ids.get(&e.from), ids.get(&e.to)) else { continue };
        let mut t = Transition::new(from.clone(), to.clone(), e.event.clone());
        t.kind = helpers::transition_kind_from_names(&e.from, &e.to, e.event.as_deref());
        sm.add_transition(t);
    }
    sm
}

/// Shared helpers for all parsers
pub(crate) mod helpers {
    use crate::models::{StateKind, TransitionKind};

    /// `PaymentFailed` → ["payment", "failed"]; `order_done` → ["order", "done"].
    fn words(name: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut current = String::new();
        for c in name.chars() {
            if c == '_' || c == '-' || c == ' ' {
                if !current.is_empty() { out.push(std::mem::take(&mut current)); }
            } else if c.is_uppercase() && current.chars().last().is_some_and(|p| p.is_lowercase()) {
                out.push(std::mem::take(&mut current));
                current.extend(c.to_lowercase());
            } else {
                current.extend(c.to_lowercase());
            }
        }
        if !current.is_empty() { out.push(current); }
        out
    }

    fn has_word(name: &str, list: &[&str]) -> bool {
        words(name).iter().any(|w| list.contains(&w.as_str()))
    }

    pub fn is_error_name(name: &str) -> bool {
        has_word(name, &["error", "failed", "failure", "fail", "fatal", "rejected", "crashed"])
    }

    /// For sources without code structure (logs, AI answers): whole words only,
    /// so "pending" is not final because it contains "end".
    pub fn state_kind_from_name(name: &str) -> StateKind {
        if is_error_name(name) {
            StateKind::Error
        } else if has_word(name, &["init", "initial", "start", "idle", "new", "created"]) {
            StateKind::Initial
        } else if has_word(name, &["done", "success", "succeeded", "complete", "completed", "finished", "end", "closed"]) {
            StateKind::Final
        } else {
            StateKind::Normal
        }
    }

    pub fn transition_kind_from_names(_from: &str, to: &str, event: Option<&str>) -> TransitionKind {
        let event_lower = event.map(|e| e.to_lowercase()).unwrap_or_default();
        if is_error_name(to) {
            TransitionKind::Error
        } else if event_lower.contains("timeout") || event_lower.contains("expire") {
            TransitionKind::Timeout
        } else if event.is_none() {
            TransitionKind::Auto
        } else {
            TransitionKind::Normal
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn kinds_match_whole_words() {
            assert_eq!(state_kind_from_name("pending"), StateKind::Normal);
            assert_eq!(state_kind_from_name("PaymentFailed"), StateKind::Error);
            assert_eq!(state_kind_from_name("order_done"), StateKind::Final);
            assert_eq!(state_kind_from_name("Idle"), StateKind::Initial);
        }
    }
}
