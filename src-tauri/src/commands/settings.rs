use tauri::State;
use crate::state::{AppState, AppSettings};
use crate::error::Result;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings> {
    Ok(state.settings.read().await.clone())
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<()> {
    let json = serde_json::to_string(&settings)
        .map_err(|e| crate::error::SfError::Other(e.to_string()))?;
    sf_core::db::queries::set_setting(&state.pool, "app_settings", &json).await?;
    *state.settings.write().await = settings;
    Ok(())
}

#[tauri::command]
pub async fn save_api_key(key: String) -> Result<()> {
    keyring::Entry::new("stateforge", "claude_api_key")?.set_password(&key)?;
    Ok(())
}

#[tauri::command]
pub async fn has_api_key() -> bool {
    keyring::Entry::new("stateforge", "claude_api_key")
        .ok().and_then(|e| e.get_password().ok())
        .map(|k| !k.is_empty())
        .unwrap_or(false)
}
