use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// ClawDuel CLI configuration.
///
/// Stored at `~/.config/clawduel/config.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Map of address -> private_key (hex with 0x prefix).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub wallets: HashMap<String, String>,
}

pub const BACKEND_URL: &str = "http://localhost:8787";
pub const RPC_URL: &str = "http://localhost:8545";

/// Returns the config directory path (`~/.config/clawduel/`).
pub fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join("clawduel"))
}

/// Returns the config file path (`~/.config/clawduel/config.json`).
pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

/// Load config from disk. Returns `Ok(None)` if no config file exists.
pub fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    load_config_from(&path)
}

/// Load config from a specific path (for testing).
pub fn load_config_from(path: &std::path::Path) -> Result<Option<Config>> {
    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(anyhow::anyhow!(e).context(format!("Failed to read {}", path.display())));
        }
    };
    let config: Config = serde_json::from_str(&data)
        .context(format!("Invalid JSON in config file {}", path.display()))?;
    Ok(Some(config))
}

/// Save config to disk, creating directories with secure permissions.
pub fn save_config(config: &Config) -> Result<()> {
    save_config_to(config, &config_path()?)
}

/// Save config to a specific path (for testing).
pub fn save_config_to(config: &Config, path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
    }

    let json = serde_json::to_string_pretty(config)?;

    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .context("Failed to create config file")?;
        file.write_all(json.as_bytes())
            .context("Failed to write config file")?;
    }

    #[cfg(not(unix))]
    {
        fs::write(path, &json).context("Failed to write config file")?;
    }

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
