//! Queue for matchmaking with EIP-712 attestation signing.
//! Default: multi-competition (up to 20 players). Use --duel for 1v1.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol_types::{Eip712Domain, SolStruct};
use anyhow::{Context, Result};

use crate::commands::poll;
use crate::contracts::{
    self, ICompetition, IMultiCompetition, JoinCompetitionAttestation,
    JoinMultiCompetitionAttestation,
};
use crate::http::HttpClient;
use crate::output::OutputFormat;
use crate::security;

/// Queue for a match at the given bet tier with EIP-712 attestation.
///
/// When `duel` is false (default), queues for multi-competition.
/// When `duel` is true, queues for 1v1 duel.
/// When `games > 1`, runs a sequential game loop.
pub async fn execute(
    client: &HttpClient,
    entry_fee_usdc: u64,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
    games: u64,
    duel: bool,
) -> Result<()> {
    if games <= 1 {
        return queue_once(client, entry_fee_usdc, timeout_secs, address, signer, rpc_url, fmt, duel)
            .await;
    }

    games_loop(client, entry_fee_usdc, timeout_secs, address, signer, rpc_url, fmt, games, duel)
        .await
}

/// Run the sequential multi-game loop.
async fn games_loop(
    client: &HttpClient,
    entry_fee_usdc: u64,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
    games: u64,
    duel: bool,
) -> Result<()> {
    let mut results: Vec<serde_json::Value> = Vec::new();
    let mode_label = if duel { "duel" } else { "competition" };

    for game_num in 1..=games {
        if matches!(fmt, OutputFormat::Table) {
            println!("\n=== Game {game_num}/{games} ({mode_label}) ===");
        }

        queue_once(client, entry_fee_usdc, timeout_secs, address, signer, rpc_url, fmt, duel)
            .await
            .map_err(|e| {
                if matches!(fmt, OutputFormat::Table) && game_num > 1 {
                    eprintln!("Completed {}/{games} games before error", game_num - 1);
                }
                e
            })?;

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
            .and_then(|m| m.get("id").or_else(|| m.get("matchId")).and_then(|v| v.as_str()))
            .unwrap_or("unknown");

        if matches!(fmt, OutputFormat::Table) {
            let problem = match_data
                .get("match")
                .and_then(|m| m.get("problemTitle").and_then(|t| t.as_str()))
                .unwrap_or("unknown");
            println!("Match assigned: {match_id} (problem: {problem})");
        }

        if matches!(fmt, OutputFormat::Table) {
            println!("Waiting for match resolution...");
        }

        let resolved = wait_for_resolution(client, match_id, 10, 600).await.map_err(|e| {
            if matches!(fmt, OutputFormat::Table) && game_num > 1 {
                eprintln!("Completed {}/{games} games before error", game_num - 1);
            }
            e
        })?;

        let winner = resolved.get("winner").and_then(|w| w.as_str()).unwrap_or("draw");
        let status = resolved.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");

        if matches!(fmt, OutputFormat::Table) {
            println!("Game {game_num}: {status} - winner: {winner}");
        }

        results.push(serde_json::json!({
            "game": game_num,
            "matchId": match_id,
            "status": status,
            "winner": winner,
        }));

        if game_num < games {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    if matches!(fmt, OutputFormat::Json) {
        crate::output::print_json(&serde_json::Value::Array(results))?;
    }

    if matches!(fmt, OutputFormat::Table) {
        println!("\nAll {games} games completed.");
    }

    Ok(())
}

/// Execute a single queue operation.
async fn queue_once(
    client: &HttpClient,
    entry_fee_usdc: u64,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
    duel: bool,
) -> Result<()> {
    let mode_label = if duel { "duel" } else { "competition" };

    if matches!(fmt, OutputFormat::Table) {
        println!("Queuing for {mode_label} at {entry_fee_usdc} USDC tier...");
    }

    let entry_fee = contracts::parse_usdc(entry_fee_usdc as f64);
    let provider = contracts::create_provider(rpc_url).await?;

    let (signature, nonce, deadline, mode) = if duel {
        let (sig, nonce, dl) = sign_duel_attestation(&provider, entry_fee, timeout_secs, address, signer).await?;
        (sig, nonce, dl, "duel")
    } else {
        let (sig, nonce, dl) = sign_multi_attestation(&provider, entry_fee, timeout_secs, address, signer).await?;
        (sig, nonce, dl, "multi")
    };

    if matches!(fmt, OutputFormat::Table) {
        println!("Attestation signed, sending to matchmaker...");
    }

    let body = serde_json::json!({
        "agentAddress": format!("{address:#x}"),
        "entryFee": entry_fee.to_string(),
        "signature": signature,
        "nonce": nonce.to_string(),
        "deadline": deadline.to_string(),
        "mode": mode,
    });
    let (status, response) = client.post("/competitions/queue", &body).await?;

    let mut output = response.clone();
    output["status"] = serde_json::json!(status);

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            if (200..300).contains(&status) {
                println!("OK: Queued for {mode_label}");
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

// ── Signing helpers ──────────────────────────────────────────────────

/// Sign a JoinCompetitionAttestation for 1v1 duels.
async fn sign_duel_attestation(
    provider: &impl Provider,
    entry_fee: U256,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
) -> Result<(String, U256, U256)> {
    let nonce = generate_duel_nonce(provider, &contracts::competition_address(), address).await?;
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let deadline = U256::from(now_secs + timeout_secs);

    let chain_id = provider.get_chain_id().await.context("Failed to get chain ID")?;

    let domain = Eip712Domain {
        name: Some("ClawDuel".into()),
        version: Some("1".into()),
        chain_id: Some(U256::from(chain_id)),
        verifying_contract: Some(contracts::competition_address()),
        salt: None,
    };

    let attestation = JoinCompetitionAttestation {
        agent: *address,
        entryFee: entry_fee,
        nonce,
        deadline,
    };

    let signing_hash = attestation.eip712_signing_hash(&domain);
    let sig = signer.sign_hash(&signing_hash).await.context("Failed to sign")?;
    Ok((format!("0x{}", hex::encode(sig.as_bytes())), nonce, deadline))
}

/// Sign a JoinMultiCompetitionAttestation for multi-competitions.
async fn sign_multi_attestation(
    provider: &impl Provider,
    entry_fee: U256,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
) -> Result<(String, U256, U256)> {
    let nonce = generate_multi_nonce(provider, &contracts::multi_competition_address(), address).await?;
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let deadline = U256::from(now_secs + timeout_secs);

    let chain_id = provider.get_chain_id().await.context("Failed to get chain ID")?;

    let domain = Eip712Domain {
        name: Some("ClawDuel".into()),
        version: Some("1".into()),
        chain_id: Some(U256::from(chain_id)),
        verifying_contract: Some(contracts::multi_competition_address()),
        salt: None,
    };

    let attestation = JoinMultiCompetitionAttestation {
        agent: *address,
        competitionId: U256::ZERO,
        entryFee: entry_fee,
        nonce,
        deadline,
    };

    let signing_hash = attestation.eip712_signing_hash(&domain);
    let sig = signer.sign_hash(&signing_hash).await.context("Failed to sign")?;
    Ok((format!("0x{}", hex::encode(sig.as_bytes())), nonce, deadline))
}

// ── Nonce generation ──────────────────────────────────────────────────

async fn generate_duel_nonce(
    provider: &impl Provider,
    competition_address: &Address,
    agent: &Address,
) -> Result<U256> {
    let contract = ICompetition::new(*competition_address, provider);
    loop {
        let random_bytes: [u8; 32] = rand::random();
        let nonce = U256::from_be_bytes(random_bytes);
        if nonce.is_zero() { continue; }
        let used = contract.usedNonces(*agent, nonce).call().await?;
        if !used {
            return Ok(nonce);
        }
    }
}

async fn generate_multi_nonce(
    provider: &impl Provider,
    multi_competition_address: &Address,
    agent: &Address,
) -> Result<U256> {
    let contract = IMultiCompetition::new(*multi_competition_address, provider);
    loop {
        let random_bytes: [u8; 32] = rand::random();
        let nonce = U256::from_be_bytes(random_bytes);
        if nonce.is_zero() { continue; }
        let used = contract.usedNonces(*agent, nonce).call().await?;
        if !used {
            return Ok(nonce);
        }
    }
}

// ── Wait helpers ──────────────────────────────────────────────────────

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
            anyhow::bail!("Timeout waiting for match assignment after {}s", timeout_secs);
        }
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

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
            anyhow::bail!("Timeout waiting for match resolution after {}s", timeout_secs);
        }
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}
