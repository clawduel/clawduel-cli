//! Show single match details with resolution summary.

use std::time::{Duration, Instant};

use anyhow::Result;

use crate::http::HttpClient;
use crate::output::OutputFormat;
use crate::security;

/// Show details for a single match.
///
/// When `wait_for_resolution` is true, polls repeatedly until the match
/// status is `resolved`, or until `timeout_secs` elapses.
pub async fn execute(
    client: &HttpClient,
    match_id: &str,
    fmt: OutputFormat,
    wait_for_resolution: bool,
    interval_secs: u64,
    timeout_secs: u64,
) -> Result<()> {
    let safe_id = security::sanitize_path_segment(match_id);

    if !wait_for_resolution {
        let data = fetch_match(client, &safe_id).await?;
        return display_match(&data, &safe_id, fmt);
    }

    // Polling loop for resolution
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    let interval = Duration::from_secs(interval_secs);

    loop {
        let data = fetch_match(client, &safe_id).await?;
        let elapsed = start.elapsed();

        let status = data.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");

        if status == "resolved" {
            if matches!(fmt, OutputFormat::Table) {
                println!(
                    "[{:>3}s] Match resolved!",
                    elapsed.as_secs()
                );
            }
            return display_match(&data, &safe_id, fmt);
        }

        // Print progress in table mode
        if matches!(fmt, OutputFormat::Table) {
            println!(
                "[{:>3}s] Waiting for resolution... status: {status}",
                elapsed.as_secs()
            );
        }

        // Check timeout
        if elapsed >= timeout {
            if matches!(fmt, OutputFormat::Table) {
                println!("Timeout after {}s", timeout_secs);
            }
            return display_match(&data, &safe_id, fmt);
        }

        tokio::time::sleep(interval).await;
    }
}

/// Fetch match data from the API and return parsed JSON.
pub async fn fetch_match(client: &HttpClient, safe_id: &str) -> Result<serde_json::Value> {
    let data = client.get(&format!("/api/matches/{safe_id}")).await?;

    if let Some(error) = data.get("error").and_then(|e| e.as_str()) {
        anyhow::bail!("API error: {error}");
    }

    Ok(data)
}

/// Display match details in the requested output format.
fn display_match(data: &serde_json::Value, safe_id: &str, fmt: OutputFormat) -> Result<()> {
    let mut result = serde_json::json!({
        "matchId": data.get("id").or_else(|| data.get("matchId")),
        "duelId": data.get("duelId"),
        "status": data.get("status"),
        "agents": data.get("agents"),
        "betSize": format_usdc_from_value(data.get("betSize")),
        "problemTitle": data.get("problemTitle"),
        "problemCategory": data.get("problemCategory"),
        "problemPrompt": data.get("problemPrompt"),
        "predictions": data.get("predictions"),
        "actualValue": data.get("actualValue"),
        "winner": data.get("winner"),
    });

    // Add resolution summary for resolved matches
    if data.get("status").and_then(|s| s.as_str()) == Some("resolved") {
        let predictions = data.get("predictions");
        let p1 = predictions
            .and_then(|p| p.get("agent1"))
            .and_then(|v| v.as_str());
        let p2 = predictions
            .and_then(|p| p.get("agent2"))
            .and_then(|v| v.as_str());
        let actual = data.get("actualValue").and_then(|v| v.as_str());
        let winner = data
            .get("winner")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let resolution = compute_resolution(p1, p2, actual, &winner);
        result["resolution"] = resolution;

        if let Some(payout) = data.get("payout") {
            result["payout"] = serde_json::json!(format_usdc_from_value(Some(payout)));
        }
    }

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&result)?;
        }
        OutputFormat::Table => {
            let match_display = result
                .get("matchId")
                .and_then(|v| v.as_str())
                .unwrap_or(safe_id);
            let status_display = result
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let mut rows = vec![
                ("Match ID", match_display.to_string()),
                ("Status", status_display.to_string()),
            ];

            if let Some(title) = result.get("problemTitle").and_then(|v| v.as_str()) {
                rows.push(("Problem", title.to_string()));
            }
            if let Some(bet) = result.get("betSize").and_then(|v| v.as_str()) {
                rows.push(("Bet Size", bet.to_string()));
            }
            if let Some(w) = result.get("winner").and_then(|v| v.as_str()) {
                rows.push(("Winner", w.to_string()));
            }
            if let Some(verdict) = result
                .get("resolution")
                .and_then(|r| r.get("verdict"))
                .and_then(|v| v.as_str())
            {
                rows.push(("Verdict", verdict.to_string()));
            }

            crate::output::print_detail(rows);
        }
    }

    Ok(())
}

fn compute_resolution(
    p1: Option<&str>,
    p2: Option<&str>,
    actual: Option<&str>,
    winner: &serde_json::Value,
) -> serde_json::Value {
    match (p1, p2) {
        (None, None) => {
            serde_json::json!({ "verdict": "DRAW - both agents failed to submit" })
        }
        (None, _) | (_, None) => {
            serde_json::json!({ "verdict": "WIN_BY_FORFEIT - opponent did not submit" })
        }
        (Some(pred1), Some(pred2)) => {
            if let (Some(actual_str), Ok(a)) = (actual, actual.unwrap_or("").parse::<f64>()) {
                if let (Ok(v1), Ok(v2)) = (pred1.parse::<f64>(), pred2.parse::<f64>()) {
                    let err1 = (v1 - a).abs();
                    let err2 = (v2 - a).abs();
                    let verdict = if winner.is_null() || winner.as_str() == Some("") {
                        "DRAW"
                    } else if err1 < err2 {
                        "AGENT_1_CLOSER"
                    } else {
                        "AGENT_2_CLOSER"
                    };
                    return serde_json::json!({
                        "actualValue": actual_str,
                        "agent1Error": err1,
                        "agent2Error": err2,
                        "verdict": verdict,
                    });
                }
            }
            let verdict = if winner.is_null() || winner.as_str() == Some("") {
                "DRAW"
            } else {
                "WINNER_DECLARED"
            };
            serde_json::json!({
                "actualValue": actual,
                "verdict": verdict,
            })
        }
    }
}

/// Format a USDC value from a JSON number/string to "X.XX USDC".
fn format_usdc_from_value(value: Option<&serde_json::Value>) -> String {
    let raw = match value {
        Some(v) if v.is_number() => v.as_f64().unwrap_or(0.0),
        Some(v) if v.is_string() => v.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
        _ => return "0.00 USDC".to_string(),
    };
    let usdc = raw / 1_000_000.0;
    format!("{usdc:.2} USDC")
}
