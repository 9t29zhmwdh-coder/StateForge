pub mod swift;
pub mod kotlin;
pub mod typescript;
pub mod go;
pub mod generic;

use crate::models::{StateMachine, Language, AnalysisSource};
use anyhow::Result;

pub trait CodeParser: Send + Sync {
    fn language(&self) -> Language;
    fn can_parse(&self, content: &str) -> bool;
    fn parse(&self, content: &str, source_path: Option<&str>) -> Result<StateMachine>;
}

pub fn parse_file(path: &str, content: &str) -> Result<StateMachine> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let lang = Language::from_extension(ext);

    let parser: Box<dyn CodeParser> = match lang {
        Language::Swift      => Box::new(swift::SwiftParser),
        Language::Kotlin     => Box::new(kotlin::KotlinParser),
        Language::TypeScript => Box::new(typescript::TypeScriptParser),
        Language::Go         => Box::new(go::GoParser),
        _                    => Box::new(generic::GenericParser),
    };

    parser.parse(content, Some(path))
}

pub fn parse_with_language(content: &str, lang: Language) -> Result<StateMachine> {
    let parser: Box<dyn CodeParser> = match lang {
        Language::Swift      => Box::new(swift::SwiftParser),
        Language::Kotlin     => Box::new(kotlin::KotlinParser),
        Language::TypeScript => Box::new(typescript::TypeScriptParser),
        Language::Go         => Box::new(go::GoParser),
        _                    => Box::new(generic::GenericParser),
    };
    parser.parse(content, None)
}

/// Shared helpers for all parsers
pub(crate) mod helpers {
    use crate::models::{State, StateKind, Transition, TransitionKind};

    pub fn state_kind_from_name(name: &str) -> StateKind {
        let lower = name.to_lowercase();
        if lower.contains("init") || lower.contains("start") || lower.contains("idle") && !lower.contains("error") {
            StateKind::Initial
        } else if lower.contains("error") || lower.contains("fail") || lower.contains("fatal") {
            StateKind::Error
        } else if lower.contains("done") || lower.contains("success") || lower.contains("complete") || lower.contains("finish") || lower.contains("end") {
            StateKind::Final
        } else {
            StateKind::Normal
        }
    }

    pub fn transition_kind_from_names(from: &str, to: &str, event: Option<&str>) -> TransitionKind {
        let to_lower = to.to_lowercase();
        let event_lower = event.map(|e| e.to_lowercase()).unwrap_or_default();

        if to_lower.contains("error") || to_lower.contains("fail") {
            TransitionKind::Error
        } else if event_lower.contains("timeout") || event_lower.contains("expire") {
            TransitionKind::Timeout
        } else if event.is_none() {
            TransitionKind::Auto
        } else {
            TransitionKind::Normal
        }
    }
}
