use core::str;
use std::{borrow::Cow, fmt::Display};

use futures_util::{Stream, StreamExt};

use crate::{
    api::{GeminiGenericErrorResponse, GenerationConfig, SafetySetting, Tool},
    chat::ChatSession,
    content::Content,
    error::{GeminiError, GeminiErrorKind},
    EmbedContentConfig, EmbedContentRequest, EmbedContentResponse, GeminiRequest, GeminiResponse,
};

/// The base URL for the Gemini API.
pub static BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Represents a Generative Model instance.
#[derive(Debug, Clone)]
pub struct GenerativeModel {
    /// The API key used to authenticate requests.
    pub api_key: String,
    /// The specific Gemini model to use (e.g., Pro_1_5, Flash_1_5).
    pub model: GeminiModel,
    /// Optional configuration for content generation.
    pub generation_config: Option<GenerationConfig>,
    /// Optional instructions given to the model before the prompt.
    pub system_instruction: Option<Content>,
    /// Optional safety settings to control the content generated by the model.
    pub safety_settings: Option<Vec<SafetySetting>>,
    /// Optional tools that the model can use.
    pub tools: Option<Vec<Tool>>,
}

/// A builder for creating a `GenerativeModel`.
#[derive(Debug, Clone)]
pub struct GenerativeModelBuilder {
    pub api_key: Option<String>,
    pub model: Option<GeminiModel>,
    pub system_instruction: Option<Content>,
    pub safety_settings: Option<Vec<SafetySetting>>,
    pub generation_config: Option<GenerationConfig>,
    pub tools: Option<Vec<Tool>>,
}

impl GenerativeModelBuilder {
    /// Creates a new `GenerativeModelBuilder` with default values.
    pub fn new() -> Self {
        Self {
            api_key: None,
            model: None,
            system_instruction: None,
            safety_settings: None,
            generation_config: None,
            tools: None,
        }
    }

    /// Sets the API key for the `GenerativeModel`.
    pub fn api_key(&mut self, api_key: &str) -> &mut Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Sets the specific `GeminiModel` to be used.
    pub fn model(&mut self, model: GeminiModel) -> &mut Self {
        self.model = Some(model);
        self
    }

    /// Sets the system instruction for the `GenerativeModel`.
    pub fn system_instruction(&mut self, system_instruction: impl Into<Content>) -> &mut Self {
        self.system_instruction = Some(system_instruction.into());
        self
    }

    /// Sets the generation configuration for the `GenerativeModel`.
    pub fn generation_config(&mut self, config: GenerationConfig) -> &mut Self {
        self.generation_config = Some(config);
        self
    }

    /// Adds a safety setting to the `GenerativeModel`.
    pub fn safety_setting(&mut self, setting: SafetySetting) -> &mut Self {
        if let Some(ref mut x) = self.safety_settings {
            x.push(setting);
        } else {
            self.safety_settings = Some(vec![setting]);
        }
        self
    }

    /// Adds a tool to the `GenerativeModel`.
    pub fn tool(&mut self, tool: Tool) -> &mut Self {
        if let Some(ref mut x) = self.tools {
            x.push(tool);
        } else {
            self.tools = Some(vec![tool]);
        }
        self
    }

    /// Builds the `GenerativeModel` with the configured values.
    ///
    /// # Panics
    ///
    /// Panics if the `api_key` is not set.
    pub fn build(&mut self) -> GenerativeModel {
        GenerativeModel {
            api_key: self.api_key.take().expect("API key must be set"),
            model: self.model.take().unwrap_or_default(),
            generation_config: self.generation_config.take(),
            system_instruction: self.system_instruction.take(),
            safety_settings: self.safety_settings.take(),
            tools: self.tools.take(),
        }
    }
}

impl GenerativeModel {
    /// Starts a new chat session with the given history.
    pub fn start_chat(&self, history: Vec<Content>) -> ChatSession {
        ChatSession {
            model: self.clone(),
            history,
        }
    }

    /// Generates content based on the provided prompt.
    pub async fn generate_content(
        &self,
        prompt: Vec<Content>,
    ) -> Result<GeminiResponse, GeminiError> {
        self.generate_content_with(prompt, GenerativeModelBuilder::new())
            .await
    }

    /// Generates a stream of content responses based on the provided prompt.
    pub async fn generate_content_streamed(
        &self,
        prompt: Vec<Content>,
    ) -> Result<impl Stream<Item = Result<GeminiResponse, GeminiError>>, GeminiError> {
        self.generate_content_streamed_with(prompt, GenerativeModelBuilder::new())
            .await
    }

