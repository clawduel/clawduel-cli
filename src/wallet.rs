use std::path::Path;
use std::str::FromStr;

use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result, bail};

use crate::config;

/// Load a wallet signer. If `agent` is given, select by address; otherwise auto-select
/// if only one wallet exists.
pub fn load_wallet(agent: Option<&str>) -> Result<PrivateKeySigner> {
    // Auto-migrate legacy config.json on first access
    config::migrate_legacy_config()?;

    let addresses = config::list_wallet_addresses()?;

    if addresses.is_empty() {
        bail!(
            "No wallet configured.\n\
             Run `clawduel wallet create` or `clawduel wallet import <key>` to set up."
        );
    }

    let key = if let Some(addr) = agent {
        let normalized = addr.to_lowercase();
        config::read_wallet(&normalized)?.with_context(|| {
            format!(
                "No wallet found for {addr}\nAvailable wallets:\n  {}",
                addresses.join("\n  ")
            )
        })?
    } else if addresses.len() == 1 {
        config::read_wallet(&addresses[0])?.context("Wallet file exists but has no privateKey")?
    } else {
        bail!(
            "Multiple wallets configured. Use --agent <address> to select one.\nAvailable:\n  {}",
            addresses.join("\n  ")
        );
    };

    let k = key.strip_prefix("0x").unwrap_or(&key);
    PrivateKeySigner::from_str(k).context("Invalid private key in wallet file")
}

/// Add a wallet. Each wallet gets its own file — no contention.
pub fn add_wallet(private_key: &str) -> Result<String> {
    let key = private_key.strip_prefix("0x").unwrap_or(private_key);
    let signer = PrivateKeySigner::from_str(key).context("Invalid private key")?;
    let address = format!("{:?}", signer.address());

    config::write_wallet(&address, &format!("0x{key}"))?;

    Ok(address)
}

pub fn load_wallet_from(config_path: &Path, agent: Option<&str>) -> Result<PrivateKeySigner> {
    let cfg = config::load_config_from(config_path)?.unwrap_or_default();
    if cfg.wallets.is_empty() {
        bail!("No wallet configured");
    }

    let key = if let Some(addr) = agent {
        let normalized = addr.to_lowercase();
        cfg.wallets
            .get(&normalized)
            .or_else(|| cfg.wallets.get(addr))
            .with_context(|| format!("No wallet found for {addr}"))?
            .clone()
    } else if cfg.wallets.len() == 1 {
        cfg.wallets.values().next().unwrap().clone()
    } else {
        bail!("Multiple wallets configured. Use --agent <address> to select one.");
    };

    let k = key.strip_prefix("0x").unwrap_or(&key);
    PrivateKeySigner::from_str(k).context("Invalid private key in wallet file")
}

pub fn add_wallet_to(private_key: &str, config_path: &Path) -> Result<String> {
    let key = private_key.strip_prefix("0x").unwrap_or(private_key);
    let signer = PrivateKeySigner::from_str(key).context("Invalid private key")?;
    let address = format!("{:?}", signer.address());
    let normalized = address.to_lowercase();

    let mut cfg = config::load_config_from(config_path)?.unwrap_or_default();
    cfg.wallets.insert(normalized, format!("0x{key}"));
    config::save_config_to(&cfg, config_path)?;

    Ok(address)
}

pub fn remove_wallet_from(address: &str, config_path: &Path) -> Result<()> {
    let mut cfg = config::load_config_from(config_path)?.unwrap_or_default();
    let normalized = address.to_lowercase();
    if cfg.wallets.remove(&normalized).is_none() && cfg.wallets.remove(address).is_none() {
        bail!("No wallet found for {address}");
    }
    config::save_config_to(&cfg, config_path)
}

pub fn list_wallets_from(config_path: &Path) -> Result<Vec<String>> {
    let cfg = config::load_config_from(config_path)?.unwrap_or_default();
    Ok(cfg.wallets.keys().cloned().collect())
}

pub fn has_wallet_at(config_path: &Path) -> Result<bool> {
    let cfg = config::load_config_from(config_path)?.unwrap_or_default();
    Ok(!cfg.wallets.is_empty())
}

pub fn delete_all_wallets_at(config_path: &Path) -> Result<()> {
    match std::fs::remove_file(config_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => {
            Err(anyhow::anyhow!(e).context(format!("Failed to delete {}", config_path.display())))
        }
    }
}

/// Remove a wallet by address.
pub fn remove_wallet(address: &str) -> Result<()> {
    let deleted = config::delete_wallet(address)?;
    if !deleted {
        bail!("No wallet found for {address}");
    }
    Ok(())
}

/// List all wallet addresses.
pub fn list_wallets() -> Result<Vec<String>> {
    config::migrate_legacy_config()?;
    config::list_wallet_addresses()
}

/// Check if any wallets exist.
pub fn has_wallet() -> Result<bool> {
    config::migrate_legacy_config()?;
    let addresses = config::list_wallet_addresses()?;
    Ok(!addresses.is_empty())
}

/// Delete all wallets.
pub fn delete_all_wallets() -> Result<()> {
    config::delete_all_wallets()
}
