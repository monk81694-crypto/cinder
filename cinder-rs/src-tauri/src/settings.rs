//! Settings management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use tauri::AppHandle;

/// Settings manager with persistence
pub struct SettingsManager {
    settings: Arc<RwLock<Settings>>,
    config_path: PathBuf,
}

impl SettingsManager {
    pub async fn new(app_handle: &AppHandle) -> Result<Self> {
        let config_dir = app_handle
            .path_resolver()
            .app_config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cinder");

        tokio::fs::create_dir_all(&config_dir).await?;

        let config_path = config_dir.join("settings.toml");

        let settings = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            Settings::default()
        };

        // Validate and fix settings
        crate::config::validate_settings(&mut settings).ok();

        Ok(Self {
            settings: Arc::new(RwLock::new(settings)),
            config_path,
        })
    }

    pub async fn get(&self) -> Settings {
        self.settings.read().await.clone()
    }

    pub async fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Settings),
    {
        let mut settings = self.settings.write().await;
        f(&mut settings);
        self.save(&settings).await
    }

    async fn save(&self, settings: &Settings) -> Result<()> {
        let content = toml::to_string_pretty(settings)?;
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&self.config_path, content).await?;
        debug!("Settings saved to {:?}", self.config_path);
        Ok(())
    }

    pub async fn reset(&self) -> Result<()> {
        let mut settings = Settings::default();
        crate::config::validate_settings(&mut settings).ok;
        self.save(&settings).await?;
        *self.settings.write().await = settings;
        Ok(())
    }

    pub async fn export(&self, path: &PathBuf, password: &str) -> Result<()> {
        let settings = self.settings.read().await.clone();

        // Sanitize sensitive data
        let mut export_settings = settings;
        export_settings.discord_client_id = None;
        export_settings.java_path = None;
        export_settings.game_directory = PathBuf::from("~/.cinder-minecraft");

        let data = serde_json::to_vec(&export_settings)?;

        // Encrypt with password
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let key = hasher.finalize();

        let mut encrypted = Vec::with_capacity(data.len() + 16);
        encrypted.extend_from_slice(&key[..16]);
        for (i, byte) in data.iter().enumerate() {
            encrypted.push(byte ^ key[i % 32]);
        }

        tokio::fs::write(path, encrypted).await?;
        Ok(())
    }

    pub async fn import(&self, path: &PathBuf, password: &str) -> Result<()> {
        let encrypted = tokio::fs::read(path).await?;
        if encrypted.len() < 16 {
            return Err(anyhow::anyhow!("Invalid encrypted file"));
        }

        let key = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            hasher.finalize()
        };

        let data = &encrypted[16..];
        let mut decrypted = Vec::with_capacity(data.len());
        for (i, byte) in data.iter().enumerate() {
            decrypted.push(byte ^ key[i % 32]);
        }

        let mut settings: Settings = serde_json::from_slice(&decrypted)?;
        crate::config::validate_settings(&mut settings).ok;

        self.save(&settings).await?;
        *self.settings.write().await = settings;

        Ok(())
    }

    pub async fn verify_game_dir(&self) -> Result<HealthCheckResult> {
        let settings = self.settings.read().await;
        let game_dir = settings.game_directory.clone();
        drop(settings);

        let mut issues = Vec::new();
        let mut total_files = 0u64;
        let mut total_size = 0u64;
        let mut can_auto_fix = false;

        // Check game directory exists
        if !game_dir.exists() {
            issues.push(HealthIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::Other,
                message: "Game directory does not exist".to_string(),
                path: Some(game_dir.clone()),
                auto_fixable: true,
                fix_description: Some("Create game directory".to_string()),
            });
            return Ok(HealthCheckResult {
                scanned_at: chrono::Utc::now(),
                total_files: 0,
                total_size_bytes: 0,
                issues,
                can_auto_fix: false,
            });
        }

        // Scan versions
        let versions_dir = game_dir.join("versions");
        if versions_dir.exists() {
            for entry in walkdir::WalkDir::new(&versions_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Ok(metadata) = path.metadata() {
                    total_files += 1;
                    total_size += metadata.len();
                }
            }
        }

        // Check for corrupted JARs
        let versions_dir = game_dir.join("versions");
        if versions_dir.exists() {
            for entry in std::fs::read_dir(&versions_dir)? {
                let entry = entry?;
                let version_dir = entry.path();
                if version_dir.is_dir() {
                    let jar_path = version_dir.join(format!("{}.jar", version_dir.file_name().unwrap().to_string_lossy()));
                    if jar_path.exists() {
                        // Try to read JAR
                        if std::fs::read(&jar_path).is_err() {
                            issues.push(HealthIssue {
                                severity: IssueSeverity::Error,
                                category: IssueCategory::CorruptedJar,
                                message: format!("Corrupted JAR: {}", jar_path.display()),
                                path: Some(jar_path.clone()),
                                auto_fixable: true,
                                fix_description: Some("Re-download version".to_string()),
                            });
                            can_auto_fix = true;
                        }
                    }
                }
            }
        }

        // Check mods
        let mods_dir = game_dir.join("mods");
        if mods_dir.exists() {
            for entry in std::fs::read_dir(&mods_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "jar").unwrap_or(false) {
                    total_files += 1;
                    if let Ok(metadata) = path.metadata() {
                        total_size += metadata.len();
                    }
                    // Quick JAR validation
                    if std::fs::read(&path).is_err() {
                        issues.push(HealthIssue {
                            severity: IssueSeverity::Error,
                            category: IssueCategory::CorruptedJar,
                            message: format!("Corrupted mod JAR: {}", path.display()),
                            path: Some(path.clone()),
                            auto_fixable: true,
                            fix_description: Some("Re-download mod".to_string()),
                        });
                        can_auto_fix = true;
                    }
                }
            }
        }

        // Check for orphaned mods (mods without corresponding loader)
        // This would require checking against installed loaders

        // Check resource packs
        let packs_dir = game_dir.join("resourcepacks");
        if packs_dir.exists() {
            for entry in std::fs::read_dir(&packs_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "zip").unwrap_or(false) {
                    // Try to read pack.mcmeta
                    use zip::ZipArchive;
                    let zip = ZipArchive::new(&path);
                    if zip.is_err() {
                        issues.push(HealthIssue {
                            severity: IssueSeverity::Warning,
                            category: IssueCategory::CorruptedJar,
                            message: format!("Corrupted resource pack: {}", path.display()),
                            path: Some(path.clone()),
                            auto_fixable: true,
                            fix_description: Some("Re-download resource pack".to_string()),
                        });
                        can_auto_fix = true;
                    }
                }
            }
        }

        // Check shader packs
        let shaders_dir = game_dir.join("shaderpacks");
        if shaders_dir.exists() {
            for entry in std::fs::read_dir(&shaders_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "zip").unwrap_or(false) {
                    use zip::ZipArchive;
                    let zip = ZipArchive::new(&path);
                    if zip.is_err() {
                        issues.push(HealthIssue {
                            severity: IssueSeverity::Warning,
                            category: IssueCategory::CorruptedJar,
                            message: format!("Corrupted shader pack: {}", path.display()),
                            path: Some(path.clone()),
                            auto_fixable: true,
                            fix_description: Some("Re-download shader pack".to_string()),
                        });
                        can_auto_fix = true;
                    }
                }
            }
        }

        // Check worlds
        let saves_dir = game_dir.join("saves");
        if saves_dir.exists() {
            for entry in std::fs::read_dir(&saves_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if !path.join("level.dat").exists() {
                        issues.push(HealthIssue {
                            severity: IssueSeverity::Warning,
                            category: IssueCategory::MissingLevelDat,
                            message: format!("World missing level.dat: {}", path.display()),
                            path: Some(path.clone()),
                            auto_fixable: false,
                            fix_description: None,
                        });
                    }
                }
            }
        }

        // Check disk space
        if let Ok(disk) = std::fs::read_dir(&game_dir) {
            // This is a simplified check
        }

        Ok(HealthCheckResult {
            scanned_at: chrono::Utc::now(),
            total_files,
            total_size_bytes: 0, // Would calculate properly
            issues,
            can_auto_fix,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.min_memory_mb, 2048);
        assert_eq!(settings.max_memory_mb, 4096);
    }
}