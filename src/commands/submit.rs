//! Submit prediction for a match with text sanitization.

use anyhow::Result;

use crate::http::HttpClient;
use crate::output::OutputFormat;
use crate::security;

/// Submit a prediction for the given match.
///
/// The prediction text is sanitized before submission:
/// - Control characters removed (except newline, carriage return, tab)
/// - Line endings normalized
/// - Tabs converted to spaces
/// - Multiple spaces collapsed
/// - Consecutive newlines limited to 2
pub async fn execute(
    client: &HttpClient,
    match_id: &str,
    prediction: &str,
    fmt: OutputFormat,
    multi_override: bool,
) -> Result<()> {
    let sanitized = sanitize_prediction(prediction);
    let safe_id = security::sanitize_path_segment(match_id);

    // Auto-detect multi-competition from match type (unless explicitly overridden)
    let is_multi = if multi_override {
        true
    } else {
        // Fetch match to check competitionType
        match client.get(&format!("/api/matches/{safe_id}")).await {
            Ok(data) => data
                .get("match")
                .and_then(|m| m.get("competitionType"))
                .and_then(|t| t.as_str())
                == Some("multi_competition"),
            Err(_) => false, // Fall back to 1v1 if can't determine
        }
    };

    if matches!(fmt, OutputFormat::Table) {
        let mode = if is_multi { "multi-competition " } else { "" };
        println!("Submitting {mode}prediction for match {safe_id}...");
        if sanitized != prediction.trim() {
            println!("  (Prediction text was sanitized for submission)");
        }
    }

    // Wrap prediction in the JSON format the backend validator expects:
    // The validator searches for {"prediction": <value>} inside the prediction text
    let wrapped = if sanitized.parse::<f64>().is_ok()
        || sanitized == "true"
        || sanitized == "false"
    {
        format!(r#"{{"prediction": {sanitized}}}"#)
    } else {
        format!(r#"{{"prediction": "{sanitized}"}}"#)
    };
    let body = serde_json::json!({ "prediction": wrapped });
    let endpoint = if is_multi {
        format!("/matches/{safe_id}/submit/multi")
    } else {
        format!("/matches/{safe_id}/submit")
    };
    let (status, response) = client.post(&endpoint, &body).await?;

    let mut output = response.clone();
    output["status"] = serde_json::json!(status);

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&output)?;
        }
        OutputFormat::Table => {
            if (200..300).contains(&status) {
                println!("OK: Prediction submitted");
            } else {
                let error = response
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("Submission failed ({status}): {error}");
            }
        }
    }

    Ok(())
}

/// Sanitize prediction text before submission.
fn sanitize_prediction(raw: &str) -> String {
    let mut s = raw.trim().to_string();

    // Remove control characters (keep \n=0x0A, \r=0x0D, \t=0x09)
    s = s
        .chars()
        .filter(|c| {
            !matches!(*c,
                '\x00'..='\x08' | '\x0B' | '\x0C' | '\x0E'..='\x1F' | '\x7F'
            )
        })
        .collect();

    // Normalize line endings
    s = s.replace("\r\n", "\n").replace('\r', "\n");

    // Tabs to spaces
    s = s.replace('\t', " ");

    // Collapse multiple spaces (not newlines)
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                result.push(c);
            }
            prev_space = true;
        } else {
            prev_space = false;
            result.push(c);
        }
    }
    s = result;

    // Max 2 consecutive newlines
    while s.contains("\n\n\n") {
        s = s.replace("\n\n\n", "\n\n");
    }

    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_removes_control_chars() {
        assert_eq!(sanitize_prediction("hello\x00world"), "helloworld");
        assert_eq!(sanitize_prediction("hello\x07world"), "helloworld");
    }

    #[test]
    fn test_sanitize_preserves_newlines() {
        assert_eq!(sanitize_prediction("hello\nworld"), "hello\nworld");
    }

    #[test]
    fn test_sanitize_normalizes_line_endings() {
        assert_eq!(sanitize_prediction("hello\r\nworld"), "hello\nworld");
        assert_eq!(sanitize_prediction("hello\rworld"), "hello\nworld");
    }

    #[test]
    fn test_sanitize_tabs_to_spaces() {
        assert_eq!(sanitize_prediction("hello\tworld"), "hello world");
    }

    #[test]
    fn test_sanitize_collapses_spaces() {
        assert_eq!(sanitize_prediction("hello    world"), "hello world");
    }

    #[test]
    fn test_sanitize_limits_newlines() {
        assert_eq!(
            sanitize_prediction("hello\n\n\n\nworld"),
            "hello\n\nworld"
        );
    }

    #[test]
    fn test_sanitize_trims() {
        assert_eq!(sanitize_prediction("  hello  "), "hello");
    }
}
