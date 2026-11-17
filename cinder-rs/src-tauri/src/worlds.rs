//! World management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use zip::{ZipArchive, ZipWriter, write::FileOptions};
use sha2::{Digest, Sha256};

/// World manager
pub struct WorldManager {
    worlds: Arc<RwLock<HashMap<Uuid, World>>>,
    game_dir: PathBuf,
}

impl WorldManager {
    pub fn new(game_dir: PathBuf) -> Self {
        Self {
            worlds: Arc::new(RwLock::new(HashMap::new())),
            game_dir,
        }
    }

    pub async fn load_worlds(&self) -> Result<()> {
        let saves_dir = self.game_dir.join("saves");
        if !saves_dir.exists() {
            tokio::fs::create_dir_all(&saves_dir).await?;
            return Ok(());
        }

        let mut worlds = self.worlds.write().await;
        worlds.clear();

        for entry in std::fs::read_dir(&saves_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(world) = self.parse_world(&path).await {
                    worlds.insert(world.id, world);
                }
            }
        }

        info!("Loaded {} worlds", worlds.len());
        Ok(())
    }

    async fn parse_world(&self, path: &Path) -> Result<World> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown World")
            .to_string;

        let metadata = std::fs::metadata(path)?;
        let size_bytes = self.calculate_dir_size(path).await?;
        let last_played = metadata.modified().ok()
            .and_then(|t| chrono::DateTime::<chrono::Utc>::from_std(t).ok())
            .unwrap_or_else(chrono::Utc::now);

        let level_dat_path = path.join("level.dat");
        let level_dat = if level_dat_path.exists() {
            Some(self.parse_level_dat(&level_dat_path).await?)
        } else {
            None
        };

        let icon_path = path.join("icon.png");
        let icon_path = if icon_path.exists() { Some(icon_path) } else { None };

        let created_at = metadata.created().ok()
            .and_then(|t| chrono::DateTime::<chrono::Utc>::from_std(t).ok())
            .unwrap_or_else(chrono::Utc::now);

        let id = Uuid::new_v4();

        Ok(World {
            id,
            name,
            path: path.to_path_buf(),
            size_bytes,
            last_played,
            created_at,
            version: level_dat.as_ref().and_then(|ld| Some(ld.version.name.clone())),
            game_mode: level_dat.as_ref().map(|ld| ld.level_dat.game_type.to_string()),
            difficulty: level_dat.as_ref().map(|ld| ld.level_dat.difficulty.to_string()),
            hardcore: level_dat.map(|ld| ld.level_dat.hardcore).unwrap_or(false),
            level_dat: None,
            icon_path,
        })
    }

    async fn calculate_dir_size(&self, path: &Path) -> Result<u64> {
        let mut total = 0u64;
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                total += metadata.len();
            }
        }
        Ok(total)
    }

    async fn parse_level_dat(&self, path: &Path) -> Result<LevelDatWrapper> {
        // This would parse the level.dat NBT file
        // For now, return a placeholder
        Ok(LevelDatWrapper {
            level_dat: LevelDat {
                game_type: 0,
                difficulty: 0,
                hardcore: false,
                seed: 0,
                version: VersionInfo { name: "Unknown".to_string(), protocol: 0 },
            },
            version: VersionInfo { name: "Unknown".to_string(), protocol: 0 },
        })
    }

    pub async fn list_worlds(&self) -> Vec<World> {
        self.worlds.read().await.values().cloned().collect()
    }

    /// Backup a world
    pub async fn backup_world(&self, world_id: Uuid) -> Result<PathBuf> {
        let world = self.worlds.read().await.get(&world_id).cloned()
            .ok_or_else(|| anyhow::anyhow!("World not found"))?;
        drop(self.worlds.read().await);

        let backups_dir = self.game_dir.join("backups");
        tokio::fs::create_dir_all(&backups_dir).await?;

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_name = format!("{}-{}.zip", world.name.replace(' ', "_"), timestamp);
        let backup_path = backups_dir.join(&backup_name);

        // Create zip
        self.create_zip(&world.path, &backup_path).await?;

        info!("Backed up world: {} -> {}", world.name, backup_path.display());
        Ok(backup_path)
    }

    async fn create_zip(&self, source: &Path, dest: &Path) -> Result<()> {
        use zip::{ZipArchive, ZipWriter, write::FileOptions};
        let mut zip = ZipWriter::new(std::fs::File::create(dest)?);

        let walker = walkdir::WalkDir::new(source);
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative = path.strip_prefix(source).unwrap();
            let name = relative.to_string_lossy().to_string;

            if entry.path().is_file() {
                zip.start_file(name, CompressionMethod::Deflated)?;
                let content = tokio::fs::read(path).await?;
                std::io::Write::write_all(&mut zip, &content)?;
            } else {
                zip.add_directory(name, CompressionMethod::Stored)?;
            }
        }

        zip.finish()?;
        Ok(())
    }

    /// Restore a world from backup
    pub async fn restore_world(&self, world_id: Uuid, backup_path: &Path) -> Result<()> {
        let world = self.worlds.read().await.get(&world_id).cloned()
            .ok_or_else(|| anyhow::anyhow!("World not found"))?;
        drop(self.worlds.read().await);

        // Backup current world first
        let current_backup = self.game_dir.join("saves").join(format!("{}.bak-{}", world.name, chrono::Utc::now().format("%Y%m%d-%H%M%S")));
        self.create_zip(&world.path, &current_backup).await?;

        // Remove current world
        tokio::fs::remove_dir_all(&world.path).await?;

        // Extract backup
        self.extract_zip(backup_path, &world.path).await?;

        info!("Restored world: {} from {}", world.name, backup_path.display());
        Ok(())
    }

    async fn extract_zip(&self, zip_path: &Path, dest: &Path) -> Result<()> {
        use zip::{ZipArchive, ZipWriter, write::FileOptions};
        let mut zip = ZipArchive::new(std::fs::File::open(zip_path)?)?;

        tokio::fs::create_dir_all(dest).await?;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let outpath = dest.join(file.name());

            if file.name().ends_with('/') {
                tokio::fs::create_dir_all(&outpath).await?;
            } else {
                if let Some(parent) = outpath.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                let mut outfile = tokio::fs::File::create(outpath).await?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    /// Delete a world
    pub async fn delete_world(&self, world_id: Uuid) -> Result<()> {
        let world = self.worlds.read().await.get(&world_id).cloned()
            .ok_or_else(|| anyhow::anyhow!("World not found"))?;
        drop(self.worlds.read().await);

        // Create backup before deletion
        let _ = self.backup_world(world_id).await;

        tokio::fs::remove_dir_all(&world.path).await?;

        let mut worlds = self.worlds.write().await;
        worlds.remove(&world_id);

        info!("Deleted world: {}", world.name);
        Ok(())
    }

    /// Open world folder in file explorer
    pub async fn open_world_folder(&self, world_id: Uuid) -> Result<()> {
        let world = self.worlds.read().await.get(&world_id).cloned()
            .ok_or_else(|| anyhow::anyhow!("World not found"))?;
        drop(self.worlds.read().await);

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&world.path)
                .spawn()?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&world.path)
                .spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&world.path)
                .spawn()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LevelDatWrapper {
    level_dat: LevelDat,
    version: VersionInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_serialization() {
        let world = World::default();
        let json = serde_json::to_string(&world).unwrap();
        assert!(json.contains("name"));
    }
}