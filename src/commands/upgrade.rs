//! Self-upgrade command: check for latest release and update binary.

use std::env;
use std::fs;
use std::process::Command;

use anyhow::{Context, bail};

use crate::output::OutputFormat;

const REPO: &str = "Clawduel/clawduel-cli";
const BINARY: &str = "clawduel";

pub fn execute(fmt: OutputFormat) -> anyhow::Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    if matches!(fmt, OutputFormat::Table) {
        println!("Current version: v{current_version}");
        println!("Checking for updates...");
    }

    let latest_tag = match get_latest_tag() {
        Ok(tag) => tag,
        Err(_) => {
            // Fallback: print manual upgrade instructions
            return print_manual_upgrade(current_version, fmt);
        }
    };

    let latest_version = latest_tag.trim_start_matches('v');

    if latest_version == current_version {
        match fmt {
            OutputFormat::Json => {
                crate::output::print_json(&serde_json::json!({
                    "current": current_version,
                    "latest": latest_version,
                    "up_to_date": true,
                }))?;
            }
            OutputFormat::Table => {
                println!("Already up to date (v{current_version}).");
            }
        }
        return Ok(());
    }

    if matches!(fmt, OutputFormat::Table) {
        println!("New version available: {latest_tag}");
    }

    let target = detect_target()?;
    let url = format!(
        "https://github.com/{REPO}/releases/download/{latest_tag}/{BINARY}-{latest_tag}-{target}.tar.gz"
    );

    let current_exe = env::current_exe().context("Failed to determine current executable path")?;

    let tmpdir = tempdir()?;
    let tarball = format!("{tmpdir}/{BINARY}.tar.gz");

    let tarball_name = format!("{BINARY}-{latest_tag}-{target}.tar.gz");
    let checksums_url =
        format!("https://github.com/{REPO}/releases/download/{latest_tag}/checksums.txt");

    if matches!(fmt, OutputFormat::Table) {
        println!("Downloading {latest_tag} ({target})...");
    }

    let status = Command::new("curl")
        .args(["-sSfL", "-o", &tarball, &url])
        .status()
        .context("Failed to run curl")?;
    if !status.success() {
        bail!("Download failed (HTTP error)");
    }

    let checksums_file = format!("{tmpdir}/checksums.txt");
    let status = Command::new("curl")
        .args(["-sSfL", "-o", &checksums_file, &checksums_url])
        .status()
        .context("Failed to download checksums")?;
    if !status.success() {
        bail!("Failed to download checksums.txt -- cannot verify integrity");
    }

    verify_checksum(&tarball, &checksums_file, &tarball_name)?;

    let status = Command::new("tar")
        .args(["xzf", &tarball, "-C", &tmpdir])
        .status()
        .context("Failed to extract archive")?;
    if !status.success() {
        bail!("Failed to extract archive");
    }

    let new_binary = format!("{tmpdir}/{BINARY}");

    // Replace the current binary
    let exe_path = current_exe.to_str().context("Non-UTF8 executable path")?;
    let backup = format!("{exe_path}.bak");

    fs::rename(exe_path, &backup)
        .or_else(|_| sudo_mv(exe_path, &backup))
        .context("Failed to replace binary (try running with sudo)")?;

    if let Err(e) = fs::rename(&new_binary, exe_path).or_else(|_| sudo_mv(&new_binary, exe_path)) {
        // Restore backup on failure
        let _ = fs::rename(&backup, exe_path);
        return Err(e).context("Failed to install new binary");
    }

    // Set executable permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(exe_path, fs::Permissions::from_mode(0o755));
    }

    let _ = fs::remove_file(&backup);
    let _ = fs::remove_dir_all(&tmpdir);

    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&serde_json::json!({
                "current": current_version,
                "latest": latest_version,
                "updated": true,
            }))?;
        }
        OutputFormat::Table => {
            println!("Updated to {latest_tag}");
        }
    }

    Ok(())
}

fn print_manual_upgrade(current_version: &str, fmt: OutputFormat) -> anyhow::Result<()> {
    match fmt {
        OutputFormat::Json => {
            crate::output::print_json(&serde_json::json!({
                "current": current_version,
                "up_to_date": false,
                "manual_upgrade": true,
                "instructions": format!("cargo install clawduel-cli --force"),
            }))?;
        }
        OutputFormat::Table => {
            println!("Could not check GitHub releases.");
            println!("To upgrade manually:");
            println!("  cargo install clawduel-cli --force");
            println!("  # or download from https://github.com/{REPO}/releases");
        }
    }
    Ok(())
}

fn get_latest_tag() -> anyhow::Result<String> {
    let output = Command::new("curl")
        .args([
            "-sSf",
            &format!("https://api.github.com/repos/{REPO}/releases/latest"),
        ])
        .output()
        .context("Failed to check for latest release")?;

    if !output.status.success() {
        bail!("Failed to fetch latest release info from GitHub");
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&body).context("Failed to parse GitHub API response")?;

    json["tag_name"]
        .as_str()
        .map(String::from)
        .context("No tag_name in release response")
}

fn detect_target() -> anyhow::Result<&'static str> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        _ => bail!("Unsupported platform: {os}/{arch}"),
    }
}

fn tempdir() -> anyhow::Result<String> {
    let output = Command::new("mktemp")
        .args(["-d"])
        .output()
        .context("Failed to create temp directory")?;
    if !output.status.success() {
        bail!("mktemp failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn verify_checksum(
    file_path: &str,
    checksums_file: &str,
    expected_name: &str,
) -> anyhow::Result<()> {
    let checksums = fs::read_to_string(checksums_file).context("Failed to read checksums.txt")?;

    let expected_hash = checksums
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let name = parts.next()?;
            if name == expected_name || name.trim_start_matches("./") == expected_name {
                Some(hash.to_string())
            } else {
                None
            }
        })
        .context(format!(
            "No checksum found for {expected_name} in checksums.txt"
        ))?;

    let output = Command::new("shasum")
        .args(["-a", "256", file_path])
        .output()
        .or_else(|_| Command::new("sha256sum").arg(file_path).output())
        .context("Failed to compute SHA256 (need shasum or sha256sum)")?;

    if !output.status.success() {
        bail!("Failed to compute SHA256 of downloaded file");
    }

    let actual_hash = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();

    if actual_hash != expected_hash {
        bail!(
            "Checksum mismatch!\n  Expected: {expected_hash}\n  Got:      {actual_hash}\n\nThe downloaded binary may have been tampered with. Aborting."
        );
    }

    if matches!(std::io::IsTerminal::is_terminal(&std::io::stdout()), true) {
        println!("Checksum verified.");
    }

    Ok(())
}

fn sudo_mv(from: &str, to: &str) -> std::io::Result<()> {
    let status = Command::new("sudo").args(["mv", from, to]).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "sudo mv failed",
        ))
    }
}
