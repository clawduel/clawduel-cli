//! Deposit USDC to the bank contract.

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Result, bail};

use crate::contracts::{self, IERC20, IBank};

/// Deposit USDC: approve then deposit to the bank.
pub async fn execute(
    amount_usdc: f64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
) -> Result<()> {
    println!("Depositing {amount_usdc} USDC...");

    let amount = contracts::parse_usdc(amount_usdc);
    let addresses = contracts::resolve_addresses()?;

    // Create a provider with the wallet signer for sending transactions
    let url: reqwest::Url = rpc_url.parse()?;
    let provider = alloy::providers::ProviderBuilder::new()
        .wallet(alloy::network::EthereumWallet::from(signer.clone()))
        .connect_http(url);

    let usdc = IERC20::new(addresses.usdc, &provider);
    let bank = IBank::new(addresses.bank, &provider);

    // Check USDC balance
    let balance = usdc.balanceOf(*address).call().await?;
    if balance < amount {
        let have = contracts::format_usdc(balance);
        bail!("Insufficient USDC. Have {have}, need {amount_usdc}");
    }

    // Approve
    println!("Approving USDC...");
    let tx1 = usdc.approve(addresses.bank, amount).send().await?;
    let _receipt1 = tx1.watch().await?;

    // Deposit
    println!("Depositing to Bank...");
    let tx2 = bank.deposit(amount).send().await?;
    let _receipt2 = tx2.watch().await?;

    println!("OK: Deposited {amount_usdc} USDC");
    println!(
        "{}",
        serde_json::json!({ "ok": true, "deposited": amount_usdc })
    );

    Ok(())
}
