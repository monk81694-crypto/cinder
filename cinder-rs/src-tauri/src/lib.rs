//! Cinder Launcher v3.0 - Rust/Tauri Core Library
//!
//! A modern Minecraft Java Edition launcher built with Tauri 2 and Rust.
//! Features: offline/Microsoft auth, Fabric/Quilt/Forge/NeoForge, Modrinth,
//! resource packs, shader packs, launch profiles, server management, Discord RPC.

#![allow(clippy::too_many_arguments)]
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

pub mod accounts;
pub mod config;
pub mod diagnostics;
pub mod discord;
pub mod java;
pub mod launch;
pub mod minecraft;
pub mod mods;
pub mod profiles;
pub mod resources;
pub mod servers;
pub mod settings;
pub mod shaders;
pub mod skins;
pub mod updater;
pub mod utils;
pub mod worlds;

use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize logging
            #[cfg(debug_assertions)]
            {
                use tracing_subscriber::{fmt, EnvFilter};
                fmt().with_env_filter(EnvFilter::from_default_env()).init();
            }

            let app_handle = app.handle().clone();

            // Start background services
            tokio::spawn(async move {
                // Auto-update check on launch
                if let Err(e) = updater::check_on_startup(app_handle.clone()).await {
                    tracing::warn!("Auto-update check failed: {}", e);
                }

                // Discord RPC connection
                if let Err(e) = discord::connect(app_handle.clone()).await {
                    tracing::warn!("Discord RPC connect failed: {}", e);
                }

                // Background mod update checker
                if let Err(e) = mods::start_background_updater(app_handle.clone()).await {
                    tracing::warn!("Mod updater failed: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Accounts
            accounts::add_offline_account,
            accounts::remove_account,
            accounts::list_accounts,
            accounts::set_active_account,
            accounts::microsoft_login,
            accounts::suggest_names,
            accounts::import_multimc_accounts,
            accounts::export_accounts,
            accounts::import_accounts,
            // Profiles
            profiles::create_profile,
            profiles::update_profile,
            profiles::delete_profile,
            profiles::list_profiles,
            profiles::get_profile,
            profiles::set_active_profile,
            profiles::duplicate_profile,
            // Groups
            profiles::create_group,
            profiles::update_group,
            profiles::delete_group,
            profiles::list_groups,
            profiles::move_account_to_group,
            // Settings
            settings::get_settings,
            settings::update_settings,
            settings::reset_settings,
            settings::export_settings,
            settings::import_settings,
            settings::verify_game_dir,
            // Java
            java::list_java,
            java::detect_java,
            java::install_recommended_java,
            java::set_java_path,
            // Minecraft versions
            minecraft::list_versions,
            minecraft::install_version,
            minecraft::uninstall_version,
            minecraft::get_version_manifest,
            // Launch
            launch::launch_game,
            launch::kill_game,
            launch::get_launch_args,
            launch::validate_launch_config,
            // Mods
            mods::list_mods,
            mods::search_modrinth,
            mods::get_modrinth_project,
            mods::install_mod,
            mods::uninstall_mod,
            mods::toggle_mod,
            mods::get_mod_dependencies,
            mods::check_mod_updates,
            mods::update_mod,
            mods::update_all_mods,
            mods::pin_mod_version,
            // Resource packs
            resources::list_resource_packs,
            resources::install_resource_pack,
            resources::uninstall_resource_pack,
            resources::toggle_resource_pack,
            resources::reorder_resource_packs,
            resources::fetch_vanilla_tweaks,
            // Shader packs
            shaders::list_shader_packs,
            shaders::install_shader_pack,
            shaders::uninstall_shader_pack,
            shaders::toggle_shader_pack,
            shaders::preview_shader_pack,
            shaders::detect_shader_loader,
            // Skins
            skins::get_skin,
            skins::upload_skin,
            skins::save_skin,
            skins::get_skin_preview,
            // Worlds
            worlds::list_worlds,
            worlds::backup_world,
            worlds::restore_world,
            worlds::delete_world,
            worlds::open_world_folder,
            // Servers
            servers::list_servers,
            servers::add_server,
            servers::remove_server,
            servers::update_server,
            servers::ping_server,
            servers::ping_all_servers,
            servers::create_group,
            servers::update_group,
            servers::delete_group,
            servers::move_server_to_group,
            // Discord
            discord::update_presence,
            discord::set_profile_status,
            discord::test_connection,
            // Diagnostics
            diagnostics::run_diagnostics,
            diagnostics::copy_diagnostics,
            // Updater
            updater::check_for_updates,
            updater::download_and_install_update,
            // Utils
            utils::open_folder,
            utils::open_external_url,
            utils::compute_sha256,
            utils::get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert!(semver::Version::parse("1.21.4").is_ok());
        assert!(semver::Version::parse("2.0.1").is_ok());
    }

    #[test]
    fn test_uuid_generation() {
        let id = uuid::Uuid::new_v4();
        assert!(!id.is_nil());
    }
}