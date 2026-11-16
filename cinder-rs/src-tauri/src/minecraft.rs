//! Minecraft version management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use reqwest::Client;
use zip::ZipArchive;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Minecraft version manager
pub struct MinecraftManager {
    client: Client,
    versions_cache: Arc<RwLock<Option<VersionManifest>>>,
    cache_path: PathBuf,
    game_dir: PathBuf,
}

impl MinecraftManager {
    pub fn new(game_dir: PathBuf, cache_path: PathBuf) -> Self {
        Self {
            client: Client::new(),
            versions_cache: Arc::new(RwLock::new(None)),
            cache_path,
            game_dir,
        }
    }

    /// Get version manifest from Mojang (with caching)
    pub async fn get_version_manifest(&self, force_refresh: bool) -> Result<VersionManifest> {
        // Check cache first
        if !force_refresh {
            if let Some(cached) = self.versions_cache.read().await.as_ref() {
                if cached.is_fresh() {
                    return Ok(cached.clone());
                }
            }
        }

        // Check disk cache
        let cache_file = self.cache_path.join("version_manifest.json");
        if !force_refresh && cache_file.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&cache_file).await {
                if let Ok(manifest) = serde_json::from_str::<VersionManifest>(&content) {
                    if manifest.is_fresh() {
                        *self.versions_cache.write().await = Some(manifest.clone());
                        return Ok(manifest);
                    }
                }
            }
        }

        // Fetch from Mojang
        let response = self.client
            .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .header("User-Agent", "Cinder/3.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch version manifest: {}", response.status()));
        }

        let manifest: VersionManifest = response.json().await?;

        // Cache in memory and on disk
        *self.versions_cache.write().await = Some(manifest.clone());
        if let Some(parent) = cache_file.parent() {
            tokio::fs::create_dir_all(parent).await.ok;
        }
        tokio::fs::write(&cache_file, serde_json::to_string_pretty(&manifest)?).await.ok;

        Ok(manifest)
    }

    /// List all versions with install status
    pub async fn list_versions(&self, force_refresh: bool) -> Result<Vec<MinecraftVersion>> {
        let manifest = self.get_version_manifest(force_refresh).await?;
        let installed = self.get_installed_versions().await?;

        let mut versions = Vec::new();
        for v in manifest.versions {
            let installed_info = installed.get(&v.id);
            versions.push(MinecraftVersion {
                id: v.id.clone(),
                version_type: v.version_type,
                url: v.url,
                time: v.time,
                release_time: v.release_time,
                sha1: v.sha1,
                compliant: true,
                installed: installed_info.is_some(),
                install_path: installed_info.map(|p| p.clone()),
                required_java_version: self.get_required_java_version(&v.id),
                loader_support: HashMap::new(),
            });
        }

        Ok(versions)
    }

    /// Get installed versions
    async fn get_installed_versions(&self) -> Result<HashMap<String, PathBuf>> {
        let mut installed = HashMap::new();
        let versions_dir = self.game_dir.join("versions");

        if versions_dir.exists() {
            for entry in std::fs::read_dir(&versions_dir)? {
                let entry = entry?;
                let version_dir = entry.path();
                if version_dir.is_dir() {
                    let version_id = version_dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    if !version_id.is_empty() {
                        installed.insert(version_id, version_dir);
                    }
                }
            }
        }

        Ok(installed)
    }

    /// Get required Java version for a Minecraft version
    fn get_required_java_version(&self, version: &str) -> u8 {
        if version.starts_with("1.") {
            let parts: Vec<&str> = version.split('.').collect();
            if parts.len() >= 2 {
                if let Ok(minor) = parts[1].parse::<u32>() {
                    if minor >= 21 { return 21; }
                    if minor >= 18 { return 17; }
                    if minor >= 17 { return 16; }
                }
            }
            8
        } else if version.starts_with("2") {
            21 // Snapshots like 24w10a
        } else {
            21
        }
    }

    /// Install a Minecraft version
    pub async fn install_version(&self, version_id: &str, progress_callback: Option<Box<dyn Fn(f32) + Send>>) -> Result<PathBuf> {
        let manifest = self.get_version_manifest(false).await?;

        let version_info = manifest.versions.iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| anyhow::anyhow!("Version not found: {}", version_id))?;

        // Fetch version details
        let version_detail: VersionDetail = self.client
            .get(&version_info.url)
            .header("User-Agent", "Cinder/3.0.0")
            .send()
            .await?
            .json()
            .await?;

        // Create version directory
        let version_dir = self.game_dir.join("versions").join(version_id);
        tokio::fs::create_dir_all(&version_dir).await?;

        // Save version.json
        let version_json = serde_json::to_string_pretty(&version_detail)?;
        tokio::fs::write(version_dir.join(format!("{}.json", version_id)), version_json).await?;

        // Download client jar
        if let Some(client) = &version_detail.downloads.client {
            let jar_path = version_dir.join(format!("{}.jar", version_id));
            self.download_file_with_progress(&client.url, &jar_path, client.sha1.as_deref(), progress_callback.as_deref()).await?;
        }

        // Download libraries
        for lib in &version_detail.libraries {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let lib_path = self.game_dir.join("libraries").join(&artifact.path);
                    tokio::fs::create_dir_all(lib_path.parent().unwrap()).await?;
                    self.download_file_with_progress(&artifact.url, &lib_path, artifact.sha1.as_deref(), None).await?;
                }
            }
        }

        // Download assets
        if let Some(asset_index) = &version_detail.asset_index {
            let asset_index_path = self.game_dir.join("assets/indexes").join(format!("{}.json", asset_index.id));
            tokio::fs::create_dir_all(asset_index_path.parent().unwrap()).await?;
            self.download_file_with_progress(&asset_index.url, &asset_index_path, asset_index.sha1.as_deref(), None).await?;

            // Parse asset index and download assets
            let asset_index_data: AssetIndex = self.client
                .get(&asset_index.url)
                .header("User-Agent", "Cinder/3.0.0")
                .send()
                .await?
                .json()
                .await?;

            let objects_dir = self.game_dir.join("assets/objects");
            tokio::fs::create_dir_all(&objects_dir).await?;

            for (_, asset) in asset_index_data.objects {
                let hash = asset.hash;
                let first_two = &hash[..2];
                let asset_path = objects_dir.join(first_two).join(&hash);
                if !asset_path.exists() {
                    let asset_url = format!("https://resources.download.minecraft.net/{}/{}", first_two, hash);
                    self.download_file_with_progress(&asset_url, &asset_path, Some(&hash), None).await?;
                }
            }
        }

        info!("Installed Minecraft version: {}", version_id);
        Ok(version_dir)
    }

    /// Download file with progress and SHA1 verification
    async fn download_file_with_progress(
        &self,
        url: &str,
        dest: &Path,
        expected_sha1: Option<&str>,
        progress_callback: Option<&dyn Fn(f32)>,
    ) -> Result<()> {
        if dest.exists() {
            if let Some(expected) = expected_sha1 {
                if let Ok(hash) = self.compute_sha1(dest).await {
                    if hash == expected {
                        return Ok(()); // Already downloaded and verified
                    }
                }
            } else {
                return Ok(()); // Exists and no verification needed
            }
        }

        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0) as f32;
        let mut downloaded = 0f32;

        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            downloaded += chunk.len() as f32;

            if let Some(cb) = progress_callback {
                if total_size > 0.0 {
                    cb(downloaded / total_size);
                }
            }
        }

        file.flush().await?;

        // Verify SHA1
        if let Some(expected) = expected_sha1 {
            let actual = self.compute_sha1(dest).await?;
            if actual != expected {
                tokio::fs::remove_file(dest).await.ok;
                return Err(anyhow::anyhow!("SHA1 mismatch: expected {}, got {}", expected, actual));
            }
        }

        Ok(())
    }

    /// Compute SHA1 of a file
    async fn compute_sha1(&self, path: &Path) -> Result<String> {
        let content = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }

    /// Uninstall a version
    pub async fn uninstall_version(&self, version_id: &str) -> Result<()> {
        let version_dir = self.game_dir.join("versions").join(version_id);
        if version_dir.exists() {
            tokio::fs::remove_dir_all(version_dir).await?;
            info!("Uninstalled version: {}", version_id);
        }
        Ok(())
    }

    /// Get version detail for launch
    pub async fn get_version_detail(&self, version_id: &str) -> Result<VersionDetail> {
        let manifest = self.get_version_manifest(false).await?;
        let version_info = manifest.versions.iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| anyhow::anyhow!("Version not found: {}", version_id))?;

        let detail: VersionDetail = self.client
            .get(&version_info.url)
            .header("User-Agent", "Cinder/3.0.0")
            .send()
            .await?
            .json()
            .await?;

        Ok(detail)
    }
}

