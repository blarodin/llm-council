use crate::config::{get_chairman_model, get_council_models};
use crate::models::*;
use crate::openrouter::{query_model, query_models_parallel};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use serde_json::json;
use std::collections::HashMap;

pub fn extract_text_from_files(files: &[FileAttachment]) -> String {
    let binary_types = vec![
        "application/pdf",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ];

    let mut text_parts = Vec::new();

    for file in files {
        // Skip images and binary formats
        if file.file_type.starts_with("image/") || binary_types.contains(&file.file_type.as_str()) {
            continue;
        }

        // Extract base64 data from data URL
        if let Some(base64_data) = file.data.split("base64,").nth(1) {
            match general_purpose::STANDARD.decode(base64_data) {
                Ok(decoded) => match String::from_utf8(decoded) {
                    Ok(text_content) => {
                        text_parts.push(format!(
                            "--- File: {} ---\n{}\n--- End of {} ---",
                            file.name, text_content, file.name
                        ));
                    }
                    Err(_) => continue, // Binary file that can't decode as text
                },
                Err(e) => {
                    eprintln!("Warning: Could not decode base64 from {}: {}", file.name, e);
                    continue;
                }
            }
        }
    }

    if !text_parts.is_empty() {
        format!("\n\n{}", text_parts.join("\n\n"))
    } else {
        String::new()
    }
}

pub async fn stage1_collect_responses(
    user_query: &str,
    files: Option<&Vec<FileAttachment>>,
) -> Result<Vec<Stage1Result>> {
    let mut text_content = user_query.to_string();
    let has_images = files
        .map(|f| f.iter().any(|file| file.file_type.starts_with("image/")))
        .unwrap_or(false);

    // Add text file contents to query
    if let Some(files) = files {
        let file_text = extract_text_from_files(files);
        if !file_text.is_empty() {
            text_content.push_str(&file_text);
        }
    }

    let messages = if has_images {
        // Vision models can handle images
        let mut content = vec![json!({"type": "text", "text": text_content})];

        if let Some(files) = files {
            for file in files {
                if file.file_type.starts_with("image/") {
                    content.push(json!({
                        "type": "image_url",
                        "image_url": {"url": file.data}
                    }));
                }
            }
        }

        vec![json!({"role": "user", "content": content})]
    } else {
        // Text-only message
        vec![json!({"role": "user", "content": text_content})]
    };

    // Query all models in parallel
    let council_models = get_council_models();
    let model_refs: Vec<&str> = council_models.iter().map(|s| s.as_str()).collect();
    let responses = query_models_parallel(&model_refs, messages).await;

    // Format results
    let mut stage1_results = Vec::new();
    for (model, response) in responses {
        if let Some(resp) = response {
            stage1_results.push(Stage1Result {
                model,
                response: resp.content,
                usage: resp.usage,
            });
        }
    }

    Ok(stage1_results)
}

