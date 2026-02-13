use thiserror::Error;

/// Errors from the Semantic Scholar SDK.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to initialize the HTTP client.
    #[error("failed to build HTTP client: {0}")]
    ClientBuild(reqwest::Error),

    /// HTTP transport error during a request.
    #[error("HTTP request failed for `{endpoint}`: {source}")]
    Http {
        endpoint: String,
        source: reqwest::Error,
    },

    /// API returned an error response.
    #[error("API error ({status}) for `{endpoint}`: {message}")]
    Api {
        status: u16,
        endpoint: String,
        message: String,
    },

    /// Rate limited by the API (HTTP 429), retries exhausted.
    #[error("rate limited for `{endpoint}` after {retries} retries")]
    RateLimited { endpoint: String, retries: u32 },

    /// JSON deserialization failed.
    #[error("failed to deserialize response from `{endpoint}`: {source}")]
    Deserialize {
        endpoint: String,
        source: serde_json::Error,
    },

    /// Invalid parameter provided by the caller.
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result type alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