/// Version manifest from Mojang
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "releaseTime")]
    pub release_time: chrono::DateTime<chrono::Utc>,
    pub sha1: Option<String>,
    #[serde(default)]
    pub compliant: bool,
}

impl VersionManifest {
    pub fn is_fresh(&self) -> bool {
        // Cache for 6 hours
        self.versions.first()
            .map(|v| chrono::Utc::now().signed_duration_since(v.time).num_hours() < 6)
            .unwrap_or(false)
    }
}

/// Version detail from Mojang
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VersionDetail {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: u32,
    #[serde(rename = "assets")]
    pub asset_index: Option<AssetIndexRef>,
    pub downloads: VersionDownloads,
    pub libraries: Vec<Library>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    #[serde(rename = "arguments")]
    pub arguments: Option<GameArguments>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VersionDownloads {
    pub client: Option<DownloadInfo>,
    pub server: Option<DownloadInfo>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DownloadInfo {
    pub sha1: Option<String>,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: Option<String>,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Library {
    pub name: String,
    #[serde(rename = "downloads")]
    pub downloads: Option<LibraryDownloads>,
    #[serde(rename = "rules")]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub extract: Option<ExtractRules>,
    #[serde(rename = "natives")]
    pub natives: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadInfo>,
    #[serde(rename = "classifiers")]
    pub classifiers: Option<HashMap<String, DownloadInfo>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ExtractRules {
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum GameArguments {
    Simple(Vec<String>),
    Complex { game: Vec<Argument>, jvm: Vec<Argument> },
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Argument {
    pub value: ArgumentValue,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    #[serde(rename = "map_to_resources")]
    pub map_to_resources: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let manifest_json = r#"{
            "latest": {"release": "1.21.4", "snapshot": "24w10a"},
            "versions": [
                {"id": "1.21.4", "type": "release", "url": "http://example.com", "time": "2024-01-01T00:00:00Z", "releaseTime": "2024-01-01T00:00:00Z", "sha1": "abc", "compliant": true}
            ]
        }"#;
        let manifest: VersionManifest = serde_json::from_str(manifest_json).unwrap();
        assert_eq!(manifest.latest.release, "1.21.4");
        assert_eq!(manifest.versions.len(), 1);
    }
}