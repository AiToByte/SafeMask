use crate::common::state::AppState;
use crate::common::errors::AppResult;
use tauri::State;

#[tauri::command]
pub async fn mask_text(
    state: State<'_, AppState>,
    text: String
) -> AppResult<String> {
    let engine = state.engine.read();
    let result = engine.mask_line(text.as_bytes());
    Ok(String::from_utf8_lossy(&result).to_string())
}