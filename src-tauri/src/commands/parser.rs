use tauri::State;
use sf_core::models::{StateMachine, Language};
use crate::state::AppState;
use crate::error::Result;

#[tauri::command]
pub async fn parse_code(
    content: String,
    language: String,
    source_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<StateMachine> {
    let lang = lang_from_str(&language);
    let mut sm = match source_path {
        Some(ref path) => sf_core::parser::parse_file(path, &content)?,
        None => sf_core::parser::parse_with_language(&content, lang)?,
    };
    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

#[tauri::command]
pub async fn parse_log(
    content: String,
    source_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<StateMachine> {
    let sm = sf_core::log_analyzer::LogAnalyzer::analyze(
        &content, source_path.as_deref()
    )?;
    sf_core::db::queries::insert_machine(&state.pool, &sm).await?;
    Ok(sm)
}

#[tauri::command]
pub async fn detect_language(content: String) -> Result<String> {
    // Heuristics to suggest a language
    let c = &content;
    let lang = if c.contains("enum ") && c.contains("case ") && c.contains("func ") {
        "swift"
    } else if c.contains("sealed class") || c.contains("data class") {
        "kotlin"
    } else if c.contains("createMachine") || (c.contains("type ") && c.contains("| '")) {
        "typescript"
    } else if c.contains(" iota") || c.contains("func (") {
        "go"
    } else {
        "generic"
    };
    Ok(lang.to_string())
}

fn lang_from_str(s: &str) -> Language {
    match s {
        "swift"      => Language::Swift,
        "kotlin"     => Language::Kotlin,
        "typescript" | "ts" => Language::TypeScript,
        "go"         => Language::Go,
        "rust"       => Language::Rust,
        _            => Language::Generic,
    }
}
