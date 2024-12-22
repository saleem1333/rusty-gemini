use crate::{content::Content, error::GeminiError, model::GenerativeModel, GeminiResponse};

pub struct ChatSession {
    pub(crate) model: GenerativeModel,
    pub(crate) history: Vec<Content>,
}

impl ChatSession {
    pub async fn send_message(&mut self, content: Content) -> Result<GeminiResponse, GeminiError> {
        self.history.push(content);
        let response = self.model.generate_content(self.history.clone()).await;
        if let Ok(ref response) = response {
            self.history.push(response.candidates[0].content.clone());
        }
        response
    }
    // pub async fn send_message_streamed(&mut self, content: Content) -> GeminiResponse {
    //     self.history.push(content);
    //     self.model.generate_content(self.history.clone()).await
    // }
}
