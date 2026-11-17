//! Resource pack management for Cinder Launcher

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

/// Resource pack manager
pub struct ResourceManager {
    packs: Arc<RwLock<HashMap<Uuid, ResourcePack>>>,
    game_dir: PathBuf,
    client: Client,
}

impl ResourceManager {
    pub fn new(game_dir: PathBuf) -> Self {
        Self {
            packs: Arc::new(RwLock::new(HashMap::new())),
            game_dir,
            client: Client::new(),
        }
    }

    pub async fn load_packs(&self) -> Result<()> {
        let packs_dir = self.game_dir.join("resourcepacks");
        if !packs_dir.exists() {
            tokio::fs::create_dir_all(&packs_dir).await?;
            return Ok(());
        }

        let mut packs = self.packs.write().await;
        packs.clear();

        for entry in std::fs::read_dir(&packs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "zip").unwrap_or(false) {
                if let Ok(pack) = self.parse_resource_pack(&path).await {
                    packs.insert(pack.id, pack);
                }
            }
        }

        info!("Loaded {} resource packs", packs.len());
        Ok(())
    }

    async fn parse_resource_pack(&self, path: &Path) -> Result<ResourcePack> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.zip")
            .to_string();

        let file_size = std::fs::metadata(path)?.len();
        let file_hash = self.compute_sha256(path).await?;

        // Read pack.mcmeta
        let (name, description, pack_format, supported_formats, icon_path) =
            self.read_pack_mcmeta(path).await?;

        let id = Uuid::new_v4();

        Ok(ResourcePack {
            id,
            name,
            description,
            file_name,
            file_path: path.to_path_buf(),
            file_size,
            file_hash,
            pack_format,
            supported_formats,
            icon_path,
            author: None,
            version: None,
            enabled: true,
            priority: 0,
            source: PackSource::Local,
            installed_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn read_pack_mcmeta(&self, path: &Path) -> Result<(String, Option<String>, u32, Vec<u32>, Option<PathBuf>)> {
        use zip::ZipArchive;

        let zip = ZipArchive::new(path)?;

        let mut name = "Unknown Pack".to_string();
        let mut description = None;
        let mut pack_format = 1;
        let mut supported_formats = Vec::new();
        let mut icon_path = None;

        if let Ok(mut file) = zip.by_name("pack.mcmeta") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)?;

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(pack) = json.get("pack") {
                    name = pack.get("description").and_then(|v| v.as_str()).unwrap_or("Unknown Pack").to_string;
                    pack_format = pack.get("pack_format").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                    if let Some(formats) = pack.get("supported_formats").and_then(|v| v.as_array()) {
                        supported_formats = formats.iter().filter_map(|v| v.as_u64()).map(|v| v as u32).collect();
                    }
                }
            }
        }

        // Check for pack.png
        if zip.by_name("pack.png").is_ok() {
            // Extract icon to cache
            icon_path = Some(self.extract_icon(&zip).await?);
        }

        Ok((name, description, pack_format, supported_formats, icon_path))
    }

    async fn extract_icon(&self, zip: &ZipArchive) -> Result<PathBuf> {
        // Extract pack.png to cache
        let cache_dir = self.game_dir.join("cache").join("icons");
        tokio::fs::create_dir_all(&cache_dir).await?;

        let icon_path = cache_dir.join(format!("{}.png", Uuid::new_v4()));

        if let Ok(mut file) = zip.by_name("pack.png") {
            let mut content = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut content)?;
            tokio::fs::write(&icon_path, content).await?;
            return Ok(icon_path);
        }

        Err(anyhow::anyhow!("No pack.png found"))
    }

    async fn compute_sha256(&self, path: &Path) -> Result<String> {
        let content = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }

    pub async fn list_packs(&self) -> Vec<ResourcePack> {
        self.packs.read().await.values().cloned().collect()
    }

    pub async fn install_pack(&self, path: &Path, source: PackSource) -> Result<ResourcePack> {
        let packs_dir = self.game_dir.join("resourcepacks");
        tokio::fs::create_dir_all(&packs_dir).await?;

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("pack.zip")
            .to_string();

        let dest = packs_dir.join(&file_name);

        // Copy or download
        if path.starts_with("http") {
            self.download_file(path, &dest, None).await?;
        } else {
            tokio::fs::copy(path, &dest).await?;
        }

        let mut pack = self.parse_resource_pack(&dest).await?;
        pack.source = source;
        pack.enabled = true;

        self.packs.write().await.insert(pack.id, pack.clone());

        info!("Installed resource pack: {} ({})", pack.name, pack.id);
        Ok(pack)
    }

    async fn download_file(&self, url: &str, dest: &Path, _expected_sha1: Option<&str>) -> Result<()> {
        let response = self.client.get(url).send().await?;
        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        }
        file.flush().await?;
        Ok(())
    }

    pub async fn uninstall_pack(&self, pack_id: Uuid) -> Result<()> {
        let mut packs = self.packs.write().await;
        if let Some(pack) = packs.remove(&pack_id) {
            tokio::fs::remove_file(&pack.file_path).await.ok;
            info!("Uninstalled resource pack: {}", pack.name);
        }
        Ok(())
    }

    pub async fn toggle_pack(&self, pack_id: Uuid) -> Result<bool> {
        let mut packs = self.packs.write().await;
        if let Some(pack) = packs.get_mut(&pack_id) {
            pack.enabled = !pack.enabled;
            pack.updated_at = Utc::now();
            Ok(pack.enabled)
        } else {
            Err(anyhow::anyhow!("Pack not found"))
        }
    }

    pub async fn reorder_packs(&self, pack_ids: Vec<Uuid>) -> Result<()> {
        let mut packs = self.packs.write().await;
        for (index, id) in pack_ids.into_iter().enumerate() {
            if let Some(pack) = packs.get_mut(&id) {
                pack.priority = index as i32;
            }
        }
        Ok(())
    }

    pub async fn fetch_vanilla_tweaks(&self) -> Result<Vec<VanillaTweaksPack>> {
        let url = "https://api.vanillatweaks.net/v1/packs";
        let response = self.client.get(url)
            .header("User-Agent", "Cinder/3.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch Vanilla Tweaks: {}", response.status()));
        }

        let packs: Vec<VanillaTweaksPack> = response.json().await?;
        Ok(packs)
    }

    pub async fn install_vanilla_tweaks_pack(&self, pack_id: &str) -> Result<ResourcePack> {
        let packs = self.fetch_vanilla_tweaks().await?;
        let pack = packs.iter().find(|p| p.id == pack_id)
            .ok_or_else(|| anyhow::anyhow!("Vanilla Tweaks pack not found"))?;

        let packs_dir = self.game_dir.join("resourcepacks");
        tokio::fs::create_dir_all(&packs_dir).await?;

        let dest = packs_dir.join(format!("{}.zip", pack_id));
        self.download_file(&pack.download_url, &dest, None).await?;

        let mut resource_pack = self.parse_resource_pack(&dest).await?;
        resource_pack.source = PackSource::VanillaTweaks;
        resource_pack.enabled = true;

        self.packs.write().await.insert(resource_pack.id, resource_pack.clone());

        info!("Installed Vanilla Tweaks pack: {} ({})", pack.name, resource_pack.id);
        Ok(resource_pack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_format_validation() {
        assert!(ResourcePack::default().pack_format >= 1);
    }
}