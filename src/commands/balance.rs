//! Check agent on-chain balance.

use alloy::primitives::Address;
use anyhow::Result;

use crate::contracts::{self, IERC20, IPrizePool};
use crate::output::OutputFormat;

/// Display PrizePool and wallet USDC balances.
pub async fn execute(address: &Address, rpc_url: &str, fmt: OutputFormat) -> Result<()> {
    let provider = contracts::create_provider(rpc_url).await?;
    let bank = IPrizePool::new(contracts::prize_pool_address(), &provider);
    let usdc = IERC20::new(contracts::usdc_address(), &provider);

    let prize_pool = bank.balanceOf(*address).call().await?;
    let wallet = usdc.balanceOf(*address).call().await?;

    let prize_pool_fmt = contracts::format_usdc(prize_pool);
    let wallet_fmt = contracts::format_usdc(wallet);

    let data = serde_json::json!({
        "address": format!("{address:?}"),
        "prizePool": prize_pool_fmt,
        "wallet": wallet_fmt,
    });

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&data)?;
        }
        OutputFormat::Table => {
            crate::output::print_detail(vec![
                ("Address", format!("{address:?}")),
                ("PrizePool", format!("{prize_pool_fmt} USDC")),
                ("Wallet USDC", format!("{wallet_fmt} USDC")),
            ]);
        }
    }

    Ok(())
}
