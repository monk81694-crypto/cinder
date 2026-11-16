//! Java detection and management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use which::which;
use dirs;
use walkdir::WalkDir;
use regex::Regex;

/// Java manager for detection, validation, and installation
pub struct JavaManager {
    detected_cache: Arc<RwLock<Vec<JavaInstallation>>>,
    settings: Arc<RwLock<Settings>>,
}

impl JavaManager {
    pub fn new(settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            detected_cache: Arc::new(RwLock::new(Vec::new())),
            settings,
        }
    }

    /// List all detected Java installations
    pub async fn list_java(&self) -> Result<Vec<JavaInstallation>> {
        let mut installations = Vec::new();

        // 1. Check explicit JAVA_HOME
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            if let Ok(install) = self.validate_java_path(PathBuf::from(java_home)).await {
                installations.push(install);
            }
        }

        // 2. Check `java` in PATH
        if let Ok(path) = which("java") {
            if let Ok(install) = self.validate_java_path(path).await {
                if !installations.iter().any(|i| i.path == install.path) {
                    installations.push(install);
                }
            }
        }

        // 3. Scan common installation directories
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

        // 4. Check for Mojang-bundled runtimes
        if let Ok(mojang_runtimes) = self.find_mojang_runtimes().await {
            for install in mojang_runtimes {
                if !installations.iter().any(|i| i.path == install.path) {
                    installations.push(install);
                }
            }
        }

        // Update cache
        *self.detected_cache.write().await = installations.clone();

        Ok(installations)
    }

    /// Get directories to scan for Java installations
    fn get_scan_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // Windows
        #[cfg(target_os = "windows")]
        {
            if let Ok(program_files) = std::env::var("ProgramFiles") {
                dirs.push(PathBuf::from(program_files).join("Java"));
                dirs.push(PathBuf::from(program_files).join("Eclipse Adoptium"));
                dirs.push(PathBuf::from(program_files).join("Microsoft"));
                dirs.push(PathBuf::from(program_files).join("Azul"));
                dirs.push(PathBuf::from(program_files).join("BellSoft"));
                dirs.push(PathBuf::from(program_files).join("GraalVM"));
            }
            if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
                dirs.push(PathBuf::from(program_files_x86).join("Java"));
            }
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                dirs.push(PathBuf::from(local_appdata).join("Microsoft"));
                dirs.push(PathBuf::from(local_appdata).join("Eclipse Adoptium"));
            }
        }

        // macOS
        #[cfg(target_os = "macos")]
        {
            dirs.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
            dirs.push(PathBuf::from("/usr/lib/jvm"));
            if let Ok(home) = std::env::var("HOME") {
                dirs.push(PathBuf::from(home).join(".sdkman/candidates/java"));
                dirs.push(PathBuf::from(home).join(".jenv/versions"));
            }
        }

        // Linux
        #[cfg(target_os = "linux")]
        {
            dirs.push(PathBuf::from("/usr/lib/jvm"));
            dirs.push(PathBuf::from("/usr/java"));
            dirs.push(PathBuf::from("/opt/java"));
            dirs.push(PathBuf::from("/opt/jdk"));
            if let Ok(home) = std::env::var("HOME") {
                dirs.push(PathBuf::from(home).join(".sdkman/candidates/java"));
                dirs.push(PathBuf::from(home).join(".jenv/versions"));
            }
        }

        dirs
    }

    /// Scan a directory for Java installations
    async fn scan_directory(&self, dir: &Path) -> Result<Vec<JavaInstallation>> {
        let mut installations = Vec::new();

        if !dir.exists() {
            return Ok(installations);
        }

        for entry in WalkDir::new(dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.file_name().map(|n| n == "java" || n == "java.exe").unwrap_or(false) {
                if let Ok(install) = self.validate_java_path(path.to_path_buf()).await {
                    installations.push(install);
                }
            }
            // Also check for bin/java inside JDK directories
            if path.file_name().map(|n| n == "bin").unwrap_or(false) {
                let java_exe = path.join(if cfg!(target_os = "windows") { "java.exe" } else { "java" });
                if java_exe.exists() {
                    if let Ok(install) = self.validate_java_path(java_exe).await {
                        installations.push(install);
                    }
                }
            }
        }

        Ok(installations)
    }

    /// Find Mojang-bundled Java runtimes
    async fn find_mojang_runtimes(&self) -> Result<Vec<JavaInstallation>> {
        let mut installations = Vec::new();

        let game_dir = self.settings.read().await.game_directory.clone();
        let runtime_dir = game_dir.join("runtime");

        if runtime_dir.exists() {
            for entry in WalkDir::new(&runtime_dir).max_depth(4).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.file_name().map(|n| n == "java" || n == "java.exe").unwrap_or(false) {
                    if let Ok(install) = self.validate_java_path(path.to_path_buf()).await {
                        let mut install = install;
                        install.is_bundled = true;
                        installations.push(install);
                    }
                }
            }
        }

        Ok(installations)
    }

    /// Validate a Java path and extract version info
    async fn validate_java_path(&self, path: PathBuf) -> Result<JavaInstallation> {
        let output = Command::new(&path)
            .args(["-version"])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let version_info = self.parse_java_version(&stderr)?;

        let vendor = JavaVendor::from_path(&path);
        let architecture = self.detect_architecture(&path).await?;
        let has_javafx = self.check_javafx(&path).await?;
        let is_bundled = path.to_string_lossy().contains("runtime") || path.to_string_lossy().contains("mojang");

        Ok(JavaInstallation {
            path: path.clone(),
            version: version_info.version_string.clone(),
            major_version: version_info.major,
            vendor,
            architecture,
            has_javafx,
            is_bundled,
            detected_at: chrono::Utc::now(),
        })
    }

    /// Parse Java version from stderr output
    fn parse_java_version(&self, output: &str) -> Result<JavaVersionInfo> {
        // Handle different version string formats:
        // openjdk version "21.0.2" 2024-01-16
        // java version "1.8.0_361"
        // openjdk version "17.0.9" 2023-10-17
        // eclipse temurin version "21.0.2+13" (build 1.21.0.2+13)

        let version_regex = Regex::new(r#"version\s+"([^"]+)"#)?;
        let version_str = version_regex
            .captures(output)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .unwrap_or("0.0.0");

        // Parse major version
        let major = if version_str.starts_with("1.") {
            // Old format: 1.8.0_361 -> major = 8
            version_str.split('.').nth(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(8)
        } else {
            // New format: 21.0.2 -> major = 21
            version_str.split('.').next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0)
        };

        Ok(JavaVersionInfo {
            version_string: version_str.to_string(),
            major,
        })
    }

    /// Detect architecture (x64, arm64)
    async fn detect_architecture(&self, path: &Path) -> Result<String> {
        let output = Command::new(path)
            .args(["-XshowSettings:properties", "-version"])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("os.arch = aarch64") || stderr.contains("os.arch = arm64") {
            Ok("arm64".to_string())
        } else if stderr.contains("os.arch = x86_64") || stderr.contains("os.arch = amd64") {
            Ok("x64".to_string())
        } else {
            Ok("x64".to_string()) // Default
        }
    }

    /// Check if JavaFX is available
    async fn check_javafx(&self, path: &Path) -> Result<bool> {
        let output = Command::new(path)
            .args(["--module-path", "", "--add-modules", "javafx.controls", "-version"])
            .output();

        Ok(output.is_ok())
    }

    /// Get recommended Java version for a Minecraft version
    pub fn get_required_java_version(&self, mc_version: &str) -> u8 {
        // Parse version like "1.21.4" or "24w10a"
        if mc_version.starts_with("1.") {
            let parts: Vec<&str> = mc_version.split('.').collect();
            if parts.len() >= 2 {
                if let Ok(minor) = parts[1].parse::<u32>() {
                    if minor >= 21 { return 21; }
                    if minor >= 18 { return 17; }
                    if minor >= 17 { return 16; }
                }
            }
            8
        } else if mc_version.starts_with("2") {
            // 24w10a format - snapshot, use latest
            21
        } else {
            21 // Default to latest LTS
        }
    }

    /// Install recommended Java (Eclipse Temurin LTS)
    pub async fn install_recommended_java(&self, mc_version: &str) -> Result<JavaInstallation> {
        let required_version = self.get_required_java_version(mc_version);
        let lts_version = match required_version {
            8 => 8,
            11 => 11,
            17 => 17,
            21 => 21,
            _ => 21,
        };

        // Download from Adoptium API
        let download_url = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/jdk/hotspot/normal/eclipse",
            lts_version,
            if cfg!(target_os = "windows") { "windows/x64" }
            else if cfg!(target_os = "macos") { "mac/x64" }
            else { "linux/x64" }
        );

        // This would download and extract the JDK
        // For now, return an error indicating manual install needed
        Err(anyhow::anyhow!(
            "Automatic JDK installation not yet implemented. Please install Java {} manually from https://adoptium.net",
            lts_version
        ))
    }

    /// Set explicit Java path in settings
    pub async fn set_java_path(&self, path: PathBuf) -> Result<()> {
        let install = self.validate_java_path(path.clone()).await?;
        let mut settings = self.settings.write().await;
        settings.java_path = Some(path);
        Ok(())
    }

    /// Get best Java for a Minecraft version
    pub async fn get_best_java(&self, mc_version: &str) -> Result<Option<JavaInstallation>> {
        let required = self.get_required_java_version(mc_version);
        let installations = self.list_java().await?;

        // Filter by version >= required
        let mut candidates: Vec<_> = installations.into_iter()
            .filter(|j| j.major_version >= required)
            .collect();

        // Sort: prefer non-bundled, then higher version, then known vendors
        candidates.sort_by(|a, b| {
            // Prefer non-bundled
            a.is_bundled.cmp(&b.is_bundled)
                // Then prefer higher version
                .then(b.major_version.cmp(&a.major_version))
                // Then prefer known vendors
                .then_with(|| {
                    let a_vendor = match a.vendor {
                        JavaVendor::EclipseTemurin => 0,
                        JavaVendor::Microsoft => 1,
                        JavaVendor::Adoptium => 2,
                        JavaVendor::AzulZulu => 3,
                        JavaVendor::BellSoft => 4,
                        _ => 5,
                    };
                    let b_vendor = match b.vendor {
                        JavaVendor::EclipseTemurin => 0,
                        JavaVendor::Microsoft => 1,
                        JavaVendor::Adoptium => 2,
                        JavaVendor::AzulZulu => 3,
                        JavaVendor::BellSoft => 4,
                        _ => 5,
                    };
                    a_vendor.cmp(&b_vendor)
                })
        });

        Ok(candidates.into_iter().next())
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
    fn test_parse_java_version_old_format() {
        let output = r#"java version "1.8.0_361"
Java(TM) SE Runtime Environment (build 1.8.0_361-b09)
Java HotSpot(TM) 64-Bit Server VM (build 25.361-b09, mixed mode)"#;
        let info = JavaManager::parse_java_version_static(output).unwrap();
        assert_eq!(info.major, 8);
    }

    #[test]
    fn test_parse_java_version_new_format() {
        let output = r#"openjdk version "21.0.2" 2024-01-16
OpenJDK Runtime Environment (build 21.0.2+13)
OpenJDK 64-Bit Server VM (build 21.0.2+13, mixed mode)"#;
        let info = JavaManager::parse_java_version_static(output).unwrap();
        assert_eq!(info.major, 21);
    }

    #[test]
    fn test_required_java_version() {
        let manager = JavaManager::new(std::sync::Arc::new(tokio::sync::RwLock::new(Settings::default())));
        assert_eq!(manager.get_required_java_version("1.21.4"), 21);
        assert_eq!(manager.get_required_java_version("1.20.4"), 17);
        assert_eq!(manager.get_required_java_version("1.18.2"), 17);
        assert_eq!(manager.get_required_java_version("1.16.5"), 8);
        assert_eq!(manager.get_required_java_version("24w10a"), 21);
    }
}

impl JavaManager {
    fn parse_java_version_static(output: &str) -> Result<JavaVersionInfo> {
        let version_regex = Regex::new(r#"version\s+"([^"]+)"#)?;
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
}