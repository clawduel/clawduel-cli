use alloy::signers::local::PrivateKeySigner;
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
    /// Generate a new random wallet and add to config
    Create,
    /// Import an existing private key into config
    Import {
        /// Private key (hex, with or without 0x prefix)
        key: String,
    },
    /// List all configured wallets
    List,
    /// Show details of a specific wallet
    Show {
        /// Address of wallet to show (optional if only one exists)
        #[arg(long)]
        agent: Option<String>,
    },
    /// Remove a wallet from config
    Remove {
        /// Address of wallet to remove
        address: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Delete all wallet config
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

pub async fn execute(args: WalletArgs) -> Result<()> {
    match args.command {
        WalletCommand::Create => cmd_create(),
        WalletCommand::Import { key } => cmd_import(&key),
        WalletCommand::List => cmd_list(),
        WalletCommand::Show { agent } => cmd_show(agent.as_deref()),
        WalletCommand::Remove { address, force } => cmd_remove(&address, force),
        WalletCommand::Reset { force } => cmd_reset(force),
    }
}

fn cmd_create() -> Result<()> {
    let signer = PrivateKeySigner::random();
    let key_hex = format!("0x{}", hex::encode(signer.to_bytes()));

    let address = wallet::add_wallet(&key_hex)?;
    let config_path = config::config_path()?;

    println!("Wallet created successfully!");
    println!("Address: {address}");
    println!("Config:  {}", config_path.display());

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "address": address,
            "config": config_path.display().to_string(),
        })
    );

    Ok(())
}

fn cmd_import(key: &str) -> Result<()> {
    let trimmed = key.trim();
    let test_key = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if test_key.len() != 64 || hex::decode(test_key).is_err() {
        bail!("Invalid private key format. Expected 64 hex characters (with or without 0x prefix).");
    }

    let address = wallet::add_wallet(trimmed)?;
    let config_path = config::config_path()?;

    println!("Wallet imported successfully!");
    println!("Address: {address}");
    println!("Config:  {}", config_path.display());

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "address": address,
            "config": config_path.display().to_string(),
        })
    );

    Ok(())
}

fn cmd_list() -> Result<()> {
    let addresses = wallet::list_wallets()?;

    if addresses.is_empty() {
        println!("No wallets configured.");
        println!("Run `clawduel wallet create` or `clawduel wallet import <key>` to set up.");
        return Ok(());
    }

    println!("Configured wallets:");
    for addr in &addresses {
        println!("  {addr}");
    }

    Ok(())
}

fn cmd_show(agent: Option<&str>) -> Result<()> {
    if !wallet::has_wallet()? {
        println!("No wallet configured.");
        println!("Run `clawduel wallet create` or `clawduel wallet import <key>` to set up.");
        return Ok(());
    }

    let signer = wallet::load_wallet(agent)?;
    let address = format!("{:?}", signer.address());
    let config_path = config::config_path()?;

    println!("Address: {address}");
    println!("Config:  {}", config_path.display());

    Ok(())
}

fn cmd_remove(address: &str, force: bool) -> Result<()> {
    if !force && config::is_interactive() {
        print!("Remove wallet {address}? [y/N] ");
        use std::io::{BufRead, Write};
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input)?;
        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    wallet::remove_wallet(address)?;
    println!("Wallet {address} removed.");
    Ok(())
}

fn cmd_reset(force: bool) -> Result<()> {
    if !wallet::has_wallet()? {
        println!("No wallets configured. Nothing to reset.");
        return Ok(());
    }

    if !force && config::is_interactive() {
        print!("Delete all wallet config? [y/N] ");
        use std::io::{BufRead, Write};
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input)?;
        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    wallet::delete_all_wallets()?;
    println!("All wallet config deleted.");
    Ok(())
}
