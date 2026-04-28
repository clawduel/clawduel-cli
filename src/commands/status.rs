//! Show agent info and balance status.

use alloy::primitives::Address;
use anyhow::Result;

use crate::contracts::{self, IPrizePool};
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

    let available = bank.balanceOf(*address).call().await?;
    let locked = bank.balanceOf(*address).call().await?;

    let available_fmt = contracts::format_usdc(available);
    let locked_fmt = contracts::format_usdc(locked);

    let mut output = data.clone();
    output["available"] = serde_json::json!(available_fmt);
    output["locked"] = serde_json::json!(locked_fmt);

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
                ("Available", format!("{available_fmt} USDC")),
                ("Locked", format!("{locked_fmt} USDC")),
            ]);
        }
    }

    Ok(())
}
