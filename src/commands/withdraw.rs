//! Withdraw USDC from the PrizePool using a gasless relayed authorization.

use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, U256};
use alloy::signers::Signer;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::{Eip712Domain, SolStruct};
use anyhow::{Context, Result, bail};

use crate::contracts::{self, IPrizePool, WithdrawAuthorization};
use crate::http::HttpClient;
use crate::output::OutputFormat;

pub async fn execute(
    client: &HttpClient,
    amount_usdc: f64,
    recipient: Option<Address>,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    let recipient = recipient.unwrap_or(*address);
    if matches!(fmt, OutputFormat::Table) {
        println!("Preparing gasless withdrawal of {amount_usdc} USDC...");
    }

    let amount = contracts::parse_usdc(amount_usdc);
    let config = client.get("/api/gasless/config").await?;
    let fee_amount = u256_from_json(&config, "withdrawFee")?;
    let auth_valid_seconds = config
        .get("authValidSeconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(600);
    let chain_id = config
        .get("chainId")
        .and_then(|v| v.as_u64())
        .context("Missing chainId in gasless config")?;
    let prize_pool_address: Address = config
        .get("prizePoolAddress")
        .and_then(|v| v.as_str())
        .context("Missing prizePoolAddress in gasless config")?
        .parse()
        .context("Invalid prizePoolAddress in gasless config")?;

    let provider = contracts::create_provider(rpc_url).await?;
    let bank = IPrizePool::new(prize_pool_address, &provider);
    let balance = bank.balanceOf(*address).call().await?;
    let debit_amount = amount + fee_amount;
    if balance < debit_amount {
        let have = contracts::format_usdc(balance);
        let need = contracts::format_usdc(debit_amount);
        bail!("Insufficient PrizePool balance. Have {have}, need {need}");
    }

    let safe_address = format!("{address:#x}");
    let nonce_response = client
        .get(&format!("/api/withdrawals/nonce/{safe_address}"))
        .await?;
    let nonce = nonce_response
        .get("nonce")
        .and_then(|v| v.as_str())
        .context("Missing withdrawal nonce")?;
    let nonce = U256::from_str_radix(nonce, 10).context("Invalid withdrawal nonce")?;
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let deadline = U256::from(now_secs + auth_valid_seconds);

    let domain = Eip712Domain {
        name: Some("ClawDuelPrizePool".into()),
        version: Some("1".into()),
        chain_id: Some(U256::from(chain_id)),
        verifying_contract: Some(prize_pool_address),
        salt: None,
    };
    let authorization = WithdrawAuthorization {
        agent: *address,
        recipient,
        amount,
        feeAmount: fee_amount,
        nonce,
        deadline,
    };
    let signing_hash = authorization.eip712_signing_hash(&domain);
    let sig = signer
        .sign_hash(&signing_hash)
        .await
        .context("Failed to sign withdrawal authorization")?;
    let signature = format!("0x{}", hex::encode(sig.as_bytes()));

    if matches!(fmt, OutputFormat::Table) {
        println!("Authorization signed, relaying withdrawal...");
    }

    let body = serde_json::json!({
        "agentAddress": format!("{address:#x}"),
        "recipient": format!("{recipient:#x}"),
        "amount": amount.to_string(),
        "feeAmount": fee_amount.to_string(),
        "nonce": nonce.to_string(),
        "deadline": deadline.to_string(),
        "signature": signature,
    });
    let (status, response) = client.post("/api/withdrawals/gasless", &body).await?;
    if !(200..300).contains(&status) {
        let error = response
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Unknown error");
        bail!("Gasless withdrawal failed ({status}): {error}");
    }

    match fmt {
        OutputFormat::Json => crate::output::print_json(&response)?,
        OutputFormat::Table => {
            let fee = contracts::format_usdc(fee_amount);
            let withdrawn = contracts::format_usdc(amount);
            let tx = response
                .get("txHash")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            println!("OK: Withdrew {withdrawn} USDC gaslessly (fee {fee} USDC)");
            println!("Recipient: {recipient:#x}");
            println!("Tx: {tx}");
        }
    }

    Ok(())
}

fn u256_from_json(value: &serde_json::Value, key: &str) -> Result<U256> {
    let raw = value
        .get(key)
        .and_then(|v| v.as_str())
        .with_context(|| format!("Missing {key} in response"))?;
    U256::from_str_radix(raw, 10).with_context(|| format!("Invalid {key}: {raw}"))
}
