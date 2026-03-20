//! Queue for a duel with EIP-712 attestation signing.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol_types::{Eip712Domain, SolStruct};
use anyhow::{Context, Result};

use crate::commands::poll;
use crate::contracts::{self, IClawDuel, JoinDuelAttestation};
use crate::http::HttpClient;
use crate::output::OutputFormat;
use crate::security;

/// Queue for a duel at the given bet tier with EIP-712 attestation.
///
/// When `games > 1`, runs a sequential game loop: queue -> wait for match ->
/// wait for resolution -> re-queue, for `games` total rounds.
pub async fn execute(
    client: &HttpClient,
    bet_tier_usdc: u64,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
    games: u64,
) -> Result<()> {
    if games <= 1 {
        // Single game: original behavior (queue once, return immediately)
        return queue_once(client, bet_tier_usdc, timeout_secs, address, signer, rpc_url, fmt)
            .await;
    }

    // Multi-game sequential loop
    games_loop(
        client,
        bet_tier_usdc,
        timeout_secs,
        address,
        signer,
        rpc_url,
        fmt,
        games,
    )
    .await
}

/// Run the sequential multi-game loop.
async fn games_loop(
    client: &HttpClient,
    bet_tier_usdc: u64,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
    games: u64,
) -> Result<()> {
    let mut results: Vec<serde_json::Value> = Vec::new();

    for game_num in 1..=games {
        if matches!(fmt, OutputFormat::Table) {
            println!("\n=== Game {game_num}/{games} ===");
        }

        // Step 1: Queue for a duel
        queue_once(client, bet_tier_usdc, timeout_secs, address, signer, rpc_url, fmt).await
            .map_err(|e| {
                if matches!(fmt, OutputFormat::Table) && game_num > 1 {
                    eprintln!("Completed {}/{games} games before error", game_num - 1);
                }
                e
            })?;

        // Step 2: Wait for match assignment (poll until waiting_submissions with problem)
        if matches!(fmt, OutputFormat::Table) {
            println!("Waiting for match assignment...");
        }

        let match_data = wait_for_match(client, address, 3, 300).await.map_err(|e| {
            if matches!(fmt, OutputFormat::Table) && game_num > 1 {
                eprintln!("Completed {}/{games} games before error", game_num - 1);
            }
            e
        })?;

        let match_id = match_data
            .get("match")
            .and_then(|m| {
                m.get("id")
                    .or_else(|| m.get("matchId"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("unknown");

        if matches!(fmt, OutputFormat::Table) {
            let problem = match_data
                .get("match")
                .and_then(|m| m.get("problemTitle").and_then(|t| t.as_str()))
                .unwrap_or("unknown");
            println!("Match assigned: {match_id} (problem: {problem})");
        }

        // Step 3: Wait for resolution
        if matches!(fmt, OutputFormat::Table) {
            println!("Waiting for match resolution...");
        }

        let resolved = wait_for_resolution(client, match_id, 10, 600).await.map_err(|e| {
            if matches!(fmt, OutputFormat::Table) && game_num > 1 {
                eprintln!("Completed {}/{games} games before error", game_num - 1);
            }
            e
        })?;

        // Extract result summary
        let winner = resolved
            .get("winner")
            .and_then(|w| w.as_str())
            .unwrap_or("draw");
        let status = resolved
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        if matches!(fmt, OutputFormat::Table) {
            println!("Game {game_num}: {status} - winner: {winner}");
        }

        // Collect result for JSON mode
        results.push(serde_json::json!({
            "game": game_num,
            "matchId": match_id,
            "status": status,
            "winner": winner,
        }));

        // Sleep before next game (unless last)
        if game_num < games {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    // In JSON mode, emit the collected results array
    if matches!(fmt, OutputFormat::Json) {
        let output = serde_json::Value::Array(results);
        crate::output::print_json(&output)?;
    }

    if matches!(fmt, OutputFormat::Table) {
        println!("\nAll {games} games completed.");
    }

    Ok(())
}

/// Execute a single queue operation (sign EIP-712 attestation, POST to matchmaker).
async fn queue_once(
    client: &HttpClient,
    bet_tier_usdc: u64,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Queuing for duel at {bet_tier_usdc} USDC tier...");
    }

    let bet_tier = contracts::parse_usdc(bet_tier_usdc as f64);
    let provider = contracts::create_provider(rpc_url).await?;

    // Generate unused nonce
    let nonce = generate_nonce(&provider, &contracts::claw_duel_address(), address).await?;

    // Calculate deadline
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs();
    let deadline = U256::from(now_secs + timeout_secs);

    // Get chain ID
    let chain_id = provider
        .get_chain_id()
        .await
        .context("Failed to get chain ID")?;

    // Build EIP-712 domain
    let domain = Eip712Domain {
        name: Some("ClawDuel".into()),
        version: Some("1".into()),
        chain_id: Some(U256::from(chain_id)),
        verifying_contract: Some(contracts::claw_duel_address()),
        salt: None,
    };

    // Build attestation value
    let attestation = JoinDuelAttestation {
        agent: *address,
        betTier: bet_tier,
        nonce,
        deadline,
    };

    // Sign EIP-712 typed data (compute hash then sign)
    let signing_hash = attestation.eip712_signing_hash(&domain);
    let sig = signer
        .sign_hash(&signing_hash)
        .await
        .context("Failed to sign EIP-712 attestation")?;
    let signature = format!("0x{}", hex::encode(sig.as_bytes()));

    if matches!(fmt, OutputFormat::Table) {
        println!("Attestation signed, sending to matchmaker...");
    }

    let body = serde_json::json!({
        "agentAddress": format!("{address:#x}"),
        "betTier": bet_tier.to_string(),
        "signature": signature,
        "nonce": nonce.to_string(),
        "deadline": deadline.to_string(),
    });
    let (status, response) = client.post("/duels/queue", &body).await?;

    let mut output = response.clone();
    output["status"] = serde_json::json!(status);

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            if (200..300).contains(&status) {
                println!("OK: Queued for duel");
            } else {
                let error = response
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("Queue failed ({status}): {error}");
            }
        }
    }

    Ok(())
}

/// Wait for a match to be assigned by polling the active match endpoint.
///
/// Returns the poll data once a match with `waiting_submissions` status and
/// a non-null problem is found.
async fn wait_for_match(
    client: &HttpClient,
    address: &Address,
    interval_secs: u64,
    timeout_secs: u64,
) -> Result<serde_json::Value> {
    let start = Instant::now();

    loop {
        let data = poll::poll_once(client, address).await?;

        if let Some(m) = data.get("match") {
            let status = m.get("status").and_then(|s| s.as_str()).unwrap_or("");
            let has_problem = m.get("problem").map_or(false, |p| !p.is_null());

            if status == "waiting_submissions" && has_problem {
                return Ok(data);
            }
        }

        if start.elapsed().as_secs() > timeout_secs {
            anyhow::bail!(
                "Timeout waiting for match assignment after {}s",
                timeout_secs
            );
        }

        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

/// Wait for a match to reach `resolved` status by polling its detail endpoint.
async fn wait_for_resolution(
    client: &HttpClient,
    match_id: &str,
    interval_secs: u64,
    timeout_secs: u64,
) -> Result<serde_json::Value> {
    let start = Instant::now();

    loop {
        let safe_id = security::sanitize_path_segment(match_id);
        let data = client.get(&format!("/api/matches/{safe_id}")).await?;

        if data.get("status").and_then(|s| s.as_str()) == Some("resolved") {
            return Ok(data);
        }

        if start.elapsed().as_secs() > timeout_secs {
            anyhow::bail!(
                "Timeout waiting for match resolution after {}s",
                timeout_secs
            );
        }

        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

/// Generate a random unused nonce for attestations.
async fn generate_nonce(
    provider: &impl Provider,
    claw_duel_address: &Address,
    agent: &Address,
) -> Result<U256> {
    let claw_duel = IClawDuel::new(*claw_duel_address, provider);

    loop {
        // Generate random 32 bytes
        let random_bytes: [u8; 32] = rand::random();
        let nonce = U256::from_be_bytes(random_bytes);

        // Skip zero
        if nonce.is_zero() {
            continue;
        }

        // Check if already used on-chain
        let used = claw_duel.usedNonces(*agent, nonce).call().await?;
        if !used {
            return Ok(nonce);
        }
    }
}
