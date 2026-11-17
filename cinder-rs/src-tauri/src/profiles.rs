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

type ProfileGroupMap = HashMap<Uuid, ProfileGroup>;
type GroupsMap = Arc<RwLock<ProfileGroupMap>>;

/// Profile manager for launch profiles and groups
pub struct ProfilesManager {
    profiles: Arc<RwLock<HashMap<Uuid, LaunchProfile>>>,
    groups: GroupsMap,
    profiles_dir: PathBuf,
    settings: Arc<RwLock<Settings>>,
}

impl ProfilesManager {
    pub fn new(profiles_dir: PathBuf, settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            groups: GroupsMap::default(),
            profiles_dir,
            settings,
        }
    }
}