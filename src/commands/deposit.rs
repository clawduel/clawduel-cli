//! Deposit USDC to the bank contract.

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Result, bail};

use crate::contracts::{self, IERC20, IPrizePool};
use crate::output::OutputFormat;

/// Deposit USDC: approve then deposit to the bank.
pub async fn execute(
    amount_usdc: f64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Depositing {amount_usdc} USDC...");
    }

    let amount = contracts::parse_usdc(amount_usdc);

    // Create a provider with the wallet signer for sending transactions
    let url: reqwest::Url = rpc_url.parse()?;
    let provider = alloy::providers::ProviderBuilder::new()
        .wallet(alloy::network::EthereumWallet::from(signer.clone()))
        .connect_http(url);

    let usdc = IERC20::new(contracts::usdc_address(), &provider);
    let bank = IPrizePool::new(contracts::prize_pool_address(), &provider);

    // Check USDC balance
    let balance = usdc.balanceOf(*address).call().await?;
    if balance < amount {
        let have = contracts::format_usdc(balance);
        bail!("Insufficient USDC. Have {have}, need {amount_usdc}");
    }

    // Approve
    if matches!(fmt, OutputFormat::Table) {
        println!("Approving USDC...");
    }
    let tx1 = usdc.approve(contracts::prize_pool_address(), amount).send().await?;
    let _receipt1 = tx1.watch().await?;

    // Deposit
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
