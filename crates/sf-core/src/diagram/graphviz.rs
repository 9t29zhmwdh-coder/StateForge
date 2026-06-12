use crate::models::{StateMachine, StateKind, TransitionKind};
use crate::models::diagram::{DiagramConfig, LayoutDirection};
use anyhow::Result;

pub fn render(sm: &StateMachine, config: &DiagramConfig) -> Result<String> {
    let rankdir = match config.direction {
        LayoutDirection::LeftRight => "LR",
        LayoutDirection::TopDown   => "TB",
        LayoutDirection::BottomUp  => "BT",
        LayoutDirection::RightLeft => "RL",
    };

    let mut out = format!(
        "digraph {} {{\n    rankdir={};\n    node [fontname=\"Helvetica\",fontsize=12];\n    edge [fontname=\"Helvetica\",fontsize=10];\n\n",
        dot_id(&sm.name), rankdir
    );

    // Initial pseudo-state
    if sm.initial_state.is_some() {
        out.push_str("    __start__ [shape=point,width=0.2,style=filled,fillcolor=black];\n");
    }

    // Nodes
    for state in &sm.states {
        let id = dot_id(&state.name);
        let (shape, style, fillcolor) = match state.kind {
            StateKind::Initial  => ("circle", "filled", "#4A90D9"),
            StateKind::Final    => ("doublecircle", "filled", "#27AE60"),
            StateKind::Error    => ("rectangle", "filled", "#E74C3C"),
            StateKind::Parallel => ("rectangle", "dashed", "#F39C12"),
            StateKind::History  => ("diamond", "filled", "#9B59B6"),
            StateKind::Normal   => ("rectangle", "filled", "#ECF0F1"),
        };

        let label = match state.kind {
            StateKind::Error => format!("⚠ {}", state.name),
            _ => state.name.clone(),
        };

        out.push_str(&format!(
            "    {} [label=\"{}\",shape={},style={},fillcolor=\"{}\",color=\"#2C3E50\"];\n",
            id, label, shape, style, fillcolor
        ));
    }

    // Initial edge
    if let Some(ref init_id) = sm.initial_state {
        if let Some(init) = sm.state_by_id(init_id) {
            out.push_str(&format!("    __start__ -> {};\n", dot_id(&init.name)));
        }
    }

    out.push('\n');

    // Edges
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
            label_parts.push(format!("/{}", t.actions.join(",")));
        }

        let label = label_parts.join("\\n");
        let (color, style) = if config.highlight_error_paths && t.kind == TransitionKind::Error {
            ("#E74C3C", "dashed")
        } else {
            ("#2C3E50", "solid")
        };

        out.push_str(&format!(
            "    {} -> {} [label=\"{}\",color=\"{}\",style={}];\n",
            dot_id(&from.name), dot_id(&to.name), label, color, style
        ));
    }

    out.push_str("}\n");
    Ok(out)
}

fn dot_id(name: &str) -> String {
    let sanitized: String = name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    // Ensure doesn't start with digit
    if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("s_{}", sanitized)
    } else {
        sanitized
    }
}
