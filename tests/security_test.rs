use clawduel_cli::security;

// --- detect_secret_leak tests ---

#[test]
fn detects_ethereum_private_key_0x_prefixed() {
    let key = format!("0x{}", "a".repeat(64));
    let result = security::detect_secret_leak(&key);
    assert!(result.is_some(), "should detect 0x-prefixed private key");
    assert!(result.unwrap().contains("private key") || result.unwrap().contains("Ethereum"));
}

#[test]
fn detects_ethereum_private_key_raw_hex() {
    let key = "a".repeat(64);
    let result = security::detect_secret_leak(&key);
    assert!(result.is_some(), "should detect raw hex private key");
}

#[test]
fn does_not_detect_normal_text() {
    let result = security::detect_secret_leak("Hello, this is normal text.");
    assert!(result.is_none(), "should not detect normal text as secret");
}

#[test]
fn detects_mnemonic_12_words() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let result = security::detect_secret_leak(mnemonic);
    assert!(result.is_some(), "should detect 12-word mnemonic");
    assert!(result.unwrap().contains("Mnemonic"));
}

#[test]
fn detects_mnemonic_24_words() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let result = security::detect_secret_leak(mnemonic);
    assert!(result.is_some(), "should detect 24-word mnemonic");
}

#[test]
fn detects_api_key_sk_prefix() {
    let key = "sk-abcdefghijklmnopqrst1234";
    let result = security::detect_secret_leak(key);
    assert!(result.is_some(), "should detect sk- API key");
    assert!(result.unwrap().contains("API key"));
}

#[test]
fn detects_api_key_sk_ant_prefix() {
    let key = "sk-ant-abcdefghijklmnopqrst1234";
    let result = security::detect_secret_leak(key);
    assert!(result.is_some(), "should detect sk-ant- API key");
}

#[test]
fn detects_extended_private_key_xprv() {
    let xprv = format!("xprv{}", "A".repeat(108));
    let result = security::detect_secret_leak(&xprv);
    assert!(result.is_some(), "should detect xprv extended key");
    assert!(result.unwrap().contains("Extended"));
}

#[test]
fn detects_aws_secret_key() {
    let aws = r#"aws_secret_key="AKIAIOSFODNN7EXAMPLEabcdefghij1234567890""#;
    let result = security::detect_secret_leak(aws);
    assert!(result.is_some(), "should detect AWS secret key");
    assert!(result.unwrap().contains("AWS"));
}

// --- assert_no_secret_leak tests ---

#[test]
fn assert_no_secret_leak_clean_body() {
    let result = security::assert_no_secret_leak(
        r#"{"nickname": "agent007"}"#,
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
    );
    assert!(result.is_ok(), "clean body should pass");
}

#[test]
fn assert_no_secret_leak_body_with_own_key() {
    let key = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let body = format!(r#"{{"data": "{}"}}"#, key);
    let result = security::assert_no_secret_leak(&body, key);
    assert!(result.is_err(), "body containing own key should be blocked");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("SECURITY BLOCKED"));
}

#[test]
fn assert_no_secret_leak_body_with_0x_key() {
    let key = "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let body = format!(r#"{{"data": "{}"}}"#, key);
    let result = security::assert_no_secret_leak(&body, key);
    assert!(
        result.is_err(),
        "body containing 0x-prefixed own key should be blocked"
    );
}

#[test]
fn assert_no_secret_leak_body_with_mnemonic() {
    let body = r#"{"data": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"}"#;
    let result = security::assert_no_secret_leak(
        body,
        "unrelatedkey1234567890abcdef1234567890abcdef1234567890abcdef1234",
    );
    assert!(
        result.is_err(),
        "body containing mnemonic should be blocked"
    );
}

// --- redact_secrets tests ---

#[test]
fn redact_secrets_replaces_0x_hex_key() {
    let key = format!("0x{}", "ab".repeat(32));
    let input = format!("Error: key was {}", key);
    let result = security::redact_secrets(&input, Some("cd".repeat(32).as_str()));
    assert!(!result.contains(&key), "should redact 0x-prefixed key");
    assert!(result.contains("[REDACTED_KEY]"));
}

#[test]
fn redact_secrets_replaces_own_key() {
    let own_key = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let input = format!("Error contained {}", own_key);
    let result = security::redact_secrets(&input, Some(own_key));
    assert!(!result.contains(own_key), "should redact own key");
    assert!(result.contains("[REDACTED_KEY]"));
}

#[test]
fn redact_secrets_replaces_sk_api_key() {
    let input = "API key: sk-abcdefghijklmnopqrstuvwxyz";
    let result = security::redact_secrets(input, None);
    assert!(!result.contains("sk-abcdefghijklmnopqrstuvwxyz"));
    assert!(result.contains("sk-[REDACTED]"));
}

#[test]
fn redact_secrets_replaces_xprv() {
    let xprv = format!("xprv{}", "B".repeat(60));
    let input = format!("key: {}", xprv);
    let result = security::redact_secrets(&input, None);
    assert!(!result.contains(&xprv));
    assert!(result.contains("xprv[REDACTED]"));
}

// --- validate_backend_url tests ---

#[test]
fn validate_backend_url_accepts_http_localhost() {
    let result = security::validate_backend_url("http://localhost:8787");
    assert!(result.is_ok());
}

#[test]
fn validate_backend_url_accepts_https() {
    let result = security::validate_backend_url("https://api.clawduel.ai");
    assert!(result.is_ok());
}

#[test]
fn validate_backend_url_rejects_ssrf_metadata() {
    let result = security::validate_backend_url("http://169.254.169.254/latest/meta-data");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("metadata"));
}

#[test]
fn validate_backend_url_rejects_ftp() {
    let result = security::validate_backend_url("ftp://evil.com");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("http"));
}

#[test]
fn validate_backend_url_rejects_not_a_url() {
    let result = security::validate_backend_url("not-a-url");
    assert!(result.is_err());
}

// --- sanitize_path_segment tests ---

#[test]
fn sanitize_path_segment_keeps_normal() {
    let result = security::sanitize_path_segment("normal-path_123");
    assert_eq!(result, "normal-path_123");
}

#[test]
fn sanitize_path_segment_strips_traversal() {
    let result = security::sanitize_path_segment("../etc/passwd");
    assert_eq!(result, "..etcpasswd");
}

#[test]
fn sanitize_path_segment_strips_special_chars() {
    let result = security::sanitize_path_segment("hello world!@#$");
    assert_eq!(result, "helloworld");
}
