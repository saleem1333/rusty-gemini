use serde::{Deserialize, Serialize};

use crate::content::Content;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingAtrribution {
    pub source_id: AtrributionSourceId,
    pub content: Content,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingMetadata {
    pub grounding_chunk: GroundingChunk,
    pub grounding_supports: Vec<GroundingSupport>,
    pub web_search_queries: Vec<String>,
    pub search_entry_point: Option<SearchEntryPoint>,
    pub retrieval_metadata: RetrievalMetadata,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AtrributionSourceId {
    #[serde(rename_all = "camelCase")]
    GroundingPassageId {
        passage_id: String,
        part_index: String,
    },
    SemanticRetrieverChunk {
        source: String,
        chunk: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct SemanticRetrieverChunk {
    pub source: String,
    pub chunk: String,
}

#[derive(Debug, Deserialize)]
pub enum GroundingChunk {
    /// A chunk from the web
    #[serde(rename = "web")]
    Web {
        /// URI reference of the chunk
        uri: String,
        /// The title of the chunk
        title: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingSupport {
    pub grounding_chunk_indices: Vec<i32>,
    pub confidence_scores: Vec<f64>,
    pub segment: Segment,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEntryPoint {
    pub rendered_content: String,
    pub sdk_blob: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalMetadata {
    pub google_search_dynamic_retrieval_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub part_index: i32,
    pub start_index: i32,
    pub end_index: i32,
    pub text: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearchRetrieval {
    pub dynamic_retrieval_config: DynamicRetrievalConfig,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicRetrievalConfig {
    pub mode: Mode,
    pub dynamic_threshold: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub enum Mode {
    ModeUnspecified,
    ModeDynamic,
}
