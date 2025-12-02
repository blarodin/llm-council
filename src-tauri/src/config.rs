use crate::settings;

pub fn get_council_models() -> Vec<String> {
    settings::get_council_models()
}

pub fn get_chairman_model() -> String {
    settings::get_chairman_model()
}

pub const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

pub const DATA_DIR: &str = "data/conversations";

pub fn get_openrouter_api_key() -> Result<String, String> {
    settings::get_openrouter_api_key()
        .map_err(|e| e.to_string())
}
