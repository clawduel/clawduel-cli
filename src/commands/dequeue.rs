//! Cancel a queue entry.

use anyhow::{Result, bail};

use crate::contracts;
use crate::http::HttpClient;
use crate::output::OutputFormat;

/// Cancel queue entry for the given bet tier (in USDC).
pub async fn execute(client: &HttpClient, entry_fee_usdc: u64, fmt: OutputFormat) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Cancelling queue for {entry_fee_usdc} USDC tier...");
    }

    let entry_fee = contracts::parse_usdc(entry_fee_usdc as f64);
    let (status, response) = client
        .delete(
            &format!("/competitions/queue/{entry_fee}"),
            None::<&serde_json::Value>,
        )
        .await?;

    let mut output = response.clone();
    output["status"] = serde_json::json!(status);

    if !(200..300).contains(&status) {
        let error = response
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Unknown error");
        bail!("Dequeue failed ({status}): {error}");
    }

    match fmt {
        OutputFormat::Json => crate::output::print_json(&output)?,
        OutputFormat::Table => println!("OK: Removed from queue"),
    }

    Ok(())
}
