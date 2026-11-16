# Changelog

All notable changes to Cinder.

## [2.0.1] - 2026-11-16

### Added
- **Resource Packs tab**: Full resource pack management — drag & drop .zip packs, enable/disable per profile, auto-sort by priority, live preview of pack.mcmeta, open resource packs folder. Supports built-in Vanilla Tweaks catalog (fetch list, one-click install).
- **Shader Packs tab**: Iris/OptiFine shader management — add shader packs (.zip), toggle per profile, preview with screenshot, auto-detect Iris vs OptiFine, open shaders folder.
- **MultiMC instance import**: Import existing MultiMC/Prism/ATLauncher instances — copies versions, mods, configs, resource packs, shader packs, Java settings. One-click migration.
- **Profile grouping**: Organize accounts into groups (Survival, Creative, Modded, Testing) — collapsible sidebar sections, drag-to-reorder, per-group default version/loader/server.
- **Launch profiles**: Named launch configurations per profile — save multiple setups (e.g., "Modded 1.21.4 + Shaders", "Vanilla 1.20.4 + Sodium") with version, loader, mods, resource packs, shaders, JVM args, memory, resolution. Switch via dropdown on Play tab.
- **Modrinth project page**: Click any mod → opens in-app detail view with changelog, dependencies, versions table, screenshots, "Install specific version" picker, "Open on Modrinth" link.
- **Auto-update mods**: Background check for mod updates (Modrinth) — badge on Mods tab, one-click "Update all" or per-mod, respects pinned versions, configurable interval (daily/weekly/never).
- **Server groups & ping regions**: Organize servers into groups (Favorites, Minigames, SMP, Anarchy) — collapsible, drag reorder. Region-aware ping (US-EU-AS) with colored latency badges.
- **Discord RPC per-profile status**: Custom status text per account/profile — "Playing Cinder · Modded 1.21" or "Building on Creative" — set in profile editor, shows in Discord presence.
- **Java vendor detection UI**: Settings → Java shows vendor badge (Eclipse Temurin, Microsoft, Oracle, Azul Zulu, BellSoft, Adoptium, GraalVM) + architecture (x64/arm64) + FX support. Warn if vendor mismatched for version.
- **JDK download helper**: "Install recommended Java" button — downloads Eclipse Temurin JDK (LTS matching version) via adoptium.net API, extracts to game dir, auto-selects.
- **Game directory health check**: Settings → "Verify game dir" — scans for corrupted version jars, missing libraries, orphaned mods, duplicate resource packs, invalid symlinks. Report + auto-fix button.
- **Export/import settings**: Settings → "Export config" (JSON) / "Import config" — migrates accounts (encrypted), versions, mods, resource packs, shaders, JVM presets, server groups, Discord RPC config. Versioned schema with migration.
- **Command palette enhancements**: `Ctrl+K` now searches mods, resource packs, shaders, servers, settings, accounts — fuzzy match, keyboard nav, recent items at top.
- **Accessibility**: Full keyboard nav for all tabs, ARIA live regions for toasts/progress, high-contrast mode toggle (Settings → Appearance), screen-reader labels on all interactive elements, focus trap in modals.

### Changed
- Version bumped to 2.0.1 (`package.json`, `index.html` titlebar, `README.md`, website).
- **Play tab redesign**: Instance pill now shows launch profile dropdown; skin canvas larger; server quick-join shows group chips; launch button splits into "Launch" / "Launch with profile…" submenu.
- **Accounts tab**: Added profile group sidebar; Microsoft login now shows Xbox gamertag + Xbox profile pic; offline name generator adds "Custom pattern" style (regex).
- **Mods tab**: Mod list now shows dependency graph toggle (tree view); install flow shows compatibility warnings before download; "Open folder" opens profile-specific mods dir.
- **Servers tab**: Added group chips above grid; ping column shows region flag (🇺🇸🇪🇺🇸🇬); "Copy IP" button on each card; custom server saves to group.
- **Settings**: Reorganized into sections (Game, Java, Performance, Appearance, Discord, Advanced); added search filter; JVM preset cards with descriptions.
- **Logs**: Added level filter pills (ERROR/WARN/INFO/DEBUG), "Copy last 100 lines" button, timestamp toggle, "Save to file" (timestamped).
- **Website**: All 6 pages updated with 2.0.1 features; new screenshots for Resource Packs, Shader Packs, Profiles, Launch Profiles tabs; download page shows 2.0.1 checksum.

### Fixed
- Microsoft token refresh race condition on rapid relaunch.
- Modrinth version picker showing incompatible loaders for Fabric/Quilt.
- World backup timestamp collision on rapid successive backups.
- Discord RPC disconnect on sleep/wake (now reconnects within 5s).
- Java auto-detect picking wrong architecture on arm64 macOS.
- Mod enable/disable not persisting across launcher restarts.
- Command palette not closing on ESC in some cases.
- High-contrast mode not applying to custom scrollbars.

### Security
- Account encryption: offline account UUIDs now encrypted at rest (AES-256-GCM, key derived from machine ID + user password if set).
- Modrinth API calls use pinned TLS certificates (certificate pinning).
- Diagnostic copy sanitizes file paths (replaces user home with ~).

## [2.0.0] - 2026-09-20

### Added
- Skins tab: 3D preview (skinview3d) by username via `skin:get`, My profile skin, Upload PNG (64x64), Save PNG.
- Worlds tab: saves list with size/mtime, zip backup / restore / delete via `adm-zip` (`worlds:list`, `worlds:backup`, `worlds:restore`, `worlds:delete`).
- Live server pings via `server:status`: Rust `slp-probe` in `tools/slp-probe` (fast path) with pure-Node SLP fallback; 60 s cache.
- Performance & updates settings: JVM presets `balanced` / `performance` / `latency` (`jvmPreset`), snowfall toggle (`showSnow`), update checker (`checkUpdates` + `update:check`), copy diagnostics (`diag:run`).
- Scripts: `scripts/dev.ps1` (install + start), `scripts/build.ps1` (warn-only typecheck + must-pass tests), `npm run typecheck` (`tsc --noEmit`).
- Tools: `tools/assets.py --check` / `--fetch` (validate + re-download `assets/img/*`), `tools/release` (`go run .` release readiness check).
- Deps: `adm-zip`, `skinview3d`, `three`; `bin/` shipped in electron-builder files.

### Changed
- Version bumped to 2.0.0 (`package.json`, `index.html` pills, README title).
- Settings extended with `jvmPreset`, `showSnow`, `checkUpdates` (sanitized + persisted).
- Features list + docs expanded for Skins, Worlds, pings, performance, scripts, tools.

### Notes
- Binaries are optional: app runs without `bin/slp-probe`; JS fallbacks cover pings.
- Rust toolchain required only to rebuild `slp-probe` (`cargo build --release`, copy to `bin/`).
- Go toolchain required only to run `tools/release` (`go run .`).
- Rust/Go toolchains are NOT required to run the app (`npm install` + `npm start` is enough).

## [1.0.0] - baseline

- Initial Electron launcher: Play / Servers / Versions / Profiles / Mods / Settings / Logs, offline + Microsoft auth, name forge, Fabric/Quilt auto-install, Modrinth browser, Discord Rich Presence, portable game dir, Java auto-detect.