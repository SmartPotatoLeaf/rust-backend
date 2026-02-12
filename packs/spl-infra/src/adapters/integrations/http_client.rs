use reqwest::{Client, Response};
use spl_shared::error::{AppError, Result};
use std::time::Duration;

/// HTTP client with retry logic for external integrations
pub struct RetryableHttpClient {
    client: Client,
    max_retries: u32,
    timeout: Duration,
}

impl RetryableHttpClient {
    pub fn new(max_retries: u32, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            max_retries,
            timeout,
        }
    }

    pub async fn get(&self, url: &str) -> Result<Response> {
        self.execute_with_retry(|| self.client.get(url).send())
            .await
    }

    pub async fn post(&self, url: &str, body: &[u8]) -> Result<Response> {
        self.execute_with_retry(|| {
            self.client
                .post(url)
                .header("Content-Type", "application/json")
                .body(body.to_vec())
                .send()
        })
        .await
    }

    async fn execute_with_retry<F, Fut>(&self, request_fn: F) -> Result<Response>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = reqwest::Result<Response>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match request_fn().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        return Err(AppError::IntegrationError {
                            integration: "http_client".to_string(),
                            message: format!(
                                "HTTP {}: {}",
                                response.status(),
                                response.status().canonical_reason().unwrap_or("Unknown")
                            ),
                        });
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        // Exponential backoff
                        let backoff = Duration::from_millis(100 * 2_u64.pow(attempt));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(map_reqwest_error(last_error.unwrap()))
    }
}

fn map_reqwest_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::IntegrationTimeout("HTTP request timed out".to_string())
    } else if error.is_connect() {
        AppError::IntegrationUnavailable("Failed to connect to external service".to_string())
    } else {
        AppError::IntegrationError {
            integration: "http_client".to_string(),
            message: error.to_string(),
        }
    }
}
