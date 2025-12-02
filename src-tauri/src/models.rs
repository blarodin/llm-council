use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub size: usize,
    pub data: String, // Base64-encoded data URL
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileAttachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage1: Option<Vec<Stage1Result>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage2: Option<Vec<Stage2Result>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage3: Option<Stage3Result>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub created_at: String,
    pub title: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub id: String,
    pub created_at: String,
    pub title: String,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage1Result {
    pub model: String,
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage2Result {
    pub model: String,
    pub ranking: String,
    pub parsed_ranking: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage3Result {
    pub model: String,
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateRanking {
    pub model: String,
    pub average_rank: f64,
    pub rankings_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub stage1_total: Usage,
    pub stage2_total: Usage,
    pub stage3_total: Usage,
    pub grand_total: Usage,
    pub by_model: HashMap<String, Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_to_model: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate_rankings: Option<Vec<AggregateRanking>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_summary: Option<UsageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

// API request/response types
#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(default)]
    pub files: Vec<FileAttachment>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTitleRequest {
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub service: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub status: String,
    pub message: String,
}
