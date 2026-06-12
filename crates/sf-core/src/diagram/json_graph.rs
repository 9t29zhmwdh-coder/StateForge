use crate::models::{StateMachine, StateKind, TransitionKind};
use crate::models::diagram::{DiagramGraph, DiagramNode, DiagramEdge, NodeKind, EdgeKind, NodePosition, DiagramConfig};
use super::layout::compute_layout;

pub fn to_graph(sm: &StateMachine, _config: &DiagramConfig) -> DiagramGraph {
    let positions = compute_layout(sm);
    let mut graph = DiagramGraph::new();

    for state in &sm.states {
        let kind = match state.kind {
            StateKind::Initial  => NodeKind::Initial,
            StateKind::Final    => NodeKind::Final,
            StateKind::Error    => NodeKind::Error,
            StateKind::Parallel => NodeKind::Parallel,
            _ => NodeKind::Normal,
        };
        let mut node = DiagramNode::new(&state.id, &state.name, kind);
        if let Some(&(x, y)) = positions.get(&state.id) {
            node.position = Some(NodePosition { x, y });
            node.width = Some(120.0);
            node.height = Some(44.0);
        }
        node.data.insert("description".to_string(),
            serde_json::Value::String(state.description.clone().unwrap_or_default()));
        node.data.insert("entry_actions".to_string(),
            serde_json::to_value(&state.entry_actions).unwrap_or_default());
        node.data.insert("exit_actions".to_string(),
            serde_json::to_value(&state.exit_actions).unwrap_or_default());
        graph.nodes.push(node);
    }

    for t in &sm.transitions {
        let kind = match t.kind {
            TransitionKind::Error    => EdgeKind::Error,
            TransitionKind::Auto     => EdgeKind::Auto,
            TransitionKind::Timeout  => EdgeKind::Timeout,
            TransitionKind::Internal => EdgeKind::Internal,
            TransitionKind::Normal   => EdgeKind::Normal,
        };
        let mut edge = DiagramEdge::new(&t.id, &t.from_state, &t.to_state);
        edge.kind = kind;
        edge.label = t.event.clone();
        if let Some(ref guard) = t.guard {
            edge.data.insert("guard".to_string(), serde_json::Value::String(guard.clone()));
        }
        if !t.actions.is_empty() {
            edge.data.insert("actions".to_string(), serde_json::to_value(&t.actions).unwrap_or_default());
        }
        graph.edges.push(edge);
    }

    graph
}
