use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::warn;

use crate::error::{Error, Result};

const BASE_URL: &str = "https://api.semanticscholar.org";
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;

#[derive(Clone, Debug)]
pub(crate) struct HttpClient {
    http: reqwest::Client,
    base_url: String,
}

impl HttpClient {
    pub(crate) fn new(api_key: Option<&str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(key) = api_key {
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(key).map_err(|_| {
                    Error::InvalidParameter(
                        "API key contains invalid header characters (only visible ASCII allowed)"
                            .into(),
                    )
                })?,
            );
        }

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(Error::ClientBuild)?;

        Ok(Self {
            http,
            base_url: BASE_URL.to_string(),
        })
    }

    pub(crate) fn set_base_url(&mut self, url: impl Into<String>) {
        self.base_url = url.into();
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        self.request_with_retry(path, || self.http.get(&url).query(params))
            .await
    }

    async fn request_with_retry<T, F>(&self, endpoint: &str, build_request: F) -> Result<T>
    where
        T: DeserializeOwned,
        F: Fn() -> reqwest::RequestBuilder,
    {
        let mut backoff = INITIAL_BACKOFF_MS;

        for attempt in 0..=MAX_RETRIES {
            let response = build_request().send().await.map_err(|e| Error::Http {
                endpoint: endpoint.to_string(),
                source: e,
            })?;
            let status = response.status();

            if status.is_success() {
                let text = response.text().await.map_err(|e| Error::Http {
                    endpoint: endpoint.to_string(),
                    source: e,
                })?;
                return serde_json::from_str(&text).map_err(|e| Error::Deserialize {
                    endpoint: endpoint.to_string(),
                    source: e,
                });
            }

            let is_retryable = status.as_u16() == 429 || status.is_server_error();

            if is_retryable && attempt < MAX_RETRIES {
                let wait = if status.as_u16() == 429 {
                    response
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .map(|secs| secs * 1000)
                        .unwrap_or(backoff)
                } else {
                    backoff
                };
                drop(response);

                warn!(attempt, wait_ms = wait, %status, "Retrying request");
                tokio::time::sleep(Duration::from_millis(wait)).await;
                backoff *= 2;
                continue;
            }

            if status.as_u16() == 429 {
                return Err(Error::RateLimited {
                    endpoint: endpoint.to_string(),
                    retries: MAX_RETRIES,
                });
            }

            let message = response
                .text()
                .await
                .unwrap_or_else(|e| format!("(failed to read response body: {e})"));
            return Err(Error::Api {
                status: status.as_u16(),
                endpoint: endpoint.to_string(),
                message,
            });
        }

        Err(Error::RateLimited {
            endpoint: endpoint.to_string(),
            retries: MAX_RETRIES,
        })
    }
}
