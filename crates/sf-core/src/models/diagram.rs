use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiagramFormat {
    MermaidState,
    MermaidSequence,
    MermaidFlowchart,
    GraphvizDot,
    Svg,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutDirection {
    TopDown,
    LeftRight,
    BottomUp,
    RightLeft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramConfig {
    pub format: DiagramFormat,
    pub direction: LayoutDirection,
    pub include_guards: bool,
    pub include_actions: bool,
    pub include_entry_exit: bool,
    pub highlight_error_paths: bool,
    pub compact: bool,
}

impl Default for DiagramConfig {
    fn default() -> Self {
        Self {
            format: DiagramFormat::MermaidState,
            direction: LayoutDirection::TopDown,
            include_guards: true,
            include_actions: true,
            include_entry_exit: false,
            highlight_error_paths: true,
            compact: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Initial,
    Normal,
    Final,
    Error,
    Parallel,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramNode {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub position: Option<NodePosition>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub data: HashMap<String, serde_json::Value>,
}

impl DiagramNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            position: None,
            width: None,
            height: None,
            data: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Normal,
    Error,
    Auto,
    Timeout,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub kind: EdgeKind,
    pub data: HashMap<String, serde_json::Value>,
}

impl DiagramEdge {
    pub fn new(id: impl Into<String>, source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            label: None,
            kind: EdgeKind::Normal,
            data: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramGraph {
    pub nodes: Vec<DiagramNode>,
    pub edges: Vec<DiagramEdge>,
    pub viewport: Option<Viewport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

impl DiagramGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new(), viewport: None }
    }
}

impl Default for DiagramGraph {
    fn default() -> Self { Self::new() }
}
