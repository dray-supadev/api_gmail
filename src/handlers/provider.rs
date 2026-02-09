use async_trait::async_trait;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageSummary {
    pub id: String,
    pub thread_id: String,
    pub snippet: String,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub date: Option<String>,
    pub unread: bool,
    pub has_attachments: bool,
    pub messages_in_thread: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CleanMessage {
    pub id: String,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<String>,
    pub snippet: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<AttachmentSummary>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttachmentSummary {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub mime_type: String,
}

#[derive(Deserialize, Debug)]
pub struct SendMessageRequest {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub subject: String,
    pub body: String,
    pub thread_id: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub label_type: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BatchModifyRequest {
    pub ids: Vec<String>,
    pub add_label_ids: Option<Vec<String>>,
    pub remove_label_ids: Option<Vec<String>>,
}

#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn list_messages(&self, token: &str, params: ListParams) -> Result<serde_json::Value, AppError>;
    async fn get_message(&self, token: &str, id: &str) -> Result<CleanMessage, AppError>;
    async fn send_message(&self, token: &str, req: SendMessageRequest) -> Result<serde_json::Value, AppError>;
    async fn list_labels(&self, token: &str) -> Result<Vec<Label>, AppError>;
    async fn batch_modify_labels(&self, token: &str, req: BatchModifyRequest) -> Result<(), AppError>;
}

#[derive(Deserialize, Debug)]
pub struct ListParams {
    pub label_ids: Option<String>,
    pub max_results: Option<u32>,
    pub q: Option<String>,
    pub page_token: Option<String>,
    pub page_number: Option<u32>,
    pub collapse_threads: Option<bool>,
}
