//! Show agent info and balance status.

use alloy::primitives::Address;
use anyhow::Result;

use crate::contracts::{self, IBank};
use crate::http::HttpClient;
use crate::security;

/// Show agent status: backend info + on-chain balance.
pub async fn execute(client: &HttpClient, address: &Address, rpc_url: &str) -> Result<()> {
    let safe_address = security::sanitize_path_segment(&format!("{address:?}"));
    let data = client.get(&format!("/api/agents/{safe_address}")).await?;

    // Get on-chain balance
    let provider = contracts::create_provider(rpc_url).await?;
    let addresses = contracts::resolve_addresses()?;
    let bank = IBank::new(addresses.bank, &provider);

    let available = bank.balanceOf(*address).call().await?;
    let locked = bank.lockedBalanceOf(*address).call().await?;

    let available_fmt = contracts::format_usdc(available);
    let locked_fmt = contracts::format_usdc(locked);

    println!("\n  Agent Status");
    println!("  {}", "-".repeat(44));
    println!("  Address        {address:?}");
    if let Some(nickname) = data.get("nickname").and_then(|n| n.as_str()) {
        println!("  Nickname       {nickname}");
    }
    if let Some(elo) = data.get("elo") {
        println!("  ELO            {elo}");
    }
    println!("  Available      {available_fmt} USDC");
    println!("  Locked         {locked_fmt} USDC");
    println!();

    let mut output = data.clone();
    output["available"] = serde_json::json!(available_fmt);
    output["locked"] = serde_json::json!(locked_fmt);
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}
