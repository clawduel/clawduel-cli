//! Deposit USDC to the bank contract.

use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, FixedBytes, U256};
use alloy::signers::Signer;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::{Eip712Domain, SolStruct};
use anyhow::{Context, Result, bail};

use crate::contracts::{self, IERC20, IPrizePool, ReceiveWithAuthorization};
use crate::http::HttpClient;
use crate::output::OutputFormat;

/// Deposit USDC. Gasless by default; use `direct` for approve + deposit.
pub async fn execute(
    client: &HttpClient,
    amount_usdc: f64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
    direct: bool,
) -> Result<()> {
    if direct {
        return execute_direct(amount_usdc, address, signer, rpc_url, fmt).await;
    }
    execute_gasless(client, amount_usdc, address, signer, rpc_url, fmt).await
}

async fn execute_gasless(
    client: &HttpClient,
    amount_usdc: f64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Preparing gasless deposit of {amount_usdc} USDC...");
    }

    let credit_amount = contracts::parse_usdc(amount_usdc);
    let config = client.get("/api/gasless/config").await?;
    let fee_amount = u256_from_json(&config, "depositFee")?;
    let auth_valid_seconds = config
        .get("authValidSeconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(600);
    let usdc_address: Address = config
        .get("usdcAddress")
        .and_then(|v| v.as_str())
        .context("Missing usdcAddress in gasless config")?
        .parse()
        .context("Invalid usdcAddress in gasless config")?;
    let prize_pool_address: Address = config
        .get("prizePoolAddress")
        .and_then(|v| v.as_str())
        .context("Missing prizePoolAddress in gasless config")?
        .parse()
        .context("Invalid prizePoolAddress in gasless config")?;

    let provider = contracts::create_provider(rpc_url).await?;
    let usdc = IERC20::new(usdc_address, &provider);
    let transfer_amount = credit_amount + fee_amount;

    let balance = usdc.balanceOf(*address).call().await?;
    if balance < transfer_amount {
        let have = contracts::format_usdc(balance);
        let need = contracts::format_usdc(transfer_amount);
        bail!("Insufficient USDC. Have {have}, need {need}");
    }

    let chain_id = config
        .get("chainId")
        .and_then(|v| v.as_u64())
        .context("Missing chainId in gasless config")?;
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let valid_after = U256::from(now_secs.saturating_sub(60));
    let valid_before = U256::from(now_secs + auth_valid_seconds);
    let nonce_bytes: [u8; 32] = rand::random();
    let nonce = FixedBytes::<32>::from(nonce_bytes);

    let domain = Eip712Domain {
        name: Some("USD Coin".into()),
        version: Some("2".into()),
        chain_id: Some(U256::from(chain_id)),
        verifying_contract: Some(usdc_address),
        salt: None,
    };
    let authorization = ReceiveWithAuthorization {
        from: *address,
        to: prize_pool_address,
        value: transfer_amount,
        validAfter: valid_after,
        validBefore: valid_before,
        nonce,
    };
    let signing_hash = authorization.eip712_signing_hash(&domain);
    let sig = signer
        .sign_hash(&signing_hash)
        .await
        .context("Failed to sign USDC authorization")?;
    let signature = format!("0x{}", hex::encode(sig.as_bytes()));
    let nonce_hex = format!("0x{}", hex::encode(nonce_bytes));

    if matches!(fmt, OutputFormat::Table) {
        println!("Authorization signed, relaying deposit...");
    }

    let body = serde_json::json!({
        "agentAddress": format!("{address:#x}"),
        "creditAmount": credit_amount.to_string(),
        "feeAmount": fee_amount.to_string(),
        "validAfter": valid_after.to_string(),
        "validBefore": valid_before.to_string(),
        "nonce": nonce_hex,
        "signature": signature,
    });
    let (status, response) = client.post("/api/deposits/gasless", &body).await?;
    if !(200..300).contains(&status) {
        let error = response
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Unknown error");
        bail!("Gasless deposit failed ({status}): {error}");
    }

    match fmt {
        OutputFormat::Json => crate::output::print_json(&response)?,
        OutputFormat::Table => {
            let fee = contracts::format_usdc(fee_amount);
            let credited = contracts::format_usdc(credit_amount);
            let tx = response
                .get("txHash")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            println!("OK: Deposited {credited} USDC gaslessly (fee {fee} USDC)");
            println!("Tx: {tx}");
        }
    }

    Ok(())
}

/// Legacy direct deposit: approve then deposit to the PrizePool.
async fn execute_direct(
    amount_usdc: f64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Depositing {amount_usdc} USDC directly...");
    }

    let amount = contracts::parse_usdc(amount_usdc);
    let url: reqwest::Url = rpc_url.parse()?;
    let provider = alloy::providers::ProviderBuilder::new()
        .wallet(alloy::network::EthereumWallet::from(signer.clone()))
        .connect_http(url);

    let usdc = IERC20::new(contracts::usdc_address(), &provider);
    let bank = IPrizePool::new(contracts::prize_pool_address(), &provider);

    let balance = usdc.balanceOf(*address).call().await?;
    if balance < amount {
        let have = contracts::format_usdc(balance);
        bail!("Insufficient USDC. Have {have}, need {amount_usdc}");
    }

    if matches!(fmt, OutputFormat::Table) {
        println!("Approving USDC...");
    }
    let tx1 = usdc
        .approve(contracts::prize_pool_address(), amount)
        .send()
        .await?;
    let _receipt1 = tx1.watch().await?;

    if matches!(fmt, OutputFormat::Table) {
        println!("Depositing to Prize Pool...");
    }
    let tx2 = bank.deposit(amount).send().await?;
    let _receipt2 = tx2.watch().await?;

    let data = serde_json::json!({ "ok": true, "deposited": amount_usdc });

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&data)?;
        }
        OutputFormat::Table => {
            println!("OK: Deposited {amount_usdc} USDC");
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
