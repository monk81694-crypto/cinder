//! Configuration management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration manager with hot-reload support
pub struct ConfigManager {
    settings: Arc<RwLock<Settings>>,
    config_path: PathBuf,
    watcher: Option<notify::RecommendedWatcher>,
}

impl ConfigManager {
    pub async fn new(config_path: Option<PathBuf>) -> Result<Self> {
        let config_path = config_path.unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("cinder")
                .join("settings.toml")
        });

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let settings = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            Settings::default()
        };

        Ok(Self {
            settings: Arc::new(RwLock::new(settings)),
            config_path,
            watcher: None,
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

    pub async fn save(&self, settings: &Settings) -> Result<()> {
        let content = toml::to_string_pretty(settings)?;
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&self.config_path, content).await?;
        debug!("Settings saved to {:?}", self.config_path);
        Ok(())
    }

    pub async fn start_watching(&mut self) -> Result<()> {
        use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

        let config_path = self.config_path.clone();
        let settings = self.settings.clone();

        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        if event.paths.contains(&config_path) {
                            if let Ok(content) = std::fs::read_to_string(&config_path) {
                                if let Ok(new_settings) = toml::from_str::<Settings>(&content) {
                                    let settings = settings.clone();
                                    tokio::spawn(async move {
                                        *settings.write().await = new_settings;
                                        info!("Settings hot-reloaded from file");
                                    });
                                }
                            }
                        }
                    }
                }
            },
            Config::default(),
        )?;

        watcher.watch(config_path.parent().unwrap(), RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);
        info!("Started watching config file for changes");
        Ok(())
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

/// Validate settings on load
pub fn validate_settings(settings: &mut Settings) -> Result<()> {
    // Ensure memory values are reasonable
    settings.min_memory_mb = settings.min_memory_mb.clamp(512, 16384);
    settings.max_memory_mb = settings.max_memory_mb.clamp(1024, 65536);

    if settings.min_memory_mb > settings.max_memory_mb {
        std::mem::swap(&mut settings.min_memory_mb, &mut settings.max_memory_mb);
    }

    // Ensure window dimensions are reasonable
    settings.window_width = settings.window_width.clamp(640, 3840);
    settings.window_height = settings.window_height.clamp(480, 2160);

    // Ensure launch timeout is reasonable
    settings.launch_timeout_sec = settings.launch_timeout_sec.clamp(10, 600);

    // Validate game directory
    if !settings.game_directory.is_absolute() {
        if let Some(data_dir) = dirs::data_dir() {
            settings.game_directory = data_dir.join(".cinder-minecraft");
        }
    }

    Ok(())
}

/// Get default JVM arguments for a version
pub fn default_jvm_args(java_version: u8, preset: &JvmPreset) -> Vec<String> {
    let mut args = vec![
        "-XX:+UnlockExperimentalVMOptions".to_string(),
        "-Dfml.earlyLoadingIgnore=net.minecraftforge.fml.loading.FMLLoader".to_string(),
    ];

    // Add preset-specific args
    args.extend(preset.args(java_version));

    // Common Minecraft JVM args
    args.extend(vec![
        "-Dminecraft.launcher.brand=Cinder".to_string(),
        "-Dminecraft.launcher.version=3.0.0".to_string(),
        "-Dlog4j2.formatMsgNoLookups=true".to_string(),
    ]);

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_settings() {
        let mut settings = Settings::default();
        settings.min_memory_mb = 100;
        settings.max_memory_mb = 50;
        validate_settings(&mut settings).unwrap();
        assert!(settings.min_memory_mb >= 512);
        assert!(settings.max_memory_mb >= settings.min_memory_mb);
    }

    #[test]
    fn test_jvm_preset_args() {
        let balanced = JvmPreset::Balanced.args(21);
        assert!(balanced.iter().any(|a| a.contains("G1GC")));

        let latency = JvmPreset::Latency.args(21);
        assert!(latency.iter().any(|a| a.contains("ZGC")));

        let latency_old = JvmPreset::Latency.args(17);
        assert!(!latency_old.iter().any(|a| a.contains("ZGC")));
    }
}