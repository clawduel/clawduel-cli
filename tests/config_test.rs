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
fn resolve_backend_url_default_when_nothing_set() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { unset("CLAW_BACKEND_URL") };
    let url = config::resolve_backend_url(None);
    assert_eq!(url, "http://localhost:3001");
}

#[test]
fn resolve_backend_url_env_overrides_default() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { set("CLAW_BACKEND_URL", "https://api.clawduel.ai") };
    let url = config::resolve_backend_url(None);
    assert_eq!(url, "https://api.clawduel.ai");
    unsafe { unset("CLAW_BACKEND_URL") };
}

#[test]
fn resolve_backend_url_flag_overrides_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { set("CLAW_BACKEND_URL", "https://api.clawduel.ai") };
    let url = config::resolve_backend_url(Some("https://custom.url"));
    assert_eq!(url, "https://custom.url");
    unsafe { unset("CLAW_BACKEND_URL") };
}

#[test]
fn resolve_rpc_url_default() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { unset("CLAW_RPC_URL") };
    let url = config::resolve_rpc_url(None);
    assert_eq!(url, "http://localhost:8545");
}

#[test]
fn resolve_rpc_url_flag_overrides_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { set("CLAW_RPC_URL", "https://rpc.example.com") };
    let url = config::resolve_rpc_url(Some("https://custom-rpc.url"));
    assert_eq!(url, "https://custom-rpc.url");
    unsafe { unset("CLAW_RPC_URL") };
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

    let original = config::Config {
        backend_url: Some("https://api.test.com".to_string()),
        rpc_url: Some("https://rpc.test.com".to_string()),
        agent_address: Some("0xdeadbeef".to_string()),
    };

    config::save_config_to(&original, &path).unwrap();
    let loaded = config::load_config_from(&path).unwrap().expect("config should exist");

    assert_eq!(loaded.backend_url.as_deref(), Some("https://api.test.com"));
    assert_eq!(loaded.rpc_url.as_deref(), Some("https://rpc.test.com"));
    assert_eq!(loaded.agent_address.as_deref(), Some("0xdeadbeef"));
}

#[test]
fn non_interactive_when_env_set() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { set("CLAW_NON_INTERACTIVE", "1") };
    assert!(!config::is_interactive());
    unsafe { unset("CLAW_NON_INTERACTIVE") };
}

#[test]
fn resolve_agent_address_flag_overrides_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { set("CLAW_AGENT_ADDRESS", "0xenv_addr") };
    let addr = config::resolve_agent_address(Some("0xflag_addr"));
    assert_eq!(addr, Some("0xflag_addr".to_string()));
    unsafe { unset("CLAW_AGENT_ADDRESS") };
}

#[test]
fn resolve_agent_address_env_when_no_flag() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { set("CLAW_AGENT_ADDRESS", "0xenv_addr") };
    let addr = config::resolve_agent_address(None);
    assert_eq!(addr, Some("0xenv_addr".to_string()));
    unsafe { unset("CLAW_AGENT_ADDRESS") };
}

#[test]
fn resolve_agent_address_none_when_nothing_set() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { unset("CLAW_AGENT_ADDRESS") };
    let addr = config::resolve_agent_address(None);
    assert!(addr.is_none());
}
