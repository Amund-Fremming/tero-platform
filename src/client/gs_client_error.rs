use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum GSClientError {
    #[error("The game is full")]
    Full,

    #[error("The game has started")]
    Started,

    #[error("Http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Api error: {0} - {1}")]
    ApiError(StatusCode, String),

    #[error("Failed to serialize object: {0}")]
    Serialize(#[from] serde_json::Error),
}
