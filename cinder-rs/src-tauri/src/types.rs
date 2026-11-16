//! Core type definitions for Cinder Launcher

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use semver::Version;

/// Account types supported by Cinder
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Offline,
    Microsoft,
}

/// Minecraft account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub account_type: AccountType,
    pub username: String,
    pub display_name: Option<String>,
    pub uuid: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub xbox_gamertag: Option<String>,
    pub xbox_xuid: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub group_id: Option<Uuid>,
    pub is_active: bool,
}

/// Profile group for organizing accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub default_version: Option<String>,
    pub default_loader: Option<String>,
    pub default_server: Option<String>,
    pub account_ids: Vec<Uuid>,
    pub order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Launch profile - named configuration per account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchProfile {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
    pub mods: Vec<ModReference>,
    pub resource_packs: Vec<ResourcePackReference>,
    pub shader_packs: Vec<ShaderPackReference>,
    pub jvm_args: String,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub server: Option<String>,
    pub discord_status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl Default for LaunchProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id: Uuid::nil(),
            name: "Default".to_string(),
            description: None,
            version: "1.21.4".to_string(),
            loader: Some("fabric".to_string()),
            loader_version: None,
            mods: Vec::new(),
            resource_packs: Vec::new(),
            shader_packs: Vec::new(),
            jvm_args: String::new(),
            min_memory_mb: 2048,
            max_memory_mb: 4096,
            width: 854,
            height: 480,
            fullscreen: false,
            server: None,
            discord_status: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_used_at: None,
        }
    }
}

/// Reference to a mod in a launch profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModReference {
    pub project_id: String,
    pub version_id: String,
    pub file_name: String,
    pub enabled: bool,
    pub pinned_version: Option<String>,
}

/// Reference to a resource pack in a launch profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackReference {
    pub id: Uuid,
    pub enabled: bool,
    pub priority: i32,
}

/// Reference to a shader pack in a launch profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPackReference {
    pub id: Uuid,
    pub enabled: bool,
}

/// Mod loader types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Vanilla,
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}

impl ModLoader {
    pub fn as_str(&self) -> &str {
        match self {
            ModLoader::Vanilla => "vanilla",
            ModLoader::Fabric => "fabric",
            ModLoader::Quilt => "quilt",
            ModLoader::Forge => "forge",
            ModLoader::NeoForge => "neoforge",
        }
    }

    pub fn supports_mods(&self) -> bool {
        !matches!(self, ModLoader::Vanilla)
    }
}

/// Minecraft version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub version_type: String,
    pub url: String,
    pub time: DateTime<Utc>,
    pub release_time: DateTime<Utc>,
    pub sha1: Option<String>,
    pub compliant: bool,
    pub installed: bool,
    pub install_path: Option<std::path::PathBuf>,
    pub required_java_version: u8,
    pub loader_support: std::collections::HashMap<String, Vec<String>>,
}

/// Java installation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInstallation {
    pub path: std::path::PathBuf,
    pub version: String,
    pub major_version: u8,
    pub vendor: JavaVendor,
    pub architecture: String,
    pub has_javafx: bool,
    pub is_bundled: bool,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum JavaVendor {
    EclipseTemurin,
    Microsoft,
    Oracle,
    AzulZulu,
    BellSoft,
    Adoptium,
    GraalVM,
    AmazonCorretto,
    SapMachine,
    Liberica,
    IbmSemeru,
    RedHat,
    Unknown(String),
}

