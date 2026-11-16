//! Profile management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;

/// Profile manager for launch profiles and groups
pub struct ProfilesManager {
    profiles: Arc<RwLock<HashMap<Uuid, LaunchProfile>>>,
    groups: Arc<RwLock<HashMap<Uuid, ProfileGroup>>>,
    profiles_dir: PathBuf,
    settings: Arc<RwLock<Settings>>,
}

impl ProfilesManager {
    pub fn new(profiles_dir: PathBuf, settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc<RwLock<HashMap<Uuid, ProfileGroup>>>(),
            profiles_dir,
            settings,
        }
    }

    pub async fn load(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.profiles_dir).await?;

        // Load profiles
        let profiles_file = self.profiles_dir.join("profiles.json");
        if profiles_file.exists() {
            let content = tokio::fs::read_to_string(&profiles_file).await?;
            let profiles: Vec<LaunchProfile> = serde_json::from_str(&content)?;
            let mut map = self.profiles.write().await;
            for p in profiles {
                map.insert(p.id, p);
            }
        }

        // Load groups
        let groups_file = self.profiles_dir.join("groups.json");
        if groups_file.exists() {
            let content = tokio::fs::read_to_string(&groups_file).await?;
            let groups: Vec<ProfileGroup> = serde_json::from_str(&content)?;
            let mut map = self.groups.write().await;
            for g in groups {
                map.insert(g.id, g);
            }
        }

        // Create default group if none exist
        let mut groups = self.groups.write().await;
        if groups.is_empty() {
            let default_group = ProfileGroup {
                id: Uuid::new_v4(),
                name: "Default".to_string(),
                description: Some("Default profile group".to_string()),
                color: Some("#FF6B1A".to_string()),
                icon: Some("users".to_string()),
                default_version: None,
                default_loader: None,
                default_server: None,
                account_ids: Vec::new(),
                order: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            groups.insert(default_group.id, default_group.clone());
            drop(groups);
            self.save_groups().await?;
        }

        info!("Loaded profiles and groups");
        Ok(())
    }

    async fn save_profiles(&self) -> Result<()> {
        let profiles_file = self.profiles_dir.join("profiles.json");
        let profiles = self.profiles.read().await;
        let content = serde_json::to_string_pretty(&profiles.values().cloned().collect::<Vec<_>>())?;
        tokio::fs::write(profiles_file, content).await?;
        Ok(())
    }

    async fn save_groups(&self) -> Result<()> {
        let groups_file = self.profiles_dir.join("groups.json");
        let groups = self.groups.read().await;
        let content = serde_json::to_string_pretty(&groups.values().cloned().collect::<Vec<_>>())?;
        tokio::fs::write(groups_file, content).await?;
        Ok(())
    }

    // ===== Profiles =====

    pub async fn create_profile(&self, mut profile: LaunchProfile) -> Result<LaunchProfile> {
        profile.id = Uuid::new_v4();
        profile.created_at = Utc::now();
        profile.updated_at = Utc::now();

        let mut profiles = self.profiles.write().await;
        profiles.insert(profile.id, profile.clone());
        drop(profiles);

        self.save_profiles().await?;
        info!("Created profile: {} ({})", profile.name, profile.id);
        Ok(profile)
    }

    pub async fn update_profile(&self, id: Uuid, updates: ProfileUpdates) -> Result<LaunchProfile> {
        let mut profiles = self.profiles.write().await;
        let profile = profiles.get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Profile not found"))?;

        if let Some(name) = updates.name { profile.name = name; }
        if let Some(description) = updates.description { profile.description = Some(description); }
        if let Some(version) = updates.version { profile.version = version; }
        if let Some(loader) = updates.loader { profile.loader = Some(loader); }
        if let Some(loader_version) = updates.loader_version { profile.loader_version = Some(loader_version); }
        if let Some(mods) = updates.mods { profile.mods = mods; }
        if let Some(resource_packs) = updates.resource_packs { profile.resource_packs = resource_packs; }
        if let Some(shader_packs) = updates.shader_packs { profile.shader_packs = shader_packs; }
        if let Some(jvm_args) = updates.jvm_args { profile.jvm_args = jvm_args; }
        if let Some(min_memory_mb) = updates.min_memory_mb { profile.min_memory_mb = min_memory_mb; }
        if let Some(max_memory_mb) = updates.max_memory_mb { profile.max_memory_mb = max_memory_mb; }
        if let Some(width) = updates.width { profile.width = width; }
        if let Some(height) = updates.height { profile.height = height; }
        if let Some(fullscreen) = updates.fullscreen { profile.fullscreen = fullscreen; }
        if let Some(server) = updates.server { profile.server = Some(server); }
        if let Some(discord_status) = updates.discord_status { profile.discord_status = Some(discord_status); }

        profile.updated_at = Utc::now();

        let updated = profile.clone();
        drop(profiles);
        self.save_profiles().await?;

        info!("Updated profile: {} ({})", updated.name, updated.id);
        Ok(updated)
    }

    pub async fn delete_profile(&self, id: Uuid) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        profiles.remove(&id);
        drop(profiles);
        self.save_profiles().await?;
        info!("Deleted profile: {}", id);
        Ok(())
    }

    pub async fn list_profiles(&self) -> Vec<LaunchProfile> {
        self.profiles.read().await.values().cloned().collect()
    }

    pub async fn get_profile(&self, id: Uuid) -> Result<LaunchProfile> {
        self.profiles.read().await.get(&id).cloned()
            .ok_or_else(|| anyhow::anyhow!("Profile not found"))
    }

    pub async fn get_default_profile(&self) -> Result<LaunchProfile> {
        let profiles = self.profiles.read().await;
        if let Some(profile) = profiles.values().next() {
            Ok(profile.clone())
        } else {
            // Create default profile
            let mut default = LaunchProfile::default();
            default.name = "Default".to_string();
            drop(profiles);
            self.create_profile(default).await
        }
    }

    pub async fn duplicate_profile(&self, id: Uuid, new_name: Option<String>) -> Result<LaunchProfile> {
        let profile = self.get_profile(id).await?;
        let mut new_profile = profile.clone();
        new_profile.id = Uuid::new_v4();
        new_profile.name = new_name.unwrap_or_else(|| format!("{} (Copy)", profile.name));
        new_profile.created_at = Utc::now();
        new_profile.updated_at = Utc::now();
        new_profile.last_used_at = None;

        self.profiles.write().await.insert(new_profile.id, new_profile.clone());
        self.save_profiles().await?;

        Ok(new_profile)
    }

    pub async fn set_active_profile(&self, id: Uuid) -> Result<()> {
        let mut settings = self.settings.write().await;
        settings.last_launch_profile_id = Some(id);
        Ok(())
    }

    pub async fn get_active_profile(&self) -> Option<LaunchProfile> {
        let settings = self.settings.read().await;
        if let Some(id) = settings.last_launch_profile_id {
            self.profiles.read().await.get(&id).cloned()
        } else {
            None
        }
    }

    pub async fn update_last_used(&self, id: Uuid) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        if let Some(profile) = profiles.get_mut(&id) {
            profile.last_used_at = Some(Utc::now());
        }
        Ok(())
    }

    // ===== Groups =====

    pub async fn create_group(&self, mut group: ProfileGroup) -> Result<ProfileGroup> {
        group.id = Uuid::new_v4();
        group.created_at = Utc::now();
        group.updated_at = Utc::now();

        let order = self.groups.read().await.len() as i32;
        group.order = order;

        let mut groups = self.groups.write().await;
        groups.insert(group.id, group.clone());
        drop(groups);

        self.save_groups().await?;
        info!("Created group: {} ({})", group.name, group.id);
        Ok(group)
    }

    pub async fn update_group(&self, id: Uuid, updates: GroupUpdates) -> Result<ProfileGroup> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        if let Some(name) = updates.name { group.name = name; }
        if let Some(description) = updates.description { group.description = Some(description); }
        if let Some(color) = updates.color { group.color = Some(color); }
        if let Some(icon) = updates.icon { group.icon = Some(icon); }
        if let Some(default_version) = updates.default_version { group.default_version = Some(default_version); }
        if let Some(default_loader) = updates.default_loader { group.default_loader = Some(default_loader); }
        if let Some(default_server) = updates.default_server { group.default_server = Some(default_server); }
        if let Some(account_ids) = updates.account_ids { group.account_ids = account_ids; }
        if let Some(order) = updates.order { group.order = order; }

        group.updated_at = Utc::now();

        let updated = group.clone();
        drop(groups);
        self.save_groups().await?;

        Ok(updated)
    }

    pub async fn delete_group(&self, id: Uuid) -> Result<()> {
        let mut groups = self.groups.write().await;
        groups.remove(&id);
        drop(groups);
        self.save_groups().await?;
        Ok(())
    }

    pub async fn list_groups(&self) -> Vec<ProfileGroup> {
        let mut groups: Vec<_> = self.groups.read().await.values().cloned().collect();
        groups.sort_by_key(|g| g.order);
        groups
    }

    pub async fn move_account_to_group(&self, account_id: Uuid, group_id: Option<Uuid>) -> Result<()> {
        let mut groups = self.groups.write().await;

        // Remove from all groups
        for group in groups.values_mut() {
            group.account_ids.retain(|&id| id != account_id);
        }

        // Add to target group
        if let Some(id) = group_id {
            if let Some(group) = groups.get_mut(&id) {
                if !group.account_ids.contains(&account_id) {
                    group.account_ids.push(account_id);
                }
            }
        }

        self.save_groups().await?;
        Ok(())
    }

    pub async fn get_accounts_in_group(&self, group_id: Uuid) -> Vec<Uuid> {
        self.groups.read().await
            .get(&group_id)
            .map(|g| g.account_ids.clone())
            .unwrap_or_default()
    }
}

