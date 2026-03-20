//! Multi-duel lobby commands: create, join, list, and status.

use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol_types::{Eip712Domain, SolStruct};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use tabled::Tabled;

use crate::contracts::{self, IMultiDuel, JoinMultiAttestation};
use crate::http::HttpClient;
use crate::output::OutputFormat;
use crate::security;

#[derive(Args)]
pub struct LobbyArgs {
    #[command(subcommand)]
    pub command: LobbyCommand,
}

#[derive(Subcommand)]
pub enum LobbyCommand {
    /// Create a new multi-duel lobby (auto-joins as first participant)
    Create {
        /// Bet size in USDC
        bet_size: u64,
        /// Maximum participants (default: 5)
        #[arg(long, default_value = "5")]
        max_participants: u32,
        /// Attestation timeout in seconds
        #[arg(long, default_value = "3600")]
        timeout: u64,
    },
    /// Join an existing multi-duel lobby
    Join {
        /// Lobby ID
        lobby_id: String,
        /// Attestation timeout in seconds
        #[arg(long, default_value = "3600")]
        timeout: u64,
    },
    /// List open lobbies
    List,
    /// Show lobby details and participants
    Status {
        /// Lobby ID
        lobby_id: String,
    },
}

pub async fn execute(
    args: LobbyArgs,
    client: &HttpClient,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    match args.command {
        LobbyCommand::Create {
            bet_size,
            max_participants,
            timeout,
        } => {
            cmd_create(
                client,
                bet_size,
                max_participants,
                timeout,
                address,
                signer,
                rpc_url,
                fmt,
            )
            .await
        }
        LobbyCommand::Join { lobby_id, timeout } => {
            cmd_join(client, &lobby_id, timeout, address, signer, rpc_url, fmt).await
        }
        LobbyCommand::List => cmd_list(client, fmt).await,
        LobbyCommand::Status { lobby_id } => cmd_status(client, &lobby_id, fmt).await,
    }
}

/// Generate a random unused nonce for multi-duel attestations.
async fn generate_nonce(
    provider: &impl Provider,
    multi_duel_address: &Address,
    agent: &Address,
) -> Result<U256> {
    let multi_duel = IMultiDuel::new(*multi_duel_address, provider);

    loop {
        let random_bytes: [u8; 32] = rand::random();
        let nonce = U256::from_be_bytes(random_bytes);

        if nonce.is_zero() {
            continue;
        }

        let used = multi_duel.usedNonces(*agent, nonce).call().await?;
        if !used {
            return Ok(nonce);
        }
    }
}

/// Sign a JoinMultiAttestation EIP-712 message.
///
/// Returns `(signature, nonce, deadline)`.
async fn sign_multi_attestation(
    bet_tier: U256,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
) -> Result<(String, U256, U256)> {
    let addresses = contracts::resolve_addresses()?;
    let provider = contracts::create_provider(rpc_url).await?;
    let nonce = generate_nonce(&provider, &addresses.multi_duel, address).await?;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs();
    let deadline = U256::from(now_secs + timeout_secs);

    let chain_id = provider
        .get_chain_id()
        .await
        .context("Failed to get chain ID")?;

    let domain = Eip712Domain {
        name: Some("ClawDuel".into()),
        version: Some("1".into()),
        chain_id: Some(U256::from(chain_id)),
        verifying_contract: Some(addresses.multi_duel),
        salt: None,
    };

    let attestation = JoinMultiAttestation {
        agent: *address,
        betTier: bet_tier,
        nonce,
        deadline,
    };

    let signing_hash = attestation.eip712_signing_hash(&domain);
    let sig = signer
        .sign_hash(&signing_hash)
        .await
        .context("Failed to sign EIP-712 attestation")?;
    let signature = format!("0x{}", hex::encode(sig.as_bytes()));

    Ok((signature, nonce, deadline))
}

/// Create a new multi-duel lobby.
async fn cmd_create(
    client: &HttpClient,
    bet_size: u64,
    max_participants: u32,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Creating multi-duel lobby at {bet_size} USDC tier...");
    }

    let bet_tier = contracts::parse_usdc(bet_size as f64);
    let (signature, nonce, deadline) =
        sign_multi_attestation(bet_tier, timeout_secs, address, signer, rpc_url).await?;

    let body = serde_json::json!({
        "betTier": bet_tier.to_string(),
        "maxParticipants": max_participants,
        "signature": signature,
        "nonce": nonce.to_string(),
        "deadline": deadline.to_string(),
    });

    let (status, response) = client.post("/lobbies", &body).await?;
    let mut output = response.clone();
    output["status"] = serde_json::json!(status);

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            if (200..300).contains(&status) {
                let lobby_id = response
                    .get("lobbyId")
                    .or_else(|| response.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                println!("Lobby created: {lobby_id}");
            } else {
                let error = response
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("Create lobby failed ({status}): {error}");
            }
        }
    }

    Ok(())
}

