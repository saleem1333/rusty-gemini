use crate::api::GeminiGenericError;

#[derive(Debug)]
pub struct GeminiError {
    pub kind: GeminiErrorKind,
    pub message: String,
}

impl GeminiError {
    pub(crate) fn message(msg: &str) -> Self {
        Self {
            kind: GeminiErrorKind::Other,
            message: msg.to_string(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum GeminiErrorKind {
    InvalidArgument,
    UnsupportedCountry,
    PermissionDenied,
    ResourceExhausted,
    Internal,
    ServiceUnavailable,
    /// This can be returned due to errors in t serialization etc
    /// And not necessarily by the Gemini API
    Other,
}

impl From<GeminiGenericError> for GeminiError {
    fn from(value: GeminiGenericError) -> Self {
        let kind = match value.status.as_str() {
            "INVALID_ARGUMENT" => GeminiErrorKind::InvalidArgument,
            "FAILED_PRECONDITION" => GeminiErrorKind::UnsupportedCountry,
            "PERMISSION_DENIED" => GeminiErrorKind::PermissionDenied,
            "RESOURCE_EXHAUSTED" => GeminiErrorKind::ResourceExhausted,
            "INTERNAL" => GeminiErrorKind::Internal,
            "UNAVAILABLE" => GeminiErrorKind::ServiceUnavailable,
            _ => GeminiErrorKind::Other,
        };

        let message = value.message;
        Self { kind, message }
    }
}
