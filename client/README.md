# Cinder Client

A client-side Fabric mod for Minecraft 1.21.11: HUD modules, menus, a config
screen, an editor, performance helpers, and utilities â€” all **original code**
written for this project (see "Original code & fair play" below).

- Mod id: `cinder-client`
- Root package: `gg.cinder.client`
- License: MIT
- Entry point: `gg.cinder.client.CinderClient` (implements `ModInitializer`,
  registered as a `client` entrypoint in `fabric.mod.json`)
- Environment: **client only** (`"environment": "client"` in `fabric.mod.json`)

## Toolchain verification (checked live, 2026-09-20)

| Pin | Value | How verified |
| --- | ----- | ------------ |
| Minecraft | 1.21.11 | Stable release; Yarn `1.21.11+build.6` exists on meta.fabricmc.net. (26.x was tried first but has NO published mappings — Mojang ships it unobfuscated and Yarn is deprecated — so 1.21.11 is the newest compilable stable.) |
| Mappings | Yarn 1.21.11+build.6 | `meta.fabricmc.net/v2/versions/yarn/1.21.11` returns builds, latest `1.21.11+build.6`. |
| Fabric Loader | 0.19.3 | Game-agnostic line (newest stable is 0.19.5; 0.19.3 kept). |
| Fabric API | 0.141.6+1.21.11 | Release file on CurseForge; resolves from maven.fabricmc.net. |
| Loom | 1.17.21 | Newest `1.17.x` on maven.fabricmc.net. |
| Gradle | 9.5.1 | Via the wrapper (`gradle/wrapper/gradle-wrapper.properties`). |
| Java | 21 (compiled) / 25 (Gradle JVM) | Temurin JDK 25 present; `build.gradle` compiles with `--release 21` so the jar runs on any Java 21+ runtime; `fabric.mod.json` requires `"java": ">=21"`. |

## Prerequisites

- **JDK 21+ to run, JDK 25 used here to build** (Temurin 25 confirmed on this machine). No Gradle install needed —
  the wrapper (`gradlew` / `gradlew.bat` + `gradle/wrapper/gradle-wrapper.jar`,
  already vendored in this repo) downloads Gradle 9.5.1 automatically.

## Build

```sh
./gradlew build        # Linux/macOS
.\gradlew.bat build    # Windows
```

The mod jar lands in `build/libs/` (archive name `cinder-client-1.0.0.jar`).

> Wrapper jar note: `gradle/wrapper/gradle-wrapper.jar` is already present and
> was validated (ZIP archive, `org/gradle/wrapper` classes). If you ever need
> to re-fetch it manually, download
> `https://github.com/gradle/gradle/raw/master/gradle/wrapper/gradle-wrapper.jar`
> into `gradle/wrapper/` (or run `gradle wrapper --gradle-version 9.5.1` with a
> local Gradle install).

Run tests:

```sh
./gradlew test
```

## Install into the launcher instance

1. Build the jar (see above) â€” or take it from `build/libs/`.
2. Drop `cinder-client-1.0.0.jar` **plus** the matching Fabric API jar
   (`fabric-api-0.141.6+1.21.11`) into the instance's `mods/` folder — or use
   the launcher's Client tab, which does this for you.
3. Launch with Fabric Loader `>=0.19.3` on Minecraft 1.21.11 + Java 25.

## Original code & fair play

All code in this project is written from scratch for Cinder (MIT licensed).
No code is copied from other mods â€” only standard, publicly documented Fabric
Loom build patterns and APIs are used. Cinder is a fair-play client: it does
not ship cheats, exploits, or anything designed to bypass server-side
anti-cheat or to gain unfair advantages in multiplayer.