/// Profile update parameters
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ProfileUpdates {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
    pub mods: Option<Vec<ModReference>>,
    pub resource_packs: Option<Vec<ResourcePackReference>>,
    pub shader_packs: Option<Vec<ShaderPackReference>>,
    pub jvm_args: Option<String>,
    pub min_memory_mb: Option<u32>,
    pub max_memory_mb: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fullscreen: Option<bool>,
    pub server: Option<String>,
    pub discord_status: Option<String>,
}

/// Group update parameters
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct GroupUpdates {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub default_version: Option<String>,
    pub default_loader: Option<String>,
    pub default_server: Option<String>,
    pub account_ids: Option<Vec<Uuid>>,
    pub order: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_default() {
        let profile = LaunchProfile::default();
        assert_eq!(profile.name, "Default");
        assert_eq!(profile.version, "1.21.4");
        assert_eq!(profile.loader, Some("fabric".to_string()));
    }

    #[test]
    fn test_group_ordering() {
        let mut groups = vec![
            ProfileGroup { order: 2, ..Default::default() },
            ProfileGroup { order: 0, ..Default::default() },
            ProfileGroup { order: 1, ..Default::default() },
        ];
        groups.sort_by_key(|g| g.order);
        assert_eq!(groups[0].order, 0);
        assert_eq!(groups[1].order, 1);
        assert_eq!(groups[2].order, 2);
    }
}