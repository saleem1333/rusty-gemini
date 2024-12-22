use serde::{Deserialize, Serialize};

use crate::{content::{Content, Part}, grounding::GroundingAtrribution};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    /// Generated content returned from the model.
    pub content: Content,

    /// List of ratings for the safety of a response candidate.
    ///
    /// There is at most one rating per category.
    pub safety_ratings: Option<Vec<SafetyRating>>,

    /// Citation information for model-generated candidate.
    ///
    /// This field may be populated with recitation information for any text
    /// included in the [content]. These are passages that are "recited" from
    /// copyrighted material in the foundational LLM's training data.
    pub citation_metadata: Option<CitationMetadata>,

    /// The reason why the model stopped generating tokens.
    ///
    /// If None, the model has not stopped generating the tokens.
    pub finish_reason: Option<FinishReason>,

    pub grounding_attributions: Option<Vec<GroundingAtrribution>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SafetyRating {
    /// The category for this rating.
    category: HarmCategory,

    /// The probability of harm for this content.
    probability: HarmProbability,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HarmCategory {
    #[serde(rename = "HARM_CATEGORY_UNSPECIFIED")]
    Unspecified,
    /// Malicious, intimidating, bullying, or abusive comments targeting another
    /// individual.
    #[serde(rename = "HARM_CATEGORY_HARASSMENT")]
    Harassment,

    /// Negative or harmful comments targeting identity and/or protected
    /// attributes.
    #[serde(rename = "HARM_CATEGORY_HATE_SPEECH")]
    HateSpeech,

    /// Contains references to sexual acts or other lewd content.
    #[serde(rename = "HARM_CATEGORY_SEXUALLY_EXPLICIT")]
    SexuallyExplicit,

    /// Promotes or enables access to harmful goods, services, and activities.
    #[serde(rename = "HARM_CATEGORY_DANGEROUS_CONTENT")]
    DangerousContent,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmProbability {
    /// Probability is unspecified.
    Unspecified,

    /// Content has a negligible probability of being unsafe.
    Negligible,

    /// Content has a low probability of being unsafe.
    Low,

    /// Content has a medium probability of being unsafe.
    Medium,

    /// Content has a high probability of being unsafe.
    High,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    pub start_index: i32,
    pub end_index: i32,
    pub uri: String,
    pub license: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum FinishReason {
    #[serde(rename = "FINISH_REASON_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "STOP")]
    Stop,
    #[serde(rename = "MAX_TOKENS")]
    MaxTokens,
    #[serde(rename = "SAFTEY")]
    Safety,
    #[serde(rename = "RECITATION")]
    Recitation,
    #[serde(rename = "LANGUAGE")]
    Language,
    #[serde(rename = "OTHER")]
    Other,
    #[serde(rename = "BLOCKLIST")]
    BlockList,
    #[serde(rename = "PROHIBITED_CONTENT")]
    ProhibitedContent,
    SPII,
    #[serde(rename = "MALFORMED_FUNCTION_CALL")]
    MalformedFunctionCall,
}


/// Represents a chunk


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetySetting {
    /// The category for this setting.
    pub category: HarmCategory,
    pub threshold: HarmBlockThreshold,
}

/// Probability of harm which causes content to be blocked.
///
/// When provided in [SafetySetting.threshold], a predicted harm probability at
/// or above this level will block content from being returned.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HarmBlockThreshold {
    /// Threshold is unspecified, block using default threshold.
    #[serde(rename = "HARM_BLOCK_THRESHOLD_UNSPECIFIED")]
    Unspecified,

    /// Block when medium or high probability of unsafe content.
    #[serde(rename = "BLOCK_LOW_AND_ABOVE")]
    Low,

    /// Block when medium or high probability of unsafe content.
    #[serde(rename = "BLOCK_MEDIUM_AND_ABOVE")]
    Medium,

    /// Block when high probability of unsafe content.
    #[serde(rename = "BLOCK_ONLY_HIGH")]
    High,

    /// Always show regardless of probability of unsafe content.
    #[serde(rename = "BLOCK_NONE")]
    None,
}

#[derive(Debug, Deserialize)]
pub enum BlockReason {
    #[serde(rename = "BLOCK_REASON_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "SAFTEY")]
    Saftey,
    #[serde(rename = "OTHER")]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    pub block_reason: Option<BlockReason>,
    pub block_reason_message: Option<String>,
    pub saftey_ratings: Vec<SafetyRating>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    pub prompt_token_count: Option<i32>,
    pub candidates_token_count: Option<i32>,
    pub cached_content_token_count: Option<i32>,
    pub total_token_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ContentEmbedding {
    pub values: Vec<f64>,
}

pub struct CountTokenResponse {
    pub total_tokens: i32,
}

impl Candidate {
    pub fn text(&self) -> String {
        let text = self
            .content
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        text
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<ResponseMimeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The maximum number of tokens to include in a response candidate
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_logprobs: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename = "camelCase")]
pub enum ResponseMimeType {
    #[serde(rename = "text/plain")]
    TextPlain,
    #[serde(rename = "application/json")]
    ApplicationJson,
}

#[derive(Debug, Serialize, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskType {
    #[serde(rename = "TASK_TYPE_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "RETRIEVAL_QUERY")]
    RetrievalQuery,
    #[serde(rename = "RETRIEVAL_DOCUMENT")]
    RetrievalDocument,
    #[serde(rename = "SEMANTIC_SIMILARITY")]
    SemanticSimilarity,
    #[serde(rename = "CLASSIFICATION")]
    Classification,
    #[serde(rename = "CLUSTERING")]
    Clustering,
    #[serde(rename = "QUESTION_ANSWERING")]
    QuestionAnswering,
    #[serde(rename = "FACT_VERIFICATION")]
    FactVerification,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiGenericErrorResponse {
    pub(crate) error: GeminiGenericError,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiGenericError {
    pub code: i32,
    pub message: String,
    pub status: String,
}