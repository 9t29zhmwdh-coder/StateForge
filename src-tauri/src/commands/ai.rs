use tauri::State;
use sf_core::models::StateMachine;
use sf_core::ai::{AiAnalyzer, claude::ClaudeAnalyzer, ollama::OllamaAnalyzer};
use crate::state::AppState;
use crate::error::{Result, SfError};

/// Waehlt das Backend nach der Einstellung des Nutzers.
///
/// Bis 1.0.12 stand hier ausschliesslich der Claude-Client. Die
/// Ollama-Auswahl in den Einstellungen existierte, wurde aber nie gelesen,
/// also ging die Zustandsmaschine auch dann an Anthropic, wenn "lokal"
/// eingestellt war.
async fn get_analyzer(state: &AppState) -> Result<Box<dyn AiAnalyzer>> {
    let settings = state.settings.read().await.clone();

    if settings.ai_backend == "ollama" {
        return Ok(Box::new(OllamaAnalyzer::new(
            settings.ollama_url,
            settings.ollama_model,
        )));
    }

    let key = keyring::Entry::new("stateforge", "claude_api_key")?
        .get_password()?;
    if key.is_empty() {
        return Err(SfError::Other(
            "No Claude API key configured. Set one in settings, or switch the AI backend to Ollama.".to_string(),
        ));
    }
    Ok(Box::new(ClaudeAnalyzer::new(key)))
}

#[tauri::command]
pub async fn ai_enhance_machine(
    machine_id: String,
    state: State<'_, AppState>,
) -> Result<StateMachine> {
    let mut sm = sf_core::db::queries::get_machine(&state.pool, &machine_id).await?
        .ok_or_else(|| SfError::Other("Not found".to_string()))?;

    let analyzer = get_analyzer(&state).await?;
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
    let analyzer = get_analyzer(&state).await?;
    let sm = analyzer.extract_from_description(&description).await?;
    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

#[tauri::command]
pub async fn check_ai_available(state: State<'_, AppState>) -> Result<bool> {
    match get_analyzer(&state).await {
        Ok(a) => Ok(a.is_available().await),
        Err(_) => Ok(false),
    }
}
