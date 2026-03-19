use anyhow::{Result, bail};
use clap::{Args, Subcommand};

use crate::config;
use crate::wallet;

#[derive(Args)]
pub struct WalletArgs {
    #[command(subcommand)]
    pub command: WalletCommand,
}

#[derive(Subcommand)]
pub enum WalletCommand {
    /// Generate a new random wallet and save to encrypted keystore
    Create {
        /// Overwrite existing keystore for this address
        #[arg(long)]
        force: bool,
    },
    /// Import an existing private key into an encrypted keystore
    Import {
        /// Private key (hex, with or without 0x prefix)
        key: String,
        /// Overwrite existing keystore for this address
        #[arg(long)]
        force: bool,
    },
    /// Show the address and key source of the active wallet
    Show,
    /// Delete a keystore file
    Delete {
        /// Address of the keystore to delete (required if multiple exist)
        #[arg(long)]
        address: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

pub async fn execute(args: WalletArgs, agent: Option<&str>) -> Result<()> {
    let keystores_dir = wallet::default_keystores_dir()?;
    let agent_address = config::resolve_agent_address(agent);

    match args.command {
        WalletCommand::Create { force } => {
            cmd_create(force, &keystores_dir).await
        }
        WalletCommand::Import { key, force } => {
            cmd_import(&key, force, &keystores_dir).await
        }
        WalletCommand::Show => {
            cmd_show(agent_address.as_deref(), &keystores_dir).await
        }
        WalletCommand::Delete { address, force } => {
            let addr = address.or(agent_address);
            cmd_delete(addr.as_deref(), force, &keystores_dir).await
        }
    }
}

async fn cmd_create(force: bool, keystores_dir: &std::path::Path) -> Result<()> {
    if !force {
        let existing = wallet::discover_keystores(keystores_dir);
        if !existing.is_empty() && !config::is_interactive() {
            // In non-interactive mode, don't create if keystores already exist
            // unless --force is passed
        }
    }

    let password = get_password("Enter password for new keystore: ")?;
    let confirm = get_password("Confirm password: ")?;

    if password != confirm {
        bail!("Passwords do not match");
    }

    let (address, path) = wallet::create_keystore(&password, keystores_dir)?;

    println!("Wallet created successfully!");
    println!("Address:  {address}");
    println!("Keystore: {}", path.display());
    println!();
    println!("IMPORTANT: Remember your password. If lost, the keystore cannot be decrypted.");

    // Also output JSON for machine parsing
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "address": address,
            "keystore": path.display().to_string(),
        })
    );

    Ok(())
}

async fn cmd_import(key: &str, force: bool, keystores_dir: &std::path::Path) -> Result<()> {
    // Validate key first
    let trimmed = key.trim();
    let test_key = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if test_key.len() != 64 || hex::decode(test_key).is_err() {
        bail!("Invalid private key format. Expected 64 hex characters (with or without 0x prefix).");
    }

    if !force {
        // Check if this key's address already has a keystore
        let signer = alloy::signers::local::PrivateKeySigner::from_str(test_key)
            .map_err(|e| anyhow::anyhow!("Invalid private key: {e}"))?;
        let address = format!("{:?}", signer.address());
        let normalized = address.to_lowercase().replace("0x", "");
        let target = keystores_dir.join(format!("{normalized}.json"));
        if target.exists() {
            bail!(
                "A keystore already exists for {address}. Use --force to overwrite."
            );
        }
    }

    let password = get_password("Enter password for keystore: ")?;
    let confirm = get_password("Confirm password: ")?;

    if password != confirm {
        bail!("Passwords do not match");
    }

    let (address, path) = wallet::import_keystore(trimmed, &password, keystores_dir)?;

    println!("Wallet imported successfully!");
    println!("Address:  {address}");
    println!("Keystore: {}", path.display());

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "address": address,
            "keystore": path.display().to_string(),
        })
    );

    Ok(())
}

async fn cmd_show(agent_address: Option<&str>, keystores_dir: &std::path::Path) -> Result<()> {
    // Try keystore
    if let Some(path) = wallet::select_keystore(agent_address, keystores_dir)? {
        let filename = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let address = format!("0x{filename}");

        println!("Address:    {address}");
        println!("Key source: encrypted keystore");
        println!("Keystore:   {}", path.display());
        return Ok(());
    }

    // Try legacy keyfile
    if let Ok(legacy) = wallet::legacy_keyfile_path() {
        if legacy.exists() {
            println!("Key source: legacy keyfile");
            println!("Keyfile:    {}", legacy.display());
            println!("(Run `clawduel wallet create` to upgrade to a keystore)");
            return Ok(());
        }
    }

    // Try env var
    if let Ok(key) = std::env::var("AGENT_PRIVATE_KEY") {
        if !key.is_empty() {
            let k = key.strip_prefix("0x").unwrap_or(&key);
            if let Ok(signer) = alloy::signers::local::PrivateKeySigner::from_str(k) {
                let address = format!("{:?}", signer.address());
                println!("Address:    {address}");
                println!("Key source: AGENT_PRIVATE_KEY env var");
                return Ok(());
            }
        }
    }

    println!("No wallet configured.");
    println!("Run `clawduel wallet create` or `clawduel wallet import <key>` to set up.");
    println!("Or set AGENT_PRIVATE_KEY env var.");
    Ok(())
}

async fn cmd_delete(
    address: Option<&str>,
    force: bool,
    keystores_dir: &std::path::Path,
) -> Result<()> {
    let address = match address {
        Some(a) => a.to_string(),
        None => {
            // Try to auto-select
            match wallet::select_keystore(None, keystores_dir)? {
                Some(path) => {
                    let stem = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    format!("0x{stem}")
                }
                None => bail!("No keystores found to delete"),
            }
        }
    };

    if !force && config::is_interactive() {
        print!("Delete keystore for {address}? [y/N] ");
        use std::io::{Write, BufRead};
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input)?;
        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    wallet::delete_keystore(&address, keystores_dir)?;
    println!("Keystore deleted for {address}");
    Ok(())
}

/// Get password from env var or interactive prompt.
fn get_password(prompt: &str) -> Result<String> {
    if let Ok(pw) = std::env::var("CLAW_KEY_PASSWORD") {
        if !pw.is_empty() {
            return Ok(pw);
        }
    }
    if config::is_interactive() {
        let pw = rpassword::prompt_password_stderr(prompt)
            .map_err(|e| anyhow::anyhow!("Failed to read password: {e}"))?;
        if pw.is_empty() {
            bail!("No password provided");
        }
        return Ok(pw);
    }
    bail!("No password available. Set CLAW_KEY_PASSWORD env var for non-interactive mode.")
}

use std::str::FromStr;
