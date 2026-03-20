use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// ClawDuel CLI configuration.
///
/// Stored at `~/.config/clawduel/config.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_address: Option<String>,
}

const DEFAULT_BACKEND_URL: &str = "http://localhost:3001";
const DEFAULT_RPC_URL: &str = "http://localhost:8545";

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
    let data = match fs::read_to_string(&path) {
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

/// Resolve backend URL with priority: flag > env > config file > default.
pub fn resolve_backend_url(flag: Option<&str>) -> String {
    resolve_with_priority(flag, "CLAW_BACKEND_URL", |c| c.backend_url.as_deref(), DEFAULT_BACKEND_URL)
}

/// Resolve RPC URL with priority: flag > env > config file > default.
pub fn resolve_rpc_url(flag: Option<&str>) -> String {
    resolve_with_priority(flag, "CLAW_RPC_URL", |c| c.rpc_url.as_deref(), DEFAULT_RPC_URL)
}

/// Resolve agent address with priority: flag > env > config file > None.
pub fn resolve_agent_address(flag: Option<&str>) -> Option<String> {
    if let Some(val) = flag {
        if !val.is_empty() {
            return Some(val.to_string());
        }
    }
    if let Ok(val) = std::env::var("CLAW_AGENT_ADDRESS") {
        if !val.is_empty() {
            return Some(val);
        }
    }
    if let Ok(Some(config)) = load_config() {
        if let Some(addr) = config.agent_address {
            if !addr.is_empty() {
                return Some(addr);
            }
        }
    }
    None
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

/// Generic priority resolution: flag > env > config > default.
fn resolve_with_priority(
    flag: Option<&str>,
    env_var: &str,
    config_getter: impl FnOnce(&Config) -> Option<&str>,
    default: &str,
) -> String {
    if let Some(val) = flag {
        if !val.is_empty() {
            return val.to_string();
        }
    }
    if let Ok(val) = std::env::var(env_var) {
        if !val.is_empty() {
            return val;
        }
    }
    if let Ok(Some(config)) = load_config() {
        if let Some(val) = config_getter(&config) {
            if !val.is_empty() {
                return val.to_string();
            }
        }
    }
    default.to_string()
}
