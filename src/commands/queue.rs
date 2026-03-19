//! Queue for a duel with EIP-712 attestation signing.

use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::signers::Signer;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::{Eip712Domain, SolStruct};
use anyhow::{Context, Result};

use crate::contracts::{self, IClawDuel, JoinDuelAttestation};
use crate::http::HttpClient;

/// Queue for a duel at the given bet tier with EIP-712 attestation.
pub async fn execute(
    client: &HttpClient,
    bet_tier_usdc: u64,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
) -> Result<()> {
    println!("Queuing for duel at {bet_tier_usdc} USDC tier...");

    let bet_tier = contracts::parse_usdc(bet_tier_usdc as f64);
    let addresses = contracts::resolve_addresses()?;
    let provider = contracts::create_provider(rpc_url).await?;

    // Generate unused nonce
    let nonce = generate_nonce(&provider, &addresses.claw_duel, address).await?;

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
        verifying_contract: Some(addresses.claw_duel),
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

    println!("Attestation signed, sending to matchmaker...");

    let body = serde_json::json!({
        "betTier": bet_tier.to_string(),
        "signature": signature,
        "nonce": nonce.to_string(),
        "deadline": deadline.to_string(),
    });
    let (status, response) = client.post("/duels/queue", &body).await?;

    if (200..300).contains(&status) {
        println!("OK: Queued for duel");
    } else {
        let error = response
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Unknown error");
        eprintln!("Queue failed ({status}): {error}");
    }

    let mut output = response.clone();
    output["status"] = serde_json::json!(status);
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
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
