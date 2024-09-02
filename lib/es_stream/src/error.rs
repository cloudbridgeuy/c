use thiserror::Error;

/// Error type returned from this library's functions
#[derive(Debug, Error)]
pub enum Error {
    /// An error when creating the SSE stream.
    #[error("Eventsource Client error: {0}")]
    EventsourceClient(#[from] eventsource_client::Error),
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
}
