use crate::council::{
    calculate_aggregate_rankings, calculate_usage_summary, generate_conversation_title,
    run_full_council, stage1_collect_responses, stage2_collect_rankings, stage3_synthesize_final,
};
use crate::models::*;
use crate::settings;
use crate::storage;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Response, Sse,
    },
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower_http::cors::{CorsLayer, Any};

#[derive(Clone)]
pub struct AppState {}

pub fn create_router() -> Router {
    let state = AppState {};

    // Configure CORS to allow frontend access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(health_check))
        .route("/api/conversations", get(list_conversations))
        .route("/api/conversations", post(create_conversation))
        .route("/api/conversations/:id", get(get_conversation))
        .route("/api/conversations/:id", delete(delete_conversation))
        .route("/api/conversations/:id/title", patch(update_title))
        .route("/api/conversations/:id/message", post(send_message))
        .route(
            "/api/conversations/:id/message/stream",
            post(send_message_stream),
        )
        .route("/api/settings/openrouter-key", get(get_api_key))
        .route("/api/settings/openrouter-key", post(set_api_key))
        .route("/api/settings/openrouter-key", delete(clear_api_key))
        .route("/api/settings/models", get(get_models))
        .route("/api/settings/models", post(set_models))
        .layer(cors)
        .with_state(Arc::new(state))
}

async fn health_check() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok".to_string(),
        service: "LLM Council API".to_string(),
    })
}

async fn list_conversations() -> Result<Json<Vec<ConversationMetadata>>, (StatusCode, String)> {
    match storage::list_conversations() {
        Ok(conversations) => Ok(Json(conversations)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list conversations: {}", e),
        )),
    }
}

async fn create_conversation(
    Json(_payload): Json<CreateConversationRequest>,
) -> Result<Json<Conversation>, (StatusCode, String)> {
    let conversation_id = uuid::Uuid::new_v4().to_string();

    match storage::create_conversation(&conversation_id) {
        Ok(conversation) => Ok(Json(conversation)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create conversation: {}", e),
        )),
    }
}

async fn get_conversation(
    Path(conversation_id): Path<String>,
) -> Result<Json<Conversation>, (StatusCode, String)> {
    match storage::get_conversation(&conversation_id) {
        Ok(Some(conversation)) => Ok(Json(conversation)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Conversation not found".to_string())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get conversation: {}", e),
        )),
    }
}

async fn delete_conversation(
    Path(conversation_id): Path<String>,
) -> Result<Json<DeleteResponse>, (StatusCode, String)> {
    match storage::delete_conversation(&conversation_id) {
        Ok(_) => Ok(Json(DeleteResponse {
            status: "ok".to_string(),
            message: "Conversation deleted".to_string(),
        })),
        Err(e) => Err((StatusCode::NOT_FOUND, format!("{}", e))),
    }
}

async fn update_title(
    Path(conversation_id): Path<String>,
    Json(payload): Json<UpdateTitleRequest>,
) -> Result<Json<DeleteResponse>, (StatusCode, String)> {
    match storage::update_conversation_title(&conversation_id, payload.title) {
        Ok(_) => Ok(Json(DeleteResponse {
            status: "ok".to_string(),
            message: "Title updated".to_string(),
        })),
        Err(e) => Err((StatusCode::NOT_FOUND, format!("{}", e))),
    }
}

