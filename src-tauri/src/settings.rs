use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub provider: String,
    pub model: String,
    pub api_keys: ApiKeys,
    pub instructions: String,
    pub auto_mode: bool,
    pub language: String,
    pub mini_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeys {
    pub openrouter: String,
    pub openai: String,
    pub google: String,
    pub deepseek: String,
    pub xai: String,
    pub nvidia: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            provider: "openrouter-free".to_string(),
            model: "stepfun/step-3.5-flash:free".to_string(),
            api_keys: ApiKeys::default(),
            instructions: String::new(),
            auto_mode: false,
            language: "en".to_string(),
            mini_mode: true,
        }
    }
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("interview-cracker");
    
    std::fs::create_dir_all(&config_dir).ok();
    config_dir.join("settings.json")
}

pub fn load_settings() -> Result<AppSettings> {
    let path = get_config_path();
    
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    
    let content = std::fs::read_to_string(&path)?;
    let settings: AppSettings = serde_json::from_str(&content)?;
    Ok(settings)
}

pub fn save_settings(settings: &AppSettings) -> Result<()> {
    let path = get_config_path();
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(&path, content)?;
    log::info!("Settings saved to {:?}", path);
    Ok(())
}

pub fn get_api_key_for_provider(settings: &AppSettings, provider: &str) -> String {
    match provider {
        "openrouter" | "openrouter-free" => settings.api_keys.openrouter.clone(),
        "openai" => settings.api_keys.openai.clone(),
        "google" => settings.api_keys.google.clone(),
        "deepseek" => settings.api_keys.deepseek.clone(),
        "xai" => settings.api_keys.xai.clone(),
        "nvidia" => settings.api_keys.nvidia.clone(),
        _ => String::new(),
    }
}