    /// Generates content based on the provided prompt, overriding some of the model's configurations using the provided builder.
    pub async fn generate_content_with(
        &self,
        prompt: Vec<Content>,
        config: GenerativeModelBuilder,
    ) -> Result<GeminiResponse, GeminiError> {
        let response = self.send_request(prompt, config, false).await?;

        let text = response.text().await.map_err(|err| GeminiError {
            kind: GeminiErrorKind::Other,
            message: err.to_string(),
        })?;

        if let Ok(response) = serde_json::from_str::<GeminiResponse>(&text) {
            Ok(response)
        } else {
            Err(serde_json::from_str::<GeminiGenericErrorResponse>(&text)
                .map(|x| GeminiError::from(x.error))
                .unwrap_or_else(|x| GeminiError::message(&x.to_string())))
        }
    }

    /// Generates a stream of content responses based on the provided prompt, overriding some of the model's configurations using the provided builder.
    pub async fn generate_content_streamed_with(
        &self,
        prompt: Vec<Content>,
        config: GenerativeModelBuilder,
    ) -> Result<impl Stream<Item = Result<GeminiResponse, GeminiError>>, GeminiError> {
        let response = self.send_request(prompt, config, true).await?;

        let stream = response.bytes_stream().filter_map(|chunk| async move {
            match chunk {
                Ok(chunk) => {
                    // we skip either '[' (which happens in the first chunk) or ',' in the subsequent chunks
                    let str = &str::from_utf8(&chunk)
                        .expect("Unexpected: this should not happen. Please report this bug to rusty-gemini repo.")[1..];

                    // in the last chunk, str should be empty
                    if str.is_empty() {
                        None
                    } else if let Ok(response) = serde_json::from_str::<GeminiResponse>(&str) {
                        Some(Ok(response))
                    } else {
                        Some(Err(serde_json::from_str::<GeminiGenericErrorResponse>(
                            &str,
                        )
                        .map(|x| GeminiError::from(x.error))
                        .unwrap_or_else(|err| GeminiError::message(&err.to_string()))))
                    }
                }
                Err(err) => Some(Err(GeminiError::message(&err.to_string()))),
            }
        });
        Ok(stream)
    }

    /// Embeds the content using the model's embedding capabilities.
    pub async fn embed_content(
        &self,
        content: impl Into<Content>,
        config: EmbedContentConfig,
    ) -> Result<EmbedContentResponse, GeminiError> {
        let content = content.into();
        let request = EmbedContentRequest { content, config };

        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "{BASE_URL}/models/{}:embedContent?key={}",
                self.model, self.api_key
            ))
            .json(&request)
            .send()
            .await
            .map_err(|err| GeminiError::message(&err.to_string()))?;

        let text = response
            .text()
            .await
            .map_err(|err| GeminiError::message(&err.to_string()))?;
        if let Ok(response) = serde_json::from_str::<EmbedContentResponse>(&text) {
            Ok(response)
        } else {
            Err(serde_json::from_str::<GeminiGenericErrorResponse>(&text)
                .map(|x| GeminiError::from(x.error))
                .unwrap_or_else(|x| GeminiError::message(&x.to_string())))
        }
    }

    async fn send_request(
        &self,
        prompt: Vec<Content>,
        config: GenerativeModelBuilder,
        stream: bool,
    ) -> Result<reqwest::Response, GeminiError> {
        let request = GeminiRequest {
            contents: prompt,
            tools: config.tools.or_else(|| self.tools.clone()),
            safety_settings: config
                .safety_settings
                .or_else(|| self.safety_settings.clone()),
            system_instruction: config
                .system_instruction
                .or_else(|| self.system_instruction.clone()),
            generation_config: config
                .generation_config
                .or_else(|| self.generation_config.clone()),
        };
        let client = reqwest::Client::new();
        let suffix = if stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };
        let response = client
            .post(format!(
                "{BASE_URL}/models/{}:{}?key={}",
                config.model.as_ref().unwrap_or(&self.model),
                suffix,
                self.api_key
            ))
            .json(&request)
            .send()
            .await
            .map_err(|err| GeminiError {
                kind: GeminiErrorKind::Other,
                message: err.to_string(),
            })?;
        Ok(response)
    }
}

/// Represents the different Gemini models available.
#[derive(Debug, Default, Clone)]
#[allow(non_camel_case_types)]
pub enum GeminiModel {
    /// The Gemini 1.5 Pro model.
    #[default]
    Pro_1_5,
    /// The Gemini 1.5 Flash model.
    Flash_1_5,
    /// The Gemini 1.5 Flash 8B model.
    Flash_1_5_8B,
    /// The Text Embedding 004 model.
    TextEmbedding004,
    /// A custom Gemini model specified by its name.
    Custom(Cow<'static, str>),
}

impl Display for GeminiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GeminiModel::Pro_1_5 => "gemini-1.5-pro",
                GeminiModel::Flash_1_5 => "gemini-1.5-flash",
                GeminiModel::Flash_1_5_8B => "gemini-1.5-flash-8b",
                GeminiModel::TextEmbedding004 => "text-embedding-004",
                GeminiModel::Custom(custom) => custom,
            }
        )
    }
}
