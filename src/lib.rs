use core::str;
use std::{fmt::Display};

use api::{Candidate, ContentEmbedding, GenerationConfig, PromptFeedback, SafetySetting, TaskType, Tool, UsageMetadata};
use content::Content;
use serde::{Deserialize, Serialize};

pub mod api;
pub mod chat;
pub mod model;
pub mod content;

pub static BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

#[derive(Debug, Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
    pub usage_metadata: UsageMetadata,
    pub prompt_feedback: Option<PromptFeedback>,

}

impl GeminiResponse {
    pub fn text(&self) -> String {
        self.candidates[0].text()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedContentRequest {
    pub content: Content,
    #[serde(flatten)]
    pub config: EmbedContentConfig,
}


#[derive(Debug, Deserialize)]
pub struct EmbedContentResponse {
    pub embedding: ContentEmbedding,
}


#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmbedContentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimensionality: Option<i32>,
}







#[cfg(test)]
mod tests {}
