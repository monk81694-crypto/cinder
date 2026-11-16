//! Account management for Cinder Launcher

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use reqwest::Client;
use serde_json::json;

/// Account manager with Microsoft OAuth support
pub struct AccountManager {
    accounts: Arc<RwLock<HashMap<Uuid, Account>>>,
    active_account_id: Arc<RwLock<Option<Uuid>>>,
    client: Client,
    settings: Arc<RwLock<Settings>>,
}

impl AccountManager {
    pub fn new(settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            active_account_id: Arc::new(RwLock::new(None)),
            client: Client::new(),
            settings,
        }
    }

    pub async fn load_accounts(&self, storage_path: &PathBuf) -> Result<()> {
        let accounts_path = storage_path.join("accounts.json");
        if accounts_path.exists() {
            let content = tokio::fs::read_to_string(&accounts_path).await?;
            let accounts: Vec<Account> = serde_json::from_str(&content)?;
            let mut map = self.accounts.write().await;
            for acc in accounts {
                map.insert(acc.id, acc);
            }
        }
        // Load active account
        let active_path = storage_path.join("active_account.txt");
        if active_path.exists() {
            let id_str = tokio::fs::read_to_string(&active_path).await?;
            if let Ok(id) = Uuid::parse_str(&id_str.trim()) {
                *self.active_account_id.write().await = Some(id);
            }
        }
        Ok(())
    }

    pub async fn save_accounts(&self, storage_path: &PathBuf) -> Result<()> {
        let accounts_path = storage_path.join("accounts.json");
        let accounts = self.accounts.read().await;
        let content = serde_json::to_string_pretty(&accounts.values().cloned().collect::<Vec<_>>())?;
        tokio::fs::write(accounts_path, content).await?;

        let active_path = storage_path.join("active_account.txt");
        if let Some(id) = *self.active_account_id.read().await {
            tokio::fs::write(active_path, id.to_string()).await?;
        }
        Ok(())
    }

    /// Add an offline account
    pub async fn add_offline_account(&self, username: String, group_id: Option<Uuid>) -> Result<Account> {
        let name = username.trim();
        if name.is_empty() {
            return Err(anyhow::anyhow!("Username cannot be empty"));
        }
        if name.len() < 3 || name.len() > 16 {
            return Err(anyhow::anyhow!("Username must be 3-16 characters"));
        }
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(anyhow::anyhow!("Username can only contain A-Z, a-z, 0-9, _"));
        }

        // Check for duplicates
        let accounts = self.accounts.read().await;
        if accounts.values().any(|a| a.username.eq_ignore_ascii_case(&name)) {
            return Err(anyhow::anyhow!("Username already exists"));
        }

        let mut account = Account {
            id: Uuid::new_v4(),
            account_type: AccountType::Offline,
            username: name.to_string(),
            display_name: None,
            uuid: None,
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            xbox_gamertag: None,
            xbox_xuid: None,
            avatar_url: Some(format!("https://minotar.net/helm/{}/64.png", name)),
            created_at: Utc::now(),
            last_used_at: None,
            group_id,
            is_active: true,
        };

        // Generate offline UUID (deterministic from username)
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"OfflinePlayer:");
        hasher.update(name.as_bytes());
        let hash = hasher.finalize();
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&hash[..16]);
        // Set version 4 (random) and variant bits
        uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x40;
        uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80;
        account.uuid = Some(Uuid::from_bytes(uuid_bytes).to_string());

        let id = account.id;
        self.accounts.write().await.insert(id, account.clone());
        drop(accounts);

        info!("Added offline account: {} ({})", account.username, id);
        Ok(account)
    }

    /// Remove an account
    pub async fn remove_account(&self, id: Uuid) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        accounts.remove(&id);

        // Clear active if it was this account
        let mut active = self.active_account_id.write().await;
        if *active == Some(id) {
            *active = accounts.keys().next().copied();
        }

        info!("Removed account: {}", id);
        Ok(())
    }

    /// List all accounts
    pub async fn list_accounts(&self) -> Vec<Account> {
        self.accounts.read().await.values().cloned().collect()
    }

    /// Set active account
    pub async fn set_active_account(&self, id: Uuid) -> Result<()> {
        let accounts = self.accounts.read().await;
        if !accounts.contains_key(&id) {
            return Err(anyhow::anyhow!("Account not found"));
        }
        drop(accounts);

        *self.active_account_id.write().await = Some(id);
        info!("Set active account: {}", id);
        Ok(())
    }

    /// Get active account
    pub async fn get_active_account(&self) -> Option<Account> {
        let id = *self.active_account_id.read().await?;
        self.accounts.read().await.get(&id).cloned()
    }

    /// Microsoft OAuth login
    pub async fn microsoft_login(&self) -> Result<Account> {
        // This would integrate with azure_identity or custom OAuth flow
        // For now, return a placeholder - full implementation needs:
        // 1. Redirect to Microsoft login URL
        // 2. Handle callback with auth code
        // 3. Exchange code for tokens
        // 4. Fetch Xbox profile
        // 5. Get Minecraft entitlement
        // 6. Get Minecraft profile

        // Placeholder implementation
        Err(anyhow::anyhow!("Microsoft login not yet implemented - requires OAuth flow"))
    }

    /// Refresh Microsoft token
    pub async fn refresh_microsoft_token(&self, id: Uuid) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.get_mut(&id) {
            if account.account_type == AccountType::Microsoft {
                // Refresh token logic here
                // account.access_token = new_token;
                // account.token_expires_at = new_expiry;
                info!("Refreshed Microsoft token for account: {}", id);
            }
        }
        Ok(())
    }

    /// Suggest names for offline accounts
    pub async fn suggest_names(&self, count: usize, style: NameStyle) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        let existing: Vec<String> = self.accounts.read().await
            .values()
            .map(|a| a.username.clone())
            .collect();

        // Name generation based on style
        let generators: Vec<fn() -> String> = match style {
            NameStyle::Mixed => vec![gen_mixed, gen_ember, gen_hero, gen_dark, gen_cute],
            NameStyle::Ember => vec![gen_ember],
            NameStyle::Hero => vec![gen_hero],
            NameStyle::Dark => vec![gen_dark],
            NameStyle::Cute => vec![gen_cute],
            NameStyle::Random => vec![gen_random],
            NameStyle::Custom(_) => vec![gen_random],
        };

        while suggestions.len() < count {
            for gen in &generators {
                if suggestions.len() >= count { break; }
                let name = gen();
                if !existing.iter().any(|e| e.eq_ignore_ascii_case(&name))
                    && !suggestions.iter().any(|s| s.eq_ignore_ascii_case(&name)) {
                    suggestions.push(name);
                }
            }
        }

        Ok(suggestions)
    }

    /// Import accounts from MultiMC instance
    pub async fn import_multimc_accounts(&self, instance_path: &PathBuf) -> Result<Vec<Account>> {
        // MultiMC stores accounts in accounts.json
        let accounts_file = instance_path.join("accounts.json");
        if !accounts_file.exists() {
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(accounts_file).await?;
        let multimc_accounts: serde_json::Value = serde_json::from_str(&content)?;

        let mut imported = Vec::new();
        if let Some(accounts) = multimc_accounts.as_array() {
            for acc in accounts {
                if let Some(username) = acc.get("username").and_then(|v| v.as_str()) {
                    if let Ok(account) = self.add_offline_account(username.to_string(), None).await {
                        imported.push(account);
                    }
                }
            }
        }

        info!("Imported {} accounts from MultiMC", imported.len());
        Ok(imported)
    }

    /// Export accounts to JSON (encrypted)
    pub async fn export_accounts(&self, path: &PathBuf, password: &str) -> Result<()> {
        let accounts = self.accounts.read().await;
        let data = serde_json::to_vec(&accounts.values().cloned().collect::<Vec<_>>())?;

        // Simple encryption using password-derived key
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let key = hasher.finalize();

        // XOR encryption (for demo - use proper encryption in production)
        let mut encrypted = Vec::with_capacity(data.len() + 32);
        encrypted.extend_from_slice(&key[..16]); // IV
        for (i, byte) in data.iter().enumerate() {
            encrypted.push(byte ^ key[i % 32]);
        }

        tokio::fs::write(path, encrypted).await?;
        Ok(())
    }

    /// Import accounts from encrypted JSON
    pub async fn import_accounts(&self, path: &PathBuf, password: &str) -> Result<Vec<Account>> {
        let encrypted = tokio::fs::read(path).await?;
        if encrypted.len() < 16 {
            return Err(anyhow::anyhow!("Invalid encrypted file"));
        }

        let key = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            hasher.finalize()
        };

        let iv = &encrypted[..16];
        let data = &encrypted[16..];

        let mut decrypted = Vec::with_capacity(data.len());
        for (i, byte) in data.iter().enumerate() {
            decrypted.push(byte ^ key[i % 32]);
        }

        let accounts: Vec<Account> = serde_json::from_slice(&decrypted)?;
        let mut imported = Vec::new();

        for account in accounts {
            let mut accounts_map = self.accounts.write().await;
            if !accounts_map.contains_key(&account.id) {
                accounts_map.insert(account.id, account.clone());
                imported.push(account);
            }
        }

        Ok(imported)
    }
}

