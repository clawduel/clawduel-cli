//! Poll for active match and handle ready/waiting flows.

use alloy::primitives::Address;
use anyhow::Result;
use url::Url;

use crate::http::HttpClient;
use crate::security;

/// Poll for the agent's active match.
///
/// Handles:
/// - `waiting_ready`: sends ready signal, waits if `startsAt` is set, then re-polls
/// - `waiting_start`: waits until `startsAt`, then re-polls
/// - Other states: prints the data as-is
pub async fn execute(client: &HttpClient, address: &Address) -> Result<()> {
    let safe_address = security::sanitize_path_segment(&format!("{address:?}"));
    let data = client.get(&format!("/matches/active/{safe_address}")).await?;

    // Handle ready acknowledgement flow
    if let Some(m) = data.get("match") {
        let status = m.get("status").and_then(|s| s.as_str()).unwrap_or("");
        let ready_url = m.get("readyUrl").and_then(|u| u.as_str());
        let problem_is_null = m.get("problem").map_or(true, |p| p.is_null());

        if status == "waiting_ready" && ready_url.is_some() && problem_is_null {
            println!("Match found, sending ready signal...");

            let url_str = ready_url.unwrap();
            let parsed = Url::parse(url_str).unwrap_or_else(|_| {
                // If it's a relative path, just use it directly
                Url::parse(&format!("http://placeholder{url_str}")).unwrap()
            });
            let path = parsed.path();

            let (resp_status, body) = client
                .post(path, &serde_json::json!({}))
                .await?;

            if (200..300).contains(&resp_status) {
                println!("OK: Ready signal sent");
                if let Some(starts_at) = body.get("startsAt").and_then(|s| s.as_str()) {
                    wait_until(starts_at).await;
                } else {
                    println!("Waiting for opponent...");
                }
            } else {
                let error = body
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("Ready acknowledgement failed ({resp_status}): {error}");
            }

            // Re-poll
            let updated = client
                .get(&format!("/matches/active/{safe_address}"))
                .await?;
            println!("{}", serde_json::to_string(&updated)?);
            return Ok(());
        }

        // Both agents ready, waiting for synchronized start
        if status == "waiting_start" {
            if let Some(starts_at) = m.get("startsAt").and_then(|s| s.as_str()) {
                wait_until(starts_at).await;
            }

            let updated = client
                .get(&format!("/matches/active/{safe_address}"))
                .await?;
            println!("{}", serde_json::to_string(&updated)?);
            return Ok(());
        }
    }

    println!("{}", serde_json::to_string(&data)?);
    Ok(())
}

/// Wait until the given ISO 8601 timestamp.
async fn wait_until(starts_at: &str) {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    // Parse ISO 8601 timestamp
    let target_ms = match parse_iso8601_to_epoch_ms(starts_at) {
        Some(ms) => ms,
        None => return,
    };

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_millis() as u64;

    if target_ms > now_ms {
        let wait_ms = target_ms - now_ms;
        let wait_secs = (wait_ms + 999) / 1000;
        println!("Match starts in {wait_secs}s, waiting...");
        tokio::time::sleep(Duration::from_millis(wait_ms)).await;
    }
}

/// Simple ISO 8601 parser to epoch milliseconds.
fn parse_iso8601_to_epoch_ms(s: &str) -> Option<u64> {
    // Handle format: "2026-03-19T22:30:00.000Z" or similar
    // We use a simple approach since we don't want to add chrono dependency
    let s = s.trim().trim_end_matches('Z');
    let parts: Vec<&str> = s.split('T').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<u64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    if date_parts.len() != 3 {
        return None;
    }

    let time_str = parts[1];
    let time_parts: Vec<&str> = time_str.split('.').collect();
    let hms: Vec<u64> = time_parts[0]
        .split(':')
        .filter_map(|p| p.parse().ok())
        .collect();
    if hms.len() != 3 {
        return None;
    }

    let millis = if time_parts.len() > 1 {
        let frac = time_parts[1];
        let padded = format!("{:0<3}", &frac[..frac.len().min(3)]);
        padded.parse::<u64>().unwrap_or(0)
    } else {
        0
    };

    // Simple days-since-epoch calculation (approximate but sufficient for wait times)
    let (year, month, day) = (date_parts[0], date_parts[1], date_parts[2]);
    let (hour, min, sec) = (hms[0], hms[1], hms[2]);

    // Use a simple epoch calculation
    let days = days_from_civil(year as i64, month as u32, day as u32);
    let epoch_secs = days as u64 * 86400 + hour * 3600 + min * 60 + sec;

    Some(epoch_secs * 1000 + millis)
}

/// Convert civil date to days since Unix epoch (1970-01-01).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let m_adj = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * m_adj + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}
