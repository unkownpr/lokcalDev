use crate::error::AppError;
use crate::services::ai_service::{self, AiMessage, AiModel};

#[tauri::command]
pub async fn ai_fetch_models(api_key: String) -> Result<Vec<AiModel>, AppError> {
    ai_service::fetch_models(&api_key).await
}

#[tauri::command]
pub async fn ai_chat(app: tauri::AppHandle, messages: Vec<AiMessage>) -> Result<(), AppError> {
    ai_service::chat_stream(&app, messages).await
}

#[tauri::command]
pub fn ai_execute_tool(tool_name: String, arguments: String) -> Result<String, AppError> {
    ai_service::execute_tool(&tool_name, &arguments)
}
