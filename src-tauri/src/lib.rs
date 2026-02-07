mod ollama;
mod redactor;
mod regex_engine;

#[tauri::command]
async fn redact_text(text: String) -> Result<String, String> {
    redactor::redact(&text).await
}

#[tauri::command]
async fn check_ollama() -> bool {
    ollama::check_connection().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![redact_text, check_ollama])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