impl JavaVendor {
    pub fn from_path(path: &std::path::PathBuf) -> Self {
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("temurin") || path_str.contains("adoptium") {
            JavaVendor::EclipseTemurin
        } else if path_str.contains("microsoft") {
            JavaVendor::Microsoft
        } else if path_str.contains("oracle") {
            JavaVendor::Oracle
        } else if path_str.contains("azul") || path_str.contains("zulu") {
            JavaVendor::AzulZulu
        } else if path_str.contains("bellsoft") || path_str.contains("liberica") {
            JavaVendor::BellSoft
        } else if path_str.contains("adoptium") {
            JavaVendor::Adoptium
        } else if path_str.contains("graalvm") {
            JavaVendor::GraalVM
        } else if path_str.contains("corretto") || path_str.contains("amazon") {
            JavaVendor::AmazonCorretto
        } else if path_str.contains("sapmachine") {
            JavaVendor::SapMachine
        } else if path_str.contains("ibm") || path_str.contains("semeru") {
            JavaVendor::IbmSemeru
        } else if path_str.contains("redhat") {
            JavaVendor::RedHat
        } else {
            JavaVendor::Unknown(path_str)
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            JavaVendor::EclipseTemurin => "Eclipse Temurin",
            JavaVendor::Microsoft => "Microsoft Build of OpenJDK",
            JavaVendor::Oracle => "Oracle JDK",
            JavaVendor::AzulZulu => "Azul Zulu",
            JavaVendor::BellSoft => "BellSoft Liberica",
            JavaVendor::Adoptium => "Adoptium Temurin",
            JavaVendor::GraalVM => "GraalVM",
            JavaVendor::AmazonCorretto => "Amazon Corretto",
            JavaVendor::SapMachine => "SAP Machine",
            JavaVendor::Liberica => "BellSoft Liberica",
            JavaVendor::IbmSemeru => "IBM Semeru",
            JavaVendor::RedHat => "Red Hat Build of OpenJDK",
            JavaVendor::Unknown(s) => s.as_str(),
        }
    }
}

/// Mod info from Modrinth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthProject {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
    pub icon_url: Option<String>,
    pub gallery: Vec<String>,
    pub downloads: u64,
    pub followers: u32,
    pub license: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub version_number: String,
    pub name: String,
    pub changelog: Option<String>,
    pub dependencies: Vec<ModrinthDependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub file_name: String,
    pub file_url: String,
    pub file_size: u64,
    pub file_hashes: ModrinthFileHashes,
    pub date_published: DateTime<Utc>,
    pub downloads: u64,
    pub version_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthFileHashes {
    pub sha1: String,
    pub sha512: String,
}

/// Local mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMod {
    pub id: Uuid,
    pub project_id: String,
    pub version_id: String,
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub file_path: std::path::PathBuf,
    pub file_size: u64,
    pub file_hash: String,
    pub loader: ModLoader,
    pub game_versions: Vec<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub enabled: bool,
    pub pinned_version: Option<String>,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dependencies: Vec<ModDependency>,
    pub compatible_loaders: Vec<ModLoader>,
    pub source: ModSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModSource {
    Modrinth,
    CurseForge,
    Local,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub project_id: String,
    pub version: Option<String>,
    pub dependency_type: String,
}

/// Resource pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePack {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub file_name: String,
    pub file_path: std::path::PathBuf,
    pub file_size: u64,
    pub file_hash: String,
    pub pack_format: u32,
    pub supported_formats: Vec<u32>,
    pub icon_path: Option<std::path::PathBuf>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub source: PackSource,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackSource {
    VanillaTweaks,
    Local,
    Custom,
    Modrinth,
}

/// Shader pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPack {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub file_name: String,
    pub file_path: std::path::PathBuf,
    pub file_size: u64,
    pub file_hash: String,
    pub shader_type: ShaderType,
    pub preview_image: Option<std::path::PathBuf>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub enabled: bool,
    pub compatible_loaders: Vec<ModLoader>,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShaderType {
    Iris,
    OptiFine,
    Universal,
}

/// Skin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skin {
    pub id: Uuid,
    pub name: String,
    pub username: String,
    pub file_path: std::path::PathBuf,
    pub texture_url: Option<String>,
    pub variant: SkinVariant,
    pub preview_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkinVariant {
    Classic,
    Slim,
}

/// World
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub id: Uuid,
    pub name: String,
    pub path: std::path::PathBuf,
    pub size_bytes: u64,
    pub last_played: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub version: Option<String>,
    pub game_mode: Option<String>,
    pub difficulty: Option<String>,
    pub hardcore: bool,
    pub level_dat: Option<LevelDat>,
    pub icon_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelDat {
    pub game_type: u8,
    pub difficulty: u8,
    pub hardcore: bool,
    pub seed: i64,
    pub version: VersionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: i32,
}

