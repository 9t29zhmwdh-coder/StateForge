pub mod sequence;
pub mod grouper;
pub mod error_paths;

pub use sequence::LogSequenceExtractor;
pub use grouper::FlowGrouper;
pub use error_paths::ErrorPathDetector;

use crate::models::{StateMachine, State, Transition, StateKind, TransitionKind, AnalysisSource};
use anyhow::Result;

pub struct LogAnalyzer;

impl LogAnalyzer {
    pub fn analyze(content: &str, source_path: Option<&str>) -> Result<StateMachine> {
        let source = match source_path {
            Some(p) => AnalysisSource::LogFile { path: p.to_string() },
            None => AnalysisSource::LogContent { content: content.chars().take(200).collect() },
        };
        let mut sm = StateMachine::new("LogFlowMachine", source);

        let extractor = LogSequenceExtractor::new();
        let sequences = extractor.extract(content);

        if sequences.is_empty() {
            return Ok(sm);
        }

        let grouper = FlowGrouper::new();
        let groups = grouper.group(&sequences);

        // Build state machine from flow groups
        let mut state_index: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        let get_or_create = |sm: &mut StateMachine, state_index: &mut std::collections::HashMap<String, String>, name: &str| -> String {
            if let Some(id) = state_index.get(name) {
                return id.clone();
            }
            let kind = crate::parser::helpers::state_kind_from_name(name);
            let s = State::new(name, kind);
            let id = s.id.clone();
            state_index.insert(name.to_string(), id.clone());
            sm.add_state(s);
            id
        };

        for group in &groups {
            for seq in &group.sequences {
                let events = &seq.events;
                for i in 0..events.len().saturating_sub(1) {
                    let from_name = events[i].from_state.as_deref()
                        .or(events[i].to_state.as_deref())
                        .unwrap_or(&events[i].name);
                    let to_name = events[i + 1].to_state.as_deref()
                        .or(events[i + 1].from_state.as_deref())
                        .unwrap_or(&events[i + 1].name);

                    let from_id = get_or_create(&mut sm, &mut state_index, from_name);
                    let to_id = get_or_create(&mut sm, &mut state_index, to_name);

                    let event_name = events[i].name.clone();
                    let already = sm.transitions.iter().any(|t| {
                        t.from_state == from_id && t.to_state == to_id
                            && t.event.as_deref() == Some(&event_name)
                    });

                    if !already {
                        let mut t = Transition::new(from_id, to_id, Some(event_name.clone()));
                        if seq.is_error_path {
                            t.kind = TransitionKind::Error;
                        }
                        sm.add_transition(t);
                    }
                }
            }
        }

        // Mark error paths
        let error_detector = ErrorPathDetector::new();
        let error_state_names = error_detector.detect(content);
        for s in sm.states.iter_mut() {
            if error_state_names.contains(&s.name) {
                s.kind = StateKind::Error;
            }
        }

        Ok(sm)
    }
}
