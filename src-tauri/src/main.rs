mod ai;
mod auto_mode;
mod audio;
mod db;
mod documents;
mod screenshot;
mod screen_hiding;
mod settings;
mod transcription;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tauri::Manager;
use tauri::Emitter;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<db::Database>,
    pub current_session_id: Arc<RwLock<Option<String>>>,
    pub audio_capture: Arc<RwLock<audio::AudioCapture>>,
    pub stealth_mode: Arc<RwLock<bool>>,
    pub auto_mode: Arc<RwLock<auto_mode::AutoModeState>>,
    pub auto_mode_config: Arc<RwLock<auto_mode::AutoModeConfig>>,
    pub documents: Arc<RwLock<Vec<documents::Document>>>,
}

#[tauri::command]
async fn get_providers() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "openrouter-free",
            "name": "OpenRouter Free",
            "models": [
                "openrouter/hunter-alpha",
                "openrouter/healer-alpha",
                "stepfun/step-3.5-flash:free",
                "z-ai/glm-4.5-air:free",
                "qwen/qwen3-coder:free"
            ]
        }),
        serde_json::json!({
            "id": "openrouter",
            "name": "OpenRouter Pro",
            "models": [
                "openai/gpt-5.4",
                "openai/gpt-5.4-pro",
                "openai/gpt-5.3-chat",
                "openai/gpt-5.2-chat",
                "anthropic/claude-opus-4.6",
                "anthropic/claude-sonnet-4.6",
                "anthropic/claude-haiku-4.5",
                "google/gemini-3.1-pro-preview",
                "google/gemini-3.1-flash-lite-preview",
                "deepseek/deepseek-v3.2",
                "deepseek/deepseek-v3.2-speciale",
                "x-ai/grok-4.20-beta",
                "x-ai/grok-4.1-fast",
                "qwen/qwen3-max",
                "qwen/qwen3.5-397b-a17b",
                "moonshotai/kimi-k2.5",
                "mistralai/mistral-large-2512"
            ]
        }),
        serde_json::json!({
            "id": "openai",
            "name": "OpenAI",
            "models": [
                "gpt-5.4",
                "gpt-5.4-pro",
                "gpt-5.3-chat",
                "gpt-5.2-chat",
                "gpt-5.1",
                "o3",
                "o4-mini"
            ]
        }),
        serde_json::json!({
            "id": "google",
            "name": "Google AI",
            "models": [
                "gemini-3.1-pro-preview",
                "gemini-3.1-flash-lite-preview",
                "gemini-3-pro-preview",
                "gemini-2.5-flash",
                "gemini-2.5-pro"
            ]
        }),
        serde_json::json!({
            "id": "deepseek",
            "name": "DeepSeek",
            "models": [
                "deepseek-v3.2",
                "deepseek-v3.2-speciale",
                "deepseek-v3.1-terminus",
                "deepseek-chat-v3.1",
                "deepseek-r1"
            ]
        }),
        serde_json::json!({
            "id": "xai",
            "name": "xAI (Grok)",
            "models": [
                "grok-4.20-beta",
                "grok-4.20-multi-agent-beta",
                "grok-4.1-fast",
                "grok-4-fast"
            ]
        }),
    ]
}

