//! Skin management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use reqwest::Client;
use sha2::{Digest, Sha256};
use image::GenericImageView;

/// Skin manager
pub struct SkinManager {
    skins: Arc<RwLock<HashMap<Uuid, Skin>>>,
    skins_dir: PathBuf,
    client: Client,
}

impl SkinManager {
    pub fn new(skins_dir: PathBuf) -> Self {
        Self {
            skins: Arc::new(RwLock::new(HashMap::new())),
            skins_dir,
            client: Client::new(),
        }
    }

    pub async fn load_skins(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.skins_dir).await?;

        let mut skins = self.skins.write().await;
        skins.clear();

        for entry in std::fs::read_dir(&self.skins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "png").unwrap_or(false) {
                if let Ok(skin) = self.parse_skin(&path).await {
                    skins.insert(skin.id, skin);
                }
            }
        }

        info!("Loaded {} skins", skins.len());
        Ok(())
    }

    async fn parse_skin(&self, path: &Path) -> Result<Skin> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("skin.png")
            .to_string();

        let file_size = std::fs::metadata(path)?.len();
        let dimensions = image::io::Reader::open(path)?.decode()?.dimensions();

        let variant = if dimensions.1 == 32 { SkinVariant::Classic } else { SkinVariant::Slim };

        let id = Uuid::new_v4();
        let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("skin").to_string();

        Ok(Skin {
            id,
            name,
            username: name.clone(),
            file_path: path.to_path_buf(),
            texture_url: None,
            variant,
            preview_url: None,
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn list_skins(&self) -> Vec<Skin> {
        self.skins.read().await.values().cloned().collect()
    }

    /// Fetch skin by username from Minotar
    pub async fn get_skin(&self, username: &str) -> Result<Skin> {
        let url = format!("https://api.minotar.net/skin/{}", urlencoding::encode(username));

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch skin for {}", username));
        }

        let bytes = response.bytes().await?;

        // Save to cache
        tokio::fs::create_dir_all(&self.skins_dir).await?;
        let cache_path = self.skins_dir.join(format!("{}.png", username));
        tokio::fs::write(&cache_path, &bytes).await?;

        let dimensions = image::load_from_memory(&bytes)?.dimensions();
        let variant = if dimensions.1 == 32 { SkinVariant::Classic } else { SkinVariant::Slim };

        let id = Uuid::new_v4();

        let skin = Skin {
            id,
            name: username.to_string(),
            username: username.to_string(),
            file_path: cache_path.clone(),
            texture_url: Some(format!("https://minotar.net/skin/{}", urlencoding::encode(username))),
            variant,
            preview_url: Some(format!("https://minotar.net/armor/body/{}/100.png", urlencoding::encode(username))),
            created_at: chrono::Utc::now(),
        };

        self.skins.write().await.insert(id, skin.clone());
        Ok(skin)
    }

    /// Upload custom skin
    pub async fn upload_skin(&self, path: &Path, name: &str) -> Result<Skin> {
        tokio::fs::create_dir_all(&self.skins_dir).await?;

        let dest = self.skins_dir.join(format!("{}.png", name));
        tokio::fs::copy(path, &dest).await?;

        let dimensions = image::io::Reader::open(&dest)?.decode()?.dimensions();
        let variant = if dimensions.1 == 32 { SkinVariant::Classic } else { SkinVariant::Slim };

        let id = Uuid::new_v4();

        let skin = Skin {
            id,
            name: name.to_string(),
            username: name.to_string(),
            file_path: dest,
            texture_url: None,
            variant,
            preview_url: None,
            created_at: chrono::Utc::now(),
        };

        self.skins.write().await.insert(id, skin.clone());
        Ok(skin)
    }

    /// Save current skin preview
    pub async fn save_skin(&self, skin_id: Uuid, dest_path: &Path) -> Result<()> {
        let skins = self.skins.read().await;
        let skin = skins.get(&skin_id).ok_or_else(|| anyhow::anyhow!("Skin not found"))?;
        tokio::fs::copy(&skin.file_path, dest_path).await?;
        Ok(())
    }

    /// Get skin preview URL
    pub async fn get_preview(&self, skin_id: Uuid) -> Result<Option<String>> {
        let skins = self.skins.read().await;
        Ok(skins.get(&skin_id).and_then(|s| s.preview_url.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_variant_serialization() {
        let classic = SkinVariant::Classic;
        let json = serde_json::to_string(&classic).unwrap();
        assert_eq!(json, "\"classic\"");
    }
}