async fn send_message(
    Path(conversation_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Response, (StatusCode, String)> {
    // Check if conversation exists
    let conversation = match storage::get_conversation(&conversation_id) {
        Ok(Some(conv)) => conv,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get conversation: {}", e),
            ));
        }
    };

    let is_first_message = conversation.messages.is_empty();

    // Add user message with files
    let files_list = if payload.files.is_empty() {
        None
    } else {
        Some(payload.files.clone())
    };

    if let Err(e) = storage::add_user_message(&conversation_id, payload.content.clone(), files_list.clone()) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to add user message: {}", e),
        ));
    }

    // If this is the first message, generate a title
    if is_first_message {
        match generate_conversation_title(&payload.content).await {
            Ok(title) => {
                let _ = storage::update_conversation_title(&conversation_id, title);
            }
            Err(_) => {} // Ignore title generation errors
        }
    }

    // Run the 3-stage council process
    let files_ref = files_list.as_ref();
    match run_full_council(&payload.content, files_ref).await {
        Ok((stage1_results, stage2_results, stage3_result, metadata)) => {
            // Add assistant message
            if let Err(e) = storage::add_assistant_message(
                &conversation_id,
                stage1_results.clone(),
                stage2_results.clone(),
                stage3_result.clone(),
                Some(metadata.clone()),
            ) {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to add assistant message: {}", e),
                ));
            }

            // Return the complete response
            let response = json!({
                "stage1": stage1_results,
                "stage2": stage2_results,
                "stage3": stage3_result,
                "metadata": metadata
            });

            Ok(Json(response).into_response())
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Council process failed: {}", e),
        )),
    }
}

struct ErrorStream<S> {
    inner: S,
}

impl<S, E> Stream for ErrorStream<S>
where
    S: Stream<Item = Result<Event, E>>,
{
    type Item = Result<Event, axum::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY: We're just repinning the inner stream
        unsafe {
            let inner = Pin::new_unchecked(&mut self.get_unchecked_mut().inner);
            match inner.poll_next(cx) {
                Poll::Ready(Some(Ok(event))) => Poll::Ready(Some(Ok(event))),
                Poll::Ready(Some(Err(_))) => Poll::Ready(None),
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

async fn send_message_stream(
    Path(conversation_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, (StatusCode, String)>
{
    // Check if conversation exists
    let conversation = match storage::get_conversation(&conversation_id) {
        Ok(Some(conv)) => conv,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get conversation: {}", e),
            ));
        }
    };

    let is_first_message = conversation.messages.is_empty();

    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Spawn the processing task
    tokio::spawn(async move {
        // Add user message with files
        let files_list = if payload.files.is_empty() {
            None
        } else {
            Some(payload.files.clone())
        };

        if let Err(e) =
            storage::add_user_message(&conversation_id, payload.content.clone(), files_list.clone())
        {
            let _ = tx
                .send(Event::default().json_data(json!({"type": "error", "message": e.to_string()})))
                .await;
            return;
        }

        // Start title generation in parallel (don't await yet)
        let title_handle = if is_first_message {
            let content = payload.content.clone();
            Some(tokio::spawn(async move {
                generate_conversation_title(&content).await
            }))
        } else {
            None
        };

        // Stage 1: Collect responses
        let _ = tx
            .send(Event::default().json_data(json!({"type": "stage1_start"})))
            .await;

        let files_ref = files_list.as_ref();
        let stage1_results = match stage1_collect_responses(&payload.content, files_ref).await {
            Ok(results) => results,
            Err(e) => {
                let _ = tx
                    .send(
                        Event::default()
                            .json_data(json!({"type": "error", "message": e.to_string()})),
                    )
                    .await;
                return;
            }
        };

        let _ = tx
            .send(
                Event::default().json_data(json!({"type": "stage1_complete", "data": stage1_results})),
            )
            .await;

        // Stage 2: Collect rankings
        let _ = tx
            .send(Event::default().json_data(json!({"type": "stage2_start"})))
            .await;

        let (stage2_results, label_to_model) =
            match stage2_collect_rankings(&payload.content, &stage1_results, files_ref).await {
                Ok(results) => results,
                Err(e) => {
                    let _ = tx
                        .send(
                            Event::default()
                                .json_data(json!({"type": "error", "message": e.to_string()})),
                        )
                        .await;
                    return;
                }
            };

        let aggregate_rankings = calculate_aggregate_rankings(&stage2_results, &label_to_model);

        let _ = tx
            .send(Event::default().json_data(json!({
                "type": "stage2_complete",
                "data": stage2_results,
                "metadata": {
                    "label_to_model": label_to_model,
                    "aggregate_rankings": aggregate_rankings
                }
            })))
            .await;

        // Stage 3: Synthesize final answer
        let _ = tx
            .send(Event::default().json_data(json!({"type": "stage3_start"})))
            .await;

        let stage3_result =
            match stage3_synthesize_final(&payload.content, &stage1_results, &stage2_results, files_ref)
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    let _ = tx
                        .send(
                            Event::default()
                                .json_data(json!({"type": "error", "message": e.to_string()})),
                        )
                        .await;
                    return;
                }
            };

        // Calculate usage summary
        let usage_summary = calculate_usage_summary(&stage1_results, &stage2_results, &stage3_result);

        let _ = tx
            .send(Event::default().json_data(json!({
                "type": "stage3_complete",
                "data": stage3_result,
                "metadata": {
                    "usage_summary": usage_summary
                }
            })))
            .await;

        // Wait for title generation if it was started
        if let Some(handle) = title_handle {
            if let Ok(Ok(title)) = handle.await {
                let _ = storage::update_conversation_title(&conversation_id, title.clone());
                let _ = tx
                    .send(
                        Event::default()
                            .json_data(json!({"type": "title_complete", "data": {"title": title}})),
                    )
                    .await;
            }
        }

        // Save complete assistant message with all metadata
        let metadata = MessageMetadata {
            label_to_model: Some(label_to_model),
            aggregate_rankings: Some(aggregate_rankings),
            usage_summary: Some(usage_summary),
        };

        let _ = storage::add_assistant_message(
            &conversation_id,
            stage1_results,
            stage2_results,
            stage3_result,
            Some(metadata),
        );

        // Send completion event
        let _ = tx
            .send(Event::default().json_data(json!({"type": "complete"})))
            .await;
    });

    let stream = ReceiverStream::new(rx);
    let error_stream = ErrorStream { inner: stream };
    Ok(Sse::new(error_stream).keep_alive(KeepAlive::default()))
}

