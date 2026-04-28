use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::Signature;
use alloy::signers::local::PrivateKeySigner;

use clawduel_cli::auth;
use clawduel_cli::security;

// --- validate_timestamp tests ---

#[test]
fn validate_timestamp_accepts_current_time() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let result = security::validate_timestamp(now);
    assert!(result.is_ok(), "current timestamp should be valid");
}

#[test]
fn validate_timestamp_rejects_6_minute_drift() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let six_min_ago = now - (6 * 60 * 1000);
    let result = security::validate_timestamp(six_min_ago);
    assert!(result.is_err(), "6 minute drift should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("drift") || err.contains("timestamp"));
}

#[test]
fn validate_timestamp_accepts_4_minute_drift() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let four_min_ago = now - (4 * 60 * 1000);
    let result = security::validate_timestamp(four_min_ago);
    assert!(result.is_ok(), "4 minute drift should be acceptable");
}

// --- auth_headers tests ---

#[tokio::test]
async fn auth_headers_contains_required_fields() {
    let signer = PrivateKeySigner::random();
    let address = signer.address();

    let headers = auth::auth_headers(&signer, &address).await.unwrap();

    assert!(headers.contains_key("Content-Type"));
    assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
    assert!(headers.contains_key("X-Agent-Address"));
    assert!(headers.contains_key("X-Agent-Signature"));
    assert!(headers.contains_key("X-Agent-Timestamp"));
}

#[tokio::test]
async fn auth_headers_address_matches_signer() {
    let signer = PrivateKeySigner::random();
    let address = signer.address();

    let headers = auth::auth_headers(&signer, &address).await.unwrap();
    let header_address = headers.get("X-Agent-Address").unwrap();

    assert_eq!(
        header_address.to_lowercase(),
        format!("{:?}", address).to_lowercase(),
        "X-Agent-Address should match signer address"
    );
}

#[tokio::test]
async fn auth_headers_signature_is_valid_eip191() {
    let signer = PrivateKeySigner::random();
    let address = signer.address();

    let headers = auth::auth_headers(&signer, &address).await.unwrap();

    let sig_hex = headers.get("X-Agent-Signature").unwrap();
    let timestamp = headers.get("X-Agent-Timestamp").unwrap();
    let addr_str = headers.get("X-Agent-Address").unwrap();

    // Reconstruct the message that was signed
    let message = format!("ClawDuel:auth:{}:{}", addr_str.to_lowercase(), timestamp);

    // Parse the signature and recover the address
    let sig_bytes = hex::decode(sig_hex.strip_prefix("0x").unwrap_or(sig_hex)).unwrap();
    let sig = Signature::try_from(sig_bytes.as_slice()).unwrap();

    let recovered = sig
        .recover_address_from_msg(message.as_bytes())
        .expect("should recover address from EIP-191 signature");

    assert_eq!(
        recovered, address,
        "recovered address should match signer address"
    );
}

// --- HttpClient tests ---

#[tokio::test]
async fn http_client_rejects_invalid_backend_url() {
    let signer = PrivateKeySigner::random();
    let address = signer.address();
    let pk_hex = hex::encode(signer.to_bytes());

    let result = clawduel_cli::http::HttpClient::new("ftp://evil.com", signer, address, &pk_hex);
    assert!(result.is_err(), "should reject non-http URL");
}

#[tokio::test]
async fn http_client_accepts_valid_backend_url() {
    let signer = PrivateKeySigner::random();
    let address = signer.address();
    let pk_hex = hex::encode(signer.to_bytes());

    let result =
        clawduel_cli::http::HttpClient::new("http://localhost:8787", signer, address, &pk_hex);
    assert!(result.is_ok(), "should accept valid http URL");
}

#[tokio::test]
async fn http_client_post_blocks_secret_in_body() {
    let signer = PrivateKeySigner::random();
    let address = signer.address();
    let pk_hex = hex::encode(signer.to_bytes());

    let client =
        clawduel_cli::http::HttpClient::new("http://localhost:8787", signer, address, &pk_hex)
            .unwrap();

    // POST a body containing the private key - should be blocked BEFORE sending
    let body = serde_json::json!({ "data": pk_hex });
    let result = client.post("/test", &body).await;
    assert!(result.is_err(), "should block POST with secret in body");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("SECURITY BLOCKED"),
        "error should mention security block"
    );
}

#[tokio::test]
async fn http_client_post_blocks_mnemonic_in_body() {
    let signer = PrivateKeySigner::random();
    let address = signer.address();
    let pk_hex = hex::encode(signer.to_bytes());

    let client =
        clawduel_cli::http::HttpClient::new("http://localhost:8787", signer, address, &pk_hex)
            .unwrap();

    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let body = serde_json::json!({ "data": mnemonic });
    let result = client.post("/test", &body).await;
    assert!(result.is_err(), "should block POST with mnemonic in body");
}
