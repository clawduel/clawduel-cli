use clawduel_cli::wallet;

const KEY1: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const ADDR1: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
const KEY2: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const ADDR2: &str = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";

#[test]
fn add_and_load_wallet_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    let address = wallet::add_wallet_to(KEY1, &config_path).unwrap();
    assert_eq!(address.to_lowercase(), ADDR1.to_lowercase());

    let signer = wallet::load_wallet_from(&config_path, None).unwrap();
    assert_eq!(
        format!("{:?}", signer.address()).to_lowercase(),
        ADDR1.to_lowercase()
    );
}

#[test]
fn multiple_wallets_and_select_by_agent() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    let addr1 = wallet::add_wallet_to(KEY1, &config_path).unwrap();
    let addr2 = wallet::add_wallet_to(KEY2, &config_path).unwrap();

    // Without --agent should fail (multiple wallets)
    let result = wallet::load_wallet_from(&config_path, None);
    assert!(result.is_err());

    // With --agent should select the right one
    let signer1 = wallet::load_wallet_from(&config_path, Some(&addr1)).unwrap();
    assert_eq!(
        format!("{:?}", signer1.address()).to_lowercase(),
        ADDR1.to_lowercase()
    );

    let signer2 = wallet::load_wallet_from(&config_path, Some(&addr2)).unwrap();
    assert_eq!(
        format!("{:?}", signer2.address()).to_lowercase(),
        ADDR2.to_lowercase()
    );
}

#[test]
fn single_wallet_auto_selects() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    wallet::add_wallet_to(KEY1, &config_path).unwrap();

    // Should auto-select the only wallet
    let signer = wallet::load_wallet_from(&config_path, None).unwrap();
    assert_eq!(
        format!("{:?}", signer.address()).to_lowercase(),
        ADDR1.to_lowercase()
    );
}

#[test]
fn has_wallet_false_when_no_config() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    assert!(!wallet::has_wallet_at(&config_path).unwrap());
}

#[test]
fn has_wallet_true_after_add() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    wallet::add_wallet_to(KEY1, &config_path).unwrap();
    assert!(wallet::has_wallet_at(&config_path).unwrap());
}

#[test]
fn remove_wallet_by_address() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    let addr1 = wallet::add_wallet_to(KEY1, &config_path).unwrap();
    wallet::add_wallet_to(KEY2, &config_path).unwrap();

    wallet::remove_wallet_from(&addr1, &config_path).unwrap();

    let wallets = wallet::list_wallets_from(&config_path).unwrap();
    assert_eq!(wallets.len(), 1);
    assert!(!wallets.iter().any(|a| a.to_lowercase() == addr1.to_lowercase()));
}

#[test]
fn remove_unknown_address_errors() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    wallet::add_wallet_to(KEY1, &config_path).unwrap();
    let result = wallet::remove_wallet_from("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef", &config_path);
    assert!(result.is_err());
}

#[test]
fn delete_all_wallets_removes_config() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    wallet::add_wallet_to(KEY1, &config_path).unwrap();
    assert!(config_path.exists());

    wallet::delete_all_wallets_at(&config_path).unwrap();
    assert!(!config_path.exists());
}

#[test]
fn add_wallet_validates_key() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    let result = wallet::add_wallet_to("not-a-valid-key", &config_path);
    assert!(result.is_err());
}

#[test]
fn load_wallet_errors_when_empty() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    std::fs::write(&config_path, "{}").unwrap();

    let result = wallet::load_wallet_from(&config_path, None);
    assert!(result.is_err());
}

#[test]
fn select_unknown_agent_errors() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    wallet::add_wallet_to(KEY1, &config_path).unwrap();

    let result = wallet::load_wallet_from(&config_path, Some("0xdeadbeef"));
    assert!(result.is_err());
}

#[test]
fn list_wallets_returns_all_addresses() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    wallet::add_wallet_to(KEY1, &config_path).unwrap();
    wallet::add_wallet_to(KEY2, &config_path).unwrap();

    let wallets = wallet::list_wallets_from(&config_path).unwrap();
    assert_eq!(wallets.len(), 2);
}
