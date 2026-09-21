# Cinder v2.0.0 — Minecraft Launcher (offline + Microsoft, mods, Discord presence)

![Cinder logo](brand/logo-light.svg)

Made by **radoslavgeme**. © 2026 radoslavgeme. All rights reserved.

Cinder is a proprietary Electron launcher for Minecraft Java: offline profiles with a
built-in **name forge**, Microsoft login for premium play, Fabric/Quilt loader
auto-install, Modrinth mods, server quick-join, and **Discord Rich Presence**
(“Playing Cinder” + logo + Download button on your profile).

v2.0.0 adds Skins + Worlds tabs, live server pings, JVM presets, and release tooling.
Tabs: Play / Servers / Versions / Profiles / Skins / Worlds / Mods / Settings / Logs.

## Quick Start

```powershell
cd tlauncher
npm install
npm start
```

First launch downloads the game version (~200 MB). See Run it for dev / dist / typecheck.

## Run it

```powershell
cd tlauncher
npm install
npm start
# with devtools:
npm run dev
# typecheck + tests:
npm run typecheck
npm test
# distributable installers (Windows NSIS / macOS dmg / Linux AppImage):
npm run dist
```

First launch downloads the game version (~200 MB). Java is auto-detected and
auto-picked per version (26.x → Java 25, 1.20.5+/1.21+ → Java 21, 1.18+ → 17).

## Which servers can I join?

| Account type | Where it works |
|---|---|
| **Offline name** (incl. everything the name forge makes) | Offline-mode / cracked servers, LAN worlds, and any server you host yourself with `online-mode=false`. |
| **Microsoft login** (owns Minecraft Java) | Everywhere, including online-mode premium servers such as Hypixel. Token auto-refreshes on launch. |

No launcher can make an offline name join an online-mode server: Hypixel-style
servers ask Mojang “is this session valid?” and reject anything unsigned with
“Failed to log in: Invalid session”. Anyone promising otherwise is selling a
bypass that will get accounts banned. Cinder tells you this in the UI instead
of pretending.

## Name forge

Accounts → **Surprise me** (dice button) generates guaranteed-valid
Minecraft names (`A–Z 0–9 _`, 3–16 chars) in five styles: Mixed, Ember, Hero,
Dark, Cute. Suggestions appear as chips — click one to fill it in, then Add.
Names are checked against your existing accounts so you never get a duplicate.

## Skins

Skins tab: live 3D preview (skinview3d, auto-spin + walking pose, zoom locked).

- **Try a name:** type any Java username → Preview fetches its live skin via `skin:get` (24 h disk cache).
- **My profile skin:** loads the active account skin in one click.
- **Upload PNG:** import a classic 64x64 PNG from disk.
- **Save PNG:** download the current preview frame.

## Worlds

Worlds tab: singleplayer saves in `<gameDir>/saves` (folders with `level.dat`).

- List with size + modified date, Refresh, Open saves folder.
- **Backup** (`worlds:backup`): zip to `<gameDir>/backups/<name>-YYYYMMDD-HHMM.zip` via `adm-zip`.
- **Restore** (`worlds:restore`): extract newest backup for that world; current folder kept as `.bak-<timestamp>`.
- **Delete** (`worlds:delete`): remove the save folder (confirm modal).

## Servers — live pings

Servers tab shows featured networks + custom quick-join with live status via `server:status` (60 s cache).

- Fast path: Rust `slp-probe` in `tools/slp-probe` (std-only, zero deps, protocol 767, one JSON line out).
- Fallback: pure-Node SLP in `main.js` when the binary is missing — same `players / max / motd / pingMs / version` fields.
- Build it (optional):
```powershell
cd tools/slp-probe
cargo build --release
copy target\release\slp-probe.exe ..\..\bin\  # -> bin/
```

## Discord profile status — zero setup for players

Just keep the Discord desktop app open. Your profile automatically shows
**“Playing Cinder”** + logo + Download button; in game it switches to
**“Playing Minecraft {version}”** + “{player} → {server}” with a timer.
No bot, no token, no portal — ever, for players. (Works like Valorant's
auto-status; technically it's one shared app ID baked into the launcher
instead of Discord-partnered detection — same experience.)

**Profile says “Minecraft” with a grass block instead of Cinder?** That is
Discord's built-in game detection (`javaw.exe` → its Minecraft entry). Our
presence replaces it — but only while connected. Checklist: Discord desktop
app open *before* launching → Settings → Discord → Test says **Connected ✓**.
Still losing? Discord → Settings → Registered Games → remove the Minecraft
entry → restart Discord → Test again. (Cinder also retries the connection
every 30s on its own.)

