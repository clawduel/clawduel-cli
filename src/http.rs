//! Authenticated HTTP client for ClawDuel backend.
//!
//! All requests include EIP-191 auth headers, POST bodies are scanned for secrets,
//! error responses are redacted, and a 30-second timeout is enforced.

use std::time::Duration;

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result, bail};
use reqwest::Method;
use serde::Serialize;

use crate::{auth, security};

/// Request timeout matching the TypeScript CLI `REQUEST_TIMEOUT_MS = 30_000`.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Authenticated HTTP client with timeout, secret scanning, and SSRF protection.
pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
    signer: PrivateKeySigner,
    address: Address,
    private_key_hex: String,
}

impl HttpClient {
    /// Create a new `HttpClient`.
    ///
    /// Validates the backend URL against SSRF vectors on construction.
    pub fn new(
        base_url: &str,
        signer: PrivateKeySigner,
        address: Address,
        private_key_hex: &str,
    ) -> Result<Self> {
        security::validate_backend_url(base_url)?;

        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            signer,
            address,
            private_key_hex: private_key_hex.to_string(),
        })
    }

    /// Build the full URL for a path.
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Inject auth headers into a request builder.
    async fn with_auth(
        &self,
        mut builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::RequestBuilder> {
        let headers = auth::auth_headers(&self.signer, &self.address).await?;
        for (key, value) in &headers {
            builder = builder.header(key, value);
        }
        Ok(builder)
    }

    /// Redact error body from a response if status >= 400.
    fn redact_error(&self, body: &mut serde_json::Value) {
        if let Some(error) = body.get("error").and_then(|e| e.as_str()) {
            let redacted = security::redact_secrets(error, Some(&self.private_key_hex));
            body["error"] = serde_json::Value::String(redacted);
        }
    }

    /// Send a GET request with auth headers.
    ///
    /// Returns the parsed JSON response body.
    pub async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let url = self.url(path);
        let builder = self.client.get(&url);
        let builder = self.with_auth(builder).await?;

        let response = builder.send().await.context("GET request failed")?;
        let status = response.status();
        let mut body: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse response JSON")?;

        if status.is_client_error() || status.is_server_error() {
            self.redact_error(&mut body);
            let error = body
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error");
            bail!("GET {path} failed ({status}): {error}");
        }

        Ok(body)
    }

    /// Send a POST request with secret scanning and auth headers.
    ///
    /// The body is serialized and scanned for secrets BEFORE sending.
    /// Returns `(status_code, response_body)`.
    pub async fn post(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<(u16, serde_json::Value)> {
        self.request(Method::POST, path, Some(body)).await
    }

    /// Send a DELETE request with optional body.
    ///
    /// Used by the dequeue command.
    pub async fn delete(
        &self,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<(u16, serde_json::Value)> {
        self.request(Method::DELETE, path, body).await
    }

    /// Generic request method for POST, DELETE, etc.
    ///
    /// If a body is provided, it is serialized and scanned for secrets before sending.
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<(u16, serde_json::Value)> {
        let url = self.url(path);

        // Serialize and scan body for secrets BEFORE sending
        let serialized_body = if let Some(b) = body {
            let json = serde_json::to_string(b).context("Failed to serialize request body")?;
            security::assert_no_secret_leak(&json, &self.private_key_hex)?;
            Some(json)
        } else {
            None
        };

        let mut builder = self.client.request(method, &url);
        builder = self.with_auth(builder).await?;

        if let Some(json) = serialized_body {
            builder = builder
                .body(json)
                .header("Content-Type", "application/json");
        }

        let response = builder.send().await.context("Request failed")?;
        let status = response.status().as_u16();
        let mut response_body: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse response JSON")?;

        if status >= 400 {
            self.redact_error(&mut response_body);
        }

        Ok((status, response_body))
    }
}
