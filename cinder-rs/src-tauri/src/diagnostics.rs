//! Diagnostics for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use reqwest::Client;
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt};

/// Diagnostics manager
pub struct DiagnosticsManager {
    settings: Arc<RwLock<Settings>>,
    client: Client,
}

impl DiagnosticsManager {
    pub fn new(settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            settings,
            client: Client::new(),
        }
    }

    /// Run full diagnostics
    pub async fn run_diagnostics(&self) -> Result<Diagnostics> {
        let settings = self.settings.read().await;
        let game_dir = settings.game_directory.clone();
        drop(settings);

        let mut sys = System::new_all();
        sys.refresh_all();

        // Java installations
        let java_installations = self.detect_java().await?;

        // Game directory info
        let (versions_cached, installed_versions, mods_count, resource_packs_count, shader_packs_count, worlds_count) =
            self.scan_game_dir(&game_dir).await?;

        // Network checks
        let network_checks = self.check_network().await?;

        // Settings hash
        let settings_hash = self.compute_settings_hash().await?;

        Ok(Diagnostics {
            timestamp: Utc::now(),
            launcher_version: "3.0.0".to_string(),
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            total_memory_gb: sys.total_memory() as f64 / 1_073_741_824.0,
            free_memory_gb: sys.available_memory() as f64 / 1_073_741_824.0,
            java_installations,
            game_directory: game_dir,
            versions_cached,
            installed_versions,
            mods_count,
            resource_packs_count,
            shader_packs_count,
            worlds_count,
            accounts_count: 0, // Would come from account manager
            network_checks,
            settings_hash,
        })
    }

    async fn detect_java(&self) -> Result<Vec<JavaInstallation>> {
        let mut installations = Vec::new();

        // Check JAVA_HOME
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            if let Ok(install) = self.validate_java_path(PathBuf::from(java_home)).await {
                installations.push(install);
            }
        }

        // Check `java` in PATH
        if let Ok(path) = which::which("java") {
            if let Ok(install) = self.validate_java_path(path).await {
                installations.push(install);
            }
        }

        // Scan common directories
        let scan_dirs = self.get_scan_directories();
        for dir in scan_dirs {
            if let Ok(entries) = self.scan_directory(&dir).await {
                for install in entries {
                    if !installations.iter().any(|i| i.path == install.path) {
                        installations.push(install);
                    }
                }
            }
        }

        Ok(installations)
    }

    fn get_scan_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        #[cfg(target_os = "windows")]
        {
            if let Ok(program_files) = std::env::var("ProgramFiles") {
                dirs.push(PathBuf::from(program_files).join("Java"));
                dirs.push(PathBuf::from(program_files).join("Eclipse Adoptium"));
                dirs.push(PathBuf::from(program_files).join("Microsoft"));
                dirs.push(PathBuf::from(program_files).join("Azul"));
                dirs.push(PathBuf::from(program_files).join("BellSoft"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            dirs.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
            dirs.push(PathBuf::from("/usr/lib/jvm"));
        }

        #[cfg(target_os = "linux")]
        {
            dirs.push(PathBuf::from("/usr/lib/jvm"));
            dirs.push(PathBuf::from("/usr/java"));
        }

        dirs
    }

    async fn scan_directory(&self, dir: &PathBuf) -> Result<Vec<JavaInstallation>> {
        let mut installations = Vec::new();

        if !dir.exists() {
            return Ok(installations);
        }

        for entry in walkdir::WalkDir::new(dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.file_name().map(|n| n == "java" || n == "java.exe").unwrap_or(false) {
                if let Ok(install) = self.validate_java_path(path.to_path_buf()).await {
                    installations.push(install);
                }
            }
        }

        Ok(installations)
    }

    async fn validate_java_path(&self, path: PathBuf) -> Result<JavaInstallation> {
        let output = Command::new(&path)
            .args(["-version"])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let version_info = self.parse_java_version(&stderr)?;

        let vendor = JavaVendor::from_path(&path);
        let architecture = self.detect_architecture(&path).await?;
        let has_javafx = self.check_javafx(&path).await?;
        let is_bundled = path.to_string_lossy().contains("runtime");

        Ok(JavaInstallation {
            path,
            version: version_info.version_string.clone(),
            major_version: version_info.major,
            vendor,
            architecture,
            has_javafx,
            is_bundled,
            detected_at: chrono::Utc::now(),
        })
    }

    fn parse_java_version(&self, output: &str) -> Result<JavaVersionInfo> {
        let version_regex = regex::Regex::new(r#"version\s+"([^"]+)"#)?;
        let version_str = version_regex
            .captures(output)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .unwrap_or("0.0.0");

        let major = if version_str.starts_with("1.") {
            version_str.split('.').nth(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(8)
        } else {
            version_str.split('.').next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0)
        };

        Ok(JavaVersionInfo {
            version_string: version_str.to_string(),
            major,
        })
    }

    async fn detect_architecture(&self, path: &PathBuf) -> Result<String> {
        let output = Command::new(path)
            .args(["-XshowSettings:properties", "-version"])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("os.arch = aarch64") || stderr.contains("os.arch = arm64") {
            Ok("arm64".to_string())
        } else {
            Ok("x64".to_string())
        }
    }

    async fn check_javafx(&self, path: &PathBuf) -> Result<bool> {
        let output = Command::new(path)
            .args(["--module-path", "", "--add-modules", "javafx.controls", "-version"])
            .output();
        Ok(output.is_ok())
    }

    async fn scan_game_dir(&self, game_dir: &PathBuf) -> Result<(u32, u32, u32, u32, u32, u32)> {
        let mut versions_cached = 0;
        let mut installed_versions = 0;
        let mut mods_count = 0;
        let mut resource_packs_count = 0;
        let mut shader_packs_count = 0;
        let mut worlds_count = 0;

        // Versions cache
        let cache_dir = game_dir.join("cache");
        if cache_dir.join("version_manifest.json").exists() {
            versions_cached = 1;
        }

        // Installed versions
        let versions_dir = game_dir.join("versions");
        if versions_dir.exists() {
            for entry in std::fs::read_dir(&versions_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    installed_versions += 1;
                }
            }
        }

        // Mods
        let mods_dir = game_dir.join("mods");
        if mods_dir.exists() {
            for entry in std::fs::read_dir(&mods_dir)? {
                let entry = entry?;
                if entry.path().extension().map(|e| e == "jar").unwrap_or(false) {
                    mods_count += 1;
                }
            }
        }

        // Resource packs
        let packs_dir = game_dir.join("resourcepacks");
        if packs_dir.exists() {
            for entry in std::fs::read_dir(&packs_dir)? {
                let entry = entry?;
                if entry.path().extension().map(|e| e == "zip").unwrap_or(false) {
                    resource_packs_count += 1;
                }
            }
        }

        // Shader packs
        let shaders_dir = game_dir.join("shaderpacks");
        if shaders_dir.exists() {
            for entry in std::fs::read_dir(&shaders_dir)? {
                let entry = entry?;
                if entry.path().extension().map(|e| e == "zip").unwrap_or(false) {
                    shader_packs_count += 1;
                }
            }
        }

        // Worlds
        let saves_dir = game_dir.join("saves");
        if saves_dir.exists() {
            for entry in std::fs::read_dir(&saves_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    worlds_count += 1;
                }
            }
        }

        Ok((versions_cached, installed_versions, mods_count, resource_packs_count, shader_packs_count, worlds_count))
    }

    async fn check_network(&self) -> Result<NetworkChecks> {
        let checks = vec![
            ("mojang", "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"),
            ("modrinth", "https://api.modrinth.com/v2/search?query=test&limit=1"),
            ("minotar", "https://minotar.net/helm/Steve/64.png"),
            ("microsoft", "https://login.microsoftonline.com"),
            ("discord", "https://discord.com/api/v10/users/@me"),
            ("github", "https://api.github.com/repos/monk81694-crypto/cinder/releases/latest"),
        ];

        let mut results = NetworkChecks::default();
        let mut handles = Vec::new();

        for (name, url) in checks {
            let client = self.client.clone();
            let url = url.to_string();
            handles.push(tokio::spawn(async move {
                let result = client.get(&url)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false);
                (name, result)
            }));
        }

        for handle in handles {
            if let Ok((name, success)) = handle.await {
                match name {
                    "mojang" => results.mojang = success,
                    "modrinth" => results.modrinth = success,
                    "minotar" => results.minotar = success,
                    "microsoft" => results.microsoft_auth = success,
                    "discord" => results.discord = success,
                    "github" => results.github = success,
                    _ => {}
                }
            }
        }

        Ok(results)
    }

    async fn compute_settings_hash(&self) -> Result<String> {
        let settings = self.get_settings_for_hash().await?;
        let json = serde_json::to_string(&settings)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(json.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    async fn get_settings_for_hash(&self) -> Result<Settings> {
        // Return sanitized settings (no tokens, paths replaced with placeholders)
        let mut settings = self.settings.read().await.clone();

        // Sanitize paths
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy().to_string();
            settings.game_directory = PathBuf::from(settings.game_directory.to_string_lossy().replace(&home_str, "~"));
            if let Some(java_path) = &settings.java_path {
                settings.java_path = Some(PathBuf::from(java_path.to_string_lossy().replace(&home_str, "~")));
            }
        }

        // Remove sensitive data
        settings.discord_client_id = None;

        Ok(settings)
    }
}

#[derive(Debug, Clone)]
struct JavaVersionInfo {
    version_string: String,
    major: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_checks_default() {
        let checks = NetworkChecks::default();
        assert!(!checks.mojang);
        assert!(!checks.modrinth);
    }
}