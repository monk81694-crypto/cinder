//! Discord Rich Presence for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use discord_rpc_client::Client as DiscordClient;

/// Discord RPC manager
pub struct DiscordManager {
    client: Option<DiscordClient>,
    client_id: String,
    connected: Arc<RwLock<bool>>,
    current_presence: Arc<RwLock<Option<DiscordPresence>>>,
    profile_statuses: Arc<RwLock<HashMap<Uuid, DiscordProfileStatus>>>,
    settings: Arc<RwLock<Settings>>,
    reconnect_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl DiscordManager {
    pub fn new(client_id: String, settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            client: None,
            client_id,
            connected: Arc::new(RwLock::new(false)),
            current_presence: Arc::new(RwLock::new(None)),
            profile_statuses: Arc::new(RwLock::new(HashMap::new())),
            settings,
            reconnect_task: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        if self.client.is_some() {
            return Ok(());
        }

        let mut client = DiscordClient::new(&self.client_id);
        client.start().map_err(|e| anyhow::anyhow!("Failed to start Discord RPC: {}", e))?;

        self.client = Some(client);
        *self.connected.write().await = true;

        // Start reconnect loop
        self.start_reconnect_loop().await;

        info!("Discord RPC connected");
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(client) = self.client.take() {
            client.close().map_err(|e| anyhow::anyhow!("Failed to close Discord RPC: {}", e))?;
        }
        *self.connected.write().await = false;

        // Stop reconnect loop
        if let Some(task) = self.reconnect_task.write().await.take() {
            task.abort();
        }

        info!("Discord RPC disconnected");
        Ok(())
    }

    async fn start_reconnect_loop(&self) {
        let connected = self.connected.clone();
        let client_id = self.client_id.clone();
        let current_presence = self.current_presence.clone();

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                let is_connected = *connected.read().await;
                if !is_connected {
                    // Try to reconnect
                    if let Ok(mut client) = DiscordClient::new(&client_id).start() {
                        *connected.write().await = true;
                        info!("Discord RPC reconnected");

                        // Restore last presence
                        if let Some(presence) = current_presence.read().await.as_ref() {
                            let _ = client.set_activity(|a| {
                                a.state(presence.state.clone())
                                    .details(presence.details.clone())
                                    .large_image(presence.large_image.clone())
                                    .large_text(presence.large_text.clone())
                                    .small_image(presence.small_image.clone())
                                    .small_text(presence.small_text.clone())
                                    .start_timestamp(presence.start_timestamp)
                            });
                        }
                    }
                }
            }
        });

        *self.reconnect_task.write().await = Some(task);
    }

    pub async fn update_presence(&self, presence: DiscordPresence) -> Result<()> {
        if let Some(client) = &self.client {
            client.set_activity(|a| {
                a.state(presence.state.clone())
                    .details(presence.details.clone())
                    .large_image(presence.large_image.clone())
                    .large_text(presence.large_text.clone())
                    .small_image(presence.small_image.clone())
                    .small_text(presence.small_text.clone())
                    .start_timestamp(presence.start_timestamp)
                    .end_timestamp(presence.end_timestamp)
                    .party_size(presence.party_size.0, presence.party_size.1)
                    .party_id(presence.party_id.clone())
                    .match_secret(presence.match_secret.clone())
                    .join_secret(presence.join_secret.clone())
                    .spectate_secret(presence.spectate_secret.clone())
            }).map_err(|e| anyhow::anyhow!("Failed to set Discord activity: {}", e))?;

            *self.current_presence.write().await = Some(presence);
        } else {
            return Err(anyhow::anyhow!("Discord RPC not connected"));
        }
        Ok(())
    }

    pub async fn set_launcher_presence(&self) -> Result<()> {
        let presence = DiscordPresence {
            state: "In Launcher".to_string(),
            details: "Cinder v3.0".to_string(),
            large_image: "cinder_logo".to_string(),
            large_text: "Cinder Launcher".to_string(),
            small_image: None,
            small_text: None,
            start_timestamp: Some(chrono::Utc::now().timestamp() as u64),
            end_timestamp: None,
            party_size: None,
            party_id: None,
            match_secret: None,
            join_secret: None,
            spectate_secret: None,
        };
        self.update_presence(presence).await
    }

    pub async fn set_game_presence(&self, version: &str, server: Option<&str>, profile_status: Option<&str>) -> Result<()> {
        let mut details = format!("Playing Minecraft {}", version);
        let mut state = "In Game".to_string();

        if let Some(server) = server {
            details = format!("{} on {}", details, server);
            state = format!("{} → {}", state, server);
        }

        if let Some(status) = profile_status {
            state = status.to_string();
        }

        let presence = DiscordPresence {
            state,
            details,
            large_image: "minecraft".to_string(),
            large_text: format!("Minecraft {}", version),
            small_image: Some("cinder_logo".to_string()),
            small_text: Some("Cinder Launcher".to_string()),
            start_timestamp: Some(chrono::Utc::now().timestamp() as u64),
            end_timestamp: None,
            party_size: None,
            party_id: None,
            match_secret: None,
            join_secret: None,
            spectate_secret: None,
        };

        self.update_presence(presence).await
    }

    pub async fn clear_presence(&self) -> Result<()> {
        if let Some(client) = &self.client {
            client.clear_activity().map_err(|e| anyhow::anyhow!("Failed to clear Discord activity: {}", e))?;
            *self.current_presence.write().await = None;
        }
        Ok(())
    }

    pub async fn set_profile_status(&self, profile_id: Uuid, status: Option<String>) -> Result<()> {
        let mut statuses = self.profile_statuses.write().await;

        if let Some(status) = status {
            let profile_status = DiscordProfileStatus {
                account_id: profile_id,
                custom_status: Some(status),
                show_server: true,
                show_modpack: true,
                updated_at: Utc::now(),
            };
            statuses.insert(profile_id, profile_status);
        } else {
            statuses.remove(&profile_id);
        }
        Ok(())
    }

    pub async fn get_profile_status(&self, profile_id: Uuid) -> Option<DiscordProfileStatus> {
        self.profile_statuses.read().await.get(&profile_id).cloned()
    }

    pub async fn test_connection(&self) -> Result<bool> {
        Ok(self.client.is_some() && *self.connected.read().await)
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiscordPresence {
    pub state: String,
    pub details: String,
    pub large_image: String,
    pub large_text: String,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub start_timestamp: Option<u64>,
    pub end_timestamp: Option<u64>,
    pub party_size: Option<(i32, i32)>,
    pub party_id: Option<String>,
    pub match_secret: Option<String>,
    pub join_secret: Option<String>,
    pub spectate_secret: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_default() {
        let presence = DiscordPresence::default();
        assert_eq!(presence.state, "");
        assert_eq!(presence.details, "");
    }
}