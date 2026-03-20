//! Multi-duel lobby commands: create, join, list, status, and play.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol_types::{Eip712Domain, SolStruct};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use tabled::Tabled;

use crate::commands::{match_detail, poll};
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
        /// Wait for lobby to fill and match to start
        #[arg(long)]
        wait: bool,
        /// Wait until match is resolved (implies --wait)
        #[arg(long)]
        wait_for_resolution: bool,
        /// Polling interval in seconds (default: 5)
        #[arg(long, default_value = "5")]
        wait_interval: u64,
        /// Maximum wait time for lobby to fill in seconds (default: 600)
        #[arg(long, default_value = "600")]
        wait_timeout: u64,
    },
    /// Join an existing multi-duel lobby
    Join {
        /// Lobby ID
        lobby_id: String,
        /// Attestation timeout in seconds
        #[arg(long, default_value = "3600")]
        timeout: u64,
        /// Wait for lobby to fill and match to start
        #[arg(long)]
        wait: bool,
        /// Wait until match is resolved (implies --wait)
        #[arg(long)]
        wait_for_resolution: bool,
        /// Polling interval in seconds (default: 5)
        #[arg(long, default_value = "5")]
        wait_interval: u64,
        /// Maximum wait time for lobby to fill in seconds (default: 600)
        #[arg(long, default_value = "600")]
        wait_timeout: u64,
    },
    /// List open lobbies
    List,
    /// Show lobby details and participants
    Status {
        /// Lobby ID
        lobby_id: String,
        /// Wait until lobby is full
        #[arg(long)]
        wait: bool,
        /// Polling interval in seconds (default: 5)
        #[arg(long, default_value = "5")]
        wait_interval: u64,
        /// Maximum wait time in seconds (default: 600)
        #[arg(long, default_value = "600")]
        wait_timeout: u64,
    },
    /// Join a lobby and play: wait for lobby to fill, poll for match, show problem
    Play {
        /// Lobby ID to join
        lobby_id: String,
        /// Attestation timeout in seconds
        #[arg(long, default_value = "3600")]
        timeout: u64,
        /// Wait until match is resolved after showing problem
        #[arg(long)]
        wait_for_resolution: bool,
        /// Lobby fill polling interval in seconds (default: 5)
        #[arg(long, default_value = "5")]
        wait_interval: u64,
        /// Maximum wait time for lobby to fill in seconds (default: 600)
        #[arg(long, default_value = "600")]
        lobby_timeout: u64,
        /// Maximum wait time for match assignment in seconds (default: 300)
        #[arg(long, default_value = "300")]
        match_timeout: u64,
        /// Resolution polling interval in seconds (default: 10)
        #[arg(long, default_value = "10")]
        resolution_interval: u64,
        /// Maximum wait time for resolution in seconds (default: 1800)
        #[arg(long, default_value = "1800")]
        resolution_timeout: u64,
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
            wait,
            wait_for_resolution,
            wait_interval,
            wait_timeout,
        } => {
            let lobby_id = cmd_create(
                client,
                bet_size,
                max_participants,
                timeout,
                address,
                signer,
                rpc_url,
                fmt,
            )
            .await?;

            if wait || wait_for_resolution {
                if let Some(ref id) = lobby_id {
                    let match_id = wait_for_lobby_and_match(
                        client, id, wait_interval, wait_timeout, 300, address, fmt,
                    )
                    .await?;

                    if wait_for_resolution {
                        if let Some(ref mid) = match_id {
                            wait_for_match_resolution(client, mid, 10, 1800, fmt).await?;
                        }
                    }
                }
            }

            Ok(())
        }
        LobbyCommand::Join {
            lobby_id,
            timeout,
            wait,
            wait_for_resolution,
            wait_interval,
            wait_timeout,
        } => {
            cmd_join(client, &lobby_id, timeout, address, signer, rpc_url, fmt).await?;

            if wait || wait_for_resolution {
                let match_id = wait_for_lobby_and_match(
                    client,
                    &lobby_id,
                    wait_interval,
                    wait_timeout,
                    300,
                    address,
                    fmt,
                )
                .await?;

                if wait_for_resolution {
                    if let Some(ref mid) = match_id {
                        wait_for_match_resolution(client, mid, 10, 1800, fmt).await?;
                    }
                }
            }

            Ok(())
        }
        LobbyCommand::List => cmd_list(client, fmt).await,
        LobbyCommand::Status {
            lobby_id,
            wait,
            wait_interval,
            wait_timeout,
        } => {
            if wait {
                wait_for_lobby_fill(client, &lobby_id, wait_interval, wait_timeout, fmt).await?;
            }
            cmd_status(client, &lobby_id, fmt).await
        }
        LobbyCommand::Play {
            lobby_id,
            timeout,
            wait_for_resolution,
            wait_interval,
            lobby_timeout,
            match_timeout,
            resolution_interval,
            resolution_timeout,
        } => {
            cmd_play(
                client,
                &lobby_id,
                timeout,
                wait_interval,
                lobby_timeout,
                match_timeout,
                wait_for_resolution,
                resolution_interval,
                resolution_timeout,
                address,
                signer,
                rpc_url,
                fmt,
            )
            .await
        }
    }
}

