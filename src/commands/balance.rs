//! Check agent on-chain balance.

use alloy::primitives::Address;
use anyhow::Result;

use crate::contracts::{self, IBank};
use crate::output::OutputFormat;

/// Display available, locked, and total USDC balance.
pub async fn execute(address: &Address, rpc_url: &str, fmt: OutputFormat) -> Result<()> {
    let provider = contracts::create_provider(rpc_url).await?;
    let addresses = contracts::resolve_addresses()?;
    let bank = IBank::new(addresses.bank, &provider);

    let available = bank.balanceOf(*address).call().await?;
    let locked = bank.lockedBalanceOf(*address).call().await?;
    let total = available + locked;

    let available_fmt = contracts::format_usdc(available);
    let locked_fmt = contracts::format_usdc(locked);
    let total_fmt = contracts::format_usdc(total);

    let data = serde_json::json!({
        "address": format!("{address:?}"),
        "available": available_fmt,
        "locked": locked_fmt,
        "total": total_fmt,
    });

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&data)?;
        }
        OutputFormat::Table => {
            crate::output::print_detail(vec![
                ("Address", format!("{address:?}")),
                ("Available", format!("{available_fmt} USDC")),
                ("Locked", format!("{locked_fmt} USDC")),
                ("Total", format!("{total_fmt} USDC")),
            ]);
        }
    }

    Ok(())
}
