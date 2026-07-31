pub mod claude;
pub mod ollama;
pub mod prompts;

use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use crate::models::{StateMachine, State, Transition, StateKind, AnalysisSource};

#[async_trait]
pub trait AiAnalyzer: Send + Sync {
    fn provider_name(&self) -> &str;
    async fn enhance(&self, sm: &mut StateMachine) -> Result<()>;
    async fn extract_from_description(&self, description: &str) -> Result<StateMachine>;
    async fn is_available(&self) -> bool;
}

/// Schneidet das erste vollstaendige JSON-Objekt aus einer Modellantwort.
///
/// Beide Backends bekommen Text zurueck, in dem das JSON von Prosa umgeben
/// sein kann, auch wenn der Prompt darum bittet, das zu lassen.
pub(crate) fn json_from_text(text: &str) -> Result<Value> {
    let start = text.find('{').ok_or_else(|| anyhow!("No JSON in model response"))?;
    let end = text.rfind('}').ok_or_else(|| anyhow!("No JSON end in model response"))?;
    if end < start {
        return Err(anyhow!("Malformed JSON braces in model response"));
    }
    Ok(serde_json::from_str(&text[start..=end])?)
}

/// Traegt Zusammenfassung, Zustandsbeschreibungen und Fehlerzustaende ein.
///
/// Liegt hier statt in den Backends, damit beide Anbieter dieselbe
/// Antwortstruktur gleich auswerten. Solange es nur einen Anbieter gab, stand
/// diese Zuordnung in dessen Datei.
pub(crate) fn apply_enhancement(sm: &mut StateMachine, j: &Value) {
    sm.ai_summary = j["summary"].as_str().map(str::to_string);

    if let Some(descs) = j["state_descriptions"].as_object() {
        for state in &mut sm.states {
            if let Some(desc) = descs.get(&state.name).and_then(|d| d.as_str()) {
                state.description = Some(desc.to_string());
            }
        }
    }

    if let Some(error_paths) = j["error_paths"].as_array() {
        for ep in error_paths {
            if let Some(name) = ep.as_str() {
                if let Some(s) = sm.states.iter_mut().find(|s| s.name == name) {
                    s.kind = StateKind::Error;
                }
            }
        }
    }
}

/// Baut eine Zustandsmaschine aus der JSON-Antwort eines Modells.
pub(crate) fn state_machine_from_json(j: &Value) -> StateMachine {
    let name = j["name"].as_str().unwrap_or("DescriptionMachine");
    let mut sm = StateMachine::new(name, AnalysisSource::Manual);

    let mut state_id_map = std::collections::HashMap::new();

    if let Some(states) = j["states"].as_array() {
        for sv in states {
            let sname = sv["name"].as_str().unwrap_or("Unknown");
            let kind = match sv["kind"].as_str().unwrap_or("normal") {
                "initial" => StateKind::Initial,
                "final"   => StateKind::Final,
                "error"   => StateKind::Error,
                _         => crate::parser::helpers::state_kind_from_name(sname),
            };
            let mut s = State::new(sname, kind);
            s.description = sv["description"].as_str().map(str::to_string);
            state_id_map.insert(sname.to_string(), s.id.clone());
            sm.add_state(s);
        }
    }

    if let Some(transitions) = j["transitions"].as_array() {
        for tv in transitions {
            let from = tv["from"].as_str().unwrap_or("");
            let to   = tv["to"].as_str().unwrap_or("");
            let event = tv["event"].as_str().map(str::to_string);

            if let (Some(from_id), Some(to_id)) = (state_id_map.get(from), state_id_map.get(to)) {
                let mut t = Transition::new(from_id, to_id, event);
                t.guard = tv["guard"].as_str().map(str::to_string);
                t.actions = tv["actions"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();
                sm.add_transition(t);
            }
        }
    }

    sm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_survives_prose_around_it() {
        let text = "Sure, here is the result:\n{\"summary\": \"ok\"}\nHope that helps.";
        let j = json_from_text(text).unwrap();
        assert_eq!(j["summary"], "ok");
    }

    #[test]
    fn a_response_without_json_is_an_error_not_a_panic() {
        assert!(json_from_text("I could not do that.").is_err());
    }

    #[test]
    fn a_closing_brace_before_the_opening_one_is_rejected() {
        // find und rfind koennen sich kreuzen, wenn ein Modell etwas wie
        // "} siehe oben {" liefert. Ohne diese Pruefung waere das ein Panic
        // beim Slicing statt eines Fehlers.
        assert!(json_from_text("} nonsense {").is_err());
    }

    #[test]
    fn error_paths_mark_the_named_state_and_leave_the_others() {
        let mut sm = StateMachine::new("M", AnalysisSource::Manual);
        sm.add_state(State::new("Failed", StateKind::Normal));
        sm.add_state(State::new("Done", StateKind::Normal));

        let j: Value = serde_json::from_str(r#"{"summary":"s","error_paths":["Failed"]}"#).unwrap();
        apply_enhancement(&mut sm, &j);

        assert_eq!(sm.ai_summary.as_deref(), Some("s"));
        assert!(matches!(
            sm.states.iter().find(|s| s.name == "Failed").unwrap().kind,
            StateKind::Error
        ));
        assert!(!matches!(
            sm.states.iter().find(|s| s.name == "Done").unwrap().kind,
            StateKind::Error
        ));
    }

    #[test]
    fn a_transition_to_an_unknown_state_is_dropped_rather_than_dangling() {
        let j: Value = serde_json::from_str(
            r#"{"name":"M","states":[{"name":"A","kind":"initial"}],
                "transitions":[{"from":"A","to":"Nowhere","event":"go"}]}"#,
        ).unwrap();
        let sm = state_machine_from_json(&j);
        assert_eq!(sm.states.len(), 1);
        assert_eq!(sm.transitions.len(), 0);
    }
}
