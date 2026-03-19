//! List matches with optional filters.

use anyhow::Result;

use crate::http::HttpClient;

/// Query parameters for match listing.
pub struct MatchFilters {
    pub status: Option<String>,
    pub page: Option<u32>,
    pub category: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// List matches with optional filters.
pub async fn execute(client: &HttpClient, filters: MatchFilters) -> Result<()> {
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
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();

    let formatted: Vec<serde_json::Value> = results
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.get("id"),
                "type": m.get("type"),
                "status": m.get("status"),
                "agents": m.get("agents"),
                "betSize": m.get("betSize"),
                "problemCategory": m.get("problemCategory"),
                "winner": m.get("winner"),
                "timestamp": m.get("timestamp"),
            })
        })
        .collect();

    let count = formatted.len();
    if count == 0 {
        if filters.status.is_some() {
            println!(
                "No {} matches found",
                filters.status.as_deref().unwrap_or("")
            );
        } else {
            println!("No matches found");
        }
    } else {
        let page = data.get("page").and_then(|p| p.as_u64()).unwrap_or(0);
        let total = data
            .get("total")
            .map(|t| t.to_string())
            .unwrap_or_else(|| "?".to_string());
        println!(
            "Found {count} match{} (page {page}, total {total})",
            if count == 1 { "" } else { "es" }
        );
    }

    let output = serde_json::json!({
        "page": data.get("page"),
        "pageSize": data.get("pageSize"),
        "total": data.get("total"),
        "count": count,
        "matches": formatted,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
