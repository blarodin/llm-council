use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SETTINGS_DIR: &str = "data";
const SETTINGS_FILE: &str = "data/settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub openrouter_api_key: Option<String>,
    pub council_models: Option<Vec<String>>,
    pub chairman_model: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            openrouter_api_key: None,
            council_models: None,
            chairman_model: None,
        }
    }
}

/// Get the settings from file
pub fn get_settings() -> Result<Settings> {
    if !PathBuf::from(SETTINGS_FILE).exists() {
        return Ok(Settings::default());
    }

    let content = fs::read_to_string(SETTINGS_FILE)?;
    let settings: Settings = serde_json::from_str(&content)?;
    Ok(settings)
}

/// Save settings to file
pub fn save_settings(settings: &Settings) -> Result<()> {
    // Ensure directory exists
    fs::create_dir_all(SETTINGS_DIR)?;

    let content = serde_json::to_string_pretty(settings)?;
    fs::write(SETTINGS_FILE, content)?;
    Ok(())
}

/// Get OpenRouter API key from settings or environment
pub fn get_openrouter_api_key() -> Result<String> {
    // First check settings file
    if let Ok(settings) = get_settings() {
        if let Some(api_key) = settings.openrouter_api_key {
            if !api_key.is_empty() {
                return Ok(api_key);
            }
        }
    }

    // Fall back to environment variable
    std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| anyhow::anyhow!("OPENROUTER_API_KEY not found in settings or environment"))
}

/// Set OpenRouter API key in settings
pub fn set_openrouter_api_key(api_key: String) -> Result<()> {
    let mut settings = get_settings().unwrap_or_default();
    settings.openrouter_api_key = Some(api_key);
    save_settings(&settings)
}

/// Clear OpenRouter API key from settings
pub fn clear_openrouter_api_key() -> Result<()> {
    let mut settings = get_settings().unwrap_or_default();
    settings.openrouter_api_key = None;
    save_settings(&settings)
}

/// Get council models (from settings or defaults)
pub fn get_council_models() -> Vec<String> {
    if let Ok(settings) = get_settings() {
        if let Some(models) = settings.council_models {
            if !models.is_empty() {
                return models;
            }
        }
    }
    // Default models
    vec![
        "openai/gpt-oss-20b:free".to_string(),
        "google/gemma-3-27b-it:free".to_string(),
        "meta-llama/llama-3.3-70b-instruct:free".to_string(),
        "x-ai/grok-4.1-fast:free".to_string(),
        "qwen/qwen3-235b-a22b:free".to_string(),
        "nousresearch/hermes-3-llama-3.1-405b:free".to_string(),
        "mistralai/mistral-small-3.1-24b-instruct:free".to_string(),
        "tngtech/deepseek-r1t2-chimera:free".to_string(),
    ]
}

/// Set council models in settings
pub fn set_council_models(models: Vec<String>) -> Result<()> {
    let mut settings = get_settings().unwrap_or_default();
    settings.council_models = Some(models);
    save_settings(&settings)
}

/// Get chairman model (from settings or default)
pub fn get_chairman_model() -> String {
    if let Ok(settings) = get_settings() {
        if let Some(model) = settings.chairman_model {
            if !model.is_empty() {
                return model;
            }
        }
    }
    "google/gemini-3-pro-preview".to_string()
}

/// Set chairman model in settings
pub fn set_chairman_model(model: String) -> Result<()> {
    let mut settings = get_settings().unwrap_or_default();
    settings.chairman_model = Some(model);
    save_settings(&settings)
}