pub async fn stage2_collect_rankings(
    user_query: &str,
    stage1_results: &[Stage1Result],
    files: Option<&Vec<FileAttachment>>,
) -> Result<(Vec<Stage2Result>, HashMap<String, String>)> {
    // Create anonymized labels for responses (Response A, Response B, etc.)
    let labels: Vec<String> = (0..stage1_results.len())
        .map(|i| format!("{}", (b'A' + i as u8) as char))
        .collect();

    // Create mapping from label to model name
    let mut label_to_model = HashMap::new();
    for (label, result) in labels.iter().zip(stage1_results.iter()) {
        label_to_model.insert(format!("Response {}", label), result.model.clone());
    }

    // Build the ranking prompt
    let responses_text: Vec<String> = labels
        .iter()
        .zip(stage1_results.iter())
        .map(|(label, result)| format!("Response {}:\n{}", label, result.response))
        .collect();
    let responses_text = responses_text.join("\n\n");

    // Build query text with file contents if present
    let mut query_text = user_query.to_string();
    if let Some(files) = files {
        let file_text = extract_text_from_files(files);
        if !file_text.is_empty() {
            query_text.push_str(&file_text);
        }
    }

    let ranking_prompt = format!(
        r#"You are evaluating different responses to the following question:

Question: {}

Here are the responses from different models (anonymized):

{}

Your task:
1. First, evaluate each response individually. For each response, explain what it does well and what it does poorly.
2. Then, at the very end of your response, provide a final ranking.

IMPORTANT: Your final ranking MUST be formatted EXACTLY as follows:
- Start with the line "FINAL RANKING:" (all caps, with colon)
- Then list the responses from best to worst as a numbered list
- Each line should be: number, period, space, then ONLY the response label (e.g., "1. Response A")
- Do not add any other text or explanations in the ranking section

Example of the correct format for your ENTIRE response:

Response A provides good detail on X but misses Y...
Response B is accurate but lacks depth on Z...
Response C offers the most comprehensive answer...

FINAL RANKING:
1. Response C
2. Response A
3. Response B

Now provide your evaluation and ranking:"#,
        query_text, responses_text
    );

    let messages = vec![json!({"role": "user", "content": ranking_prompt})];

    // Get rankings from all council models in parallel
    let council_models = get_council_models();
    let model_refs: Vec<&str> = council_models.iter().map(|s| s.as_str()).collect();
    let responses = query_models_parallel(&model_refs, messages).await;

    // Format results
    let mut stage2_results = Vec::new();
    for (model, response) in responses {
        if let Some(resp) = response {
            let full_text = resp.content;
            let parsed = parse_ranking_from_text(&full_text);
            stage2_results.push(Stage2Result {
                model,
                ranking: full_text,
                parsed_ranking: parsed,
                usage: resp.usage,
            });
        }
    }

    Ok((stage2_results, label_to_model))
}

pub async fn stage3_synthesize_final(
    user_query: &str,
    stage1_results: &[Stage1Result],
    stage2_results: &[Stage2Result],
    files: Option<&Vec<FileAttachment>>,
) -> Result<Stage3Result> {
    // Build comprehensive context for chairman
    let stage1_text: Vec<String> = stage1_results
        .iter()
        .map(|result| format!("Model: {}\nResponse: {}", result.model, result.response))
        .collect();
    let stage1_text = stage1_text.join("\n\n");

    let stage2_text: Vec<String> = stage2_results
        .iter()
        .map(|result| format!("Model: {}\nRanking: {}", result.model, result.ranking))
        .collect();
    let stage2_text = stage2_text.join("\n\n");

    // Build query text with file contents if present
    let mut query_text = user_query.to_string();
    if let Some(files) = files {
        let file_text = extract_text_from_files(files);
        if !file_text.is_empty() {
            query_text.push_str(&file_text);
        }
    }

    let chairman_prompt = format!(
        r#"You are the Chairman of an LLM Council. Multiple AI models have provided responses to a user's question, and then ranked each other's responses.

Original Question: {}

STAGE 1 - Individual Responses:
{}

STAGE 2 - Peer Rankings:
{}

Your task as Chairman is to synthesize all of this information into a single, comprehensive, accurate answer to the user's original question. Consider:
- The individual responses and their insights
- The peer rankings and what they reveal about response quality
- Any patterns of agreement or disagreement

Provide a clear, well-reasoned final answer that represents the council's collective wisdom:"#,
        query_text, stage1_text, stage2_text
    );

    let messages = vec![json!({"role": "user", "content": chairman_prompt})];

    // Query the chairman model
    let chairman_model = get_chairman_model();
    let response = query_model(&chairman_model, messages, 120).await?;

    match response {
        Some(resp) => Ok(Stage3Result {
            model: chairman_model.clone(),
            response: resp.content,
            usage: resp.usage,
        }),
        None => Ok(Stage3Result {
            model: chairman_model.clone(),
            response: "Error: Unable to generate final synthesis.".to_string(),
            usage: None,
        }),
    }
}

