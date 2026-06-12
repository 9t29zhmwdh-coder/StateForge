use tauri::State;
use sf_core::models::{StateMachine, State as MState, Transition, StateKind, TransitionKind};
use crate::state::AppState;
use crate::error::Result;

#[tauri::command]
pub async fn list_machines(state: State<'_, AppState>) -> Result<Vec<StateMachine>> {
    Ok(sf_core::db::queries::list_machines(&state.pool).await?)
}

#[tauri::command]
pub async fn get_machine(machine_id: String, state: State<'_, AppState>) -> Result<Option<StateMachine>> {
    Ok(sf_core::db::queries::get_machine(&state.pool, &machine_id).await?)
}

#[tauri::command]
pub async fn save_machine(machine: StateMachine, state: State<'_, AppState>) -> Result<()> {
    Ok(sf_core::db::queries::insert_machine(&state.pool, &machine).await?)
}

#[tauri::command]
pub async fn delete_machine(machine_id: String, state: State<'_, AppState>) -> Result<()> {
    Ok(sf_core::db::queries::delete_machine(&state.pool, &machine_id).await?)
}

#[tauri::command]
pub async fn add_state(
    machine_id: String,
    name: String,
    kind: String,
    state: State<'_, AppState>,
) -> Result<StateMachine> {
    let mut sm = sf_core::db::queries::get_machine(&state.pool, &machine_id).await?
        .ok_or_else(|| crate::error::SfError::Other("Not found".to_string()))?;

    let sk = match kind.as_str() {
        "initial"  => StateKind::Initial,
        "final"    => StateKind::Final,
        "error"    => StateKind::Error,
        "parallel" => StateKind::Parallel,
        _          => StateKind::Normal,
    };
    let new_state = MState::new(&name, sk);
    sm.add_state(new_state);
    sm.updated_at = chrono::Utc::now();

    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

#[tauri::command]
pub async fn add_transition(
    machine_id: String,
    from_state: String,
    to_state: String,
    event: Option<String>,
    guard: Option<String>,
    state: State<'_, AppState>,
) -> Result<StateMachine> {
    let mut sm = sf_core::db::queries::get_machine(&state.pool, &machine_id).await?
        .ok_or_else(|| crate::error::SfError::Other("Not found".to_string()))?;

    let mut t = Transition::new(&from_state, &to_state, event);
    t.guard = guard;
    sm.add_transition(t);
    sm.updated_at = chrono::Utc::now();

    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

#[tauri::command]
pub async fn new_machine(name: String, state: State<'_, AppState>) -> Result<StateMachine> {
    use sf_core::models::AnalysisSource;
    let sm = StateMachine::new(&name, AnalysisSource::Manual);
    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

#[tauri::command]
pub fn analyze_machine(machine: StateMachine) -> serde_json::Value {
    let unreachable = machine.unreachable_states()
        .iter().map(|s| s.name.clone()).collect::<Vec<_>>();
    let deterministic = machine.is_deterministic();
    let error_states = machine.states.iter()
        .filter(|s| s.kind == StateKind::Error)
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();

    serde_json::json!({
        "state_count": machine.states.len(),
        "transition_count": machine.transitions.len(),
        "is_deterministic": deterministic,
        "unreachable_states": unreachable,
        "error_states": error_states,
        "has_initial_state": machine.initial_state.is_some(),
        "has_final_state": machine.states.iter().any(|s| s.kind == StateKind::Final),
    })
}
