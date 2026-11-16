//! Auto-updater for Cinder Launcher using Tauri's built-in updater

use crate::types::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use tauri_plugin_updater::UpdaterExt;

/// Update manager using Tauri's built-in updater
pub struct UpdateManager {
    settings: Arc<RwLock<Settings>>,
}

impl UpdateManager {
    pub fn new(settings: Arc<RwLock<Settings>>) -> Self {
        Self { settings }
    }

    /// Check for updates on startup
    pub async fn check_on_startup(app_handle: tauri::AppHandle) -> Result<()> {
        let settings = app_handle.state::<Arc<RwLock<Settings>>>().inner().clone();
        let check_enabled = settings.read().await.check_updates;

        if !check_enabled {
            info!("Update check disabled in settings");
            return Ok(());
        }

        // Use Tauri's built-in updater
        if let Err(e) = Self::check_and_notify(app_handle).await {
            warn!("Background update check failed: {}", e);
        }

        Ok(())
    }

    /// Check for updates and notify user
    async fn check_and_notify(app_handle: tauri::AppHandle) -> Result<()> {
        let updater = app_handle.updater()?;

        match updater.check().await {
            Ok(update) => {
                if update.is_update_available() {
                    let version = update.version.clone();
                    let notes = update.body.clone().unwrap_or_default();
                    let date = update.date;

                    info!("Update available: {} (current: {})", version, env!("CARGO_PKG_VERSION"));

                    // Emit event to frontend
                    let _ = app_handle.emit("update-available", UpdateAvailableEvent {
                        version,
                        notes,
                        date: date.to_string(),
                        download_url: "https://github.com/monk81694-crypto/cinder/releases/latest".to_string(),
                    });
                } else {
                    info!("No updates available");
                }
            }
            Err(e) => {
                warn!("Update check failed: {}", e);
                return Err(e.into());
            }
        }
        Ok(())
    }

    /// Manual update check (from settings)
    pub async fn check_for_updates(app_handle: tauri::AppHandle) -> Result<UpdateInfo> {
        let updater = app_handle.updater()?;

        match updater.check().await {
            Ok(update) => {
                if update.is_update_available() {
                    Ok(UpdateInfo {
                        current_version: env!("CARGO_PKG_VERSION").to_string(),
                        latest_version: update.version.clone(),
                        newer: true,
                        release_notes: update.body,
                        download_url: "https://github.com/monk81694-crypto/cinder/releases/latest".to_string(),
                        published_at: update.date,
                        mandatory: false,
                    })
                } else {
                    Ok(UpdateInfo {
                        current_version: env!("CARGO_PKG_VERSION").to_string(),
                        latest_version: env!("CARGO_PKG_VERSION").to_string(),
                        newer: false,
                        release_notes: None,
                        download_url: "https://github.com/monk81694-crypto/cinder/releases/latest".to_string(),
                        published_at: chrono::Utc::now(),
                        mandatory: false,
                    })
                }
            }
            Err(e) => {
                Err(anyhow::anyhow!("Update check failed: {}", e))
            }
        }
    }

    /// Download and install update
    pub async fn download_and_install_update(app_handle: tauri::AppHandle) -> Result<()> {
        let updater = app_handle.updater()?;

        match updater.check().await {
            Ok(update) => {
                if update.is_update_available() {
                    // Download with progress
                    let mut downloaded = 0u64;
                    let total = update.size;

                    update.download_and_install(
                        |chunk_length, content_length| {
                            downloaded += chunk_length as u64;
                            let progress = if total > 0 { downloaded as f32 / total as f32 } else { 0.0 };
                            // Emit progress event
                            let _ = app_handle.emit("update-progress", UpdateProgressEvent {
                                progress,
                                downloaded,
                                total,
                            });
                        },
                        || {
                            // Installation completed
                            let _ = app_handle.emit("update-downloaded", ());
                        },
                    ).await.map_err(|e| anyhow::anyhow!("Update download/install failed: {}", e))?;

                    info!("Update installed successfully, restarting...");

                    // Restart app
                    app_handle.restart();
                } else {
                    return Err(anyhow::anyhow!("No update available"));
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Update check failed: {}", e));
            }
        }
        Ok(())
    }

    /// Get current version
    pub fn get_current_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

/// Update available event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateAvailableEvent {
    pub version: String,
    pub notes: String,
    pub date: String,
    pub download_url: String,
}

/// Update progress event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateProgressEvent {
    pub progress: f32,
    pub downloaded: u64,
    pub total: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert!(semver::Version::parse("3.0.0").is_ok());
        assert!(semver::Version::parse("3.0.1").is_ok());
    }
}