pub fn parse_ranking_from_text(ranking_text: &str) -> Vec<String> {
    // Look for "FINAL RANKING:" section
    if let Some(pos) = ranking_text.find("FINAL RANKING:") {
        let ranking_section = &ranking_text[pos..];

        // Try to extract numbered list format (e.g., "1. Response A")
        let re = Regex::new(r"\d+\.\s*Response [A-Z]").unwrap();
        let matches: Vec<String> = re
            .find_iter(ranking_section)
            .filter_map(|m| {
                let re2 = Regex::new(r"Response [A-Z]").unwrap();
                re2.find(m.as_str()).map(|m2| m2.as_str().to_string())
            })
            .collect();

        if !matches.is_empty() {
            return matches;
        }

        // Fallback: Extract all "Response X" patterns in order
        let re = Regex::new(r"Response [A-Z]").unwrap();
        return re
            .find_iter(ranking_section)
            .map(|m| m.as_str().to_string())
            .collect();
    }

    // Fallback: try to find any "Response X" patterns in order
    let re = Regex::new(r"Response [A-Z]").unwrap();
    re.find_iter(ranking_text)
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn calculate_usage_summary(
    stage1_results: &[Stage1Result],
    stage2_results: &[Stage2Result],
    stage3_result: &Stage3Result,
) -> UsageSummary {
    fn sum_usage(usages: &[Option<Usage>]) -> Usage {
        let mut total = Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };

        for usage in usages.iter().flatten() {
            total.prompt_tokens += usage.prompt_tokens;
            total.completion_tokens += usage.completion_tokens;
            total.total_tokens += usage.total_tokens;
        }

        total
    }

    let stage1_usages: Vec<Option<Usage>> = stage1_results.iter().map(|r| r.usage.clone()).collect();
    let stage2_usages: Vec<Option<Usage>> = stage2_results.iter().map(|r| r.usage.clone()).collect();
    let stage3_usage = vec![stage3_result.usage.clone()];

    let stage1_total = sum_usage(&stage1_usages);
    let stage2_total = sum_usage(&stage2_usages);
    let stage3_total = sum_usage(&stage3_usage);

    let grand_total = Usage {
        prompt_tokens: stage1_total.prompt_tokens
            + stage2_total.prompt_tokens
            + stage3_total.prompt_tokens,
        completion_tokens: stage1_total.completion_tokens
            + stage2_total.completion_tokens
            + stage3_total.completion_tokens,
        total_tokens: stage1_total.total_tokens
            + stage2_total.total_tokens
            + stage3_total.total_tokens,
    };

    let mut by_model: HashMap<String, Usage> = HashMap::new();

    for result in stage1_results {
        if let Some(usage) = &result.usage {
            by_model
                .entry(result.model.clone())
                .or_insert_with(|| Usage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                })
                .prompt_tokens += usage.prompt_tokens;
            by_model.get_mut(&result.model).unwrap().completion_tokens += usage.completion_tokens;
            by_model.get_mut(&result.model).unwrap().total_tokens += usage.total_tokens;
        }
    }

    for result in stage2_results {
        if let Some(usage) = &result.usage {
            by_model
                .entry(result.model.clone())
                .or_insert_with(|| Usage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                })
                .prompt_tokens += usage.prompt_tokens;
            by_model.get_mut(&result.model).unwrap().completion_tokens += usage.completion_tokens;
            by_model.get_mut(&result.model).unwrap().total_tokens += usage.total_tokens;
        }
    }

    if let Some(usage) = &stage3_result.usage {
        by_model
            .entry(stage3_result.model.clone())
            .or_insert_with(|| Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            })
            .prompt_tokens += usage.prompt_tokens;
        by_model
            .get_mut(&stage3_result.model)
            .unwrap()
            .completion_tokens += usage.completion_tokens;
        by_model
            .get_mut(&stage3_result.model)
            .unwrap()
            .total_tokens += usage.total_tokens;
    }

    UsageSummary {
        stage1_total,
        stage2_total,
        stage3_total,
        grand_total,
        by_model,
    }
}

