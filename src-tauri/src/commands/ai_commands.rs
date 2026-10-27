use crate::error::AppError;
use crate::services::ai_service::{self, AiMessage, AiModel};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn ai_fetch_models(api_key: String) -> Result<Vec<AiModel>, AppError> {
    ai_service::fetch_models(&api_key).await
}

#[tauri::command]
pub async fn ai_chat(app: tauri::AppHandle, messages: Vec<AiMessage>) -> Result<(), AppError> {
    ai_service::chat_stream(&app, messages).await
}

#[tauri::command(async)]
pub fn ai_execute_tool(
    state: State<'_, AppState>,
    tool_name: String,
    arguments: String,
) -> Result<String, AppError> {
    ai_service::execute_tool(state, &tool_name, &arguments)
}
