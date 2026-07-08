use tauri::State;
use sf_core::models::StateMachine;
use sf_core::ai::{AiAnalyzer, claude::ClaudeAnalyzer};
use crate::state::AppState;
use crate::error::{Result, SfError};

async fn get_analyzer() -> Result<ClaudeAnalyzer> {
    let key = keyring::Entry::new("stateforge", "claude_api_key")?
        .get_password()?;
    if key.is_empty() {
        return Err(SfError::Other("No API key configured".to_string()));
    }
    Ok(ClaudeAnalyzer::new(key))
}

#[tauri::command]
pub async fn ai_enhance_machine(
    machine_id: String,
    state: State<'_, AppState>,
) -> Result<StateMachine> {
    let mut sm = sf_core::db::queries::get_machine(&state.pool, &machine_id).await?
        .ok_or_else(|| SfError::Other("Not found".to_string()))?;

    let analyzer = get_analyzer().await?;
    analyzer.enhance(&mut sm).await?;
    sm.updated_at = chrono::Utc::now();

    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

#[tauri::command]
pub async fn ai_from_description(
    description: String,
    state: State<'_, AppState>,
) -> Result<StateMachine> {
    let analyzer = get_analyzer().await?;
    let sm = analyzer.extract_from_description(&description).await?;
    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

#[tauri::command]
pub async fn check_ai_available() -> Result<bool> {
    match get_analyzer().await {
        Ok(a) => Ok(a.is_available().await),
        Err(_) => Ok(false),
    }
}
