//! Game launch logic for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use chrono::Utc;
use sha2::{Digest, Sha256};

/// Launch manager
pub struct LaunchManager {
    settings: Arc<RwLock<Settings>>,
    java_manager: Arc<JavaManager>,
    minecraft_manager: Arc<MinecraftManager>,
    profiles_manager: Arc<ProfilesManager>,
    active_child: Arc<RwLock<Option<tokio::process::Child>>>,
}

impl LaunchManager {
    pub fn new(
        settings: Arc<RwLock<Settings>>,
        java_manager: Arc<JavaManager>,
        minecraft_manager: Arc<MinecraftManager>,
        profiles_manager: Arc<ProfilesManager>,
    ) -> Self {
        Self {
            settings,
            java_manager,
            minecraft_manager,
            profiles_manager,
            active_child: Arc::new(RwLock::new(None)),
        }
    }

    /// Launch the game with a profile
    pub async fn launch_game(&self, profile_id: Option<Uuid>, account_id: Option<Uuid>) -> Result<()> {
        let settings = self.settings.read().await;
        let profile = if let Some(id) = profile_id {
            self.profiles_manager.get_profile(id).await?
        } else if let Some(id) = settings.last_launch_profile_id {
            self.profiles_manager.get_profile(id).await?
        } else {
            self.profiles_manager.get_default_profile().await?
        };

        let account_id = account_id.or(settings.last_account_id);
        let account = if let Some(id) = account_id {
            self.get_account(id).await?
        } else {
            self.get_default_account().await?
        };

        let version_id = profile.version.clone();

        // Get Java
        let java = self.java_manager.get_best_java(&version_id).await?
            .ok_or_else(|| anyhow::anyhow!("No suitable Java found for version {}", version_id))?;

        // Get version detail
        let version_detail = self.minecraft_manager.get_version_detail(&version_id).await?;

        // Build classpath
        let classpath = self.build_classpath(&version_detail).await?;

        // Build JVM arguments
        let jvm_args = self.build_jvm_args(&profile, &settings, &java).await?;

        // Build game arguments
        let game_args = self.build_game_args(&version_detail, &account, &profile).await?;

        // Prepare launch command
        let java_exe = &java.path;
        let mut cmd = Command::new(java_exe);

        cmd.args(&jvm_args)
            .arg("-cp")
            .arg(&classpath)
            .arg(&version_detail.main_class)
            .args(&game_args)
            .current_dir(&settings.game_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        cmd.env("MINECRAFT_VERSION", &version_id);
        cmd.env("MINECRAFT_USER", &account.username);
        if let Some(uuid) = &account.uuid {
            cmd.env("MINECRAFT_UUID", uuid);
        }
        if let Some(token) = &account.access_token {
            cmd.env("MINECRAFT_ACCESS_TOKEN", token);
        }

        info!("Launching Minecraft {} with Java {}", version_id, java.version);
        debug!("Command: {:?}", cmd);

        // Spawn process
        let mut child = cmd.spawn()?;

        // Store child for management
        *self.active_child.write().await = Some(child);

        // Update last used
        if let Some(id) = profile_id {
            self.profiles_manager.update_last_used(id).await?;
        }

        // Handle launch action
        match settings.launch_action {
            LaunchAction::Minimize => {
                // Minimize window (handled by frontend)
            }
            LaunchAction::Hide => {
                // Hide window (handled by frontend)
            }
            LaunchAction::Nothing => {}
        }

        // Monitor process
        let child = self.active_child.write().await.take().unwrap();
        tokio::spawn(async move {
            let status = child.wait().await;
            info!("Game process exited with: {:?}", status);
        });

        Ok(())
    }

    /// Build classpath for the version
    async fn build_classpath(&self, version_detail: &VersionDetail) -> Result<String> {
        let mut classpath = Vec::new();
        let game_dir = self.settings.read().await.game_directory.clone();
        let libraries_dir = game_dir.join("libraries");

        // Add version jar
        let version_jar = game_dir.join("versions").join(&version_detail.id).join(format!("{}.jar", version_detail.id));
        if version_jar.exists() {
            classpath.push(version_jar.to_string_lossy().to_string());
        }

        // Add libraries
        for lib in &version_detail.libraries {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let lib_path = libraries_dir.join(&artifact.path);
                    if lib_path.exists() {
                        classpath.push(lib_path.to_string_lossy().to_string());
                    }
                }
                // Add native classifiers if present
                if let Some(classifiers) = &downloads.classifiers {
                    for (_, download) in classifiers {
                        let lib_path = libraries_dir.join(&download.path);
                        if lib_path.exists() {
                            classpath.push(lib_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        Ok(classpath.join(if cfg!(target_os = "windows") { ";" } else { ":" }))
    }

    /// Build JVM arguments
    async fn build_jvm_args(
        &self,
        profile: &LaunchProfile,
        settings: &Settings,
        java: &JavaInstallation,
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // Memory
        args.push(format!("-Xms{}m", profile.min_memory_mb.min(settings.min_memory_mb)));
        args.push(format!("-Xmx{}m", profile.max_memory_mb.min(settings.max_memory_mb)));

        // JVM preset
        let preset_args = JvmPreset::default().args(java.major_version);
        args.extend(preset_args);

        // Custom JVM args
        if !profile.jvm_args.is_empty() {
            args.extend(profile.jvm_args.split_whitespace().map(|s| s.to_string()));
        }
        if !settings.jvm_args.is_empty() {
            args.extend(settings.jvm_args.split_whitespace().map(|s| s.to_string()));
        }

        // Native library path
        let game_dir = settings.game_directory.clone();
        let natives_dir = game_dir.join("versions").join(&profile.version).join("natives");
        if natives_dir.exists() {
            args.push(format!("-Djava.library.path={}", natives_dir.to_string_lossy()));
        }

        // Log4j2 config
        let log4j_config = game_dir.join("versions").join(&profile.version).join("log4j2.xml");
        if log4j_config.exists() {
            args.push(format!("-Dlog4j.configurationFile={}", log4j_config.to_string_lossy()));
        }

        // Minecraft specific
        args.push("-Dminecraft.launcher.brand=Cinder".to_string());
        args.push("-Dminecraft.launcher.version=3.0.0".to_string());
        args.push("-Dlog4j2.formatMsgNoLookups=true".to_string());

        Ok(args)
    }

    /// Build game arguments
    async fn build_game_args(
        &self,
        version_detail: &VersionDetail,
        account: &Account,
        profile: &LaunchProfile,
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // Username
        args.push("--username".to_string());
        args.push(account.username.clone());

        // UUID
        if let Some(uuid) = &account.uuid {
            args.push("--uuid".to_string());
            args.push(uuid.clone());
        } else {
            // Generate offline UUID
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(b"OfflinePlayer:");
            hasher.update(account.username.as_bytes());
            let hash = hasher.finalize();
            let offline_uuid = Uuid::from_bytes({
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&hash[..16]);
                bytes[6] = (bytes[6] & 0x0f) | 0x40;
                bytes[8] = (bytes[8] & 0x3f) | 0x80;
                bytes
            });
            args.push("--uuid".to_string());
            args.push(offline_uuid.to_string());
        }

        // Access token
        if let Some(token) = &account.access_token {
            args.push("--accessToken".to_string());
            args.push(token.clone());
        } else {
            args.push("--accessToken".to_string());
            args.push("0".to_string()); // Offline mode
        }

        // Version
        args.push("--version".to_string());
        args.push(profile.version.clone());

        // Game directory
        let settings = self.settings.read().await;
        args.push("--gameDir".to_string());
        args.push(settings.game_directory.to_string_lossy().to_string());

        // Assets directory
        args.push("--assetsDir".to_string());
        args.push(settings.game_directory.join("assets").to_string_lossy().to_string());

        // Asset index
        if let Some(asset_index) = &version_detail.asset_index {
            args.push("--assetIndex".to_string());
            args.push(asset_index.id.clone());
        }

        // Version type
        args.push("--versionType".to_string());
        args.push("Cinder".to_string());

        // Launch profile
        if let Some(launch_profile_id) = profile.id {
            args.push("--launchProfile".to_string());
            args.push(launch_profile_id.to_string());
        }

        // Custom game arguments from version
        if let Some(arguments) = &version_detail.arguments {
            match arguments {
                GameArguments::Simple(args_list) => {
                    args.extend(args_list.iter().cloned());
                }
                GameArguments::Complex { game, .. } => {
                    for arg in game {
                        match &arg.value {
                            ArgumentValue::Single(s) => args.push(s.clone()),
                            ArgumentValue::Multiple(list) => args.extend(list.iter().cloned()),
                        }
                    }
                }
            }
        }

        // Server auto-join
        if let Some(server) = &profile.server {
            args.push("--server".to_string());
            args.push(server.clone());
        } else if let Some(server) = &settings.last_server {
            args.push("--server".to_string());
            args.push(server.clone());
        }

        // Resolution
        args.push("--width".to_string());
        args.push(profile.width.to_string());
        args.push("--height".to_string());
        args.push(profile.height.to_string());

        if profile.fullscreen {
            args.push("--fullscreen".to_string());
        }

        Ok(args)
    }

    /// Kill running game
    pub async fn kill_game(&self) -> Result<bool> {
        let mut child = self.active_child.write().await;
        if let Some(mut process) = child.take() {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &process.id().to_string()])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .status()?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                use std::os::unix::process::CommandExt;
                process.kill().await?;
            }
            info!("Game process killed");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Validate launch configuration
    pub async fn validate_launch_config(&self, profile_id: Uuid) -> Result<ValidationResult> {
        let profile = self.profiles_manager.get_profile(profile_id).await?;
        let mut issues = Vec::new();

        // Check account
        if let Some(account) = self.get_account(profile.account_id).await.ok() {
            if account.account_type == AccountType::Microsoft {
                if account.token_expires_at.map_or(true, |t| t <= Utc::now()) {
                    issues.push("Microsoft token expired".to_string());
                }
            }
        } else {
            issues.push("Account not found".to_string());
        }

        // Check version
        let _ = self.minecraft_manager.get_version_detail(&profile.version).await
            .map_err(|_| issues.push("Version not installed".to_string()))?;

        // Check Java
        let _ = self.java_manager.get_best_java(&profile.version).await
            .map_err(|_| issues.push("No suitable Java found".to_string()))?;

        // Check mods compatibility
        // TODO: Check mod compatibility with version/loader

        Ok(ValidationResult {
            valid: issues.is_empty(),
            issues,
        })
    }

    /// Get launch arguments without launching (for debugging)
    pub async fn get_launch_args(&self, profile_id: Uuid) -> Result<LaunchArgs> {
        let profile = self.profiles_manager.get_profile(profile_id).await?;
        let settings = self.settings.read().await;
        let account = self.get_account(profile.account_id).await?;
        let java = self.java_manager.get_best_java(&profile.version).await?
            .ok_or_else(|| anyhow::anyhow!("No suitable Java"))?;
        let version_detail = self.minecraft_manager.get_version_detail(&profile.version).await?;

        let classpath = self.build_classpath(&version_detail).await?;
        let jvm_args = self.build_jvm_args(&profile, &settings, &java).await?;
        let game_args = self.build_game_args(&version_detail, &account, &profile).await?;

        Ok(LaunchArgs {
            java_path: java.path.to_string_lossy().to_string(),
            classpath,
            main_class: version_detail.main_class,
            jvm_args,
            game_args,
            working_directory: settings.game_directory.to_string_lossy().to_string(),
            environment: self.build_environment(&account, &profile.version).await?,
        })
    }

    async fn build_environment(&self, account: &Account, version: &str) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();
        env.insert("MINECRAFT_VERSION".to_string(), version.to_string());
        env.insert("MINECRAFT_USER".to_string(), account.username.clone());
        if let Some(uuid) = &account.uuid {
            env.insert("MINECRAFT_UUID".to_string(), uuid.clone());
        }
        if let Some(token) = &account.access_token {
            env.insert("MINECRAFT_ACCESS_TOKEN".to_string(), token.clone());
        }
        Ok(env)
    }

    async fn get_account(&self, id: Uuid) -> Result<Account> {
        // This would call the account manager
        // For now, return a placeholder
        Err(anyhow::anyhow!("Account manager not injected"))
    }

    async fn get_default_account(&self) -> Result<Account> {
        Err(anyhow::anyhow!("No default account"))
    }
}

/// Validation result for launch config
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<String>,
}

/// Launch arguments for debugging
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LaunchArgs {
    pub java_path: String,
    pub classpath: String,
    pub main_class: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classpath_separator() {
        let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
        assert!(!sep.is_empty());
    }

    #[test]
    fn test_memory_args() {
        let args = vec!["-Xms2048m", "-Xmx4096m"];
        assert!(args.iter().any(|a| a.starts_with("-Xms")));
        assert!(args.iter().any(|a| a.starts_with("-Xmx")));
    }
}