// ── Shared waiting helpers ──────────────────────────────────────────

/// Poll a lobby until it fills up (status == "started" or count >= max).
/// Returns `Ok(())` on success or errors on timeout.
async fn wait_for_lobby_fill(
    client: &HttpClient,
    lobby_id: &str,
    interval_secs: u64,
    timeout_secs: u64,
    fmt: OutputFormat,
) -> Result<()> {
    let safe_id = security::sanitize_path_segment(lobby_id);
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    let interval = Duration::from_secs(interval_secs);

    if matches!(fmt, OutputFormat::Table) {
        println!("Waiting for lobby to fill...");
    }

    loop {
        let data = client
            .get(&format!("/lobbies/{safe_id}"))
            .await
            .context("Failed to poll lobby status")?;

        let status = data
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let current = data
            .get("currentParticipants")
            .or_else(|| data.get("participantCount"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let max = data
            .get("maxParticipants")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if matches!(fmt, OutputFormat::Table) {
            println!(
                "[{:>3}s] Lobby: {current}/{max} participants, status: {status}",
                start.elapsed().as_secs()
            );
        }

        if status == "started" || current >= max {
            if matches!(fmt, OutputFormat::Table) {
                println!("Lobby is full!");
            }
            return Ok(());
        }

        if start.elapsed() >= timeout {
            anyhow::bail!(
                "Timeout waiting for lobby to fill after {}s ({current}/{max} participants)",
                timeout_secs
            );
        }

        tokio::time::sleep(interval).await;
    }
}

/// Poll for the agent's active match to reach `waiting_submissions` with a problem.
/// Returns the match ID if found.
async fn wait_for_match_assignment(
    client: &HttpClient,
    address: &Address,
    timeout_secs: u64,
    fmt: OutputFormat,
) -> Result<Option<String>> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    let interval = Duration::from_secs(3);

    if matches!(fmt, OutputFormat::Table) {
        println!("Waiting for match assignment...");
    }

    loop {
        let data = poll::poll_once(client, address).await?;

        if let Some(m) = data.get("match") {
            let status = m.get("status").and_then(|s| s.as_str()).unwrap_or("");
            let has_problem = m.get("problem").map_or(false, |p| !p.is_null());
            let duel_type = m
                .get("duelType")
                .and_then(|d| d.as_str())
                .unwrap_or("");

            if matches!(fmt, OutputFormat::Table) {
                println!(
                    "[{:>3}s] Match status: {status} (type: {duel_type})",
                    start.elapsed().as_secs()
                );
            }

            if status == "waiting_submissions" && has_problem {
                let match_id = m
                    .get("id")
                    .or_else(|| m.get("matchId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                print_match_problem(m, &match_id, duel_type, fmt)?;
                return Ok(Some(match_id));
            }
        } else if matches!(fmt, OutputFormat::Table) {
            println!(
                "[{:>3}s] No active match yet...",
                start.elapsed().as_secs()
            );
        }

        if start.elapsed() >= timeout {
            anyhow::bail!(
                "Timeout waiting for match assignment after {}s",
                timeout_secs
            );
        }

        tokio::time::sleep(interval).await;
    }
}

/// Wait for lobby fill + match assignment. Returns match ID if found.
async fn wait_for_lobby_and_match(
    client: &HttpClient,
    lobby_id: &str,
    lobby_interval: u64,
    lobby_timeout: u64,
    match_timeout: u64,
    address: &Address,
    fmt: OutputFormat,
) -> Result<Option<String>> {
    wait_for_lobby_fill(client, lobby_id, lobby_interval, lobby_timeout, fmt).await?;
    wait_for_match_assignment(client, address, match_timeout, fmt).await
}

/// Wait for a match to resolve by polling /api/matches/:id.
async fn wait_for_match_resolution(
    client: &HttpClient,
    match_id: &str,
    interval_secs: u64,
    timeout_secs: u64,
    fmt: OutputFormat,
) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Waiting for match resolution...");
    }

    match_detail::execute(client, match_id, fmt, true, interval_secs, timeout_secs).await
}

/// Print the match problem details and submit hint.
fn print_match_problem(
    m: &serde_json::Value,
    match_id: &str,
    duel_type: &str,
    fmt: OutputFormat,
) -> Result<()> {
    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(m)?;
        }
        OutputFormat::Table => {
            println!("\nMatch ready!");

            let problem = m.get("problem");
            let prompt = problem
                .and_then(|p| p.get("prompt"))
                .and_then(|p| p.as_str())
                .unwrap_or("-");
            let problem_type = problem
                .and_then(|p| p.get("type"))
                .and_then(|t| t.as_str())
                .unwrap_or("-");
            let deadline = problem
                .and_then(|p| p.get("deadline"))
                .map(|d| d.to_string())
                .unwrap_or_else(|| "-".to_string());

            crate::output::print_detail(vec![
                ("Match ID", match_id.to_string()),
                ("Type", duel_type.to_string()),
                ("Problem", prompt.to_string()),
                ("Answer Type", problem_type.to_string()),
                ("Deadline", deadline),
            ]);

            let multi_flag = if duel_type == "multiduel" {
                " --multi"
            } else {
                ""
            };
            println!(
                "\nSubmit your prediction with:\n  clawduel submit --match-id {match_id} --prediction <value>{multi_flag}"
            );
        }
    }
    Ok(())
}

