use tauri::State;
use sf_core::models::StateMachine;
use sf_core::models::diagram::{DiagramConfig, DiagramFormat, DiagramGraph, LayoutDirection};
use crate::state::AppState;
use crate::error::Result;

#[tauri::command]
pub async fn render_diagram(
    machine_id: String,
    format: String,
    direction: Option<String>,
    include_guards: Option<bool>,
    include_actions: Option<bool>,
    highlight_errors: Option<bool>,
    state: State<'_, AppState>,
) -> Result<String> {
    let sm = sf_core::db::queries::get_machine(&state.pool, &machine_id).await?
        .ok_or_else(|| crate::error::SfError::Other(format!("Machine not found: {}", machine_id)))?;

    let config = DiagramConfig {
        format: format_from_str(&format),
        direction: dir_from_str(direction.as_deref()),
        include_guards: include_guards.unwrap_or(true),
        include_actions: include_actions.unwrap_or(true),
        highlight_error_paths: highlight_errors.unwrap_or(true),
        ..DiagramConfig::default()
    };

    Ok(sf_core::diagram::render(&sm, &config)?)
}

#[tauri::command]
pub async fn render_diagram_from_machine(
    machine: StateMachine,
    format: String,
) -> Result<String> {
    let config = DiagramConfig {
        format: format_from_str(&format),
        ..DiagramConfig::default()
    };
    Ok(sf_core::diagram::render(&machine, &config)?)
}

#[tauri::command]
pub async fn get_diagram_graph(
    machine_id: String,
    state: State<'_, AppState>,
) -> Result<DiagramGraph> {
    let sm = sf_core::db::queries::get_machine(&state.pool, &machine_id).await?
        .ok_or_else(|| crate::error::SfError::Other("Not found".to_string()))?;

    let config = DiagramConfig::default();
    Ok(sf_core::diagram::to_graph(&sm, &config))
}

#[tauri::command]
pub async fn update_machine_from_graph(
    machine_id: String,
    graph: DiagramGraph,
    state: State<'_, AppState>,
) -> Result<StateMachine> {
    use sf_core::models::{State as MState, Transition, StateKind, TransitionKind};
    use sf_core::models::diagram::NodeKind;

    let mut sm = sf_core::db::queries::get_machine(&state.pool, &machine_id).await?
        .ok_or_else(|| crate::error::SfError::Other("Not found".to_string()))?;

    // Sync node positions and labels back to the state machine
    for node in &graph.nodes {
        if let Some(s) = sm.states.iter_mut().find(|s| s.id == node.id) {
            s.name = node.label.clone();
        }
    }

    sm.updated_at = chrono::Utc::now();
    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

fn format_from_str(s: &str) -> DiagramFormat {
    match s {
        "graphviz" | "dot" => DiagramFormat::GraphvizDot,
        "svg"              => DiagramFormat::Svg,
        "json"             => DiagramFormat::Json,
        "sequence"         => DiagramFormat::MermaidSequence,
        "flowchart"        => DiagramFormat::MermaidFlowchart,
        _                  => DiagramFormat::MermaidState,
    }
}

fn dir_from_str(s: Option<&str>) -> LayoutDirection {
    match s.unwrap_or("td") {
        "lr" | "left-right" => LayoutDirection::LeftRight,
        "bu" | "bottom-up"  => LayoutDirection::BottomUp,
        "rl" | "right-left" => LayoutDirection::RightLeft,
        _                   => LayoutDirection::TopDown,
    }
}
