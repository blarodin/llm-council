use crate::config::DATA_DIR;
use crate::models::*;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_data_dir() -> Result<()> {
    let path = Path::new(DATA_DIR);
    if !path.exists() {
        fs::create_dir_all(path).context("Failed to create data directory")?;
    }
    Ok(())
}

fn get_conversation_path(conversation_id: &str) -> PathBuf {
    Path::new(DATA_DIR).join(format!("{}.json", conversation_id))
}

pub fn create_conversation(conversation_id: &str) -> Result<Conversation> {
    ensure_data_dir()?;

    let conversation = Conversation {
        id: conversation_id.to_string(),
        created_at: Utc::now().to_rfc3339(),
        title: "New Conversation".to_string(),
        messages: Vec::new(),
    };

    let path = get_conversation_path(conversation_id);
    let content = serde_json::to_string_pretty(&conversation)
        .context("Failed to serialize conversation")?;
    fs::write(&path, content).context("Failed to write conversation file")?;

    Ok(conversation)
}

pub fn get_conversation(conversation_id: &str) -> Result<Option<Conversation>> {
    let path = get_conversation_path(conversation_id);

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).context("Failed to read conversation file")?;
    let conversation: Conversation =
        serde_json::from_str(&content).context("Failed to deserialize conversation")?;

    Ok(Some(conversation))
}

fn save_conversation(conversation: &Conversation) -> Result<()> {
    ensure_data_dir()?;

    let path = get_conversation_path(&conversation.id);
    let content =
        serde_json::to_string_pretty(conversation).context("Failed to serialize conversation")?;
    fs::write(&path, content).context("Failed to write conversation file")?;

    Ok(())
}

pub fn list_conversations() -> Result<Vec<ConversationMetadata>> {
    ensure_data_dir()?;

    let mut conversations = Vec::new();
    let entries = fs::read_dir(DATA_DIR).context("Failed to read data directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).context("Failed to read conversation file")?;
            let conv: Conversation =
                serde_json::from_str(&content).context("Failed to deserialize conversation")?;

            conversations.push(ConversationMetadata {
                id: conv.id,
                created_at: conv.created_at,
                title: conv.title,
                message_count: conv.messages.len(),
            });
        }
    }

    // Sort by creation time, newest first
    conversations.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(conversations)
}

pub fn add_user_message(
    conversation_id: &str,
    content: String,
    files: Option<Vec<FileAttachment>>,
) -> Result<()> {
    let mut conversation = get_conversation(conversation_id)?
        .ok_or_else(|| anyhow::anyhow!("Conversation {} not found", conversation_id))?;

    let message = Message {
        role: "user".to_string(),
        content: Some(content),
        files,
        stage1: None,
        stage2: None,
        stage3: None,
        metadata: None,
    };

    conversation.messages.push(message);
    save_conversation(&conversation)?;

    Ok(())
}

pub fn add_assistant_message(
    conversation_id: &str,
    stage1: Vec<Stage1Result>,
    stage2: Vec<Stage2Result>,
    stage3: Stage3Result,
    metadata: Option<MessageMetadata>,
) -> Result<()> {
    let mut conversation = get_conversation(conversation_id)?
        .ok_or_else(|| anyhow::anyhow!("Conversation {} not found", conversation_id))?;

    let message = Message {
        role: "assistant".to_string(),
        content: None,
        files: None,
        stage1: Some(stage1),
        stage2: Some(stage2),
        stage3: Some(stage3),
        metadata,
    };

    conversation.messages.push(message);
    save_conversation(&conversation)?;

    Ok(())
}

pub fn update_conversation_title(conversation_id: &str, title: String) -> Result<()> {
    let mut conversation = get_conversation(conversation_id)?
        .ok_or_else(|| anyhow::anyhow!("Conversation {} not found", conversation_id))?;

    conversation.title = title;
    save_conversation(&conversation)?;

    Ok(())
}

pub fn delete_conversation(conversation_id: &str) -> Result<()> {
    let path = get_conversation_path(conversation_id);

    if !path.exists() {
        return Err(anyhow::anyhow!("Conversation {} not found", conversation_id));
    }

    fs::remove_file(&path).context("Failed to delete conversation file")?;

    Ok(())
}