// ── Nonce / Signing ─────────────────────────────────────────────────

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
    let provider = contracts::create_provider(rpc_url).await?;
    let nonce = generate_nonce(&provider, &contracts::multi_duel_address(), address).await?;

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
        verifying_contract: Some(contracts::multi_duel_address()),
        salt: None,
    };

    let attestation = JoinMultiAttestation {
        agent: *address,
        duelId: U256::ZERO,
        betSize: bet_tier,
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

// ── Subcommand implementations ──────────────────────────────────────

/// Create a new multi-duel lobby. Returns the lobby ID on success.
async fn cmd_create(
    client: &HttpClient,
    bet_size: u64,
    max_participants: u32,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<Option<String>> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Creating multi-duel lobby at {bet_size} USDC tier...");
    }

    let bet_tier = contracts::parse_usdc(bet_size as f64);
    let (signature, nonce, deadline) =
        sign_multi_attestation(bet_tier, timeout_secs, address, signer, rpc_url).await?;

    let body = serde_json::json!({
        "agentAddress": format!("{address:#x}"),
        "betSize": bet_tier.to_string(),
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

    let lobby_id = response
        .get("lobbyId")
        .or_else(|| response.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(lobby_id)
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
        "agentAddress": format!("{address:#x}"),
        "betSize": bet_tier.to_string(),
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
                let participants = response
                    .get("participants")
                    .and_then(|v| v.as_u64())
                    .map(|n| format!(" ({n} participants)"))
                    .unwrap_or_default();
                println!("Joined lobby: {safe_id}{participants}");
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

            if lobbies.is_empty() {
                println!("No open lobbies");
                return Ok(());
            }

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

/// Join a lobby and play the full lifecycle:
/// 1. Join the lobby (sign + POST)
/// 2. Poll until the lobby is full (started)
/// 3. Poll for match assignment (waiting_submissions with problem)
/// 4. Optionally wait for resolution
async fn cmd_play(
    client: &HttpClient,
    lobby_id: &str,
    timeout_secs: u64,
    poll_interval: u64,
    lobby_timeout: u64,
    match_timeout: u64,
    wait_for_resolution: bool,
    resolution_interval: u64,
    resolution_timeout: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    // Step 1: Join the lobby
    if matches!(fmt, OutputFormat::Table) {
        println!("Step 1/3: Joining lobby...");
    }
    cmd_join(client, lobby_id, timeout_secs, address, signer, rpc_url, fmt).await?;

    // Step 2: Wait for lobby to fill
    if matches!(fmt, OutputFormat::Table) {
        println!("\nStep 2/3: Waiting for lobby to fill...");
    }
    wait_for_lobby_fill(client, lobby_id, poll_interval, lobby_timeout, fmt).await?;

    // Step 3: Wait for match assignment
    if matches!(fmt, OutputFormat::Table) {
        println!("\nStep 3/3: Waiting for match to start...");
    }
    let match_id = wait_for_match_assignment(client, address, match_timeout, fmt).await?;

    // Step 4 (optional): Wait for resolution
    if wait_for_resolution {
        if let Some(ref mid) = match_id {
            if matches!(fmt, OutputFormat::Table) {
                println!("\nWaiting for match resolution...");
            }
            wait_for_match_resolution(client, mid, resolution_interval, resolution_timeout, fmt)
                .await?;
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

            let participants_arr = data.get("participants").and_then(|v| v.as_array());

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
