use std::env;

pub const COUNCIL_MODELS: &[&str] = &[
    "openai/gpt-oss-20b:free",
    "google/gemma-3-27b-it:free",
    "meta-llama/llama-3.3-70b-instruct:free",
    "x-ai/grok-4.1-fast:free",
    "qwen/qwen3-235b-a22b:free",
    "nousresearch/hermes-3-llama-3.1-405b:free",
    "mistralai/mistral-small-3.1-24b-instruct:free",
    "tngtech/deepseek-r1t2-chimera:free",
];

pub const CHAIRMAN_MODEL: &str = "google/gemini-3-pro-preview";

pub const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

pub const DATA_DIR: &str = "data/conversations";

pub fn get_openrouter_api_key() -> Result<String, String> {
    env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY environment variable not set".to_string())
}
