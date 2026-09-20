# bin/ — optional native helpers (built from tools/)

Cinder runs 100% without these files (every helper has a JS fallback).
Drop compiled binaries here to enable the fast paths:

| File | Source | Purpose |
|------|--------|---------|
| `slp-probe.exe` (Windows) / `slp-probe` (macOS/Linux) | `tools/slp-probe` (`cargo build --release`, then copy the binary here) | Live Minecraft server status (players, MOTD, ping) on the Servers tab |

`bin/**/*` is included in the packaged app via the `files` field in package.json.
Never commit API keys or tokens here — binaries only.
