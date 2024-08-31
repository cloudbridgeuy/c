use eventsource_client as es;
use thiserror::Error;

/// Error type returned from this library's functions
#[derive(Debug, Error)]
pub enum Error {
    /// An error when creating the SSE stream.
    #[error("sse stream creation error: {0}")]
    SseStreamCreation(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
    /// An Error returned by the API
    #[error("AuthError Error: {0}")]
    AuthError(String),
    /// An Error returned by the API
    #[error("API Error: {0}")]
    ApiError(String),
    /// An Error not related to the API
    #[error("Request Error: {0}")]
    RequestError(String),
    /// De/serialization error
    #[error("de/serialize error: {0}")]
    Serde(#[from] serde_json::error::Error),
    /// An Error occurred when performing an IO operation.
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    /// EventSource error.
    #[error("eventsource error: {0}")]
    ES(#[source] es::Error),
}

impl From<es::Error> for Error {
    fn from(error: es::Error) -> Self {
        Self::ES(error)
    }
}
