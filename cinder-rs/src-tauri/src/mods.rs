//! Mod management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use reqwest::Client;
use zip::ZipArchive;
use sha2::{Digest, Sha256};

/// Mod manager for Modrinth and local mods
pub struct ModManager {
    client: Client,
    mods: Arc<RwLock<HashMap<Uuid, LocalMod>>>,
    game_dir: PathBuf,
    settings: Arc<RwLock<Settings>>,
}

impl ModManager {
    pub fn new(game_dir: PathBuf, settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            client: Client::new(),
            mods: Arc::new(RwLock::new(HashMap::new())),
            game_dir,
            settings,
        }
    }

    pub async fn load_mods(&self) -> Result<()> {
        let mods_dir = self.game_dir.join("mods");
        if !mods_dir.exists() {
            tokio::fs::create_dir_all(&mods_dir).await?;
            return Ok(());
        }

        let mut mods = self.mods.write().await;
        mods.clear();

        for entry in std::fs::read_dir(&mods_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "jar").unwrap_or(false) {
                if let Ok(mod_info) = self.parse_mod_jar(&path).await {
                    mods.insert(mod_info.id, mod_info);
                }
            }
        }

        info!("Loaded {} mods", mods.len());
        Ok(())
    }

    /// Parse a mod JAR to extract metadata
    async fn parse_mod_jar(&self, path: &Path) -> Result<LocalMod> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.jar")
            .to_string();

        let file_size = std::fs::metadata(path)?.len();
        let file_hash = self.compute_sha256(path).await?;

        // Try to read fabric.mod.json or META-INF/mods.toml
        let (name, version, loader, game_versions, description, dependencies) =
            self.extract_mod_metadata(path).await?;

        let id = Uuid::new_v4();
        let project_id = self.guess_project_id(&name, &version).unwrap_or_else(|| Uuid::new_v4().to_string());

        Ok(LocalMod {
            id,
            project_id,
            version_id: Uuid::new_v4().to_string(),
            name,
            version,
            file_name,
            file_path: path.to_path_buf(),
            file_size,
            file_hash,
            loader,
            game_versions,
            description,
            enabled: true,
            pinned_version: None,
            installed_at: Utc::now(),
            updated_at: Utc::now(),
            dependencies,
            compatible_loaders: vec![],
            source: ModSource::Local,
        })
    }

    /// Extract metadata from mod JAR
    async fn extract_mod_metadata(&self, path: &Path) -> Result<(String, String, ModLoader, Vec<String>, Option<String>, Vec<ModDependency>)> {
        use zip::ZipArchive;

        let zip = adm_zip::adm_zip::ZipArchive::new(path)?;
        let mut name = "Unknown Mod".to_string();
        let mut version = "1.0.0".to_string();
        let mut loader = ModLoader::Fabric;
        let mut game_versions = Vec::new();
        let mut description = None;
        let mut dependencies = Vec::new();

        // Try Fabric: fabric.mod.json
        if let Ok(mut file) = zip.by_name("fabric.mod.json") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                name = json.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Mod").to_string();
                version = json.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string;
                loader = ModLoader::Fabric;
                if let Some(depends) = json.get("depends").and_then(|v| v.as_object()) {
                    for (dep_id, dep_info) in depends {
                        dependencies.push(ModDependency {
                            project_id: dep_id.clone(),
                            version: dep_info.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            dependency_type: "required".to_string(),
                        });
                    }
                }
                if let Some(env) = json.get("environment").and_then(|v| v.as_str()) {
                    // client, server, or *
                }
                if let Some(custom) = json.get("custom").and_then(|v| v.as_object()) {
                    if let Some(mc) = custom.get("minecraft").and_then(|v| v.as_array()) {
                        game_versions = mc.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect();
                    }
                }
            return Ok((name, version, loader, game_versions, description, dependencies));
        }

        // Try Forge/NeoForge: META-INF/mods.toml
        if let Ok(mut file) = zip.by_name("META-INF/mods.toml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)?;
            // Parse TOML
            if let Ok(toml) = content.parse::<toml::Value>() {
                if let Some(mods) = toml.get("mods").and_then(|v| v.as_array()) {
                    if let Some(first) = mods.first() {
                        name = first.get("modId").and_then(|v| v.as_str()).unwrap_or("Unknown Mod").to_string();
                        version = first.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string;
                        description = first.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

                        // Detect loader from modLoader field
                        if let Some(mod_loader) = first.get("modLoader").and_then(|v| v.as_str()) {
                            if mod_loader.contains("forge") {
                                loader = ModLoader::Forge;
                            } else if mod_loader.contains("neoforge") {
                                loader = ModLoader::NeoForge;
                            }
                        }

                        if let Some(deps) = first.get("dependencies").and_then(|v| v.as_array()) {
                            for dep in deps {
                                if let Some(dep_id) = dep.get("modId").and_then(|v| v.as_str()) {
                                    dependencies.push(ModDependency {
                                        project_id: dep_id.to_string(),
                                        version: dep.get("versionRange").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                        dependency_type: dep.get("type").and_then(|v| v.as_str()).unwrap_or("required").to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            return Ok((name, version, loader, game_versions, description, dependencies));
        }

        // Try Quilt: quilt.mod.json
        if let Ok(mut file) = zip.by_name("quilt.mod.json") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                name = json.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Mod").to_string;
                version = json.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string;
                loader = ModLoader::Quilt;
                // Parse dependencies similar to Fabric
            }
            return Ok((name, version, loader, game_versions, description, dependencies));
        }

        // Fallback: use filename
        name = file_name.trim_end_matches(".jar").to_string;
        Ok((name, version, loader, game_versions, description, dependencies))
    }

    /// Guess Modrinth project ID from name/version
    fn guess_project_id(&self, name: &str, version: &str) -> Option<String> {
        // This would search Modrinth API
        None
    }

    /// Compute SHA256 of a file
    async fn compute_sha256(&self, path: &Path) -> Result<String> {
        let content = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }

    /// List all mods
    pub async fn list_mods(&self) -> Vec<LocalMod> {
        self.mods.read().await.values().cloned().collect()
    }

    /// Search Modrinth
    pub async fn search_modrinth(&self, query: &str, limit: usize) -> Result<Vec<ModrinthProject>> {
        let url = format!(
            "https://api.modrinth.com/v2/search?query={}&limit={}&facets=[[\"project_type:mod\"]]",
            urlencoding::encode(query),
            limit
        );

        let response = self.client.get(&url)
            .header("User-Agent", "Cinder/3.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Modrinth search failed: {}", response.status()));
        }

        let data: ModrinthSearchResponse = response.json().await?;
        Ok(data.hits)
    }

    /// Get Modrinth project details
    pub async fn get_modrinth_project(&self, project_id: &str) -> Result<ModrinthProject> {
        let url = format!("https://api.modrinth.com/v2/project/{}", project_id);
        let response = self.client.get(&url)
            .header("User-Agent", "Cinder/3.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch project: {}", response.status()));
        }

        Ok(response.json().await?)
    }

    /// Get project versions
    pub async fn get_project_versions(&self, project_id: &str, game_version: &str, loader: &ModLoader) -> Result<Vec<ModrinthVersion>> {
        let loader_str = loader.as_str();
        let url = format!(
            "https://api.modrinth.com/v2/project/{}/version?game_versions=[\"{}\"]&loaders=[\"{}\"]",
            project_id, game_version, loader_str
        );

        let response = self.client.get(&url)
            .header("User-Agent", "Cinder/3.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch versions: {}", response.status()));
        }

        Ok(response.json().await?)
    }

    /// Install a mod from Modrinth
    pub async fn install_mod(&self, project_id: &str, version_id: &str, profile_id: Option<Uuid>) -> Result<LocalMod> {
        // Get version details
        let url = format!("https://api.modrinth.com/v2/version/{}", version_id);
        let response = self.client.get(&url)
            .header("User-Agent", "Cinder/3.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch version: {}", response.status()));
        }

        let version: ModrinthVersion = response.json().await?;

        // Find primary file
        let file = version.files.iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or_else(|| anyhow::anyhow!("No files in version"))?;

        // Download
        let mods_dir = self.game_dir.join("mods");
        tokio::fs::create_dir_all(&mods_dir).await?;

        let dest_path = mods_dir.join(&file.filename);
        self.download_file(&file.url, &dest_path, Some(&file.hashes.sha1)).await?;

        // Parse the downloaded mod
        let mut local_mod = self.parse_mod_jar(&dest_path).await?;
        local_mod.project_id = version.project_id.clone();
        local_mod.version_id = version.id.clone();
        local_mod.source = ModSource::Modrinth;
        local_mod.enabled = true;

        // Save
        self.mods.write().await.insert(local_mod.id, local_mod.clone());

        // Add to profile if specified
        if let Some(profile_id) = profile_id {
            // self.profiles_manager.add_mod_to_profile(profile_id, local_mod.id).await?;
        }

        info!("Installed mod: {} ({})", local_mod.name, local_mod.version);
        Ok(local_mod)
    }

    /// Download file with verification
    async fn download_file(&self, url: &str, dest: &Path, expected_sha1: Option<&str>) -> Result<()> {
        if dest.exists() {
            if let Some(expected) = expected_sha1 {
                if let Ok(hash) = self.compute_sha1(dest).await {
                    if hash == expected {
                        return Ok(());
                    }
                }
            } else {
                return Ok(());
            }
        }

        let response = self.client.get(url).send().await?;
        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        }
        file.flush().await?;

        if let Some(expected) = expected_sha1 {
            let actual = self.compute_sha1(dest).await?;
            if actual != expected {
                tokio::fs::remove_file(dest).await.ok;
                return Err(anyhow::anyhow!("SHA1 mismatch"));
            }
        }

        Ok(())
    }

    async fn compute_sha1(&self, path: &Path) -> Result<String> {
        let content = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }

    /// Uninstall a mod
    pub async fn uninstall_mod(&self, mod_id: Uuid) -> Result<()> {
        let mut mods = self.mods.write().await;
        if let Some(mod_info) = mods.remove(&mod_id) {
            tokio::fs::remove_file(&mod_info.file_path).await.ok;
            info!("Uninstalled mod: {}", mod_info.name);
        }
        Ok(())
    }

    /// Toggle mod enabled state
    pub async fn toggle_mod(&self, mod_id: Uuid) -> Result<bool> {
        let mut mods = self.mods.write().await;
        if let Some(mod_info) = mods.get_mut(&mod_id) {
            mod_info.enabled = !mod_info.enabled;
            mod_info.updated_at = Utc::now();
            info!("Mod {} {}", mod_info.name, if mod_info.enabled { "enabled" } else { "disabled" });
            Ok(mod_info.enabled)
        } else {
            Err(anyhow::anyhow!("Mod not found"))
        }
    }

    /// Get mod dependencies
    pub async fn get_mod_dependencies(&self, mod_id: Uuid) -> Result<Vec<ModDependency>> {
        let mods = self.mods.read().await;
        if let Some(mod_info) = mods.get(&mod_id) {
            Ok(mod_info.dependencies.clone())
        } else {
            Err(anyhow::anyhow!("Mod not found"))
        }
    }

    /// Check for mod updates
    pub async fn check_mod_updates(&self) -> Result<Vec<ModUpdateInfo>> {
        let mods = self.mods.read().await;
        let mut updates = Vec::new();

        for mod_info in mods.values() {
            if mod_info.source == ModSource::Modrinth {
                if let Ok(versions) = self.get_project_versions(&mod_info.project_id, "", &mod_info.loader).await {
                    if let Some(latest) = versions.first() {
                        if latest.version_number != mod_info.version {
                            updates.push(ModUpdateInfo {
                                mod_id: mod_info.id,
                                current_version: mod_info.version.clone(),
                                latest_version: latest.version_number.clone(),
                                latest_version_id: latest.id.clone(),
                                changelog: latest.changelog.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(updates)
    }

    /// Update a mod
    pub async fn update_mod(&self, mod_id: Uuid) -> Result<LocalMod> {
        let mut mods = self.mods.write().await;
        let mod_info = mods.get(&mod_id).ok_or_else(|| anyhow::anyhow!("Mod not found"))?.clone();

        if mod_info.source != ModSource::Modrinth {
            return Err(anyhow::anyhow!("Only Modrinth mods can be auto-updated"));
        }

        let versions = self.get_project_versions(&mod_info.project_id, "", &mod_info.loader).await?;
        let latest = versions.first().ok_or_else(|| anyhow::anyhow!("No versions found"))?;

        // Download new version
        let file = latest.files.iter().find(|f| f.primary).or_else(|| latest.files.first())
            .ok_or_else(|| anyhow::anyhow!("No files in latest version"))?;

        let mods_dir = self.game_dir.join("mods");
        let dest_path = mods_dir.join(&file.filename);

        // Backup old file
        let backup_path = mods_dir.join(format!("{}.backup", mod_info.file_name));
        tokio::fs::rename(&mod_info.file_path, &backup_path).await.ok;

        self.download_file(&file.url, &dest_path, Some(&file.hashes.sha1)).await?;

        // Parse new mod
        let mut new_mod = self.parse_mod_jar(&dest_path).await?;
        new_mod.id = mod_info.id; // Keep same ID
        new_mod.project_id = mod_info.project_id;
        new_mod.version_id = latest.id.clone();
        new_mod.enabled = mod_info.enabled;
        new_mod.pinned_version = mod_info.pinned_version;
        new_mod.updated_at = Utc::now();

        // Remove backup
        tokio::fs::remove_file(backup_path).await.ok;

        mods.insert(mod_info.id, new_mod.clone());
        info!("Updated mod: {} -> {}", mod_info.version, new_mod.version);
        Ok(new_mod)
    }

    /// Update all mods
    pub async fn update_all_mods(&self) -> Result<Vec<LocalMod>> {
        let updates = self.check_mod_updates().await?;
        let mut updated = Vec::new();

        for update in updates {
            if update.mod_id != Uuid::nil() && update.latest_version != update.current_version {
                if let Ok(mod_info) = self.update_mod(update.mod_id).await {
                    updated.push(mod_info);
                }
            }
        }

        Ok(updated)
    }

    /// Pin mod version
    pub async fn pin_mod_version(&self, mod_id: Uuid, version: Option<String>) -> Result<()> {
        let mut mods = self.mods.write().await;
        if let Some(mod_info) = mods.get_mut(&mod_id) {
            mod_info.pinned_version = version;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Mod not found"))
        }
    }

    /// Start background updater
    pub async fn start_background_updater(app_handle: tauri::AppHandle) -> Result<()> {
        let interval = tokio::time::interval(tokio::time::Duration::from_secs(6 * 60 * 60)); // 6 hours

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                // Check for updates
                if let Err(e) = app_handle.emit("mod-updates-available", Vec::<ModUpdateInfo>::new()) {
                    tracing::warn!("Failed to emit mod updates event: {}", e);
                }
            }
        });

        Ok(())
    }
}

/// Mod update info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModUpdateInfo {
    pub mod_id: Uuid,
    pub current_version: String,
    pub latest_version: String,
    pub latest_version_id: String,
    pub changelog: Option<String>,
}

/// Modrinth search response
#[derive(Debug, Clone, serde::Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthProject>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_source_serialization() {
        let source = ModSource::Modrinth;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"modrinth\"");
    }
}