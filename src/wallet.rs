use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result, bail};

/// Default keystores directory: `~/.clawduel/keystores/`
pub fn default_keystores_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".clawduel").join("keystores"))
}

/// Legacy keyfile path: `~/.clawduel/keyfile.json`
pub fn legacy_keyfile_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".clawduel").join("keyfile.json"))
}

/// Create a new random wallet and encrypt it as a keystore file.
///
/// Returns `(address_string, keystore_path)`.
pub fn create_keystore(password: &str, keystores_dir: &Path) -> Result<(String, PathBuf)> {
    let signer = PrivateKeySigner::random();
    let address = format!("{:?}", signer.address());
    let key_bytes = signer.to_bytes();

    save_keystore(&key_bytes, password, &address, keystores_dir)
        .map(|path| (address, path))
}

/// Import an existing private key and encrypt it as a keystore file.
///
/// Returns `(address_string, keystore_path)`.
pub fn import_keystore(private_key: &str, password: &str, keystores_dir: &Path) -> Result<(String, PathBuf)> {
    let key = private_key.strip_prefix("0x").unwrap_or(private_key);
    let signer = PrivateKeySigner::from_str(key).context("Invalid private key")?;
    let address = format!("{:?}", signer.address());
    let key_bytes = signer.to_bytes();

    save_keystore(&key_bytes, password, &address, keystores_dir)
        .map(|path| (address, path))
}

/// Encrypt and save a private key to a keystore file.
fn save_keystore(
    key_bytes: &[u8; 32],
    password: &str,
    address: &str,
    keystores_dir: &Path,
) -> Result<PathBuf> {
    fs::create_dir_all(keystores_dir).context("Failed to create keystores directory")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(keystores_dir, fs::Permissions::from_mode(0o700))?;
    }

    // Use eth-keystore to encrypt
    let mut rng = rand::thread_rng();
    let filename = address.strip_prefix("0x").unwrap_or(address).to_lowercase();

    // eth_keystore::encrypt_key writes a file to the directory with a UUID name.
    // We need a specific filename, so we encrypt to a temp name then rename.
    let name = eth_keystore::encrypt_key(
        keystores_dir,
        &mut rng,
        key_bytes,
        password,
        None,
    )
    .context("Failed to encrypt keystore")?;

    let temp_path = keystores_dir.join(&name);
    let final_path = keystores_dir.join(format!("{filename}.json"));

    // If target already exists, remove it first
    if final_path.exists() {
        fs::remove_file(&final_path)?;
    }
    fs::rename(&temp_path, &final_path)
        .context("Failed to rename keystore file")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&final_path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(final_path)
}

/// Decrypt a keystore file and return a LocalSigner.
pub fn decrypt_keystore(path: &Path, password: &str) -> Result<PrivateKeySigner> {
    let key_bytes = eth_keystore::decrypt_key(path, password)
        .context("Failed to decrypt keystore. Wrong password?")?;
    let key_hex = hex::encode(&key_bytes);
    PrivateKeySigner::from_str(&key_hex).context("Invalid key in keystore")
}

/// Discover all keystore files in the keystores directory.
pub fn discover_keystores(keystores_dir: &Path) -> Vec<PathBuf> {
    if !keystores_dir.exists() {
        return Vec::new();
    }
    let mut result: Vec<PathBuf> = fs::read_dir(keystores_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().and_then(|e| e.to_str()) == Some("json")
        })
        .map(|entry| entry.path())
        .collect();
    result.sort();
    result
}

