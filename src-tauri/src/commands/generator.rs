use tauri::State;
use sf_core::models::Language;
use crate::state::AppState;
use crate::error::Result;

#[tauri::command]
pub async fn generate_code(
    machine_id: String,
    language: String,
    state: State<'_, AppState>,
) -> Result<String> {
    let sm = sf_core::db::queries::get_machine(&state.pool, &machine_id).await?
        .ok_or_else(|| crate::error::SfError::Other("Not found".to_string()))?;

    let lang = match language.as_str() {
        "swift"      => Language::Swift,
        "kotlin"     => Language::Kotlin,
        "go"         => Language::Go,
        _            => Language::TypeScript,
    };

    Ok(sf_core::generator::generate(&sm, &lang)?)
}

#[tauri::command]
pub fn supported_languages() -> Vec<String> {
    vec!["swift".to_string(), "kotlin".to_string(), "typescript".to_string(), "go".to_string()]
}
