//! List matches with optional filters.

use anyhow::Result;
use tabled::Tabled;

use crate::http::HttpClient;
use crate::output::OutputFormat;

/// Query parameters for match listing.
pub struct MatchFilters {
    pub status: Option<String>,
    pub page: Option<u32>,
    pub category: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Tabled, serde::Serialize)]
struct MatchRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Type")]
    kind: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Entry Fee")]
    entry_fee: String,
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Winner")]
    winner: String,
}

/// List matches with optional filters.
pub async fn execute(client: &HttpClient, filters: MatchFilters, fmt: OutputFormat) -> Result<()> {
    let mut params = Vec::new();
    if let Some(ref s) = filters.status {
        params.push(format!("status={s}"));
    }
    if let Some(p) = filters.page {
        params.push(format!("page={p}"));
    }
    if let Some(ref c) = filters.category {
        params.push(format!("category={c}"));
    }
    if let Some(ref f) = filters.from {
        params.push(format!("from={f}"));
    }
    if let Some(ref t) = filters.to {
        params.push(format!("to={t}"));
    }

    let qs = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };

    let data = client.get(&format!("/api/matches{qs}")).await?;

    let results = data
        .get("matches")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();

    match fmt {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "page": data.get("page"),
                "pageSize": data.get("pageSize"),
                "total": data.get("total"),
                "count": results.len(),
                "matches": results,
            });
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            let rows: Vec<MatchRow> = results
                .iter()
                .map(|m| {
                    let raw_fee = json_str(m, "entryFee");
                    let fee_display = raw_fee
                        .parse::<f64>()
                        .map(|f| format!("{:.2} USDC", f / 1_000_000.0))
                        .unwrap_or(raw_fee);
                    MatchRow {
                        id: json_str(m, "id"),
                        kind: json_str(m, "competitionType"),
                        status: json_str(m, "status"),
                        entry_fee: fee_display,
                        category: json_str(m, "oracleId"),
                        winner: json_str(m, "winner"),
                    }
                })
                .collect();

            let count = rows.len();
            let page = data.get("page").and_then(|p| p.as_u64()).unwrap_or(0);
            let total = data
                .get("count")
                .map(|t| t.to_string())
                .unwrap_or_else(|| "?".to_string());

            if count == 0 {
                println!("No matches found");
            } else {
                println!(
                    "Found {count} match{} (page {page}, total {total})",
                    if count == 1 { "" } else { "es" }
                );
                crate::output::print_table(&rows);
            }
        }
    }

    Ok(())
}

fn json_str(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => "-".to_string(),
            other => other.to_string(),
        })
        .unwrap_or_else(|| "-".to_string())
}