#[tauri::command]
async fn ask_ai(
    state: tauri::State<'_, AppState>,
    provider: String,
    model: String,
    api_key: String,
    message: String,
    instructions: Option<String>,
    context_documents: Option<Vec<String>>,
) -> Result<String, String> {
    let request = ai::AiRequest {
        provider,
        model,
        api_key,
        message,
        instructions,
        context_documents: context_documents.unwrap_or_default(),
    };

    ai::ask(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_session(
    state: tauri::State<'_, AppState>,
    title: String,
    instructions: Option<String>,
) -> Result<String, String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    state
        .db
        .create_session(&session_id, &title, instructions.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let mut current = state.current_session_id.write().await;
    *current = Some(session_id.clone());

    Ok(session_id)
}

#[tauri::command]
async fn end_session(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut current = state.current_session_id.write().await;
    *current = None;

    // Stop auto mode if active
    let mut auto = state.auto_mode.write().await;
    auto.is_active = false;

    Ok(())
}

#[tauri::command]
async fn get_sessions(state: tauri::State<'_, AppState>) -> Result<Vec<db::Session>, String> {
    state
        .db
        .get_sessions()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_session_messages(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Vec<db::Message>, String> {
    state
        .db
        .get_session_messages(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_message(
    state: tauri::State<'_, AppState>,
    session_id: String,
    role: String,
    content: String,
) -> Result<(), String> {
    state
        .db
        .save_message(&session_id, &role, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_audio_devices() -> Result<Vec<String>, String> {
    audio::list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_audio_capture(
    state: tauri::State<'_, AppState>,
    capture_mic: bool,
    capture_system: bool,
) -> Result<String, String> {
    let mut capture = state.audio_capture.write().await;

    if capture.is_capturing() {
        return Err("Audio capture is already running".to_string());
    }

    if capture_mic {
        capture
            .start_microphone()
            .map_err(|e| e.to_string())?;
    }

    if capture_system {
        capture
            .start_system_audio()
            .map_err(|e| e.to_string())?;
    }

    Ok("Audio capture started".to_string())
}

#[tauri::command]
async fn stop_audio_capture(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut capture = state.audio_capture.write().await;
    capture.stop();
    Ok(())
}

#[tauri::command]
async fn get_audio_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let capture = state.audio_capture.read().await;
    Ok(serde_json::json!({
        "is_capturing": capture.is_capturing(),
        "buffer_duration": capture.buffer_duration(),
    }))
}

#[tauri::command]
async fn transcribe_audio(
    state: tauri::State<'_, AppState>,
    transcription_provider: String,
    api_key: String,
    language: Option<String>,
) -> Result<String, String> {
    let capture = state.audio_capture.read().await;
    let samples = capture.take_audio();

    if samples.is_empty() {
        return Err("No audio data available".to_string());
    }

    drop(capture);

    let wav = audio::samples_to_wav(&samples);
    let provider = transcription::TranscriptionProvider::from_str(&transcription_provider);

    transcription::transcribe(&provider, &api_key, wav, language.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn transcribe_and_ask(
    state: tauri::State<'_, AppState>,
    ai_provider: String,
    ai_model: String,
    ai_api_key: String,
    transcription_provider: String,
    transcription_api_key: String,
    instructions: Option<String>,
    language: Option<String>,
) -> Result<String, String> {
    let transcript = transcribe_audio(
        state.clone(),
        transcription_provider,
        transcription_api_key,
        language,
    )
    .await?;

    if transcript.trim().is_empty() {
        return Err("No speech detected".to_string());
    }

    ask_ai(
        state,
        ai_provider,
        ai_model,
        ai_api_key,
        transcript,
        instructions,
        None,
    )
    .await
}

#[tauri::command]
async fn enable_stealth_mode(
    state: tauri::State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        screen_hiding::hide_window_from_screen_capture(Some(hwnd.0 as isize))
            .map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        screen_hiding::hide_window_from_screen_capture(None)
            .map_err(|e| e.to_string())?;
    }

    let mut stealth = state.stealth_mode.write().await;
    *stealth = true;

    log::info!("Stealth mode enabled");
    Ok(())
}

#[tauri::command]
async fn disable_stealth_mode(
    state: tauri::State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        screen_hiding::show_window_in_screen_capture(Some(hwnd.0 as isize))
            .map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        screen_hiding::show_window_in_screen_capture(None)
            .map_err(|e| e.to_string())?;
    }

    let mut stealth = state.stealth_mode.write().await;
    *stealth = false;

    log::info!("Stealth mode disabled");
    Ok(())
}

#[tauri::command]
async fn get_stealth_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let stealth = state.stealth_mode.read().await;
    Ok(serde_json::json!({
        "enabled": *stealth,
        "supported": screen_hiding::is_screen_capture_hiding_supported(),
    }))
}

#[tauri::command]
fn is_screen_capture_hiding_supported() -> bool {
    screen_hiding::is_screen_capture_hiding_supported()
}

// Auto mode commands

#[tauri::command]
async fn start_auto_mode(
    state: tauri::State<'_, AppState>,
    window: tauri::Window,
    ai_provider: String,
    ai_model: String,
    ai_api_key: String,
    transcription_provider: String,
    transcription_api_key: String,
    instructions: Option<String>,
    language: Option<String>,
    respond_to_all: Option<bool>,
) -> Result<(), String> {
    // Update config
    {
        let mut config = state.auto_mode_config.write().await;
        if let Some(ra) = respond_to_all {
            config.respond_to_all = ra;
        }
    }

    // Update state
    {
        let mut auto = state.auto_mode.write().await;
        auto.is_active = true;
    }

    // Start audio capture
    {
        let mut capture = state.audio_capture.write().await;
        if !capture.is_capturing() {
            capture.start_microphone().map_err(|e| e.to_string())?;
        }
    }

    // Clone necessary data for the background task
    let state_clone = state.inner().clone();
    let window_clone = window.clone();

    // Spawn background task for auto mode
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        let mut last_transcript = String::new();

        loop {
            interval.tick().await;

            // Check if auto mode is still active
            let is_active = {
                let auto = state_clone.auto_mode.read().await;
                auto.is_active
            };

            if !is_active {
                log::info!("Auto mode stopped");
                break;
            }

            // Get config
            let config = {
                let c = state_clone.auto_mode_config.read().await;
                c.clone()
            };

            // Get audio duration
            let buffer_duration = {
                let capture = state_clone.audio_capture.read().await;
                capture.buffer_duration()
            };

            // Skip if not enough audio
            if buffer_duration < config.min_audio_duration {
                continue;
            }

            // Transcribe
            let samples = {
                let capture = state_clone.audio_capture.read().await;
                capture.take_audio()
            };

            if samples.is_empty() {
                continue;
            }

            let wav = audio::samples_to_wav(&samples);
            let t_provider = transcription::TranscriptionProvider::from_str(&transcription_provider);

            let transcript = match transcription::transcribe(
                &t_provider,
                &transcription_api_key,
                wav,
                language.as_deref(),
            )
            .await
            {
                Ok(t) => t,
                Err(e) => {
                    log::error!("Transcription error: {}", e);
                    continue;
                }
            };

            // Skip if same as last or empty
            if transcript.trim().is_empty() || transcript == last_transcript {
                continue;
            }

            last_transcript = transcript.clone();

            // Emit transcription event to frontend
            let _ = window_clone.emit("auto-transcription", &transcript);

            // Check if it's a question
            if !auto_mode::is_question(&transcript, &config) {
                continue;
            }

            // It's a question! Get AI response
            let _ = window_clone.emit("auto-question-detected", &transcript);

            let questions = auto_mode::extract_questions(&transcript);
            let question_text = questions.join(" ");

            let request = ai::AiRequest {
                provider: ai_provider.clone(),
                model: ai_model.clone(),
                api_key: ai_api_key.clone(),
                message: question_text.clone(),
                instructions: instructions.clone(),
                context_documents: Vec::new(),
            };

            match ai::ask(request).await {
                Ok(response) => {
                    let formatted = auto_mode::format_auto_response(&question_text, &response);
                    let _ = window_clone.emit("auto-response", &formatted);

                    // Update stats
                    let mut auto = state_clone.auto_mode.write().await;
                    auto.questions_detected += 1;
                    auto.responses_given += 1;
                    auto.last_transcription = Some(transcript);
                    auto.last_response = Some(response);
                }
                Err(e) => {
                    log::error!("AI error: {}", e);
                    let _ = window_clone.emit("auto-error", e.to_string());
                }
            }
        }
    });

    log::info!("Auto mode started");
    Ok(())
}

#[tauri::command]
async fn stop_auto_mode(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut auto = state.auto_mode.write().await;
    auto.is_active = false;
    log::info!("Auto mode stop requested");
    Ok(())
}

#[tauri::command]
async fn get_auto_mode_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let auto = state.auto_mode.read().await;
    Ok(serde_json::json!({
        "is_active": auto.is_active,
        "questions_detected": auto.questions_detected,
        "responses_given": auto.responses_given,
        "last_transcription": auto.last_transcription,
        "last_response": auto.last_response,
    }))
}

#[tauri::command]
async fn update_auto_mode_config(
    state: tauri::State<'_, AppState>,
    config: auto_mode::AutoModeConfig,
) -> Result<(), String> {
    let mut cfg = state.auto_mode_config.write().await;
    *cfg = config;
    Ok(())
}

#[tauri::command]
async fn get_auto_mode_config(state: tauri::State<'_, AppState>) -> Result<auto_mode::AutoModeConfig, String> {
    let config = state.auto_mode_config.read().await;
    Ok(config.clone())
}

// Screenshot commands

#[tauri::command]
async fn take_screenshot() -> Result<String, String> {
    let png_data = screenshot::capture_screen()
        .await
        .map_err(|e| e.to_string())?;
    Ok(screenshot::to_base64(&png_data))
}

#[tauri::command]
async fn analyze_screenshot(
    provider: String,
    model: String,
    api_key: String,
    image_base64: String,
    prompt: Option<String>,
) -> Result<String, String> {
    let default_prompt = "Analyze this screenshot from an interview. What question is being asked? Provide a concise, helpful answer.".to_string();
    let actual_prompt = prompt.unwrap_or(default_prompt);

    screenshot::analyze_screenshot(&provider, &model, &api_key, &image_base64, &actual_prompt)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn screenshot_and_analyze(
    provider: String,
    model: String,
    api_key: String,
    prompt: Option<String>,
) -> Result<String, String> {
    // Take screenshot
    let png_data = screenshot::capture_screen()
        .await
        .map_err(|e| e.to_string())?;
    let image_base64 = screenshot::to_base64(&png_data);

    // Analyze
    let default_prompt = "Analyze this screenshot from an interview. What question is being asked? Provide a concise, helpful answer.".to_string();
    let actual_prompt = prompt.unwrap_or(default_prompt);

    screenshot::analyze_screenshot(&provider, &model, &api_key, &image_base64, &actual_prompt)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_vision_models(provider: String) -> Vec<String> {
    screenshot::get_vision_models(&provider)
}

// Document commands

#[tauri::command]
async fn upload_document(
    state: tauri::State<'_, AppState>,
    file_path: String,
    doc_type: String,
) -> Result<documents::Document, String> {
    let doc_type_enum = documents::DocumentType::from_str(&doc_type);

    if !documents::is_supported_file(&file_path) {
        return Err("Unsupported file type. Supported: TXT, MD, PDF, DOC, DOCX, JSON".to_string());
    }

    let doc = documents::read_document(&file_path, doc_type_enum)
        .await
        .map_err(|e| e.to_string())?;

    let mut docs = state.documents.write().await;
    docs.push(doc.clone());

    Ok(doc)
}

#[tauri::command]
async fn add_text_document(
    state: tauri::State<'_, AppState>,
    name: String,
    doc_type: String,
    content: String,
) -> Result<documents::Document, String> {
    let doc_type_enum = documents::DocumentType::from_str(&doc_type);
    let doc = documents::create_from_text(&name, doc_type_enum, &content);

    let mut docs = state.documents.write().await;
    docs.push(doc.clone());

    Ok(doc)
}

#[tauri::command]
async fn get_documents(state: tauri::State<'_, AppState>) -> Result<Vec<documents::Document>, String> {
    let docs = state.documents.read().await;
    Ok(docs.clone())
}

#[tauri::command]
async fn remove_document(
    state: tauri::State<'_, AppState>,
    doc_id: String,
) -> Result<(), String> {
    let mut docs = state.documents.write().await;
    docs.retain(|d| d.id != doc_id);
    Ok(())
}

#[tauri::command]
async fn clear_documents(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut docs = state.documents.write().await;
    docs.clear();
    Ok(())
}

#[tauri::command]
async fn get_documents_context(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let docs = state.documents.read().await;
    Ok(documents::format_as_context(&docs))
}

#[tauri::command]
fn get_supported_file_types() -> Vec<String> {
    vec![
        "txt".to_string(),
        "md".to_string(),
        "pdf".to_string(),
        "doc".to_string(),
        "docx".to_string(),
        "json".to_string(),
    ]
}

// Settings commands

#[tauri::command]
fn load_app_settings() -> Result<settings::AppSettings, String> {
    settings::load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_app_settings(app_settings: settings::AppSettings) -> Result<(), String> {
    settings::save_settings(&app_settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_api_key(provider: String) -> Result<String, String> {
    let settings = settings::load_settings().map_err(|e| e.to_string())?;
    Ok(settings::get_api_key_for_provider(&settings, &provider))
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let db_path = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir")
                .join("interview_hunter.db");

            std::fs::create_dir_all(db_path.parent().unwrap()).ok();

            let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
            let db = rt
                .block_on(db::Database::new(&db_path.to_string_lossy()))
                .expect("failed to init database");

            app.manage(AppState {
                db: Arc::new(db),
                current_session_id: Arc::new(RwLock::new(None)),
                audio_capture: Arc::new(RwLock::new(audio::AudioCapture::new())),
                stealth_mode: Arc::new(RwLock::new(false)),
                auto_mode: Arc::new(RwLock::new(auto_mode::AutoModeState::default())),
                auto_mode_config: Arc::new(RwLock::new(auto_mode::AutoModeConfig::default())),
                documents: Arc::new(RwLock::new(Vec::new())),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_providers,
            ask_ai,
            start_session,
            end_session,
            get_sessions,
            get_session_messages,
            save_message,
            get_audio_devices,
            start_audio_capture,
            stop_audio_capture,
            get_audio_status,
            transcribe_audio,
            transcribe_and_ask,
            enable_stealth_mode,
            disable_stealth_mode,
            get_stealth_status,
            is_screen_capture_hiding_supported,
            start_auto_mode,
            stop_auto_mode,
            get_auto_mode_status,
            update_auto_mode_config,
            get_auto_mode_config,
            take_screenshot,
            analyze_screenshot,
            screenshot_and_analyze,
            get_vision_models,
            upload_document,
            add_text_document,
            get_documents,
            remove_document,
            clear_documents,
            get_documents_context,
            get_supported_file_types,
            load_app_settings,
            save_app_settings,
            get_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
