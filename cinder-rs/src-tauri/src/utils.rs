//! Utility functions for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};
use sha2::{Digest, Sha256};

/// Open folder in system file explorer
pub fn open_folder(path: &PathBuf) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()?;
    }
    Ok(())
}

/// Open external URL in default browser
pub fn open_external_url(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()?;
    }
    Ok(())
}

/// Compute SHA256 of a file
pub async fn compute_sha256(path: &PathBuf) -> Result<String> {
    let content = tokio::fs::read(path).await?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&content);
    Ok(hex::encode(hasher.finalize()))
}

/// Compute SHA1 of a file
pub async fn compute_sha1(path: &PathBuf) -> Result<String> {
    let content = tokio::fs::read(path).await?;
    let mut hasher = sha2::Sha1::new();
    hasher.update(&content);
    Ok(hex::encode(hasher.finalize()))
}

/// Get system information
pub fn get_system_info() -> SystemInfo {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    SystemInfo {
        platform: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        total_memory_gb: sys.total_memory() as f64 / 1_073_741_824.0,
        free_memory_gb: sys.available_memory() as f64 / 1_073_741_824.0,
        cpu_count: sys.cpus().len(),
        cpu_brand: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
        os_version: std::env::consts::OS.to_string(),
        hostname: hostname::get().ok().and_then(|h| h.to_str().map(|s| s.to_string())),
    }
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration as human-readable string
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Generate a random UUID
pub fn generate_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

/// Slugify a string for use in file paths
pub fn slugify(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Truncate string to max length with ellipsis
pub fn truncate(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        input.to_string()
    } else {
        format!("{}...", &input[..max_len.saturating_sub(3)])
    }
}

/// Parse semver version string
pub fn parse_version(version: &str) -> Result<semver::Version, semver::Error> {
    semver::Version::parse(version)
}

/// Compare two version strings
pub fn compare_versions(a: &str, b: &str) -> Result<std::cmp::Ordering, semver::Error> {
    let va = semver::Version::parse(a)?;
    let vb = semver::Version::parse(b)?;
    Ok(va.cmp(&vb))
}

/// Check if a version satisfies a requirement
pub fn version_satisfies(version: &str, requirement: &str) -> bool {
    // Simplified - would use semver::VersionReq in practice
    if let Ok(v) = semver::Version::parse(version) {
        if let Ok(req) = semver::VersionReq::parse(requirement) {
            return req.matches(&v);
        }
    }
    false
}

/// Deep merge two JSON values
pub fn deep_merge_json(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
            for (k, v) in b {
                deep_merge_json(a.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

/// Sanitize file path (replace home with ~)
pub fn sanitize_path(path: &PathBuf, home: Option<&PathBuf>) -> PathBuf {
    let path_str = path.to_string_lossy();
    if let Some(home_dir) = home {
        let home_str = home_dir.to_string_lossy();
        PathBuf::from(path_str.replace(&home_str, "~"))
    } else {
        path.clone()
    }
}

/// Check if a command exists in PATH
pub fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Run a command and capture output
pub fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow::anyhow!(
            "Command failed: {} {}", cmd, args.join(" ")
        ))
    }
}

/// Get current timestamp as ISO 8601 string
pub fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Parse ISO 8601 timestamp
pub fn parse_iso8601(s: &str) -> Result<chrono::DateTime<chrono::Utc>, chrono::ParseError> {
    chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc))
}

/// Check if running in a container
pub fn is_container() -> bool {
    PathBuf::from("/.dockerenv").exists() || std::env::var("container").is_ok()
}

/// Get available disk space
pub async fn get_disk_space(path: &PathBuf) -> Result<DiskSpace> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        let metadata = std::fs::metadata(path)?;
        Ok(DiskSpace {
            total: metadata.file_size(),
            free: metadata.file_size(), // Simplified
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::MetadataExt;
        let stat = nix::sys::statvfs::statvfs(path)?;
        Ok(DiskSpace {
            total: stat.blocks() * stat.block_size(),
            free: stat.blocks_free() * stat.block_size(),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiskSpace {
    pub total: u64,
    pub free: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(std::time::Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(3661)), "1h 1m");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("Test@#$%Mod"), "test-mod");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello World", 20), "Hello World");
        assert_eq!(truncate("Hello World", 8), "Hello...");
    }
}