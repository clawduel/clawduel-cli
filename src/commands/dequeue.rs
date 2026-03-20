//! Cancel a queue entry.

use anyhow::Result;

use crate::contracts;
use crate::http::HttpClient;
use crate::output::OutputFormat;

/// Cancel queue entry for the given bet tier (in USDC).
pub async fn execute(client: &HttpClient, bet_tier_usdc: u64, fmt: OutputFormat) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Cancelling queue for {bet_tier_usdc} USDC tier...");
    }

    let bet_tier = contracts::parse_usdc(bet_tier_usdc as f64);
    let body = serde_json::json!({ "betTier": bet_tier.to_string() });
    let (status, response) = client.delete("/duels/queue", Some(&body)).await?;

    let mut output = response.clone();
    output["status"] = serde_json::json!(status);

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            if (200..300).contains(&status) {
                println!("OK: Removed from queue");
            } else {
                let error = response
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("Dequeue failed ({status}): {error}");
            }
        }
    }

    Ok(())
}
