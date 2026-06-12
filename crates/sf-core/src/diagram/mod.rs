pub mod mermaid;
pub mod graphviz;
pub mod svg;
pub mod layout;
pub mod json_graph;

use crate::models::{StateMachine, diagram::{DiagramConfig, DiagramFormat, DiagramGraph}};
use anyhow::Result;

pub fn render(sm: &StateMachine, config: &DiagramConfig) -> Result<String> {
    match config.format {
        DiagramFormat::MermaidState     => mermaid::render_state(sm, config),
        DiagramFormat::MermaidSequence  => mermaid::render_sequence(sm, config),
        DiagramFormat::MermaidFlowchart => mermaid::render_flowchart(sm, config),
        DiagramFormat::GraphvizDot      => graphviz::render(sm, config),
        DiagramFormat::Svg              => svg::render(sm, config),
        DiagramFormat::Json             => {
            let g = json_graph::to_graph(sm, config);
            Ok(serde_json::to_string_pretty(&g)?)
        }
    }
}

pub fn to_graph(sm: &StateMachine, config: &DiagramConfig) -> DiagramGraph {
    json_graph::to_graph(sm, config)
}
