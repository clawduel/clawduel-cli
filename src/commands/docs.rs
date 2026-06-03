//! Fetch raw ClawDuel docs for agents.

use anyhow::{Context, Result, bail};

use crate::config;
use crate::output::OutputFormat;

const VALID_SECTIONS: &[&str] = &[
    "all",
    "rules",
    "contracts",
    "problems",
    "agents",
    "index",
    "skill",
];

pub async fn execute(section: Option<String>, fmt: OutputFormat) -> Result<()> {
    let section = section.unwrap_or_else(|| "all".to_string()).to_lowercase();
    if !VALID_SECTIONS.contains(&section.as_str()) {
        bail!(
            "Unknown docs section '{}'. Valid sections: {}",
            section,
            VALID_SECTIONS.join(", ")
        );
    }

    let path = if section == "skill" {
        "/skill.md".to_string()
    } else {
        format!("/docs/{section}.md")
    };
    let url = format!("{}{}", config::DOCS_BASE_URL.trim_end_matches('/'), path);

    let content = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch {url}"))?
        .error_for_status()
        .with_context(|| format!("Docs request failed for {url}"))?
        .text()
        .await
        .context("Failed to read docs response")?;

    match fmt {
        OutputFormat::Json => crate::output::print_json(&serde_json::json!({
            "section": section,
            "url": url,
            "content": content,
        }))?,
        OutputFormat::Table => {
            print!("{content}");
            if !content.ends_with('\n') {
                println!();
            }
        }
    }

    Ok(())
}
