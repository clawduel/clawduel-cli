//! Show agent info and balance status.

use alloy::primitives::Address;
use anyhow::Result;

use crate::contracts::{self, IERC20, IPrizePool};
use crate::http::HttpClient;
use crate::output::OutputFormat;
use crate::security;

/// Show agent status: backend info + on-chain balance.
pub async fn execute(
    client: &HttpClient,
    address: &Address,
    rpc_url: &str,
    fmt: OutputFormat,
) -> Result<()> {
    let safe_address = security::sanitize_path_segment(&format!("{address:?}"));
    let data = client.get(&format!("/api/agents/{safe_address}")).await?;

    // Get on-chain balance
    let provider = contracts::create_provider(rpc_url).await?;
    let bank = IPrizePool::new(contracts::prize_pool_address(), &provider);
    let usdc = IERC20::new(contracts::usdc_address(), &provider);

    let prize_pool = bank.balanceOf(*address).call().await?;
    let wallet = usdc.balanceOf(*address).call().await?;

    let prize_pool_fmt = contracts::format_usdc(prize_pool);
    let wallet_fmt = contracts::format_usdc(wallet);

    let mut output = data.clone();
    output["prizePool"] = serde_json::json!(prize_pool_fmt);
    output["walletUsdc"] = serde_json::json!(wallet_fmt);

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            let nickname = data.get("nickname").and_then(|n| n.as_str()).unwrap_or("-");
            let elo = data
                .get("elo")
                .map(|e| e.to_string())
                .unwrap_or_else(|| "-".to_string());

            crate::output::print_detail(vec![
                ("Address", format!("{address:?}")),
                ("Nickname", nickname.to_string()),
                ("ELO", elo),
                ("PrizePool", format!("{prize_pool_fmt} USDC")),
                ("Wallet USDC", format!("{wallet_fmt} USDC")),
            ]);
        }
    }

    Ok(())
}