/// Join an existing multi-duel lobby.
async fn cmd_join(
    client: &HttpClient,
    lobby_id: &str,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    let safe_id = security::sanitize_path_segment(lobby_id);

    if matches!(fmt, OutputFormat::Table) {
        println!("Joining lobby {safe_id}...");
    }

    // Look up lobby bet size first
    let lobby_data = client
        .get(&format!("/lobbies/{safe_id}"))
        .await
        .context("Failed to fetch lobby details")?;

    let bet_tier = lobby_data
        .get("betSize")
        .or_else(|| lobby_data.get("betTier"))
        .and_then(|v| v.as_str())
        .context("Lobby response missing betSize field")?
        .parse::<U256>()
        .context("Failed to parse lobby betSize")?;

    let (signature, nonce, deadline) =
        sign_multi_attestation(bet_tier, timeout_secs, address, signer, rpc_url).await?;

    let body = serde_json::json!({
        "betTier": bet_tier.to_string(),
        "signature": signature,
        "nonce": nonce.to_string(),
        "deadline": deadline.to_string(),
    });

    let (status, response) = client
        .post(&format!("/lobbies/{safe_id}/join"), &body)
        .await?;
    let mut output = response.clone();
    output["status"] = serde_json::json!(status);

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            if (200..300).contains(&status) {
                println!("Joined lobby: {safe_id}");
            } else {
                let error = response
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("Join lobby failed ({status}): {error}");
            }
        }
    }

    Ok(())
}

/// Tabled row for lobby list display.
#[derive(Tabled)]
struct LobbyRow {
    #[tabled(rename = "Lobby ID")]
    lobby_id: String,
    #[tabled(rename = "Bet Size")]
    bet_size: String,
    #[tabled(rename = "Participants")]
    participants: String,
    #[tabled(rename = "Status")]
    status: String,
}

/// List open lobbies.
async fn cmd_list(client: &HttpClient, fmt: OutputFormat) -> Result<()> {
    let data = client
        .get("/lobbies")
        .await
        .context("Failed to fetch lobbies")?;

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&data)?;
        }
        OutputFormat::Table => {
            let lobbies = data.as_array().unwrap_or(&Vec::new()).clone();

            let rows: Vec<LobbyRow> = lobbies
                .iter()
                .map(|lobby| {
                    let lobby_id = lobby
                        .get("id")
                        .or_else(|| lobby.get("lobbyId"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                        .to_string();

                    let bet_size = lobby
                        .get("betSize")
                        .or_else(|| lobby.get("betTier"))
                        .and_then(|v| v.as_str())
                        .map(|s| {
                            s.parse::<U256>()
                                .map(|u| contracts::format_usdc(u))
                                .unwrap_or_else(|_| s.to_string())
                        })
                        .unwrap_or_else(|| "?".to_string());

                    let current = lobby
                        .get("currentParticipants")
                        .or_else(|| lobby.get("participantCount"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let max = lobby
                        .get("maxParticipants")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let participants = format!("{current}/{max}");

                    let status = lobby
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    LobbyRow {
                        lobby_id,
                        bet_size,
                        participants,
                        status,
                    }
                })
                .collect();

            crate::output::print_table(&rows);
        }
    }

    Ok(())
}

/// Show detailed lobby status.
async fn cmd_status(client: &HttpClient, lobby_id: &str, fmt: OutputFormat) -> Result<()> {
    let safe_id = security::sanitize_path_segment(lobby_id);
    let data = client
        .get(&format!("/lobbies/{safe_id}"))
        .await
        .context("Failed to fetch lobby details")?;

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&data)?;
        }
        OutputFormat::Table => {
            let id = data
                .get("id")
                .or_else(|| data.get("lobbyId"))
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            let bet_size = data
                .get("betSize")
                .or_else(|| data.get("betTier"))
                .and_then(|v| v.as_str())
                .map(|s| {
                    s.parse::<U256>()
                        .map(|u| contracts::format_usdc(u))
                        .unwrap_or_else(|_| s.to_string())
                })
                .unwrap_or_else(|| "?".to_string());

            let max_participants = data
                .get("maxParticipants")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string());

            let participants_arr = data
                .get("participants")
                .and_then(|v| v.as_array());

            let current_participants = participants_arr
                .map(|a| a.len().to_string())
                .unwrap_or_else(|| {
                    data.get("currentParticipants")
                        .or_else(|| data.get("participantCount"))
                        .and_then(|v| v.as_u64())
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "?".to_string())
                });

            let status = data
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let created_at = data
                .get("createdAt")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            let participant_list = participants_arr
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            p.get("nickname")
                                .or_else(|| p.get("address"))
                                .and_then(|v| v.as_str())
                                .or_else(|| p.as_str())
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "-".to_string());

            crate::output::print_detail(vec![
                ("Lobby ID", id),
                ("Bet Size", bet_size),
                ("Max Participants", max_participants),
                ("Current Participants", current_participants),
                ("Status", status),
                ("Created At", created_at),
                ("Participants", participant_list),
            ]);
        }
    }

    Ok(())
}
