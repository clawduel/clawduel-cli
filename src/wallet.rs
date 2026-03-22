use std::str::FromStr;

use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result, bail};

use crate::config;

/// Load a wallet signer. If `agent` is given, select by address; otherwise auto-select
/// if only one wallet exists.
pub fn load_wallet(agent: Option<&str>) -> Result<PrivateKeySigner> {
    let cfg = config::load_config()?.unwrap_or_default();
    load_wallet_inner(&cfg, agent)
}

/// Load wallet from a specific config path (for testing).
pub fn load_wallet_from(path: &std::path::Path, agent: Option<&str>) -> Result<PrivateKeySigner> {
    let cfg = config::load_config_from(path)?.unwrap_or_default();
    load_wallet_inner(&cfg, agent)
}

fn load_wallet_inner(cfg: &config::Config, agent: Option<&str>) -> Result<PrivateKeySigner> {
    if cfg.wallets.is_empty() {
        bail!(
            "No wallet configured.\n\
             Run `clawduel wallet create` or `clawduel wallet import <key>` to set up."
        );
    }

    let key = if let Some(addr) = agent {
        let normalized = addr.to_lowercase();
        cfg.wallets
            .iter()
            .find(|(a, _)| a.to_lowercase() == normalized)
            .map(|(_, k)| k.as_str())
            .with_context(|| {
                let available: Vec<&str> = cfg.wallets.keys().map(|s| s.as_str()).collect();
                format!(
                    "No wallet found for {addr}\nAvailable wallets:\n  {}",
                    available.join("\n  ")
                )
            })?
    } else if cfg.wallets.len() == 1 {
        cfg.wallets.values().next().unwrap().as_str()
    } else {
        let available: Vec<&str> = cfg.wallets.keys().map(|s| s.as_str()).collect();
        bail!(
            "Multiple wallets configured. Use --agent <address> to select one.\nAvailable:\n  {}",
            available.join("\n  ")
        );
    };

    let k = key.strip_prefix("0x").unwrap_or(key);
    PrivateKeySigner::from_str(k).context("Invalid private key in config")
}

/// Add a wallet (address -> private_key) to config.json.
/// Uses file locking to prevent concurrent writes from clobbering each other.
pub fn add_wallet(private_key: &str) -> Result<String> {
    let key = private_key.strip_prefix("0x").unwrap_or(private_key);
    let signer = PrivateKeySigner::from_str(key).context("Invalid private key")?;
    let address = format!("{:?}", signer.address());

    let key_owned = format!("0x{key}");
    let addr_clone = address.clone();
    config::modify_config_locked(move |cfg| {
        cfg.wallets.insert(addr_clone, key_owned);
    })?;

    Ok(address)
}

/// Add a wallet to a specific config path (for testing).
pub fn add_wallet_to(private_key: &str, path: &std::path::Path) -> Result<String> {
    let key = private_key.strip_prefix("0x").unwrap_or(private_key);
    let signer = PrivateKeySigner::from_str(key).context("Invalid private key")?;
    let address = format!("{:?}", signer.address());

    let key_owned = format!("0x{key}");
    let addr_clone = address.clone();
    config::modify_config_locked_at(path, move |cfg| {
        cfg.wallets.insert(addr_clone, key_owned);
    })?;

    Ok(address)
}

/// Remove a wallet by address from config.json.
/// Uses file locking to prevent concurrent writes from clobbering each other.
pub fn remove_wallet(address: &str) -> Result<()> {
    let normalized = address.to_lowercase();
    config::modify_config_locked(move |cfg| {
        let key_to_remove = cfg
            .wallets
            .keys()
            .find(|a| a.to_lowercase() == normalized)
            .cloned();
        if let Some(k) = key_to_remove {
            cfg.wallets.remove(&k);
        }
    })
}

/// Remove a wallet from a specific config path (for testing).
pub fn remove_wallet_from(address: &str, path: &std::path::Path) -> Result<()> {
    let normalized = address.to_lowercase();
    config::modify_config_locked_at(path, move |cfg| {
        let key_to_remove = cfg
            .wallets
            .keys()
            .find(|a| a.to_lowercase() == normalized)
            .cloned();
        if let Some(k) = key_to_remove {
            cfg.wallets.remove(&k);
        }
    })
}

/// List all wallet addresses in config.
pub fn list_wallets() -> Result<Vec<String>> {
    let cfg = config::load_config()?.unwrap_or_default();
    Ok(cfg.wallets.keys().cloned().collect())
}

/// List all wallet addresses from a specific config path (for testing).
pub fn list_wallets_from(path: &std::path::Path) -> Result<Vec<String>> {
    let cfg = config::load_config_from(path)?.unwrap_or_default();
    Ok(cfg.wallets.keys().cloned().collect())
}

/// Check if any wallets exist in config.
pub fn has_wallet() -> Result<bool> {
    let cfg = config::load_config()?.unwrap_or_default();
    Ok(!cfg.wallets.is_empty())
}

/// Check if any wallets exist at a specific config path (for testing).
pub fn has_wallet_at(path: &std::path::Path) -> Result<bool> {
    let cfg = config::load_config_from(path)?.unwrap_or_default();
    Ok(!cfg.wallets.is_empty())
}

/// Delete the config file (removing all wallets).
pub fn delete_all_wallets() -> Result<()> {
    let path = config::config_path()?;
    delete_all_wallets_at(&path)
}

/// Delete a specific config file (for testing).
pub fn delete_all_wallets_at(path: &std::path::Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path).context("Failed to delete config file")?;
    }
    Ok(())
}
