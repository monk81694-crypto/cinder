# Changelog

All notable changes to Cinder.

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