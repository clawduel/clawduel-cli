//! Poll for active match and handle ready/waiting flows.

use std::time::{Duration, Instant};

use alloy::primitives::Address;
use anyhow::Result;
use url::Url;

use crate::http::HttpClient;
use crate::output::OutputFormat;
use crate::security;

/// Poll for the agent's active match.
///
/// When `wait` is true, polls repeatedly until the match reaches
/// `waiting_submissions` with a problem present, or until `timeout_secs` elapses.
pub async fn execute(
    client: &HttpClient,
    address: &Address,
    fmt: OutputFormat,
    wait: bool,
    interval_secs: u64,
    timeout_secs: u64,
) -> Result<()> {
    if !wait {
        let data = poll_once(client, address).await?;
        return print_poll_result(&data, fmt);
    }

    // Polling loop
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    let interval = Duration::from_secs(interval_secs);

    loop {
        let data = poll_once(client, address).await?;
        let elapsed = start.elapsed();

        // Check if match is ready (waiting_submissions with problem)
        if let Some(m) = data.get("match") {
            let status = m.get("status").and_then(|s| s.as_str()).unwrap_or("");
            let has_problem = m.get("problem").map_or(false, |p| !p.is_null());

            if status == "waiting_submissions" && has_problem {
                if matches!(fmt, OutputFormat::Table) {
                    println!(
                        "[{:>3}s] Polling... status: waiting_submissions (problem ready!)",
                        elapsed.as_secs()
                    );
                }
                return print_poll_result(&data, fmt);
            }

            // Print progress in table mode
            if matches!(fmt, OutputFormat::Table) {
                println!("[{:>3}s] Polling... status: {status}", elapsed.as_secs());
            }
        } else {
            // No active match
            if matches!(fmt, OutputFormat::Table) {
                println!("[{:>3}s] Polling... no active match", elapsed.as_secs());
            }
        }

        // Check timeout
        if elapsed >= timeout {
            if matches!(fmt, OutputFormat::Table) {
                println!("Timeout after {}s", timeout_secs);
            }
            return print_poll_result(&data, fmt);
        }

        tokio::time::sleep(interval).await;
    }
}

/// Execute a single poll cycle: fetch active match, handle ready acknowledgement,
/// handle waiting_start, and return the final JSON data.
pub async fn poll_once(client: &HttpClient, address: &Address) -> Result<serde_json::Value> {
    let safe_address = security::sanitize_path_segment(&format!("{address:?}"));
    let data = client
        .get(&format!("/matches/active/{safe_address}"))
        .await?;

    // Handle ready acknowledgement flow
    if let Some(m) = data.get("match") {
        let status = m.get("status").and_then(|s| s.as_str()).unwrap_or("");
        let ready_url = m.get("readyUrl").and_then(|u| u.as_str());
        let problem_is_null = m.get("problem").map_or(true, |p| p.is_null());

        if status == "waiting_ready" && ready_url.is_some() {
            let url_str = ready_url.unwrap();
            let parsed = Url::parse(url_str).unwrap_or_else(|_| {
                Url::parse(&format!("http://placeholder{url_str}")).unwrap()
            });
            let path = parsed.path();

            let (resp_status, body) = client.post(path, &serde_json::json!({})).await?;

            if (200..300).contains(&resp_status) {
                if let Some(starts_at) = body.get("startsAt").and_then(|s| s.as_str()) {
                    wait_until(starts_at).await;
                }
            } else {
                let error = body
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("Ready acknowledgement failed ({resp_status}): {error}");
            }

            // Re-poll after ready acknowledgement
            return client
                .get(&format!("/matches/active/{safe_address}"))
                .await;
        }

        // Both agents ready, waiting for synchronized start
        if status == "waiting_start" {
            if let Some(starts_at) = m.get("startsAt").and_then(|s| s.as_str()) {
                wait_until(starts_at).await;
            }

            return client
                .get(&format!("/matches/active/{safe_address}"))
                .await;
        }
    }

    Ok(data)
}

fn print_poll_result(data: &serde_json::Value, fmt: OutputFormat) -> Result<()> {
    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(data)?;
        }
        OutputFormat::Table => {
            let matches = data.get("matches").and_then(|m| m.as_array());
            let total = matches.map_or(0, |m| m.len());

            if let Some(m) = data.get("match") {
                if m.is_null() {
                    println!("No active match");
                    return Ok(());
                }

                let match_id = m
                    .get("id")
                    .or_else(|| m.get("matchId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                let status = m.get("status").and_then(|s| s.as_str()).unwrap_or("-");
                let comp_type = m.get("competitionType").and_then(|t| t.as_str()).unwrap_or("-");
                let problem = m
                    .get("problemTitle")
                    .and_then(|t| t.as_str())
                    .unwrap_or("-");

                let mut fields = vec![
                    ("Match ID", match_id.to_string()),
                    ("Type", comp_type.to_string()),
                    ("Status", status.to_string()),
                    ("Problem", problem.to_string()),
                ];

                if total > 1 {
                    fields.push(("Active Matches", format!("{} (showing oldest)", total)));
                }

                crate::output::print_detail(fields);

                // Print summary of other active matches
                if total > 1 {
                    if let Some(all) = matches {
                        println!();
                        println!("Other active matches:");
                        for (i, other) in all.iter().enumerate().skip(1) {
                            let oid = other.get("id").or_else(|| other.get("matchId"))
                                .and_then(|v| v.as_str()).unwrap_or("-");
                            let otype = other.get("competitionType").and_then(|t| t.as_str()).unwrap_or("-");
                            let oprob = other.get("problemTitle").and_then(|t| t.as_str()).unwrap_or("-");
                            println!("  {}. [{}] {} — {}", i + 1, otype, &oid[..oid.len().min(12)], &oprob[..oprob.len().min(60)]);
                        }
                    }
                }
            } else {
                println!("No active match");
            }
        }
    }
    Ok(())
}

/// Wait until the given ISO 8601 timestamp.
async fn wait_until(starts_at: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};

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

    let (year, month, day) = (date_parts[0], date_parts[1], date_parts[2]);
    let (hour, min, sec) = (hms[0], hms[1], hms[2]);

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
