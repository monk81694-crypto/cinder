//! Shader pack management for Cinder Launcher

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

/// Shader pack manager
pub struct ShaderManager {
    packs: Arc<RwLock<HashMap<Uuid, ShaderPack>>>,
    game_dir: PathBuf,
    client: Client,
}

impl ShaderManager {
    pub fn new(game_dir: PathBuf) -> Self {
        Self {
            packs: Arc::new(RwLock::new(HashMap::new())),
            game_dir,
            client: Client::new(),
        }
    }

    pub async fn load_packs(&self) -> Result<()> {
        let packs_dir = self.game_dir.join("shaderpacks");
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
                if let Ok(pack) = self.parse_shader_pack(&path).await {
                    packs.insert(pack.id, pack);
                }
            }
        }

        info!("Loaded {} shader packs", packs.len());
        Ok(())
    }

    async fn parse_shader_pack(&self, path: &Path) -> Result<ShaderPack> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.zip")
            .to_string();

        let file_size = std::fs::metadata(path)?.len();
        let file_hash = self.compute_sha256(path).await?;

        // Try to detect shader type and read metadata
        let (name, description, shader_type, preview_image, compatible_loaders) =
            self.detect_shader_type(path).await?;

        let id = Uuid::new_v4();

        Ok(ShaderPack {
            id,
            name,
            description,
            file_name,
            file_path: path.to_path_buf(),
            file_size,
            file_hash,
            shader_type,
            preview_image,
            author: None,
            version: None,
            enabled: true,
            compatible_loaders,
            installed_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn detect_shader_type(&self, path: &Path) -> Result<(String, Option<String>, ShaderType, Option<PathBuf>, Vec<ModLoader>)> {
        use zip::ZipArchive;

        let zip = adm_zip::adm_zip::ZipArchive::new(path)?;
        let mut name = "Unknown Shader".to_string();
        let mut description = None;
        let mut shader_type = ShaderType::Universal;
        let mut preview_image = None;
        let mut compatible_loaders = vec![ModLoader::Fabric, ModLoader::Forge];

        // Check for iris.json (Iris shaders)
        if let Ok(mut file) = zip.by_name("iris.json") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                name = json.get("name").and_then(|v| v.as_str()).unwrap_or("Iris Shader").to_string;
                description = json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                shader_type = ShaderType::Iris;
                compatible_loaders = vec![ModLoader::Fabric, ModLoader::Quilt];
            }
        }

        // Check for optifine.json or shaderpack manifest
        if let Ok(mut file) = zip.by_name("shaderpack.json") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                name = json.get("name").and_then(|v| v.as_str()).unwrap_or(name).to_string;
                description = json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                if shader_type == ShaderType::Universal {
                    shader_type = ShaderType::OptiFine;
                    compatible_loaders = vec![ModLoader::Forge, ModLoader::NeoForge];
                }
            }
        }

        // Check for shaderpack manifest (older format)
        if let Ok(mut file) = zip.by_name("shaderpack.properties") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)?;
            for line in content.lines() {
                if line.starts_with("name=") {
                    name = line[5..].to_string;
                } else if line.starts_with("description=") {
                    description = Some(line[12..].to_string);
                }
            }
        }

        // Try to extract preview image
        if let Ok(mut file) = zip.by_name("preview.png") {
            let mut content = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut content)?;
            let cache_dir = self.game_dir.join("cache").join("shader_previews");
            tokio::fs::create_dir_all(&cache_dir).await?;
            let preview_path = cache_dir.join(format!("{}.png", Uuid::new_v4()));
            tokio::fs::write(&preview_path, content).await?;
            preview_image = Some(preview_path);
        }

        // If name is still unknown, use filename
        if name == "Unknown Shader" {
            name = path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown Shader")
                .to_string;
        }

        Ok((name, description, shader_type, preview_image, compatible_loaders))
    }

    async fn compute_sha256(&self, path: &Path) -> Result<String> {
        let content = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }

    pub async fn list_packs(&self) -> Vec<ShaderPack> {
        self.packs.read().await.values().cloned().collect()
    }

    pub async fn install_pack(&self, path: &Path) -> Result<ShaderPack> {
        let packs_dir = self.game_dir.join("shaderpacks");
        tokio::fs::create_dir_all(&packs_dir).await?;

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("shaderpack.zip")
            .to_string;

        let dest = packs_dir.join(&file_name);

        if path.starts_with("http") {
            self.download_file(path, &dest).await?;
        } else {
            tokio::fs::copy(path, &dest).await?;
        }

        let mut pack = self.parse_shader_pack(&dest).await?;
        pack.enabled = true;

        self.packs.write().await.insert(pack.id, pack.clone());

        info!("Installed shader pack: {} ({})", pack.name, pack.id);
        Ok(pack)
    }

    async fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
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
            info!("Uninstalled shader pack: {}", pack.name);
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

    pub async fn preview_shader_pack(&self, pack_id: Uuid) -> Result<Option<PathBuf>> {
        let packs = self.packs.read().await;
        Ok(packs.get(&pack_id).and_then(|p| p.preview_image.clone()))
    }

    pub async fn detect_shader_loader(&self) -> Result<Vec<ModLoader>> {
        // Check for Iris/Quilt
        let mods_dir = self.game_dir.join("mods");
        let mut loaders = Vec::new();

        if mods_dir.exists() {
            for entry in std::fs::read_dir(&mods_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "jar").unwrap_or(false) {
                    if let Ok(name) = self.get_mod_name(&path).await {
                        if name.to_lowercase().contains("iris") {
                            loaders.push(ModLoader::Fabric);
                        } else if name.to_lowercase().contains("optifine") {
                            loaders.push(ModLoader::Forge);
                        }
                    }
                }
            }
        }

        if loaders.is_empty() {
            loaders.push(ModLoader::Fabric);
            loaders.push(ModLoader::Forge);
        }

        Ok(loaders)
    }

    async fn get_mod_name(&self, path: &Path) -> Result<String> {
        use zip::ZipArchive;
        let zip = adm_zip::adm_zip::ZipArchive::new(path)?;
        if let Ok(mut file) = zip.by_name("fabric.mod.json") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                return Ok(json.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string);
            }
        }
        Ok(path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_type_serialization() {
        let iris = ShaderType::Iris;
        let json = serde_json::to_string(&iris).unwrap();
        assert_eq!(json, "\"iris\"");
    }
}