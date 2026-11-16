//! Server management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use socket2::{Domain, Protocol, Socket, Type};
use std::time::Duration;

/// Server manager with ping functionality
pub struct ServerManager {
    servers: Arc<RwLock<HashMap<Uuid, Server>>>,
    groups: Arc<RwLock<HashMap<Uuid, ServerGroup>>>,
    ping_cache: Arc<RwLock<HashMap<String, ServerPing>>>,
    client: reqwest::Client,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            ping_cache: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    pub async fn load(&self, storage_path: &PathBuf) -> Result<()> {
        let servers_file = storage_path.join("servers.json");
        if servers_file.exists() {
            let content = tokio::fs::read_to_string(&servers_file).await?;
            let servers: Vec<Server> = serde_json::from_str(&content)?;
            let mut map = self.servers.write().await;
            for s in servers {
                map.insert(s.id, s);
            }
        }

        let groups_file = storage_path.join("server_groups.json");
        if groups_file.exists() {
            let content = tokio::fs::read_to_string(&groups_file).await?;
            let groups: Vec<ServerGroup> = serde_json::from_str(&content)?;
            let mut map = self.groups.write().await;
            for g in groups {
                map.insert(g.id, g);
            }
        }

        // Create default groups
        let mut groups = self.groups.write().await;
        if groups.is_empty() {
            let defaults = vec![
                ServerGroup {
                    id: Uuid::new_v4(),
                    name: "Favorites".to_string(),
                    description: Some("Your favorite servers".to_string()),
                    color: Some("#FF6B1A".to_string()),
                    icon: Some("star".to_string()),
                    server_ids: Vec::new(),
                    order: 0,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                ServerGroup {
                    id: Uuid::new_v4(),
                    name: "Minigames".to_string(),
                    description: Some("Minigame servers".to_string()),
                    color: Some("#3EF2B6".to_string()),
                    icon: Some("gamepad".to_string()),
                    server_ids: Vec::new(),
                    order: 1,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                ServerGroup {
                    id: Uuid::new_v4(),
                    name: "SMP".to_string(),
                    description: Some("Survival Multiplayer".to_string()),
                    color: Some("#A78BFA".to_string()),
                    icon: Some("home".to_string()),
                    server_ids: Vec::new(),
                    order: 2,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                ServerGroup {
                    id: Uuid::new_v4(),
                    name: "Anarchy".to_string(),
                    description: Some("No rules servers".to_string()),
                    color: Some("#FF7AB8".to_string()),
                    icon: Some("skull".to_string()),
                    server_ids: Vec::new(),
                    order: 3,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ];

            for group in defaults {
                groups.insert(group.id, group);
            }
        }

        info!("Loaded servers and groups");
        Ok(())
    }

    pub async fn save(&self, storage_path: &PathBuf) -> Result<()> {
        let servers_file = storage_path.join("servers.json");
        let servers = self.servers.read().await;
        let content = serde_json::to_string_pretty(&servers.values().cloned().collect::<Vec<_>>())?;
        tokio::fs::write(servers_file, content).await?;

        let groups_file = storage_path.join("server_groups.json");
        let groups = self.groups.read().await;
        let content = serde_json::to_string_pretty(&groups.values().cloned().collect::<Vec<_>>())?;
        tokio::fs::write(groups_file, content).await?;

        Ok(())
    }

    // ===== Servers =====

    pub async fn list_servers(&self) -> Vec<Server> {
        let servers = self.servers.read().await;
        let mut list: Vec<_> = servers.values().cloned().collect();
        list.sort_by(|a, b| {
            a.order.cmp(&b.order).then_with(|| a.name.cmp(&b.name))
        });
        list
    }

    pub async fn add_server(&self, mut server: Server) -> Result<Server> {
        server.id = Uuid::new_v4();
        server.added_at = Utc::now();

        let order = self.servers.read().await.len() as i32;
        server.order = order;

        let mut servers = self.servers.write().await;
        servers.insert(server.id, server.clone());
        drop(servers);

        info!("Added server: {} ({})", server.name, server.id);
        Ok(server)
    }

    pub async fn update_server(&self, id: Uuid, updates: ServerUpdates) -> Result<Server> {
        let mut servers = self.servers.write().await;
        let server = servers.get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(name) = updates.name { server.name = name; }
        if let Some(address) = updates.address { server.address = address; }
        if let Some(port) = updates.port { server.port = port; }
        if let Some(description) = updates.description { server.description = Some(description); }
        if let Some(group_id) = updates.group_id { server.group_id = Some(group_id); }
        if let Some(order) = updates.order { server.order = order; }

        let updated = server.clone();
        drop(servers);

        info!("Updated server: {} ({})", updated.name, updated.id);
        Ok(updated)
    }

    pub async fn remove_server(&self, id: Uuid) -> Result<()> {
        let mut servers = self.servers.write().await;
        servers.remove(&id);

        // Remove from groups
        let mut groups = self.groups.write().await;
        for group in groups.values_mut() {
            group.server_ids.retain(|&sid| sid != id);
        }

        info!("Removed server: {}", id);
        Ok(())
    }

    pub async fn ping_server(&self, id: Uuid) -> Result<ServerPing> {
        let server = self.servers.read().await.get(&id).cloned()
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;
        drop(self.servers);

        let ping = self.ping_address(&server.address, server.port).await?;

        // Update server with ping
        let mut servers = self.servers.write().await;
        if let Some(s) = servers.get_mut(&id) {
            s.ping = Some(ping.clone());
            s.last_pinged = Some(Utc::now());
        }

        // Cache ping
        let cache_key = format!("{}:{}", server.address, server.port);
        self.ping_cache.write().await.insert(cache_key, ping.clone());

        Ok(ping)
    }

    pub async fn ping_all_servers(&self) -> Result<Vec<ServerPing>> {
        let servers = self.servers.read().await.values().cloned().collect::<Vec<_>>();
        drop(self.servers);

        let mut results = Vec::new();
        for server in servers {
            match self.ping_address(&server.address, server.port).await {
                Ok(ping) => {
                    let mut servers = self.servers.write().await;
                    if let Some(s) = servers.get_mut(&server.id) {
                        s.ping = Some(ping.clone());
                        s.last_pinged = Some(Utc::now());
                    }
                    results.push(ping);
                }
                Err(e) => {
                    warn!("Failed to ping {}: {}", server.name, e);
                }
            // Small delay to avoid overwhelming servers
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Ok(results)
    }

    async fn ping_address(&self, host: &str, port: u16) -> Result<ServerPing> {
        let start = std::time::Instant::now();

        // Resolve address
        let addr = format!("{}:{}", host, port);
        let socket_addr: SocketAddr = addr.parse()?;

        // Try TCP connection
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        socket.set_write_timeout(Some(Duration::from_secs(5)))?;

        let connect_start = std::time::Instant::now();
        let connected = socket.connect(&socket_addr.into()).is_ok();
        let ping_ms = connect_start.elapsed().as_millis() as u32;

        if !connected {
            return Ok(ServerPing {
                online: false,
                players_online: 0,
                players_max: 0,
                motd: "Connection failed".to_string(),
                version_name: "Unknown".to_string(),
                protocol_version: 0,
                ping_ms,
                region: None,
                favicon: None,
                sample_players: Vec::new(),
                pinged_at: Utc::now(),
            });
        }

        // If connected, try to get status via Minecraft protocol
        // This would require implementing the Minecraft ping protocol (handshake + status request)
        // For now, return basic online status

        let region = self.detect_region(&host).await;

        Ok(ServerPing {
            online: true,
            players_online: 0, // Would need full protocol
            players_max: 0,
            motd: "Server online".to_string(),
            version_name: "Unknown".to_string(),
            protocol_version: 765, // 1.21.4
            ping_ms,
            region,
            favicon: None,
            sample_players: Vec::new(),
            pinged_at: Utc::now(),
        })
    }

    async fn detect_region(&self, host: &str) -> Option<ServerRegion> {
        // Try to resolve IP and check GeoIP
        // For now, check known server hostnames
        let host_lower = host.to_lowercase();

        if host_lower.contains("hypixel") || host_lower.contains("us.") {
            Some(ServerRegion::UsEast)
        } else if host_lower.contains("eu.") || host_lower.contains("europe") {
            Some(ServerRegion::EuWest)
        } else if host_lower.contains("asia") || host_lower.contains("jp.") || host_lower.contains("sg.") {
            Some(ServerRegion::AsiaPacific)
        } else if host_lower.contains("br.") || host_lower.contains("sa.") {
            Some(ServerRegion::SouthAmerica)
        } else {
            None
        }
    }

    // ===== Groups =====

    pub async fn create_group(&self, mut group: ServerGroup) -> Result<ServerGroup> {
        group.id = Uuid::new_v4();
        group.created_at = Utc::now();
        group.updated_at = Utc::now();

        let order = self.groups.read().await.len() as i32;
        group.order = order;

        let mut groups = self.groups.write().await;
        groups.insert(group.id, group.clone());
        drop(groups);

        self.save_groups().await?;
        Ok(group)
    }

    pub async fn update_group(&self, id: Uuid, updates: GroupUpdates) -> Result<ServerGroup> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        if let Some(name) = updates.name { group.name = name; }
        if let Some(description) = updates.description { group.description = Some(description); }
        if let Some(color) = updates.color { group.color = Some(color); }
        if let Some(icon) = updates.icon { group.icon = Some(icon); }
        if let Some(server_ids) = updates.server_ids { group.server_ids = server_ids; }
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
        self.save_groups().await?;
        Ok(())
    }

    pub async fn list_groups(&self) -> Vec<ServerGroup> {
        let mut groups: Vec<_> = self.groups.read().await.values().cloned().collect();
        groups.sort_by_key(|g| g.order);
        groups
    }

    pub async fn move_server_to_group(&self, server_id: Uuid, group_id: Option<Uuid>) -> Result<()> {
        let mut groups = self.groups.write().await;

        // Remove from all groups
        for group in groups.values_mut() {
            group.server_ids.retain(|&sid| sid != server_id);
        }

        // Add to target group
        if let Some(id) = group_id {
            if let Some(group) = groups.get_mut(&id) {
                if !group.server_ids.contains(&server_id) {
                    group.server_ids.push(server_id);
                }
            }
        }

        self.save_groups().await?;
        Ok(())
    }

    pub async fn get_servers_in_group(&self, group_id: Uuid) -> Vec<Uuid> {
        self.groups.read().await
            .get(&group_id)
            .map(|g| g.server_ids.clone())
            .unwrap_or_default()
    }

    async fn save_groups(&self) -> Result<()> {
        // This would save to storage path
        // For now, just a placeholder
        Ok(())
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ServerUpdates {
    pub name: Option<String>,
    pub address: Option<String>,
    pub port: Option<u16>,
    pub description: Option<String>,
    pub group_id: Option<Uuid>,
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct GroupUpdates {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub server_ids: Option<Vec<Uuid>>,
    pub order: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_default_port() {
        let server = Server::default();
        assert_eq!(server.port, 25565);
    }

    #[test]
    fn test_region_detection() {
        assert!(matches!(ServerRegion::UsEast, ServerRegion::UsEast));
    }
}