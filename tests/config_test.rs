use std::collections::HashMap;
use std::sync::Mutex;

use clawduel_cli::config;

// Mutex to serialize env var tests (set_var is not thread-safe).
static ENV_LOCK: Mutex<()> = Mutex::new(());

// SAFETY: These tests run single-threaded (--test-threads=1) and hold ENV_LOCK.
unsafe fn set(var: &str, val: &str) {
    unsafe { std::env::set_var(var, val) };
}

unsafe fn unset(var: &str) {
    unsafe { std::env::remove_var(var) };
}

#[test]
fn load_config_returns_none_when_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.json");
    let result = config::load_config_from(&path).unwrap();
    assert!(result.is_none());
}

#[test]
fn save_then_load_config_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");

    let mut wallets = HashMap::new();
    wallets.insert("0xabc".to_string(), "0xkey123".to_string());

    let original = config::Config { wallets };

    config::save_config_to(&original, &path).unwrap();
    let loaded = config::load_config_from(&path).unwrap().expect("config should exist");

    assert_eq!(loaded.wallets.get("0xabc").map(|s| s.as_str()), Some("0xkey123"));
}

#[test]
fn non_interactive_when_env_set() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { set("CLAW_NON_INTERACTIVE", "1") };
    assert!(!config::is_interactive());
    unsafe { unset("CLAW_NON_INTERACTIVE") };
}

#[test]
fn backend_url_constant() {
    assert_eq!(config::BACKEND_URL, "https://staging-api.clawduel.ai");
}

#[test]
fn rpc_url_constant() {
    assert_eq!(config::RPC_URL, "https://rpc.sepolia.org");
}

#[test]
fn config_omits_empty_wallets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");

    let config = config::Config::default();
    config::save_config_to(&config, &path).unwrap();

    let data = std::fs::read_to_string(&path).unwrap();
    assert!(!data.contains("wallets"));
}