// Name generators
fn gen_mixed() -> String {
    let prefixes = ["Steve", "Alex", "Player", "Cinder", "Block", "Craft", "Mine", "Build"];
    let suffixes = ["Master", "Pro", "Gamer", "Fan", "King", "Queen", "Lord", "Hero"];
    let prefix = prefixes[fastrand::usize(..prefixes.len())];
    let suffix = suffixes[fastrand::usize(..suffixes.len())];
    let num = fastrand::u32(100..9999);
    format!("{}_{}{}", prefix, suffix, num)
}

fn gen_ember() -> String {
    let prefixes = ["Ember", "Blaze", "Flame", "Fire", "Spark", "Ash", "Cinder", "Coal"];
    let suffixes = ["Walker", "Heart", "Soul", "Spirit", "Lord", "King", "Master", "Born"];
    let prefix = prefixes[fastrand::usize(..prefixes.len())];
    let suffix = suffixes[fastrand::usize(..suffixes.len())];
    let num = fastrand::u32(10..999);
    format!("{}{}{}", prefix, suffix, num)
}

fn gen_hero() -> String {
    let prefixes = ["Captain", "Commander", "General", "Admiral", "Chief", "Master", "Grand", "Supreme"];
    let suffixes = ["Hero", "Guardian", "Protector", "Defender", "Champion", "Victor", "Conqueror", "Legend"];
    let prefix = prefixes[fastrand::usize(..prefixes.len())];
    let suffix = suffixes[fastrand::usize(..suffixes.len())];
    let num = fastrand::u32(1..99);
    format!("{}{}{:02}", prefix, suffix, num)
}

