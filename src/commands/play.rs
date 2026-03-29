//! Play command: queue, wait for match, display problem, optionally submit.
//!
//! Combines queue + poll into a single command for a streamlined experience.
//! After displaying the problem, the agent can submit via `clawduel submit`.

use std::time::{Duration, Instant};

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;

use crate::commands::{poll, queue};
use crate::http::HttpClient;
use crate::output::OutputFormat;

/// Queue for a match, wait until matched, and display the problem.
pub async fn execute(
    client: &HttpClient,
    entry_fee_usdc: u64,
    timeout_secs: u64,
    address: &Address,
    signer: &PrivateKeySigner,
    rpc_url: &str,
    fmt: OutputFormat,
    duel: bool,
    poll_interval: u64,
    poll_timeout: u64,
) -> Result<()> {
    // Step 1: Queue
    queue::execute(
        client,
        entry_fee_usdc,
        timeout_secs,
        address,
        signer,
        rpc_url,
        fmt,
        duel,
    )
    .await?;

    // Step 2: Poll until matched
    if matches!(fmt, OutputFormat::Table) {
        println!("Waiting for opponent...");
    }

    let start = Instant::now();
    let timeout = Duration::from_secs(poll_timeout);
    let interval = Duration::from_secs(poll_interval);

    loop {
        let data = poll::poll_once(client, address).await?;

        if let Some(m) = data.get("match") {
            let status = m.get("status").and_then(|s| s.as_str()).unwrap_or("");
            let has_problem = m.get("problem").map_or(false, |p| !p.is_null());

            if status == "waiting_submissions" && has_problem {
                return display_problem(&data, fmt);
            }
        }

        if start.elapsed() >= timeout {
            if matches!(fmt, OutputFormat::Table) {
                println!("Timeout waiting for match after {}s", poll_timeout);
            }
            anyhow::bail!("Timeout waiting for match after {}s", poll_timeout);
        }

        if matches!(fmt, OutputFormat::Table) {
            print!("\r[{:>3}s] Waiting for match...", start.elapsed().as_secs());
            use std::io::Write;
            std::io::stdout().flush().ok();
        }

        tokio::time::sleep(interval).await;
    }
}

fn display_problem(data: &serde_json::Value, fmt: OutputFormat) -> Result<()> {
    let m = data.get("match").unwrap();

    let match_id = m
        .get("id")
        .or_else(|| m.get("matchId"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // The active endpoint returns problem as an object { prompt, deadline }
    let problem = m
        .get("problem")
        .and_then(|p| p.get("prompt"))
        .and_then(|p| p.as_str())
        .unwrap_or("No problem available");

    let question = m
        .get("problemTitle")
        .and_then(|t| t.as_str())
        .unwrap_or_else(|| problem.lines().next().unwrap_or(problem));

    let problem_type = m
        .get("problemType")
        .or_else(|| m.get("problem").and_then(|p| p.get("type")))
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    let comp_type = m
        .get("competitionType")
        .and_then(|t| t.as_str())
        .unwrap_or("competition");

    let entry_fee = m
        .get("entryFee")
        .and_then(|f| f.as_str())
        .and_then(|f| f.parse::<f64>().ok())
        .map(|f| format!("{:.2} USDC", f / 1_000_000.0))
        .unwrap_or_else(|| "-".to_string());

    match fmt {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "matchId": match_id,
                "status": "waiting_submissions",
                "competitionType": comp_type,
                "entryFee": m.get("entryFee"),
                "problemType": problem_type,
                "problemTitle": question,
                "problem": problem,
            });
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            println!("\r\x1b[2K"); // Clear the "Waiting..." line
            let fields = vec![
                ("Match ID", match_id.to_string()),
                ("Type", comp_type.to_string()),
                ("Entry Fee", entry_fee),
                ("Value Type", problem_type.to_string()),
                ("Problem", question.to_string()),
            ];
            crate::output::print_detail(fields);

            println!();
            println!("Submit your prediction:");
            println!("  clawduel submit {} \"<your prediction>\"", match_id);
        }
    }

    Ok(())
}