// Settings API endpoints

#[derive(Debug, Serialize, Deserialize)]
struct ApiKeyResponse {
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetApiKeyRequest {
    api_key: String,
}

async fn get_api_key() -> Result<Json<ApiKeyResponse>, (StatusCode, String)> {
    match settings::get_settings() {
        Ok(settings) => Ok(Json(ApiKeyResponse {
            api_key: settings.openrouter_api_key.unwrap_or_default(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get settings: {}", e),
        )),
    }
}

async fn set_api_key(
    Json(payload): Json<SetApiKeyRequest>,
) -> Result<Json<DeleteResponse>, (StatusCode, String)> {
    match settings::set_openrouter_api_key(payload.api_key) {
        Ok(_) => Ok(Json(DeleteResponse {
            status: "ok".to_string(),
            message: "API key saved".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save API key: {}", e),
        )),
    }
}

async fn clear_api_key() -> Result<Json<DeleteResponse>, (StatusCode, String)> {
    match settings::clear_openrouter_api_key() {
        Ok(_) => Ok(Json(DeleteResponse {
            status: "ok".to_string(),
            message: "API key cleared".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to clear API key: {}", e),
        )),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelsResponse {
    council_models: Vec<String>,
    chairman_model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetModelsRequest {
    council_models: Vec<String>,
    chairman_model: String,
}

async fn get_models() -> Result<Json<ModelsResponse>, (StatusCode, String)> {
    Ok(Json(ModelsResponse {
        council_models: settings::get_council_models(),
        chairman_model: settings::get_chairman_model(),
    }))
}

async fn set_models(
    Json(payload): Json<SetModelsRequest>,
) -> Result<Json<DeleteResponse>, (StatusCode, String)> {
    // Validate that we have at least one council model
    if payload.council_models.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "At least one council model is required".to_string(),
        ));
    }

    if payload.chairman_model.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Chairman model is required".to_string(),
        ));
    }

    // Save council models
    if let Err(e) = settings::set_council_models(payload.council_models) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save council models: {}", e),
        ));
    }

    // Save chairman model
    if let Err(e) = settings::set_chairman_model(payload.chairman_model) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save chairman model: {}", e),
        ));
    }

    Ok(Json(DeleteResponse {
        status: "ok".to_string(),
        message: "Models configuration saved".to_string(),
    }))
}
