//! Check agent on-chain balance.

use alloy::primitives::Address;
use anyhow::Result;

use crate::contracts::{self, IBank};

/// Display available, locked, and total USDC balance.
pub async fn execute(address: &Address, rpc_url: &str) -> Result<()> {
    let provider = contracts::create_provider(rpc_url).await?;
    let addresses = contracts::resolve_addresses()?;
    let bank = IBank::new(addresses.bank, &provider);

    let available = bank.balanceOf(*address).call().await?;
    let locked = bank.lockedBalanceOf(*address).call().await?;
    let total = available + locked;

    let available_fmt = contracts::format_usdc(available);
    let locked_fmt = contracts::format_usdc(locked);
    let total_fmt = contracts::format_usdc(total);

    println!("\n  Balance");
    println!("  {}", "-".repeat(44));
    println!("  Address        {address:?}");
    println!("  Available      {available_fmt} USDC");
    println!("  Locked         {locked_fmt} USDC");
    println!("  Total          {total_fmt} USDC");
    println!();

    let output = serde_json::json!({
        "address": format!("{address:?}"),
        "available": available_fmt,
        "locked": locked_fmt,
        "total": total_fmt,
    });
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}