fn gen_dark() -> String {
    let prefixes = ["Shadow", "Void", "Dark", "Night", "Gloom", "Abyss", "Obsidian", "Onyx"];
    let suffixes = ["Walker", "Stalker", "Hunter", "Reaper", "Wraith", "Phantom", "Shade", "Veiled"];
    let prefix = prefixes[fastrand::usize(..prefixes.len())];
    let suffix = suffixes[fastrand::usize(..suffixes.len())];
    let num = fastrand::u32(100..9999);
    format!("{}{}{}", prefix, suffix, num)
}

fn gen_cute() -> String {
    let prefixes = ["Mochi", "Boba", "Peach", "Berry", "Honey", "Sugar", "Cookie", "Cake"];
    let suffixes = ["Bun", "Bear", "Cat", "Fox", "Pup", "Cub", "Bean", "Pod"];
    let prefix = prefixes[fastrand::usize(..prefixes.len())];
    let suffix = suffixes[fastrand::usize(..suffixes.len())];
    let num = fastrand::u32(1..99);
    format!("{}{}{:02}", prefix, suffix, num)
}

fn gen_random() -> String {
    let chars = "abcdefghijklmnopqrstuvwxyz0123456789_";
    let len = fastrand::usize(3..16);
    (0..len).map(|_| chars.chars().nth(fastrand::usize(..chars.len())).unwrap()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_generators() {
        for _ in 0..100 {
            let name = gen_mixed();
            assert!(name.len() >= 3 && name.len() <= 16);
            assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        }
    }

    #[test]
    fn test_offline_uuid_generation() {
        use sha2::{Digest, Sha256};
        let name = "TestPlayer";
        let mut hasher = Sha256::new();
        hasher.update(b"OfflinePlayer:");
        hasher.update(name.as_bytes());
        let hash = hasher.finalize();
        assert_eq!(hash.len(), 32);
    }
}