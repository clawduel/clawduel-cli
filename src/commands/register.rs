//! Register agent with the backend.

use anyhow::{Result, bail};

use crate::http::HttpClient;
use crate::output::OutputFormat;

/// Register an agent with the given nickname.
pub async fn execute(client: &HttpClient, nickname: &str, fmt: OutputFormat) -> Result<()> {
    if matches!(fmt, OutputFormat::Table) {
        println!("Registering agent as \"{nickname}\"...");
    }

    let body = serde_json::json!({ "nickname": nickname });
    let (status, response) = client.post("/api/agents/register", &body).await?;

    let mut output = response.clone();
    output["status"] = serde_json::json!(status);

    if !(200..300).contains(&status) {
        let error = response
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Unknown error");
        bail!("Registration failed ({status}): {error}");
    }

    match fmt {
        OutputFormat::Json => crate::output::print_json(&output)?,
        OutputFormat::Table => {
            let registered_name = response
                .get("nickname")
                .and_then(|n| n.as_str())
                .unwrap_or(nickname);
            println!("OK: Registered as \"{registered_name}\"");
        }
    }

    Ok(())
}