### Owner setup (radoslavgeme — once, ~1 min, then never again)

1. https://discord.com/developers/applications → **New Application**,
   name it `Cinder` (that name is the word after “Playing”).
2. Rich Presence → Art Assets → upload **`assets/logo-512.png`**
   (already in the repo, ready to go) named **`logo`**.
3. Copy the **Application ID** → paste it as `DEFAULT_CLIENT_ID` in
   `discord.js` (replacing the placeholder) → ship. Done — every player
   from that build on gets automatic status.

Alternatively set the env var `CINDER_DISCORD_CLIENT_ID` instead of the field.

## Features

- Play: account + version picker, server quick-join (`hypixel.net`), LAUNCH +
  Kill, progress, recent-server chips, remembers last account/version/server
- Servers: featured grid + custom IP, live pings (Rust binary with JS fallback), search + filters
- Versions: live Mojang list (6 h cache + offline fallback), search,
  release/snapshot/beta/alpha filter, installed badges + filter, uninstall
- Accounts: offline + Microsoft, avatars, click-to-activate, name forge
- Skins: 3D preview by name, profile skin, PNG upload / download
- Worlds: saves list, zip backup / restore / delete via `adm-zip`
- Mods: Modrinth search + version picker, local list with enable/disable
  toggle, Fabric/Quilt auto-install, Forge/NeoForge mismatch warnings
- Settings: portable game dir (default `%AppData%/.cinder-minecraft`), Java
  dropdown (generic detection: JAVA_HOME, PATH, Adoptium/Microsoft/Java/
  Temurin/BellSoft/Zulu, Mojang-bundled runtimes — no hardcoded paths), RAM
  sliders with presets + system hint, resolution presets, fullscreen, JVM args,
  JVM presets, snowfall toggle, update checker, copy diagnostics,
  launch behavior (minimize / hide / nothing), launch timeout, accent themes,
  reduced motion, log filters, one-click Saves/Screenshots folders,
  Discord section
- Logs: colored levels, filter, follow toggle, copy, background-activity dot
- UX: toasts, confirm modals, skeletons, `Ctrl+1..8` tabs, `Ctrl+Enter` launch

## Performance & updates (Settings)

Settings → Performance & updates (keys: `jvmPreset` / `showSnow` / `checkUpdates`, IPC: `update:check`, `diag:run`).

- **JVM preset:** `balanced` (default), `performance` (G1GC throughput), `latency` (ZGC, Java 21+ only).
- **Show snowfall:** toggle the animated background.
- **Check for updates:** on-launch GitHub release check + Check now button.
- **Copy diagnostics:** copies version, Java list, settings, net probes (Mojang / Modrinth / Minotar) + log tail for support.

## Scripts & checks

```powershell
powershell -ExecutionPolicy Bypass -File scripts/dev.ps1
powershell -ExecutionPolicy Bypass -File scripts/build.ps1
npm run typecheck
```

- `scripts/dev.ps1`: `npm install` if needed, then `npm start`.
- `scripts/build.ps1`: `npm run typecheck` (warn-only) + `npm test` (must pass).
- `npm run typecheck`: `tsc --noEmit` per `tsconfig.json` / `types/`.

## Tools

```powershell
python tools/assets.py --check
python tools/assets.py --fetch
cd tools/release; go run .
```

- `tools/assets.py --check`: validate `assets/img/*` + icons (magic + size + dimensions).
- `tools/assets.py --fetch`: re-download `assets/img/*` from embedded URLs, then validate.
- `tools/release` (`go run .`): release readiness — semver version, version strings in `index.html` + `README.md`, required files, `npm test`. Rust/Go needed only for these tools, not to run the app.

## Build / ship

- `npm run dist` uses electron-builder (`build` field in package.json,
  appId `gg.cinder.launcher`). Icons are done (`assets/icon.ico` for Windows,
  `assets/icon.png` for Linux; add `assets/icon.icns` for macOS if you build
  for Mac). Before shipping to strangers:
  - Set your real repo URL and Discord Application ID default.
  - `npm audit`; re-run before release.

## Disclaimer

© 2026 radoslavgeme. All rights reserved. Unauthorized copying, modification
or distribution prohibited. Cinder is proprietary software — not open source.

Not affiliated with Mojang, Microsoft, or Discord. Minecraft requires a paid
Java account for online-mode servers. Offline mode is for servers you own or
that explicitly allow it — respect server rules and the law where you live.