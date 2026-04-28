//! Security module: secret leak detection, redaction, SSRF protection, path sanitization.
//!
//! Replicates all 7 secret detection patterns from the TypeScript CLI.

use std::sync::LazyLock;

use anyhow::{Result, bail};
use regex::Regex;
use url::Url;

/// Maximum allowed timestamp drift for auth (5 minutes).
const MAX_TIMESTAMP_DRIFT_MS: u64 = 5 * 60 * 1000;

// --- Detection patterns (all 7 from TypeScript CLI) ---

static RE_ETH_KEY_0X: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|[^a-fA-F0-9])0x[0-9a-fA-F]{64}(?:[^a-fA-F0-9]|$)").unwrap());
static RE_ETH_KEY_RAW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|[^a-fA-F0-9x])[0-9a-fA-F]{64}(?:[^a-fA-F0-9]|$)").unwrap());
static RE_MNEMONIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:[a-z]{3,8}\s){11,23}[a-z]{3,8}").unwrap());
static RE_XPRV: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"xprv[a-zA-Z0-9]{107,108}").unwrap());
static RE_SK_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"sk-[a-zA-Z0-9_\-]{20,}").unwrap());
static RE_SK_ANT_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"sk-ant-[a-zA-Z0-9_\-]{20,}").unwrap());
static RE_AWS_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?:AWS|aws).{0,20}['"][0-9a-zA-Z/+=]{40}['"]"#).unwrap());

// --- Redaction patterns ---

static RE_REDACT_0X_HEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"0x[0-9a-fA-F]{64}").unwrap());
static RE_REDACT_RAW_HEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|[^0-9a-fA-F])[0-9a-fA-F]{64}(?:[^0-9a-fA-F]|$)").unwrap());
static RE_REDACT_SK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"sk-[a-zA-Z0-9_\-]{20,}").unwrap());
static RE_REDACT_SK_ANT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"sk-ant-[a-zA-Z0-9_\-]{20,}").unwrap());
static RE_REDACT_XPRV: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"xprv[a-zA-Z0-9]{50,}").unwrap());

// --- Path sanitization ---

static RE_SAFE_PATH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9\-_.]").unwrap());

/// Named detection patterns, checked in order (matching TypeScript CLI).
const PATTERN_NAMES: &[&str] = &[
    "Ethereum private key (0x-prefixed)",
    "Ethereum private key (raw hex)",
    "Mnemonic seed phrase",
    "Extended private key (xprv)",
    "API key (sk- prefix)",
    "API key (sk-ant- prefix)",
    "AWS secret key",
];

/// Detect if data contains a secret pattern. Returns the pattern name if found.
pub fn detect_secret_leak(data: &str) -> Option<&'static str> {
    let patterns: &[&LazyLock<Regex>] = &[
        &RE_ETH_KEY_0X,
        &RE_ETH_KEY_RAW,
        &RE_MNEMONIC,
        &RE_XPRV,
        &RE_SK_KEY,
        &RE_SK_ANT_KEY,
        &RE_AWS_KEY,
    ];

    for (i, re) in patterns.iter().enumerate() {
        if re.is_match(data) {
            return Some(PATTERN_NAMES[i]);
        }
    }
    None
}

/// Assert that the body does not contain any secret patterns or the agent's own key.
///
/// Checks exact match of the agent's own key first (both with and without 0x prefix),
/// then checks pattern-based detection.
pub fn assert_no_secret_leak(body: &str, private_key: &str) -> Result<()> {
    // Exact match against the agent's own private key
    let raw_key = private_key.strip_prefix("0x").unwrap_or(private_key);
    if body.contains(raw_key) {
        bail!(
            "SECURITY BLOCKED: Request body contains the agent's own private key. Request was NOT sent."
        );
    }

    // Pattern-based detection
    if let Some(name) = detect_secret_leak(body) {
        bail!(
            "SECURITY BLOCKED: Request body appears to contain a secret ({name}). Request was NOT sent."
        );
    }

    Ok(())
}

/// Redact secrets from input text.
///
/// If `private_key` is provided, redacts exact matches of the key first,
/// then applies pattern-based redaction.
pub fn redact_secrets(input: &str, private_key: Option<&str>) -> String {
    let mut result = input.to_string();

    // Redact the agent's exact private key first (both with and without 0x)
    if let Some(pk) = private_key {
        let raw_key = pk.strip_prefix("0x").unwrap_or(pk);
        let full_key = if pk.starts_with("0x") {
            pk.to_string()
        } else {
            format!("0x{pk}")
        };
        result = result.replace(raw_key, "[REDACTED_KEY]");
        result = result.replace(&full_key, "0x[REDACTED_KEY]");
    }

    // Redact hex strings that look like private keys
    result = RE_REDACT_0X_HEX
        .replace_all(&result, "0x[REDACTED_KEY]")
        .into_owned();
    // For raw hex, the regex captures boundary chars, so we need to preserve them.
    result = RE_REDACT_RAW_HEX
        .replace_all(&result, |caps: &regex::Captures| {
            let m = caps.get(0).unwrap().as_str();
            // Find where the 64-char hex starts/ends and preserve boundary chars
            let hex_start = m.find(|c: char| c.is_ascii_hexdigit()).unwrap_or(0);
            let prefix = &m[..hex_start];
            let suffix_start = hex_start + 64;
            let suffix = if suffix_start < m.len() {
                &m[suffix_start..]
            } else {
                ""
            };
            format!("{prefix}[REDACTED_HEX]{suffix}")
        })
        .into_owned();

    // Redact API keys (sk-ant- before sk- since sk-ant- is more specific)
    result = RE_REDACT_SK_ANT
        .replace_all(&result, "sk-ant-[REDACTED]")
        .into_owned();
    result = RE_REDACT_SK
        .replace_all(&result, "sk-[REDACTED]")
        .into_owned();

    // Redact extended private keys
    result = RE_REDACT_XPRV
        .replace_all(&result, "xprv[REDACTED]")
        .into_owned();

    result
}

/// Validate that a URL is safe to use as a backend URL.
///
/// Enforces http/https protocol and blocks cloud metadata endpoints (SSRF vector).
pub fn validate_backend_url(url_str: &str) -> Result<()> {
    let parsed =
        Url::parse(url_str).map_err(|_| anyhow::anyhow!("Invalid backend URL: {url_str}"))?;

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        bail!("Backend URL must use http or https protocol, got: {scheme}:");
    }

    // Block cloud metadata endpoints (SSRF vector)
    if let Some(host) = parsed.host_str() {
        if host == "169.254.169.254" {
            bail!("Backend URL must not point to cloud metadata endpoints");
        }
    }

    Ok(())
}

/// Sanitize a path segment, keeping only `[a-zA-Z0-9\-_.]`.
pub fn sanitize_path_segment(segment: &str) -> String {
    RE_SAFE_PATH.replace_all(segment, "").into_owned()
}

/// Validate that a timestamp is within acceptable drift from the current time.
pub fn validate_timestamp(timestamp_ms: u64) -> Result<()> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_millis() as u64;

    let drift = now_ms.abs_diff(timestamp_ms);
    if drift > MAX_TIMESTAMP_DRIFT_MS {
        bail!(
            "Auth timestamp is too far from current time (drift: {drift}ms). Clock may be out of sync."
        );
    }

    Ok(())
}
