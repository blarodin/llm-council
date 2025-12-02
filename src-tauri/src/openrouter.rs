use crate::config::{get_openrouter_api_key, OPENROUTER_API_URL};
use crate::models::{ModelResponse, Usage};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

pub async fn query_model(
    model: &str,
    messages: Vec<Value>,
    timeout: u64,
) -> Result<Option<ModelResponse>> {
    let api_key = get_openrouter_api_key()
        .map_err(|e| anyhow::anyhow!(e))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .context("Failed to create HTTP client")?;

    let payload = json!({
        "model": model,
        "messages": messages,
    });

    match client
        .post(OPENROUTER_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            if !response.status().is_success() {
                eprintln!("Error querying model {}: HTTP {}", model, response.status());
                return Ok(None);
            }

            match response.json::<Value>().await {
                Ok(data) => {
                    let message = &data["choices"][0]["message"];
                    let content = message["content"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    
                    let usage = data.get("usage").and_then(|u| {
                        Some(Usage {
                            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
                        })
                    });

                    let reasoning_details = message.get("reasoning_details").cloned();

                    Ok(Some(ModelResponse {
                        content,
                        reasoning_details,
                        usage,
                    }))
                }
                Err(e) => {
                    eprintln!("Error parsing response from {}: {}", model, e);
                    Ok(None)
                }
            }
        }
        Err(e) => {
            eprintln!("Error querying model {}: {}", model, e);
            Ok(None)
        }
    }
}

pub async fn query_models_parallel(
    models: &[&str],
    messages: Vec<Value>,
) -> HashMap<String, Option<ModelResponse>> {
    let mut tasks = Vec::new();

    for &model in models {
        let messages = messages.clone();
        let model = model.to_string();
        
        let task = tokio::spawn(async move {
            let response = query_model(&model, messages, 120).await;
            (model, response.ok().flatten())
        });

        tasks.push(task);
    }

    let mut results = HashMap::new();
    for task in tasks {
        if let Ok((model, response)) = task.await {
            results.insert(model, response);
        }
    }

    results
}