pub fn calculate_aggregate_rankings(
    stage2_results: &[Stage2Result],
    label_to_model: &HashMap<String, String>,
) -> Vec<AggregateRanking> {
    let mut model_positions: HashMap<String, Vec<usize>> = HashMap::new();

    for ranking in stage2_results {
        let parsed_ranking = &ranking.parsed_ranking;

        for (position, label) in parsed_ranking.iter().enumerate() {
            if let Some(model_name) = label_to_model.get(label) {
                model_positions
                    .entry(model_name.clone())
                    .or_default()
                    .push(position + 1);
            }
        }
    }

    let mut aggregate: Vec<AggregateRanking> = model_positions
        .into_iter()
        .filter(|(_, positions)| !positions.is_empty())
        .map(|(model, positions)| {
            let avg_rank = positions.iter().sum::<usize>() as f64 / positions.len() as f64;
            AggregateRanking {
                model,
                average_rank: (avg_rank * 100.0).round() / 100.0, // Round to 2 decimal places
                rankings_count: positions.len(),
            }
        })
        .collect();

    // Sort by average rank (lower is better)
    aggregate.sort_by(|a, b| {
        a.average_rank
            .partial_cmp(&b.average_rank)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    aggregate
}

pub async fn generate_conversation_title(user_query: &str) -> Result<String> {
    let title_prompt = format!(
        r#"Generate a very short title (3-5 words maximum) that summarizes the following question.
The title should be concise and descriptive. Do not use quotes or punctuation in the title.

Question: {}

Title:"#,
        user_query
    );

    let messages = vec![json!({"role": "user", "content": title_prompt})];

    // Use gemini-2.5-flash for title generation (fast and cheap)
    let response = query_model("google/gemini-2.5-flash", messages, 30).await?;

    let title = match response {
        Some(resp) => {
            let mut title = resp.content.trim().to_string();
            // Clean up the title - remove quotes
            title = title.trim_matches(|c| c == '"' || c == '\'').to_string();
            // Truncate if too long
            if title.len() > 50 {
                title = format!("{}...", &title[..47]);
            }
            title
        }
        None => "New Conversation".to_string(),
    };

    Ok(title)
}

pub async fn run_full_council(
    user_query: &str,
    files: Option<&Vec<FileAttachment>>,
) -> Result<(
    Vec<Stage1Result>,
    Vec<Stage2Result>,
    Stage3Result,
    MessageMetadata,
)> {
    // Stage 1: Collect individual responses
    let stage1_results = stage1_collect_responses(user_query, files).await?;

    // If no models responded successfully, return error
    if stage1_results.is_empty() {
        return Ok((
            Vec::new(),
            Vec::new(),
            Stage3Result {
                model: "error".to_string(),
                response: "All models failed to respond. Please try again.".to_string(),
                usage: None,
            },
            MessageMetadata {
                label_to_model: None,
                aggregate_rankings: None,
                usage_summary: None,
            },
        ));
    }

    // Stage 2: Collect rankings
    let (stage2_results, label_to_model) =
        stage2_collect_rankings(user_query, &stage1_results, files).await?;

    // Calculate aggregate rankings
    let aggregate_rankings = calculate_aggregate_rankings(&stage2_results, &label_to_model);

    // Stage 3: Synthesize final answer
    let stage3_result =
        stage3_synthesize_final(user_query, &stage1_results, &stage2_results, files).await?;

    // Calculate usage summary
    let usage_summary = calculate_usage_summary(&stage1_results, &stage2_results, &stage3_result);

    // Prepare metadata
    let metadata = MessageMetadata {
        label_to_model: Some(label_to_model),
        aggregate_rankings: Some(aggregate_rankings),
        usage_summary: Some(usage_summary),
    };

    Ok((stage1_results, stage2_results, stage3_result, metadata))
}
