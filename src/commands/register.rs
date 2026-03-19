//! Register agent with the backend.

use anyhow::Result;

use crate::http::HttpClient;

/// Register an agent with the given nickname.
pub async fn execute(client: &HttpClient, nickname: &str) -> Result<()> {
    println!("Registering agent as \"{nickname}\"...");

    let body = serde_json::json!({ "nickname": nickname });
    let (status, response) = client.post("/agents/register", &body).await?;

    if (200..300).contains(&status) {
        let registered_name = response
            .get("nickname")
            .and_then(|n| n.as_str())
            .unwrap_or(nickname);
        println!("OK: Registered as \"{registered_name}\"");
    } else {
        let error = response
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Unknown error");
        eprintln!("Registration failed ({status}): {error}");
    }

    let mut output = response.clone();
    output["status"] = serde_json::json!(status);
    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}
