//! Show single match details with resolution summary.

use anyhow::Result;

use crate::http::HttpClient;
use crate::security;

/// Show details for a single match.
pub async fn execute(client: &HttpClient, match_id: &str) -> Result<()> {
    let safe_id = security::sanitize_path_segment(match_id);
    let data = client.get(&format!("/api/matches/{safe_id}")).await?;

    if let Some(error) = data.get("error").and_then(|e| e.as_str()) {
        eprintln!("Error: {error}");
        println!("{}", serde_json::to_string(&data)?);
        return Ok(());
    }

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
        let winner = data.get("winner").cloned().unwrap_or(serde_json::Value::Null);

        let resolution = compute_resolution(p1, p2, actual, &winner);
        result["resolution"] = resolution;

        if let Some(payout) = data.get("payout") {
            result["payout"] = serde_json::json!(format_usdc_from_value(Some(payout)));
        }
    }

    // Print header
    let match_display = result
        .get("matchId")
        .and_then(|v| v.as_str())
        .unwrap_or(&safe_id);
    let status_display = result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    println!("\n  Match {match_display}");
    println!("  {}", "-".repeat(44));
    println!("  Status         {status_display}");
    if let Some(title) = result.get("problemTitle").and_then(|v| v.as_str()) {
        println!("  Problem        {title}");
    }
    if let Some(bet) = result.get("betSize").and_then(|v| v.as_str()) {
        println!("  Bet Size       {bet}");
    }
    if let Some(w) = result.get("winner").and_then(|v| v.as_str()) {
        println!("  Winner         {w}");
    }
    println!();

    println!("{}", serde_json::to_string_pretty(&result)?);

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
            // Non-numeric or missing actual
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
    // raw is in atomic units (6 decimals)
    let usdc = raw / 1_000_000.0;
    format!("{usdc:.2} USDC")
}