/// Server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub description: Option<String>,
    pub icon_path: Option<std::path::PathBuf>,
    pub group_id: Option<Uuid>,
    pub order: i32,
    pub ping: Option<ServerPing>,
    pub last_pinged: Option<DateTime<Utc>>,
    pub added_at: DateTime<Utc>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            address: String::new(),
            port: 25565,
            description: None,
            icon_path: None,
            group_id: None,
            order: 0,
            ping: None,
            last_pinged: None,
            added_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPing {
    pub online: bool,
    pub players_online: u32,
    pub players_max: u32,
    pub motd: String,
    pub version_name: String,
    pub protocol_version: i32,
    pub ping_ms: u32,
    pub region: Option<ServerRegion>,
    pub favicon: Option<String>,
    pub sample_players: Vec<String>,
    pub pinged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ServerRegion {
    UsEast,
    UsWest,
    EuWest,
    EuCentral,
    AsiaPacific,
    SouthAmerica,
    Unknown,
}

/// Server group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub server_ids: Vec<Uuid>,
    pub order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Discord RPC profile status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordProfileStatus {
    pub account_id: Uuid,
    pub custom_status: Option<String>,
    pub show_server: bool,
    pub show_modpack: bool,
    pub updated_at: DateTime<Utc>,
}

/// Game directory health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub scanned_at: DateTime<Utc>,
    pub total_files: u64,
    pub total_size_bytes: u64,
    pub issues: Vec<HealthIssue>,
    pub can_auto_fix: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub path: Option<std::path::PathBuf>,
    pub auto_fixable: bool,
    pub fix_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum IssueCategory {
    CorruptedJar,
    MissingLibrary,
    OrphanedMod,
    DuplicateResourcePack,
    InvalidSymlink,
    MissingLevelDat,
    InvalidVersionJson,
    OutdatedMod,
    IncompatibleMod,
    DiskSpace,
    PermissionDenied,
    Other,
}

/// Diagnostics info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    pub timestamp: DateTime<Utc>,
    pub launcher_version: String,
    pub platform: String,
    pub architecture: String,
    pub total_memory_gb: f64,
    pub free_memory_gb: f64,
    pub java_installations: Vec<JavaInstallation>,
    pub game_directory: std::path::PathBuf,
    pub versions_cached: u32,
    pub installed_versions: u32,
    pub mods_count: u32,
    pub resource_packs_count: u32,
    pub shader_packs_count: u32,
    pub worlds_count: u32,
    pub accounts_count: u32,
    pub network_checks: NetworkChecks,
    pub settings_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkChecks {
    pub mojang: bool,
    pub modrinth: bool,
    pub minotar: bool,
    pub microsoft_auth: bool,
    pub discord: bool,
    pub github: bool,
}

/// Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub game_directory: std::path::PathBuf,
    pub java_path: Option<std::path::PathBuf>,
    pub auto_java: bool,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub jvm_args: String,
    pub jvm_preset: JvmPreset,
    pub launch_action: LaunchAction,
    pub launch_timeout_sec: u32,
    pub accent_theme: AccentTheme,
    pub reduced_motion: bool,
    pub show_debug_logs: bool,
    pub follow_logs: bool,
    pub show_snow: bool,
    pub check_updates: bool,
    pub discord_enabled: bool,
    pub discord_client_id: Option<String>,
    pub discord_download_url: String,
    pub discord_custom_status: Option<String>,
    pub language: String,
    pub high_contrast: bool,
    pub log_filter: LogFilter,
    pub last_version: Option<String>,
    pub last_account_id: Option<Uuid>,
    pub last_server: Option<String>,
    pub last_launch_profile_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JvmPreset {
    Balanced,
    Performance,
    Latency,
    Custom,
}

impl JvmPreset {
    pub fn args(&self, java_version: u8) -> Vec<String> {
        match self {
            JvmPreset::Balanced => vec![
                "-XX:+UseG1GC".to_string(),
                "-XX:MaxGCPauseMillis=100".to_string(),
                "-XX:+ParallelRefProcEnabled".to_string(),
            ],
            JvmPreset::Performance => vec![
                "-XX:+UseG1GC".to_string(),
                "-XX:MaxGCPauseMillis=50".to_string(),
                "-XX:+ParallelRefProcEnabled".to_string(),
                "-XX:+UnlockExperimentalVMOptions".to_string(),
                "-XX:G1HeapRegionSize=16M".to_string(),
            ],
            JvmPreset::Latency => {
                if java_version >= 21 {
                    vec![
                        "-XX:+UseZGC".to_string(),
                        "-XX:+ZGenerational".to_string(),
                        "-XX:+UnlockExperimentalVMOptions".to_string(),
                    ]
                } else {
                    vec![
                        "-XX:+UseG1GC".to_string(),
                        "-XX:MaxGCPauseMillis=10".to_string(),
                        "-XX:+ParallelRefProcEnabled".to_string(),
                    ]
                }
            }
            JvmPreset::Custom => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LaunchAction {
    Minimize,
    Hide,
    Nothing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum AccentTheme {
    Ember,
    Mint,
    Violet,
    Glacier,
    Rose,
    Indigo,
}

impl AccentTheme {
    pub fn all() -> Vec<Self> {
        vec![
            AccentTheme::Ember,
            AccentTheme::Mint,
            AccentTheme::Violet,
            AccentTheme::Glacier,
            AccentTheme::Rose,
            AccentTheme::Indigo,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFilter {
    All,
    Error,
    Warn,
    Info,
    Debug,
}

impl Default for Settings {
    fn default() -> Self {
        let game_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".cinder-minecraft");

        Self {
            game_directory: game_dir,
            java_path: None,
            auto_java: true,
            min_memory_mb: 2048,
            max_memory_mb: 4096,
            window_width: 854,
            window_height: 480,
            fullscreen: false,
            jvm_args: String::new(),
            jvm_preset: JvmPreset::Balanced,
            launch_action: LaunchAction::Minimize,
            launch_timeout_sec: 30,
            accent_theme: AccentTheme::Ember,
            reduced_motion: false,
            show_debug_logs: false,
            follow_logs: true,
            show_snow: true,
            check_updates: true,
            discord_enabled: true,
            discord_client_id: None,
            discord_download_url: "https://github.com/monk81694-crypto/cinder".to_string(),
            discord_custom_status: None,
            language: "en".to_string(),
            high_contrast: false,
            log_filter: LogFilter::All,
            last_version: None,
            last_account_id: None,
            last_server: None,
            last_launch_profile_id: None,
        }
    }
}

/// Update info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub newer: bool,
    pub release_notes: Option<String>,
    pub download_url: String,
    pub published_at: DateTime<Utc>,
    pub mandatory: bool,
}

/// Name forge styles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NameStyle {
    Mixed,
    Ember,
    Hero,
    Dark,
    Cute,
    Random,
    Custom(String),
}

/// Vanilla Tweaks pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VanillaTweaksPack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub download_url: String,
    pub file_size: u64,
    pub icon_url: Option<String>,
    pub tags: Vec<String>,
}

/// Command palette item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub action: PaletteAction,
    pub keywords: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PaletteAction {
    Navigate { tab: String },
    Launch { profile_id: Option<Uuid> },
    SetAccent { theme: AccentTheme },
    OpenSettings { section: Option<String> },
    OpenFolder { path: String },
    KillGame,
    CheckUpdates,
    RunDiagnostics,
    CopyDiagnostics,
    SearchMods { query: String },
    SearchServers { query: String },
}

/// MultiMC instance info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiMCInstance {
    pub name: String,
    pub path: std::path::PathBuf,
    pub version: String,
    pub loader: Option<ModLoader>,
    pub loader_version: Option<String>,
    pub java_version: Option<u8>,
    pub jvm_args: String,
    pub memory_max: u32,
    pub memory_min: u32,
    pub custom_resolution: Option<(u32, u32)>,
    pub fullscreen: bool,
    pub mods: Vec<MultiMCMod>,
    pub resource_packs: Vec<String>,
    pub server: Option<String>,
    pub icon: Option<std::path::PathBuf>,
    pub last_played: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiMCMod {
    pub name: String,
    pub file_name: String,
    pub enabled: bool,
    pub side: String,
}

impl Default for LaunchProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id: Uuid::nil(),
            name: "Default".to_string(),
            description: None,
            version: "1.21.4".to_string(),
            loader: Some("fabric".to_string()),
            loader_version: None,
            mods: Vec::new(),
            resource_packs: Vec::new(),
            shader_packs: Vec::new(),
            jvm_args: String::new(),
            min_memory_mb: 2048,
            max_memory_mb: 4096,
            width: 854,
            height: 480,
            fullscreen: false,
            server: None,
            discord_status: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_used_at: None,
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            address: String::new(),
            port: 25565,
            description: None,
            icon_path: None,
            group_id: None,
            order: 0,
            ping: None,
            last_pinged: None,
            added_at: chrono::Utc::now(),
        }
    }
}