/// Select a keystore by address.
///
/// - If `agent_address` is given, find the matching keystore.
/// - If none given and exactly one exists, auto-select.
/// - If none given and multiple exist, return error listing available addresses.
/// - If none exist, return `Ok(None)` to allow fallback.
pub fn select_keystore(
    agent_address: Option<&str>,
    keystores_dir: &Path,
) -> Result<Option<PathBuf>> {
    let keystores = discover_keystores(keystores_dir);

    if keystores.is_empty() {
        return Ok(None);
    }

    if let Some(address) = agent_address {
        let normalized = address.to_lowercase().replace("0x", "");
        let found = keystores.iter().find(|k| {
            let stem = k.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase()
                .replace("0x", "");
            stem == normalized
        });

        match found {
            Some(path) => Ok(Some(path.clone())),
            None => {
                let available: Vec<String> = keystores.iter()
                    .filter_map(|k| k.file_stem().and_then(|s| s.to_str()).map(|s| format!("0x{s}")))
                    .collect();
                bail!(
                    "No keystore found for agent {address}\nAvailable keystores:\n  {}",
                    available.join("\n  ")
                );
            }
        }
    } else if keystores.len() == 1 {
        Ok(Some(keystores[0].clone()))
    } else {
        let available: Vec<String> = keystores.iter()
            .filter_map(|k| k.file_stem().and_then(|s| s.to_str()).map(|s| format!("0x{s}")))
            .collect();
        bail!(
            "Multiple keystores found. Specify which agent to use:\n  --agent <address>\nAvailable:\n  {}\nOr set CLAW_AGENT_ADDRESS env var",
            available.join("\n  ")
        );
    }
}

/// Delete a keystore file by address.
pub fn delete_keystore(address: &str, keystores_dir: &Path) -> Result<()> {
    let normalized = address.to_lowercase().replace("0x", "");
    let path = keystores_dir.join(format!("{normalized}.json"));

    if !path.exists() {
        bail!("No keystore found for address {address}");
    }

    fs::remove_file(&path).context(format!("Failed to delete keystore for {address}"))?;
    Ok(())
}

/// Load a wallet signer with the following priority:
///
/// 1. Keystore (selected by `agent_address` or auto-selected)
/// 2. Legacy keyfile (`~/.clawduel/keyfile.json`)
/// 3. `AGENT_PRIVATE_KEY` env var
///
/// Password is resolved from: `password` arg > `CLAW_KEY_PASSWORD` env > interactive prompt.
pub fn load_wallet(
    agent_address: Option<&str>,
    password: Option<&str>,
    keystores_dir: &Path,
    legacy_keyfile: Option<&Path>,
) -> Result<PrivateKeySigner> {
    // 1. Try keystores
    if let Some(path) = select_keystore(agent_address, keystores_dir)? {
        let pw = resolve_password(password)?;
        return decrypt_keystore(&path, &pw);
    }

    // 2. Try legacy keyfile
    let legacy = legacy_keyfile
        .map(|p| p.to_path_buf())
        .or_else(|| legacy_keyfile_path().ok());
    if let Some(ref legacy_path) = legacy {
        if legacy_path.exists() {
            let pw = resolve_password(password)?;
            return decrypt_keystore(legacy_path, &pw);
        }
    }

    // 3. Try AGENT_PRIVATE_KEY env var
    if let Ok(key) = std::env::var("AGENT_PRIVATE_KEY") {
        if !key.is_empty() {
            let k = key.strip_prefix("0x").unwrap_or(&key);
            return PrivateKeySigner::from_str(k).context("Invalid AGENT_PRIVATE_KEY");
        }
    }

    bail!(
        "No wallet found.\n\
         Run `clawduel wallet create` or `clawduel wallet import <key>` to set up a keystore.\n\
         Or set AGENT_PRIVATE_KEY env var as a fallback."
    )
}

/// Resolve password from argument, env var, or interactive prompt.
fn resolve_password(password: Option<&str>) -> Result<String> {
    if let Some(pw) = password {
        return Ok(pw.to_string());
    }
    if let Ok(pw) = std::env::var("CLAW_KEY_PASSWORD") {
        if !pw.is_empty() {
            return Ok(pw);
        }
    }
    // Interactive prompt
    if crate::config::is_interactive() {
        let pw = rpassword::prompt_password_stderr("Enter keystore password: ")
            .context("Failed to read password")?;
        return Ok(pw);
    }
    bail!("No password available. Set CLAW_KEY_PASSWORD env var for non-interactive mode.")
}
