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
pub static BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

#[derive(Debug, Clone)]
pub struct GenerativeModel {
    pub api_key: String,
    pub model: GeminiModel,
    pub generation_config: Option<GenerationConfig>,
    pub system_instruction: Option<Content>,
    pub safety_settings: Option<Vec<SafetySetting>>,
    pub tools: Option<Vec<Tool>>,
}

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
    pub fn api_key(&mut self, api_key: &str) -> &mut Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    pub fn model(&mut self, model: GeminiModel) -> &mut Self {
        self.model = Some(model);
        self
    }

    pub fn system_instruction(&mut self, system_instruction: impl Into<Content>) -> &mut Self {
        self.system_instruction = Some(system_instruction.into());
        self
    }

    pub fn generation_config(&mut self, config: GenerationConfig) -> &mut Self {
        self.generation_config = Some(config);
        self
    }

    pub fn safety_setting(&mut self, setting: SafetySetting) -> &mut Self {
        if let Some(ref mut x) = self.safety_settings {
            x.push(setting);
        }
        self
    }

    pub fn tool(&mut self, tool: Tool) -> &mut Self {
        if let Some(ref mut x) = self.tools {
            x.push(tool);
        }
        self
    }

    pub fn build(&mut self) -> GenerativeModel {
        GenerativeModel {
            api_key: self.api_key.take().unwrap(),
            model: self.model.take().unwrap(),
            generation_config: self.generation_config.take(),
            system_instruction: self.system_instruction.take(),
            safety_settings: self.safety_settings.take(),
            tools: self.tools.take(),
        }
    }
}

impl GenerativeModel {
    pub fn start_chat(&self, history: Vec<Content>) -> ChatSession {
        ChatSession {
            model: self.clone(),
            history,
        }
    }
    pub async fn generate_content(
        &self,
        prompt: Vec<Content>,
    ) -> Result<GeminiResponse, GeminiError> {
        // equivalent to passing an empty builder to generate_content_with
        self.generate_content_with(prompt, GenerativeModelBuilder::new())
            .await
    }

    pub async fn generate_content_streamed(
        &self,
        prompt: Vec<Content>,
    ) -> impl Stream<Item = Result<GeminiResponse, GeminiError>> {
        self.generate_content_streamed_with(prompt, GenerativeModelBuilder::new())
            .await
    }

    // Only overrides the some values of builder, useful for things like temporary generating with a specific model
    pub async fn generate_content_with(
        &self,
        prompt: Vec<Content>,
        config: GenerativeModelBuilder,
    ) -> Result<GeminiResponse, GeminiError> {
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
        let response = client
            .post(format!(
                "{BASE_URL}/models/{}:generateContent?key={}",
                config.model.as_ref().unwrap_or(&self.model),
                self.api_key
            ))
            .json(&request)
            .send()
            .await
            .map_err(|err| GeminiError {
                kind: GeminiErrorKind::Other,
                message: err.to_string(),
            })?;

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

    pub async fn generate_content_streamed_with(
        &self,
        prompt: Vec<Content>,
        config: GenerativeModelBuilder,
    ) -> impl Stream<Item = Result<GeminiResponse, GeminiError>> {
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
        let response = client
            .post(format!(
                "{BASE_URL}/models/{}:streamGenerateContent?key={}",
                config.model.as_ref().unwrap_or(&self.model),
                self.api_key
            ))
            .json(&request)
            .send()
            .await
            .map_err(|err| GeminiError {
                kind: GeminiErrorKind::Other,
                message: err.to_string(),
            })
            .unwrap();

        let stream = response.bytes_stream().filter_map(|chunk| async move {
            let chunk = chunk.unwrap();

            // we skip either '[' (which happens in the first chunk) and ',' in the subsequent chunks
            let str = &str::from_utf8(&chunk).unwrap()[1..];

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
                .unwrap_or_else(|err| GeminiError {
                    kind: GeminiErrorKind::Other,
                    message: err.to_string(),
                })))
            }
        });
        stream
    }

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
}

#[derive(Debug, Default, Clone)]
#[allow(non_camel_case_types)]
pub enum GeminiModel {
    #[default]
    Pro_1_5,
    Flash_1_5,
    Flash_1_5_8B,
    TextEmbedding004,
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
