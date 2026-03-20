//! EIP-191 authentication header generation for ClawDuel backend.
//!
//! Signs messages as: `ClawDuel:auth:<address_lowercase>:<timestamp>`

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::Address;
use alloy::signers::Signer;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result};

use crate::security;

/// Generate auth headers with EIP-191 signature for ClawDuel backend.
///
/// Returns a `HashMap` with:
/// - `Content-Type: application/json`
/// - `X-Agent-Address: <checksummed address>`
/// - `X-Agent-Signature: <0x-prefixed EIP-191 signature>`
/// - `X-Agent-Timestamp: <ms since epoch>`
pub async fn auth_headers(
    signer: &PrivateKeySigner,
    address: &Address,
) -> Result<HashMap<String, String>> {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_millis() as u64;

    // Validate timestamp is reasonable (catches badly-skewed clocks)
    security::validate_timestamp(timestamp_ms)?;

    let timestamp_str = timestamp_ms.to_string();
    let address_str = format!("{:?}", address);
    let message = format!(
        "ClawDuel:auth:{}:{}",
        address_str.to_lowercase(),
        timestamp_str
    );

    let signature = signer
        .sign_message(message.as_bytes())
        .await
        .context("Failed to sign auth message")?;

    let sig_hex = format!("0x{}", hex::encode(signature.as_bytes()));

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("X-Agent-Address".to_string(), address_str);
    headers.insert("X-Agent-Signature".to_string(), sig_hex);
    headers.insert("X-Agent-Timestamp".to_string(), timestamp_str);

    Ok(headers)
}
