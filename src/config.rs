use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub const BACKEND_URL: &str = "http://localhost:8787";
pub const RPC_URL: &str = "http://localhost:8545";

/// Returns the config directory path (`~/.config/clawduel/`).
pub fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join("clawduel"))
}

/// Returns the wallets directory path (`~/.config/clawduel/wallets/`).
pub fn wallets_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("wallets"))
}

/// Ensure the wallets directory exists with secure permissions.
pub fn ensure_wallets_dir() -> Result<PathBuf> {
    let dir = wallets_dir()?;
    fs::create_dir_all(&dir).context("Failed to create wallets directory")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        if let Some(parent) = dir.parent() {
            let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
        }
    }

    Ok(dir)
}

/// Returns the path for a wallet file: `~/.config/clawduel/wallets/<address>.json`
pub fn wallet_path(address: &str) -> Result<PathBuf> {
    let normalized = address.to_lowercase();
    Ok(wallets_dir()?.join(format!("{normalized}.json")))
}

/// Read a single wallet's private key from its file.
pub fn read_wallet(address: &str) -> Result<Option<String>> {
    let path = wallet_path(address)?;
    match fs::read_to_string(&path) {
        Ok(data) => {
            let val: serde_json::Value = serde_json::from_str(&data)
                .context(format!("Invalid JSON in wallet file {}", path.display()))?;
            Ok(val.get("privateKey").and_then(|v| v.as_str()).map(|s| s.to_string()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow::anyhow!(e).context(format!("Failed to read {}", path.display()))),
    }
}

/// Write a single wallet file. Each wallet is isolated — no lock contention.
pub fn write_wallet(address: &str, private_key: &str) -> Result<()> {
    let dir = ensure_wallets_dir()?;
    let normalized = address.to_lowercase();
    let path = dir.join(format!("{normalized}.json"));

    let json = serde_json::json!({
        "address": normalized,
        "privateKey": private_key,
    });
    let data = serde_json::to_string_pretty(&json)?;

    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .context("Failed to create wallet file")?;
        file.write_all(data.as_bytes())
            .context("Failed to write wallet file")?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&path, &data).context("Failed to write wallet file")?;
    }

    Ok(())
}

/// Delete a single wallet file.
pub fn delete_wallet(address: &str) -> Result<bool> {
    let path = wallet_path(address)?;
    if path.exists() {
        fs::remove_file(&path).context("Failed to delete wallet file")?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// List all wallet addresses by scanning the wallets directory.
pub fn list_wallet_addresses() -> Result<Vec<String>> {
    let dir = wallets_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut addresses = Vec::new();
    for entry in fs::read_dir(&dir).context("Failed to read wallets directory")? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".json") {
            let addr = name_str.trim_end_matches(".json").to_string();
            addresses.push(addr);
        }
    }
    Ok(addresses)
}

/// Delete all wallet files.
pub fn delete_all_wallets() -> Result<()> {
    let dir = wallets_dir()?;
    if dir.exists() {
        fs::remove_dir_all(&dir).context("Failed to delete wallets directory")?;
    }
    // Also clean up legacy config.json if present
    let legacy = config_dir()?.join("config.json");
    if legacy.exists() {
        let _ = fs::remove_file(&legacy);
    }
    Ok(())
}

/// Migrate from legacy config.json to per-wallet files (if needed).
pub fn migrate_legacy_config() -> Result<()> {
    let legacy_path = config_dir()?.join("config.json");
    if !legacy_path.exists() {
        return Ok(());
    }

    let data = fs::read_to_string(&legacy_path)?;

    #[derive(serde::Deserialize)]
    struct LegacyConfig {
        #[serde(default)]
        wallets: std::collections::HashMap<String, String>,
    }

    let legacy: LegacyConfig = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(_) => return Ok(()), // Not a valid legacy config, skip
    };

    if legacy.wallets.is_empty() {
        return Ok(());
    }

    for (address, private_key) in &legacy.wallets {
        // Only migrate if per-wallet file doesn't already exist
        let path = wallet_path(address)?;
        if !path.exists() {
            write_wallet(address, private_key)?;
        }
    }

    // Remove legacy file after successful migration
    let _ = fs::remove_file(&legacy_path);
    let lock_path = legacy_path.with_extension("lock");
    let _ = fs::remove_file(&lock_path);

    Ok(())
}

/// Returns false if `CLAW_NON_INTERACTIVE=1` or stdin is not a TTY.
pub fn is_interactive() -> bool {
    if let Ok(val) = std::env::var("CLAW_NON_INTERACTIVE") {
        if val == "1" {
            return false;
        }
    }
    std::io::IsTerminal::is_terminal(&std::io::stdin())
}
