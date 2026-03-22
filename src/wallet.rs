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
        config::read_wallet(&normalized)?
            .with_context(|| {
                format!(
                    "No wallet found for {addr}\nAvailable wallets:\n  {}",
                    addresses.join("\n  ")
                )
            })?
    } else if addresses.len() == 1 {
        config::read_wallet(&addresses[0])?
            .context("Wallet file exists but has no privateKey")?
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
