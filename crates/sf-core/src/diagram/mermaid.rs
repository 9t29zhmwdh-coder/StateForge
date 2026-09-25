use crate::models::{StateMachine, StateKind, TransitionKind};
use crate::models::diagram::{DiagramConfig, LayoutDirection};
use anyhow::Result;

pub fn render_state(sm: &StateMachine, config: &DiagramConfig) -> Result<String> {
    let mut out = String::from("stateDiagram-v2\n");

    let dir = match config.direction {
        LayoutDirection::LeftRight => "    direction LR\n",
        LayoutDirection::TopDown   => "    direction TB\n",
        _ => "    direction TB\n",
    };
    out.push_str(dir);

    // Initial state pointer
    if let Some(ref init_id) = sm.initial_state {
        if let Some(init) = sm.state_by_id(init_id) {
            out.push_str(&format!("    [*] --> {}\n", sanitize(&init.name)));
        }
    }

    // States
    for state in &sm.states {
        let sname = sanitize(&state.name);

        match state.kind {
            StateKind::Final => {
                if sname != state.name {
                    out.push_str(&format!("    state \"{}\" as {}\n", state.name.replace('"', "'"), sname));
                }
                out.push_str(&format!("    {} --> [*]\n", sname));
            }
            StateKind::Error => {
                out.push_str(&format!("    state \"⚠ {}\" as {}\n", state.name, sname));
            }
            StateKind::Parallel => {
                out.push_str(&format!("    state {} {{\n    }}\n", sname));
            }
            _ => {
                if let Some(ref desc) = state.description {
                    out.push_str(&format!("    state \"{}\" as {}\n", desc, sname));
                } else if sname != state.name {
                    // "POST /orders/{id}/pay" keeps its readable name as the label.
                    out.push_str(&format!("    state \"{}\" as {}\n", state.name.replace('"', "'"), sname));
                }
            }
        }

        // Entry/exit actions
        if config.include_entry_exit && !state.entry_actions.is_empty() {
            out.push_str(&format!("    note right of {}\n        entry: {}\n    end note\n",
                sname, state.entry_actions.join(", ")));
        }
    }

    // Transitions
    for t in &sm.transitions {
        let Some(from) = sm.state_by_id(&t.from_state) else { continue };
        let Some(to)   = sm.state_by_id(&t.to_state)   else { continue };

        let mut label_parts = Vec::new();
        if let Some(ref event) = t.event {
            label_parts.push(event.clone());
        }
        if config.include_guards {
            if let Some(ref guard) = t.guard {
                label_parts.push(format!("[{}]", guard));
            }
        }
        if config.include_actions && !t.actions.is_empty() {
            label_parts.push(format!("/{}", t.actions.join(", ")));
        }

        // `-->>` is not state-diagram syntax: Mermaid drew a phantom ">" node and
        // put the label into the target box. Error states are coloured below instead.
        let arrow = "-->";

        if label_parts.is_empty() {
            out.push_str(&format!("    {} {} {}\n",
                sanitize(&from.name), arrow, sanitize(&to.name)));
        } else {
            out.push_str(&format!("    {} {} {} : {}\n",
                sanitize(&from.name), arrow, sanitize(&to.name),
                label_parts.join(" ")));
        }
    }

    // Styling for error states
    if config.highlight_error_paths {
        let error_states: Vec<_> = sm.states.iter()
            .filter(|s| s.kind == StateKind::Error)
            .collect();
        if !error_states.is_empty() {
            out.push_str("\n    classDef errorState fill:#ff4444,color:#fff,stroke:#cc0000\n");
            for s in &error_states {
                out.push_str(&format!("    class {} errorState\n", sanitize(&s.name)));
            }
        }
    }

    Ok(out)
}

pub fn render_sequence(sm: &StateMachine, _config: &DiagramConfig) -> Result<String> {
    let mut out = String::from("sequenceDiagram\n");

    // Group transitions by service/participant (insertion-order dedup)
    let mut _seen = std::collections::HashSet::new();
    let participants: Vec<String> = sm.transitions.iter()
        .flat_map(|t| {
            let from = sm.state_by_id(&t.from_state).map(|s| s.name.clone());
            let to = sm.state_by_id(&t.to_state).map(|s| s.name.clone());
            [from, to].into_iter().flatten()
        })
        .filter(|p| _seen.insert(p.clone()))
        .collect();

    for p in &participants {
        out.push_str(&format!("    participant {}\n", sanitize(p)));
    }

    for t in &sm.transitions {
        let Some(from) = sm.state_by_id(&t.from_state) else { continue };
        let Some(to)   = sm.state_by_id(&t.to_state)   else { continue };
        let label = t.event.as_deref().unwrap_or("→");
        let _arrow = "->>";
        out.push_str(&format!("    {}->>{}:{}\n", sanitize(&from.name), sanitize(&to.name), label));
    }

    Ok(out)
}

pub fn render_flowchart(sm: &StateMachine, config: &DiagramConfig) -> Result<String> {
    let dir = match config.direction {
        LayoutDirection::LeftRight => "LR",
        LayoutDirection::TopDown   => "TD",
        LayoutDirection::BottomUp  => "BU",
        LayoutDirection::RightLeft => "RL",
    };
    let mut out = format!("flowchart {}\n", dir);

    for state in &sm.states {
        let sname = sanitize(&state.name);
        let shape = match state.kind {
            StateKind::Initial  => format!("    {}([{}])\n", sname, state.name),
            StateKind::Final    => format!("    {}(([{}]))\n", sname, state.name),
            StateKind::Error    => format!("    {}{{⚠ {}}}\n", sname, state.name),
            StateKind::Parallel => format!("    {}[/{}\\]\n", sname, state.name),
            _                   => format!("    {}[{}]\n", sname, state.name),
        };
        out.push_str(&shape);
    }

    for t in &sm.transitions {
        let Some(from) = sm.state_by_id(&t.from_state) else { continue };
        let Some(to)   = sm.state_by_id(&t.to_state)   else { continue };
        let label = t.event.as_deref().unwrap_or("");
        let arrow = if t.kind == TransitionKind::Error { "-.->|⚠|" } else { "-->" };
        if label.is_empty() {
            out.push_str(&format!("    {} {} {}\n", sanitize(&from.name), arrow, sanitize(&to.name)));
        } else {
            out.push_str(&format!("    {} {}|{}| {}\n",
                sanitize(&from.name), "-->", label, sanitize(&to.name)));
        }
    }

    Ok(out)
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_transitions_use_valid_arrows_and_names_keep_labels() {
        let log = "order=1 state changed from pending to paid\norder=1 state changed from paid to payment_failed\n";
        let sm = crate::log_analyzer::LogAnalyzer::analyze(log, None).unwrap();
        let out = render_state(&sm, &DiagramConfig::default()).unwrap();
        assert!(!out.contains("-->>"), "{out}");
        let api = "1.2.3.4 - - [25/Sep/2026:08:00:01 +0000] \"POST /orders HTTP/1.1\" 201 5\n1.2.3.4 - - [25/Sep/2026:08:00:02 +0000] \"POST /orders/7/pay HTTP/1.1\" 200 5\n";
        let sm = crate::log_analyzer::LogAnalyzer::analyze(api, None).unwrap();
        let out = render_state(&sm, &DiagramConfig::default()).unwrap();
        assert!(out.contains("state \"POST /orders/{id}/pay\" as POST__orders__id__pay"), "{out}");
    }
}
