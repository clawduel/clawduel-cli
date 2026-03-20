use std::sync::Mutex;

use clawduel_cli::wallet;

// Mutex to serialize env var tests.
static ENV_LOCK: Mutex<()> = Mutex::new(());

unsafe fn set(var: &str, val: &str) {
    unsafe { std::env::set_var(var, val) };
}

unsafe fn unset(var: &str) {
    unsafe { std::env::remove_var(var) };
}

#[test]
fn create_keystore_produces_file_and_returns_address() {
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    let (address, path) = wallet::create_keystore("test-password", &keystores_dir).unwrap();

    // Address should be a valid hex string
    assert!(address.starts_with("0x"), "address should start with 0x");
    assert_eq!(address.len(), 42, "address should be 42 chars");

    // File should exist
    assert!(path.exists(), "keystore file should exist at {:?}", path);

    // File should be valid JSON
    let contents = std::fs::read_to_string(&path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&contents).unwrap();
}

#[test]
fn import_keystore_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    // Use a known test private key
    let test_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let password = "test-password";

    let (address, path) = wallet::import_keystore(test_key, password, &keystores_dir).unwrap();

    // Address for this well-known key
    assert_eq!(
        address.to_lowercase(),
        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
        "address should match the known key"
    );

    // File should exist
    assert!(path.exists());

    // Decrypt should yield the same address
    let decrypted = wallet::decrypt_keystore(&path, password).unwrap();
    assert_eq!(
        format!("{:?}", decrypted.address()).to_lowercase(),
        address.to_lowercase(),
    );
}

#[test]
fn select_keystore_auto_selects_single() {
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    let (address, _) = wallet::create_keystore("pw", &keystores_dir).unwrap();

    let selected = wallet::select_keystore(None, &keystores_dir).unwrap();
    assert!(selected.is_some(), "should auto-select the only keystore");

    // Verify it matches the address we created
    let selected_path = selected.unwrap();
    let filename = selected_path.file_stem().unwrap().to_str().unwrap();
    assert_eq!(
        filename.to_lowercase(),
        address.to_lowercase().trim_start_matches("0x"),
        "auto-selected keystore should match created address"
    );
}

#[test]
fn select_keystore_by_address() {
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    // Create two keystores
    let (addr1, _) = wallet::create_keystore("pw1", &keystores_dir).unwrap();
    let (_addr2, _) = wallet::create_keystore("pw2", &keystores_dir).unwrap();

    // Select by first address
    let selected = wallet::select_keystore(Some(&addr1), &keystores_dir).unwrap();
    assert!(selected.is_some());

    let selected_path = selected.unwrap();
    let filename = selected_path.file_stem().unwrap().to_str().unwrap();
    assert_eq!(
        filename.to_lowercase(),
        addr1.to_lowercase().trim_start_matches("0x"),
    );
}

#[test]
fn select_keystore_unknown_address_errors() {
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    let (_addr, _) = wallet::create_keystore("pw", &keystores_dir).unwrap();

    let result = wallet::select_keystore(Some("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"), &keystores_dir);
    assert!(result.is_err(), "should error for unknown address");
}

#[test]
fn delete_keystore_removes_file() {
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    let (address, path) = wallet::create_keystore("pw", &keystores_dir).unwrap();
    assert!(path.exists());

    wallet::delete_keystore(&address, &keystores_dir).unwrap();
    assert!(!path.exists(), "keystore file should be deleted");
}

#[test]
fn load_wallet_from_env_var() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");
    // No keystores exist, no legacy keyfile

    let test_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    unsafe { set("AGENT_PRIVATE_KEY", test_key) };

    let fake_legacy = dir.path().join("nonexistent-keyfile.json");
    let signer = wallet::load_wallet(None, None, &keystores_dir, Some(&fake_legacy)).unwrap();
    let address = format!("{:?}", signer.address());
    assert_eq!(
        address.to_lowercase(),
        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
    );

    unsafe { unset("AGENT_PRIVATE_KEY") };
}

#[test]
fn load_wallet_from_keystore() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    // Clear env to ensure we use keystore
    unsafe { unset("AGENT_PRIVATE_KEY") };

    let test_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let password = "test-pw";
    let (expected_addr, _) = wallet::import_keystore(test_key, password, &keystores_dir).unwrap();

    let fake_legacy = dir.path().join("nonexistent-keyfile.json");
    let signer = wallet::load_wallet(None, Some(password), &keystores_dir, Some(&fake_legacy)).unwrap();
    let address = format!("{:?}", signer.address());
    assert_eq!(address.to_lowercase(), expected_addr.to_lowercase());
}

#[test]
fn select_keystore_none_when_empty() {
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    let selected = wallet::select_keystore(None, &keystores_dir).unwrap();
    assert!(selected.is_none(), "should return None when no keystores exist");
}

#[test]
fn select_keystore_multiple_no_address_errors() {
    let dir = tempfile::tempdir().unwrap();
    let keystores_dir = dir.path().join("keystores");

    // Create two keystores
    let _ = wallet::create_keystore("pw1", &keystores_dir).unwrap();
    let _ = wallet::create_keystore("pw2", &keystores_dir).unwrap();

    let result = wallet::select_keystore(None, &keystores_dir);
    assert!(
        result.is_err(),
        "should error when multiple keystores exist and no address specified"
    );
}
